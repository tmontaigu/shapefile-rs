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
