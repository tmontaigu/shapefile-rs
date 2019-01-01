extern crate shapefile;

use std::fs::File;
use std::io::Read;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;

const LINE_PATH: &str = "./tests/data/line.shp";
const LINEM_PATH: &str = "./tests/data/linem.shp";
const LINEZ_PATH: &str = "./tests/data/linez.shp";

#[test]
fn read_line_header() {
    let mut file = File::open(LINE_PATH).unwrap();
    let header = shapefile::header::Header::read_from(&mut file).unwrap();

    assert_eq!(header.shape_type, shapefile::ShapeType::Polyline);
}

fn check_line<T: Read>(mut reader: shapefile::Reader<T>) {
    let shapes = reader.read().unwrap();

    assert_eq!(shapes.len(), 1);
    match &shapes[0] {
        shapefile::record::Shape::Polyline(_poly) => {}
        _ => { assert!(false); }
    }

    if let shapefile::Shape::Polyline(shape) = &shapes[0] {
        assert_eq!(shape.bbox.xmin, 1.0);
        assert_eq!(shape.bbox.ymin, 1.0);
        assert_eq!(shape.bbox.xmax, 5.0);
        assert_eq!(shape.bbox.ymax, 6.0);
        assert_eq!(shape.parts, vec![0, 5]);
        assert_eq!(shape.xs, vec![1.0, 5.0, 5.0, 3.0, 1.0, 3.0, 2.0]);
        assert_eq!(shape.ys, vec![5.0, 5.0, 1.0, 3.0, 1.0, 2.0, 6.0]);
    } else {
        assert!(false, "The shape is not a Polyline");
    }
}
#[test]
fn read_line() {
    let reader = shapefile::Reader::from_path(LINE_PATH).unwrap();
    check_line(reader);
}

fn check_linem<T: Read>(mut reader: shapefile::Reader<T>) {
    use shapefile::NO_DATA;

    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1);


    let shape = &shapes[0];
    if let shapefile::Shape::PolylineM(shape) = &shapes[0] {
        assert_eq!(shape.bbox.xmin, 1.0);
        assert_eq!(shape.bbox.ymin, 1.0);
        assert_eq!(shape.bbox.xmax, 5.0);
        assert_eq!(shape.bbox.ymax, 6.0);
        assert_eq!(shape.parts, vec![0, 5]);
        assert_eq!(shape.xs, vec![1.0, 5.0, 5.0, 3.0, 1.0, 3.0, 2.0]);
        assert_eq!(shape.ys, vec![5.0, 5.0, 1.0, 3.0, 1.0, 2.0, 6.0]);
        assert_eq!(shape.ms, vec![0.0, NO_DATA, 3.0, NO_DATA, 0.0, NO_DATA, NO_DATA]);
        assert_eq!(shape.m_range, [0.0, 3.0]);
    } else {
        assert!(false, "The shape is not a PolylineM");
    }
}

#[test]
fn read_linem() {
    let reader = shapefile::Reader::from_path(LINEM_PATH).unwrap();
    check_linem(reader);
}

fn check_linez<T: Read>(mut reader: shapefile::Reader<T>) {
    use shapefile::NO_DATA;
    let shapes = reader.read().unwrap();

    assert_eq!(shapes.len(), 1);

    for shape in shapes {
        if let shapefile::Shape::PolylineZ(shp) = shape {
            assert_eq!(shp.parts, vec![0, 5, 7]);
            assert_eq!(shp.xs, vec![1.0, 5.0, 5.0, 3.0, 1.0, 3.0, 2.0, 3.0, 2.0, 1.0]);
            assert_eq!(shp.ys, vec![5.0, 5.0, 1.0, 3.0, 1.0, 2.0, 6.0, 2.0, 6.0, 9.0]);

            assert_eq!(shp.z_range, [0.0, 22.0]);
            assert_eq!(shp.zs, vec![18.0, 20.0, 22.0, 0.0, 0.0, 0.0, 0.0, 15.0, 13.0, 14.0]);

            assert_eq!(shp.m_range, [0.0, 3.0]);
            assert_eq!(shp.ms, vec![NO_DATA, NO_DATA, NO_DATA, NO_DATA, NO_DATA, NO_DATA, NO_DATA, 0.0, 3.0, 2.0]);
        } else {
            assert!(false, "The shape is not a PolylineZ");
        }
    }
}

#[test]
fn read_linez() {
    let reader = shapefile::Reader::from_path(LINEZ_PATH).unwrap();
    check_linez(reader);

}

#[test]
fn read_point() {
    let mut reader = shapefile::Reader::from_path("./tests/data/point.shp").unwrap();

    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1);

    let points = shapefile::record::to_vec_of_point(shapes).unwrap();
    assert_eq!(points.len(), 1);

    let point = &points[0];
    assert_eq!(point.x, 122.0);
    assert_eq!(point.y, 37.0);
}



macro_rules! read_write_read_test {
    ($func:ident, $convert_func:ident, $check_func:ident, $src_file:ident) => {
        #[test]
        fn $func() {
            let mut reader = shapefile::Reader::from_path($src_file).unwrap();
            let shapes = reader.read().unwrap();
            let shapes = $convert_func(shapes).unwrap();

            let v = Vec::<u8>::new();
            let mut cursor = Cursor::new(v);
            let mut writer = shapefile::writer::Writer::new(cursor);
            writer.write_shapes(shapes).unwrap();

            cursor = writer.dest;

            cursor.seek(SeekFrom::Start(0)).unwrap();
            let reader = shapefile::Reader::new(cursor).unwrap();
            $check_func(reader);
        }
    };
}

use shapefile::record::{to_vec_of_polyline, to_vec_of_polylinem, to_vec_of_polylinez};
read_write_read_test!(read_write_read_line, to_vec_of_polyline, check_line, LINE_PATH);
read_write_read_test!(read_write_read_linem, to_vec_of_polylinem, check_linem, LINEM_PATH);
read_write_read_test!(read_write_read_linez, to_vec_of_polylinez, check_linez, LINEZ_PATH);
