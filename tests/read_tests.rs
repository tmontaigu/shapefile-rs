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
    let file = File::open("./tests/data/line.shp").unwrap();
    let mut reader = shapefile::Reader::new(file).unwrap();
    let shapes = reader.read().unwrap();

    assert_eq!(shapes.len(), 1);
    match &shapes[0] {
        shapefile::record::Shape::Polyline(_poly) => {},
        _ => {assert!(false);}
    }

    if let shapefile::record::Shape::Polyline(shape) = &shapes[0] {
        assert_eq!(shape.bbox.xmin, 1.0);
        assert_eq!(shape.bbox.ymin, 1.0);
        assert_eq!(shape.bbox.xmax, 5.0);
        assert_eq!(shape.bbox.ymax, 6.0);
        assert_eq!(shape.parts, vec![0, 5]);
        assert_eq!(shape.xs, vec![1.0, 5.0, 5.0, 3.0, 1.0, 3.0, 2.0]);
        assert_eq!(shape.ys, vec![5.0, 5.0, 1.0, 3.0, 1.0, 2.0, 6.0]);
        assert!(shape.z.is_none());
        assert!(shape.m.is_none());
    }
    else {
        assert!(false);
    }
}


#[test]
fn read_linem() {
    let file = File::open("./tests/data/linem.shp").unwrap();
    let mut reader = shapefile::Reader::new(file).unwrap();
    let shapes = reader.read().unwrap();

    let shapes = shapefile::record::to_vec_of_polyline(shapes).unwrap();
    assert_eq!(shapes.len(), 1);
    use shapefile::NO_DATA;

    let shape = &shapes[0];
    assert_eq!(shape.bbox.xmin, 1.0);
    assert_eq!(shape.bbox.ymin, 1.0);
    assert_eq!(shape.bbox.xmax, 5.0);
    assert_eq!(shape.bbox.ymax, 6.0);
    assert_eq!(shape.parts, vec![0, 5]);
    assert_eq!(shape.xs, vec![1.0, 5.0, 5.0, 3.0, 1.0, 3.0, 2.0]);
    assert_eq!(shape.ys, vec![5.0, 5.0, 1.0, 3.0, 1.0, 2.0, 6.0]);
    assert!(shape.z.is_none());
    assert!(shape.m.is_some());
    if let Some(dim) = &shape.m {
        assert_eq!(dim.values, vec![0.0, NO_DATA, 3.0, NO_DATA, 0.0, NO_DATA, NO_DATA]);
    }
}

#[test]
fn read_iter() {
    let file = File::open("./tests/data/linem.shp").unwrap();
    let mut reader = shapefile::Reader::new(file).unwrap();

    for shape in reader {
        if let shapefile::record::Shape::Polyline(poly) = shape.unwrap() {
            println!("{}", poly.num_points);
        }
    }
}
