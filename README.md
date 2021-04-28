# shapefile-rs
Rust library to read & write shapefiles
.dbf files supported via the [dbase](https://crates.io/crates/dbase) crate

```rust
let mut reader = shapefile::Reader::from_path(filename).unwrap();

for result in reader.iter_shapes_and_records() {
    let (shape, record) = result.unwrap();
    println ! ("Shape: {}, records: ", shape);
    for (name, value) in record {
        println ! ("\t{}: {:?}, ", name, value);
    }
    println ! ();
}
```

