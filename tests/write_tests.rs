extern crate shapefile;

use std::io::Cursor;

#[test]
fn write_polyline() {
    let mut polys = Vec::<shapefile::record::Polyline>::with_capacity(1);
    polys.push(shapefile::record::Polyline::default());

    let v = Vec::<u8>::new();
    let cursor = Cursor::new(v);
    let mut writer = shapefile::writer::Writer::new(cursor);
    writer.write_shapes(polys).unwrap();
}