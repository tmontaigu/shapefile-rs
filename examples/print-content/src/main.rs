extern crate shapefile;
use std::env;
use std::process::exit;
use std::mem::size_of;

fn main() {
    let args: Vec<String> =  env::args().into_iter().collect();
    let filename = match args.get(1) {
        Some(arg) => arg,
        None => {
            println!("Expected a path to a file as first argument.");
            exit(-1);
        }
    };

    let reader = shapefile::Reader::from_path(filename).unwrap();
    for (i, shape) in reader.into_iter().enumerate() {
        let shape = shape.unwrap();
    }

    let reader = shapefile::Reader::from_path(filename).unwrap();

    for result in reader.iter_shapes_and_records().unwrap() {
        let (shape, record) = result.unwrap();
        println!("Shape: {}, records: ", shape);
        for (name, value) in record {
            println!("\t{}: {:?}, ", name, value);
        }
        println!();
    }
}
