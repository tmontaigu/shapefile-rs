#![no_main]

use libfuzzer_sys::fuzz_target;
use shapefile::ShapeReader;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let _ = ShapeReader::new(Cursor::new(data));
});
