//! The reader iterators must stop after the first error.
//!
//! `ShapeIterator` decides it is done by comparing `current_pos` with the
//! header's `file_length`, but only the success path advances `current_pos`, so
//! an error used to leave the iterator in exactly the state it was already in
//! and `next()` returned the same error forever.
//!
//! Every test pulls a bounded number of items, so a regression fails instead of
//! hanging CI.

use std::io::{Cursor, Read, Seek, SeekFrom};

use shapefile::dbase;
use shapefile::{Polyline, Reader, Shape, ShapeReader};

/// A valid 100-byte main header with the given (untrusted) `file_length`.
fn main_header(shape_type: i32, file_length_words: i32) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&9994i32.to_be_bytes()); // file code
    buf.extend_from_slice(&[0u8; 20]); // 5 unused i32
    buf.extend_from_slice(&file_length_words.to_be_bytes());
    buf.extend_from_slice(&1000i32.to_le_bytes()); // version
    buf.extend_from_slice(&shape_type.to_le_bytes());
    buf.extend_from_slice(&[0u8; 64]); // bbox: 8 f64
    buf
}

/// An 8-byte record header (number, size in 16-bit words) followed by `body`.
fn record(number: i32, size_words: i32, body: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&number.to_be_bytes());
    buf.extend_from_slice(&size_words.to_be_bytes());
    buf.extend_from_slice(body);
    buf
}

fn point_body(x: f64, y: f64) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&1i32.to_le_bytes()); // ShapeType::Point
    buf.extend_from_slice(&x.to_le_bytes());
    buf.extend_from_slice(&y.to_le_bytes());
    buf
}

/// `n` Point records with a truthful `file_length`.
fn valid_points_shp(n: i32) -> Vec<u8> {
    let mut records = Vec::new();
    for i in 0..n {
        records.extend(record(i + 1, 10, &point_body(i.into(), i.into())));
    }
    let mut buf = main_header(1, ((100 + records.len()) / 2) as i32);
    buf.extend(records);
    buf
}

/// A .shx holding one 8-byte index entry per given offset (in 16-bit words).
fn shx(offsets_words: &[i32]) -> Vec<u8> {
    let mut entries = Vec::new();
    for offset in offsets_words {
        entries.extend_from_slice(&offset.to_be_bytes());
        entries.extend_from_slice(&10i32.to_be_bytes()); // content length
    }
    let mut buf = main_header(1, ((100 + entries.len()) / 2) as i32);
    buf.extend(entries);
    buf
}

/// Offsets of `n` sequential 28-byte Point records, in 16-bit words.
fn sequential_offsets(n: usize) -> Vec<i32> {
    (0..n).map(|i| ((100 + i * 28) / 2) as i32).collect()
}

const DBF_FIELD_LEN: usize = 16;
/// 32-byte header + 32 bytes per field + a 1-byte terminator.
const DBF_HEADER_LEN: usize = 32 + 32 + 1;
/// A deletion flag byte followed by the field data.
const DBF_RECORD_LEN: usize = 1 + DBF_FIELD_LEN;

fn valid_dbf(n: usize) -> Vec<u8> {
    let mut dest = Cursor::new(Vec::new());
    let writer = dbase::TableWriterBuilder::new()
        .add_character_field("NAME".try_into().unwrap(), DBF_FIELD_LEN as u8)
        .build_with_dest(&mut dest);
    let records: Vec<dbase::Record> = (0..n)
        .map(|i| {
            let mut record = dbase::Record::default();
            record.insert(
                "NAME".to_string(),
                dbase::FieldValue::Character(Some(format!("rec{i}"))),
            );
            record
        })
        .collect();
    writer.write_records(&records).unwrap();
    dest.into_inner()
}

/// A source whose `seek` always fails, to reach the seek error path.
struct FailingSeek(Cursor<Vec<u8>>);

impl Read for FailingSeek {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl Seek for FailingSeek {
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        Err(std::io::Error::other("seek refused"))
    }
}

/// Reads `expected_shapes` shapes, then requires exactly one error and `None`.
fn assert_stops_after_first_error<T, E>(
    mut iter: impl Iterator<Item = Result<T, E>>,
    expected_shapes: usize,
) {
    for i in 0..expected_shapes {
        assert!(
            iter.next().expect("a shape was expected").is_ok(),
            "shape {i} should have been read"
        );
    }
    assert!(iter.next().expect("an error was expected").is_err());
    assert!(
        iter.next().is_none(),
        "the iterator must stop after the first error"
    );
}

#[test]
fn truncated_shp_stops_after_one_error() {
    // The header claims 1000 bytes, the file is only the 100-byte header. At EOF
    // `Read::read` keeps returning `Ok(0)`, so the record header read keeps
    // failing without consuming anything.
    let mut reader = ShapeReader::new(Cursor::new(main_header(5, 500))).unwrap();
    assert_stops_after_first_error(reader.iter_shapes_as::<Shape>(), 0);
}

#[test]
fn shp_truncated_after_a_valid_record_stops_after_one_error() {
    let mut buf = main_header(1, 5000);
    buf.extend(record(1, 10, &point_body(1.0, 2.0)));
    let mut reader = ShapeReader::new(Cursor::new(buf)).unwrap();
    assert_stops_after_first_error(reader.iter_shapes_as::<Shape>(), 1);
}

#[test]
fn invalid_record_shape_type_stops_after_one_error() {
    let mut body = 60i32.to_le_bytes().to_vec(); // not a ShapeType
    body.extend_from_slice(&[0u8; 16]);
    let mut buf = main_header(5, 5000);
    buf.extend(record(1, 10, &body));
    let mut reader = ShapeReader::new(Cursor::new(buf)).unwrap();
    assert_stops_after_first_error(reader.iter_shapes_as::<Shape>(), 0);
}

#[test]
fn wrong_requested_shape_type_stops_after_one_error() {
    // Nothing is malformed here: a complete, valid Point file read as Polyline.
    // Every record yields `MismatchShapeType`, which is documented behaviour for
    // `read_as`, but used to make the iterator spin.
    let mut reader = ShapeReader::new(Cursor::new(valid_points_shp(3))).unwrap();
    assert_stops_after_first_error(reader.iter_shapes_as::<Polyline>(), 0);
}

#[test]
fn negative_record_size_stops_after_one_error() {
    // `NullShape` ignores the record size, so the record is read successfully and
    // `(record_size as usize) * 2` then overflows in the iterator itself.
    let mut buf = main_header(0, 5000);
    buf.extend(record(1, -1, &0i32.to_le_bytes())); // ShapeType::NullShape
    buf.extend_from_slice(&[0u8; 400]);
    let mut reader = ShapeReader::new(Cursor::new(buf)).unwrap();
    assert_stops_after_first_error(reader.iter_shapes_as::<Shape>(), 0);
}

#[test]
fn invalid_shx_offset_stops_after_one_error() {
    let mut reader = ShapeReader::with_shx(
        Cursor::new(valid_points_shp(3)),
        Cursor::new(shx(&[-1, -1, -1])),
    )
    .unwrap();
    assert_stops_after_first_error(reader.iter_shapes_as::<Shape>(), 0);
}

#[test]
fn size_hint_is_zero_once_the_iterator_has_stopped() {
    let mut reader = ShapeReader::with_shx(
        Cursor::new(valid_points_shp(3)),
        Cursor::new(shx(&sequential_offsets(3))),
    )
    .unwrap();
    let mut iter = reader.iter_shapes_as::<Polyline>(); // wrong type: errors at once
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert!(iter.next().expect("an error was expected").is_err());
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn failing_seek_stops_after_one_error() {
    // Offsets that differ from the current position, so a seek is attempted.
    let mut reader = ShapeReader::with_shx(
        FailingSeek(Cursor::new(valid_points_shp(3))),
        Cursor::new(shx(&[60, 74, 88])),
    )
    .unwrap();
    assert_stops_after_first_error(reader.iter_shapes_as::<Shape>(), 0);
}

#[test]
fn shape_and_record_iterator_stops_after_a_shape_error() {
    let shape_reader = ShapeReader::new(Cursor::new(main_header(5, 500))).unwrap();
    let dbase_reader = dbase::Reader::new(Cursor::new(valid_dbf(3))).unwrap();
    let mut reader = Reader::new(shape_reader, dbase_reader);
    assert_stops_after_first_error(reader.iter_shapes_and_records(), 0);
}

#[test]
fn shape_and_record_iterator_stops_after_a_record_error() {
    // A valid .shp with 3 records, but a .dbf holding only the first one: the
    // remaining shapes must not be paired with records that are not there.
    let dbf = valid_dbf(3);
    let shape_reader = ShapeReader::new(Cursor::new(valid_points_shp(3))).unwrap();
    let dbase_reader =
        dbase::Reader::new(Cursor::new(dbf[..DBF_HEADER_LEN + DBF_RECORD_LEN].to_vec())).unwrap();
    let mut reader = Reader::new(shape_reader, dbase_reader);
    assert_stops_after_first_error(reader.iter_shapes_and_records(), 1);
}

#[test]
fn valid_shp_is_read_completely() {
    let mut reader = ShapeReader::new(Cursor::new(valid_points_shp(3))).unwrap();
    let mut iter = reader.iter_shapes_as::<Shape>();
    for i in 0..3 {
        assert!(iter.next().expect("a shape was expected").is_ok(), "{i}");
    }
    for _ in 0..3 {
        assert!(iter.next().is_none());
    }
}

#[test]
fn valid_shp_with_shx_is_read_completely() {
    let mut reader = ShapeReader::with_shx(
        Cursor::new(valid_points_shp(3)),
        Cursor::new(shx(&sequential_offsets(3))),
    )
    .unwrap();
    assert_eq!(reader.iter_shapes_as::<Shape>().count(), 3);
}

#[test]
fn valid_shp_and_dbf_are_read_completely() {
    let shape_reader = ShapeReader::new(Cursor::new(valid_points_shp(3))).unwrap();
    let dbase_reader = dbase::Reader::new(Cursor::new(valid_dbf(3))).unwrap();
    let mut reader = Reader::new(shape_reader, dbase_reader);
    let pairs = reader.read().unwrap();
    assert_eq!(pairs.len(), 3);
}
