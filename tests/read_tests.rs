extern crate shapefile;

use std::fs::File;

#[test]
fn read_line_header() {
    let mut file = File::open("./tests/data/line.shp").unwrap();
    let header = shapefile::header::Header::read_from(&mut file).unwrap();

    assert_eq!(header.shape_type, shapefile::ShapeType::Polyline);
}

#[test]
fn read_line() {
    let mut reader = shapefile::Reader::from_path("./tests/data/line.shp").unwrap();
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
        assert!(false);
    }
}


#[test]
fn read_linem() {
    use shapefile::NO_DATA;
    let mut reader = shapefile::Reader::from_path("./tests/data/linem.shp").unwrap();

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
        assert!(false);
    }
}

#[test]
fn read_linez() {
    use shapefile::NO_DATA;

    let mut reader = shapefile::Reader::from_path("./tests/data/linez.shp").unwrap();
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
            assert!(false);
        }
    }
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




