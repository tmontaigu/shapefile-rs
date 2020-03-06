extern crate shapefile;

mod testfiles;

use shapefile::{Point, Polyline, Polygon, Writer};
/*
use std::io::Cursor;

#[test]
fn write_polyline() {
    let mut polys = Vec::<shapefile::record::Polyline>::with_capacity(1);
    polys.push(shapefile::record::Polyline::default());

    let v = Vec::<u8>::new();
    let cursor = Cursor::new(v);
    let mut writer = shapefile::writer::Writer::new(cursor);
    writer.write_shapes(polys).unwrap();
}*/

fn read_a_file(path: &str) -> std::io::Result<Vec<u8>> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut data: Vec<u8> = vec![];
    file.read_to_end(&mut data)?;
    Ok(data)
}

#[test]
fn single_point() {
    let point = Point::new(122.0, 37.0);
    let mut shp: Vec<u8> = vec![];
    let mut shx: Vec<u8> = vec![];
    let mut writer = Writer::new(&mut shp);
    writer.add_index_dest(&mut shx);
    writer.write_shapes(vec![point]).unwrap();

    let expected = read_a_file(testfiles::POINT_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shp, expected.unwrap());

    let expected = read_a_file(testfiles::POINT_SHX_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shx, expected.unwrap());
}

#[test]
fn multi_line() {
    let point = Polyline::with_parts(vec![
        vec![
            Point::new(1.0, 5.0),
            Point::new(5.0, 5.0),
            Point::new(5.0, 1.0),
            Point::new(3.0, 3.0),
            Point::new(1.0, 1.0),
        ],
        vec![Point::new(3.0, 2.0), Point::new(2.0, 6.0)],
    ]);
    let mut shp: Vec<u8> = vec![];
    let mut shx: Vec<u8> = vec![];
    let mut writer = Writer::new(&mut shp);
    writer.add_index_dest(&mut shx);
    writer.write_shapes(vec![point]).unwrap();

    let expected = read_a_file(testfiles::LINE_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shp, expected.unwrap());

    let expected = read_a_file(testfiles::LINE_SHX_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shx, expected.unwrap());
}

#[test]
fn polygon_inner() {
    let point = Polygon::with_parts(vec![
        vec![
            Point::new(-120.0, 60.0),
            Point::new(120.0, 60.0),
            Point::new(120.0, -60.0),
            Point::new(-120.0, -60.0),
            Point::new(-120.0, 60.0),
        ],
        vec![
            Point::new(-60.0, 30.0),
            Point::new(-60.0, -30.0),
            Point::new(60.0, -30.0),
            Point::new(60.0, 30.0),
            Point::new(-60.0, 30.0),
        ],
    ]);
    let mut shp: Vec<u8> = vec![];
    let mut shx: Vec<u8> = vec![];
    let mut writer = Writer::new(&mut shp);
    writer.add_index_dest(&mut shx);
    writer.write_shapes(vec![point]).unwrap();

    let expected = read_a_file(testfiles::POLYGON_HOLE_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shp, expected.unwrap());

    let expected = read_a_file(testfiles::POLYGON_HOLE_SHX_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shx, expected.unwrap());
}
