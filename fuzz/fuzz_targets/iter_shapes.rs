#![no_main]

use libfuzzer_sys::fuzz_target;
use shapefile::{ShapeReader, Shape};
use std::io::Cursor;

// `read()` is `iter_shapes_as().collect()`, which short-circuits on the first
// error, so the other targets never keep pulling from a failing iterator. This
// one drives the iterator the way the docs do, which is what exposes an
// iterator that does not terminate.
fuzz_target!(|data: &[u8]| {
    if let Ok(mut reader) = ShapeReader::new(Cursor::new(data)) {
        for shape in reader.iter_shapes_as::<Shape>() {
            let _ = shape;
        }
    }
});
