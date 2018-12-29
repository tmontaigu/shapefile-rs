extern crate shapefile;

use std::fs::File;

#[test]
fn read_line_header() {
    let file = File::open("./tests/data/line.shp").unwrap();
    let header = shapefile::header::Header::read_from(file).unwrap();

    assert_eq!(header.shape_type, shapefile::ShapeType::Polyline);
}