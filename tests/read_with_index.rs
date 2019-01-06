extern crate shapefile;

mod read_tests;


#[test]
fn test_line() {
    let mut reader = shapefile::reader::FileReaderBuilder::new(read_tests::LINE_PATH)
        .with_index()
        .build()
        .unwrap();

    if let Some(shape) = reader.read_nth_shape(0) {
        let shp = shape.unwrap();
        read_tests::check_line_first_shape(&shp);
    }
    else {
        assert!(false, "Should be Some(shape)")
    }

    assert_eq!(reader.read_nth_shape(1).is_none(), true);
}

