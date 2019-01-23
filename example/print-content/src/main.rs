extern crate shapefile;
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> =  env::args().into_iter().collect();
    let filename = match args.get(1) {
        Some(arg) => arg,
        None => {
            println!("Expected a path to a file as first argument.");
            exit(-1);
        }
    };

    let reader = match shapefile::Reader::from_path(filename) {
        Ok(r) => r,
        Err(e) =>  {
            println!("Error: {}", e);
            exit(-1);
        }
    };

    for (i, shape) in reader.into_iter().enumerate() {
        let shape = match shape {
            Ok(s) => println!("{}", s),
            Err(e) => {
                println!("Error reading shape n°{}: {}", i, e);
                exit(-1);
            }
        };
    }
}
