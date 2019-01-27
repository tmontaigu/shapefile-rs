extern crate shapefile;

mod testfiles;

#[test]
fn test_line() {
    let mut reader = shapefile::Reader::from_path(testfiles::LINE_PATH).unwrap();

    if let Some(shape) = reader.read_nth_shape(0) {
        let shp = shape.unwrap();
        testfiles::check_line_first_shape(&shp);
    } else {
        assert!(false, "Should be Some(shape)")
    }

    assert_eq!(reader.read_nth_shape(1).is_none(), true);
}
