extern crate shapefile;

mod testfiles;

#[test]
fn test_line_read_nth() {
    let mut reader = shapefile::ShapeReader::from_path(testfiles::LINE_PATH).unwrap();

    if let Some(shape) = reader.read_nth_shape(0) {
        let shp = shape.unwrap();
        testfiles::check_line_first_shape(&shp);
    } else {
        assert!(false, "Should be Some(shape)")
    }

    assert_eq!(reader.read_nth_shape(1).is_none(), true);
}

#[test]
fn test_size_hint() {
    let mut reader = shapefile::ShapeReader::from_path(testfiles::LINE_PATH).unwrap();
    let mut iter = reader.iter_shapes();
    assert_eq!(iter.size_hint(), (1, Some(1)));
    iter.next();

    // size_hint must return the remaining length and not the entire length at the beginning.
    assert_eq!(iter.size_hint(), (0, Some(0)));
}
