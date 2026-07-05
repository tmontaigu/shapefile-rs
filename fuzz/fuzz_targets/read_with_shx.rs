#![no_main]

use libfuzzer_sys::fuzz_target;
use shapefile::ShapeReader;
use std::io::Cursor;

fuzz_target!(|data: (&[u8], &[u8])| {
    let (shp, shx) = data;
    if let Ok(reader) = ShapeReader::with_shx(Cursor::new(shp), Cursor::new(shx)) {
        let _ = reader.read();
    }
});
