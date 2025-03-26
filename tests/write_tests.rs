extern crate shapefile;

mod testfiles;

use shapefile::writer::ShapeWriter;
use shapefile::{Point, Polygon, PolygonRing, Polyline};
use std::io::Cursor;

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
    let mut shp: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let mut shx: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let writer = ShapeWriter::with_shx(&mut shp, &mut shx);
    writer.write_shapes(&vec![point]).unwrap();

    let expected = read_a_file(testfiles::POINT_PATH).unwrap();
    assert_eq!(shp.get_ref(), &expected);

    let expected = read_a_file(testfiles::POINT_SHX_PATH).unwrap();
    assert_eq!(&shx.get_ref()[..100], &expected[..100]);
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
    let mut shp: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let mut shx: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let writer = ShapeWriter::with_shx(&mut shp, &mut shx);
    writer.write_shapes(&vec![point]).unwrap();

    let expected = read_a_file(testfiles::LINE_PATH).unwrap();
    assert_eq!(shp.get_ref(), &expected);

    let expected = read_a_file(testfiles::LINE_SHX_PATH).unwrap();
    assert_eq!(shx.get_ref(), &expected);
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
    let mut shp: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let mut shx: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let writer = ShapeWriter::with_shx(&mut shp, &mut shx);
    writer.write_shapes(&vec![point]).unwrap();

    let expected = read_a_file(testfiles::POLYGON_HOLE_PATH).unwrap();
    assert_eq!(shp.get_ref(), &expected);

    let expected = read_a_file(testfiles::POLYGON_HOLE_SHX_PATH).unwrap();
    assert_eq!(shx.get_ref(), &expected);
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
    let mut shp: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let mut shx: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let writer = ShapeWriter::with_shx(&mut shp, &mut shx);
    writer.write_shapes(&vec![point]).unwrap();

    let expected = read_a_file(testfiles::POLYGON_HOLE_PATH).unwrap();
    assert_eq!(shp.get_ref(), &expected);

    let expected = read_a_file(testfiles::POLYGON_HOLE_SHX_PATH).unwrap();
    assert_eq!(shx.get_ref(), &expected);
}

/// Same polygon as test above, but the points for the ring are in the
/// incorrect order for a shapefile, so this test if we reorder points correctly
#[test]
fn shape_writer_explicit_finalize() {
    let shape = Polygon::with_rings(vec![
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
    let mut shp: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let mut shx: Cursor<Vec<u8>> = Cursor::new(vec![]);
    let mut writer = ShapeWriter::with_shx(&mut shp, &mut shx);
    writer.write_shape(&shape).unwrap();
    writer.finalize().unwrap();
    drop(writer);

    let expected = read_a_file(testfiles::POLYGON_HOLE_PATH).unwrap();
    assert_eq!(shp.get_ref(), &expected);

    let expected = read_a_file(testfiles::POLYGON_HOLE_SHX_PATH).unwrap();
    assert_eq!(shx.get_ref(), &expected);
}
