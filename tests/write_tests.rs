extern crate shapefile;

mod testfiles;

use shapefile::{Point, Polyline, Polygon, Writer, PolygonRing};


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
    writer.write_shapes(&vec![point]).unwrap();

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
    writer.write_shapes(&vec![point]).unwrap();

    let expected = read_a_file(testfiles::LINE_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shp, expected.unwrap());

    let expected = read_a_file(testfiles::LINE_SHX_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shx, expected.unwrap());
}

#[test]
fn polygon_inner() {
    let point = Polygon::with_rings(vec![
        PolygonRing::Outer(vec![
            Point::new(-120.0, 60.0),
            Point::new(120.0, 60.0),
            Point::new(120.0, -60.0),
            Point::new(-120.0, -60.0),
            Point::new(-120.0, 60.0),
        ]),
        PolygonRing::Inner(vec![
            Point::new(-60.0, 30.0),
            Point::new(-60.0, -30.0),
            Point::new(60.0, -30.0),
            Point::new(60.0, 30.0),
            Point::new(-60.0, 30.0),
        ]),
    ]);
    let mut shp: Vec<u8> = vec![];
    let mut shx: Vec<u8> = vec![];
    let mut writer = Writer::new(&mut shp);
    writer.add_index_dest(&mut shx);
    writer.write_shapes(&vec![point]).unwrap();

    let expected = read_a_file(testfiles::POLYGON_HOLE_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shp, expected.unwrap());

    let expected = read_a_file(testfiles::POLYGON_HOLE_SHX_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shx, expected.unwrap());
}

/// Same polygon as test above, but the points for the ring are in the
/// incorrect order for a shapefile, so this test if we reorder points correctly
#[test]
fn polygon_inner_is_correctly_reordered() {
    let point = Polygon::with_rings(vec![
        PolygonRing::Outer(vec![
            Point::new(-120.0, 60.0),
            Point::new(-120.0, -60.0),
            Point::new(120.0, -60.0),
            Point::new(120.0, 60.0),
            Point::new(-120.0, 60.0),
        ]),
        PolygonRing::Inner(vec![
            Point::new(-60.0, 30.0),
            Point::new(60.0, 30.0),
            Point::new(60.0, -30.0),
            Point::new(-60.0, -30.0),
            Point::new(-60.0, 30.0),
        ]),
    ]);
    let mut shp: Vec<u8> = vec![];
    let mut shx: Vec<u8> = vec![];
    let mut writer = Writer::new(&mut shp);
    writer.add_index_dest(&mut shx);
    writer.write_shapes(&vec![point]).unwrap();

    let expected = read_a_file(testfiles::POLYGON_HOLE_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shp, expected.unwrap());

    let expected = read_a_file(testfiles::POLYGON_HOLE_SHX_PATH);
    assert_eq!(expected.is_ok(), true);
    assert_eq!(shx, expected.unwrap());
}
