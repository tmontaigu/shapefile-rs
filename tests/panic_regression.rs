//! Regression tests for denial-of-service via crafted shapefiles.
//!
//! A tiny crafted file declaring an absurd per-shape count used to either
//! force a multi-gigabyte `Vec::with_capacity` (aborting the process) or, for
//! negative counts, panic with an arithmetic overflow. Reading such a file
//! must now return an error instead.

use std::io::Cursor;

fn push_be_i32(buf: &mut Vec<u8>, v: i32) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn push_le_i32(buf: &mut Vec<u8>, v: i32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Writes a valid 100-byte main header for the given shape type, followed by a
/// single record header announcing `content_length_words` 16-bit words.
fn push_header_and_record_header(buf: &mut Vec<u8>, shape_type: i32, content_length_words: i32) {
    // ---- main header (100 bytes) ----
    push_be_i32(buf, 9994); // file code
    buf.extend_from_slice(&[0u8; 20]); // 5 unused i32
    push_be_i32(buf, 1000); // file length in 16-bit words (arbitrary, > header)
    push_le_i32(buf, 1000); // version
    push_le_i32(buf, shape_type);
    buf.extend_from_slice(&[0u8; 64]); // header bbox: 8 f64

    // ---- record header (8 bytes) ----
    push_be_i32(buf, 1); // record number
    push_be_i32(buf, content_length_words);
}

/// Builds a valid 100-byte main header with the given shape type and file length.
fn main_header(shape_type: i32, file_length: i32) -> Vec<u8> {
    let mut buf = Vec::new();
    push_be_i32(&mut buf, 9994); // file code
    buf.extend_from_slice(&[0u8; 20]); // 5 unused i32
    push_be_i32(&mut buf, file_length);
    push_le_i32(&mut buf, 1000); // version
    push_le_i32(&mut buf, shape_type);
    buf.extend_from_slice(&[0u8; 64]); // header bbox: 8 f64
    buf
}

fn read_shapes(buf: Vec<u8>) -> Result<Vec<shapefile::Shape>, shapefile::Error> {
    shapefile::ShapeReader::new(Cursor::new(buf))
        .expect("header is valid")
        .read()
}

#[test]
fn multipoint_with_huge_point_count_errors_instead_of_aborting() {
    const NUM_POINTS: i32 = 130_000_000;
    let content_len_bytes = 4 + 32 + 4 + 16i64 * i64::from(NUM_POINTS);

    let mut buf = Vec::new();
    push_header_and_record_header(&mut buf, 8, (content_len_bytes / 2) as i32);
    push_le_i32(&mut buf, 8); // record shape type = Multipoint
    buf.extend_from_slice(&[0u8; 32]); // record bbox
    push_le_i32(&mut buf, NUM_POINTS); // declared count, no points follow

    assert!(read_shapes(buf).is_err());
}

#[test]
fn multipoint_with_negative_point_count_errors_instead_of_panicking() {
    let mut buf = Vec::new();
    push_header_and_record_header(&mut buf, 8, 100);
    push_le_i32(&mut buf, 8); // record shape type = Multipoint
    buf.extend_from_slice(&[0u8; 32]); // record bbox
    push_le_i32(&mut buf, -1); // negative count

    let err = read_shapes(buf).err().expect("reading must fail");
    assert!(matches!(err, shapefile::Error::InvalidShapeRecordSize));
}

#[test]
fn negative_file_length_header_errors_instead_of_panicking() {
    // A negative file_length used to overflow `(file_length as usize) * 2`
    // when building the shape iterator (reproduced by the fuzzer).
    let buf = main_header(8, -6);
    let err = shapefile::ShapeReader::new(Cursor::new(buf))
        .err()
        .expect("header must be rejected");
    assert!(matches!(err, shapefile::Error::InvalidShapeRecordSize));
}

#[test]
fn huge_record_size_errors_instead_of_panicking() {
    // `record_size * 2` would overflow i32 while reading the record.
    let mut buf = Vec::new();
    push_header_and_record_header(&mut buf, 8, 1_200_000_000);

    let err = read_shapes(buf).err().expect("reading must fail");
    assert!(matches!(err, shapefile::Error::InvalidShapeRecordSize));
}

#[test]
fn huge_shx_file_length_errors_instead_of_panicking() {
    // In read_index_file, `file_length * 2` would overflow i32.
    let shp = main_header(1, 50);
    let shx = main_header(1, 1_200_000_000);
    let err = shapefile::ShapeReader::with_shx(Cursor::new(shp), Cursor::new(shx))
        .err()
        .expect("index must be rejected");
    assert!(matches!(err, shapefile::Error::InvalidShapeRecordSize));
}

#[test]
fn polyline_with_non_monotonic_parts_array_errors_instead_of_panicking() {
    // A part index greater than the following one (or than num_points) yields a
    // negative span; it must error rather than trip a debug assertion.
    let mut buf = Vec::new();
    push_header_and_record_header(&mut buf, 3, 24);
    push_le_i32(&mut buf, 3); // record shape type = Polyline
    buf.extend_from_slice(&[0u8; 32]); // record bbox
    push_le_i32(&mut buf, 1); // num_parts
    push_le_i32(&mut buf, 0); // num_points
    push_le_i32(&mut buf, 5); // parts_array[0] = 5 > num_points (0)

    let err = read_shapes(buf).err().expect("reading must fail");
    assert!(matches!(err, shapefile::Error::InvalidShapeRecordSize));
}

#[test]
fn polyline_with_negative_part_count_errors_instead_of_panicking() {
    let mut buf = Vec::new();
    push_header_and_record_header(&mut buf, 3, 100);
    push_le_i32(&mut buf, 3); // record shape type = Polyline
    buf.extend_from_slice(&[0u8; 32]); // record bbox
    push_le_i32(&mut buf, -1); // num_parts (negative)
    push_le_i32(&mut buf, 0); // num_points

    let err = read_shapes(buf).err().expect("reading must fail");
    assert!(matches!(err, shapefile::Error::InvalidShapeRecordSize));
}
