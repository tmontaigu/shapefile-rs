use dbase::{dbase_record, TableWriterBuilder};
use shapefile::{dbase, Point, Polyline, Writer};

fn example_1() {
    const FILE_NAME: &'static str = "hello_shape_1.shp";

    let square = Polyline::new(vec![
        Point::new(0.0, 0.0),
        Point::new(0.0, 1.0),
        Point::new(1.0, 1.0),
        Point::new(1.0, 0.0),
    ]);

    let bigger_square = Polyline::new(vec![
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
        Point::new(10.0, 10.0),
        Point::new(10.0, 0.0),
    ]);

    // Create the builder for the accompanying dbase (.dbf) file
    // For this simple example we will only have a single field
    // "Name", with 55 chars max
    let table_builder =
        TableWriterBuilder::new().add_character_field("Name".try_into().unwrap(), 55);

    {
        let mut writer = Writer::from_path(FILE_NAME, table_builder)
            .expect("Failed to create the shapefile writer");

        let mut first_record = dbase::Record::default();
        first_record.insert("Name".to_string(), "Square".to_string().into());
        writer
            .write_shape_and_record(&square, &first_record)
            .expect("Failed to write first record");

        let mut second_record = dbase::Record::default();
        second_record.insert("Name".to_string(), "Big Square".to_string().into());
        writer
            .write_shape_and_record(&bigger_square, &second_record)
            .expect("Failed to write second record");
    }

    println!("File created, you can use `cargo run --example print-content {FILE_NAME}`");
}

fn example_2() {
    dbase_record!(
        #[derive(Debug)]
        struct UserRecord {
            first_name: String,
            last_name: String,
        }
    );

    const FILE_NAME: &'static str = "hello_shape_2.shp";

    let square = Polyline::new(vec![
        Point::new(0.0, 0.0),
        Point::new(0.0, 1.0),
        Point::new(1.0, 1.0),
        Point::new(1.0, 0.0),
    ]);

    let bigger_square = Polyline::new(vec![
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
        Point::new(10.0, 10.0),
        Point::new(10.0, 0.0),
    ]);

    // Create the builder for the accompanying dbase (.dbf) file
    let table_builder = TableWriterBuilder::new()
        .add_character_field("FirstName".try_into().unwrap(), 55)
        .add_character_field("LastName".try_into().unwrap(), 55);

    {
        let mut writer = Writer::from_path(FILE_NAME, table_builder)
            .expect("Failed to create the shapefile writer");

        let first_record = UserRecord {
            first_name: "Yoshi".to_string(),
            last_name: "Green".to_string(),
        };

        writer
            .write_shape_and_record(&square, &first_record)
            .expect("Failed to write first record");

        let second_record = UserRecord {
            first_name: "Yoshi".to_string(),
            last_name: "Red".to_string(),
        };
        writer
            .write_shape_and_record(&bigger_square, &second_record)
            .expect("Failed to write second record");
    }

    println!("File created, you can use `cargo run --example print-content {FILE_NAME}`");
}

fn main() {
    example_1();
    example_2();
}
