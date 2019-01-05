extern crate shapefile;

mod read_tests;

use shapefile::Reader;


#[test]
fn test_line() {
    let mut reader = Reader::from_path(read_tests::LINE_PATH).unwrap();

    if let Some(shape) = reader.read_nth_shape(0) {
        let shp = shape.unwrap();
        read_tests::check_line_first_shape(&shp);
    }
    else {
        assert!(false, "Should be Some(shape)")
    }

    assert_eq!(reader.read_nth_shape(1).is_none(), true);
}

