use std::io::Write;

use Error;
use record::{EsriShape, RecordHeader};
use header;


pub struct Writer<T: Write> {
    pub dest: T,
}


impl<T: Write> Writer<T> {
    pub fn new(dest: T) -> Self {
        Self{dest}
    }

    //TODO This method should move (take mut self) as calling it twice would produce a shitty file
    pub fn write_shapes<S: EsriShape>(&mut self, shapes: Vec<S>) -> Result<(), Error> {
        let mut file_length = header::SHP_HEADER_SIZE as usize;
        for shape in &shapes {
            file_length += shape.size_in_bytes();
        }
        file_length /= 2; // file size is in 16bit words

        if file_length > i32::max_value() as usize {
            panic!("To big"); //TODO convert in proper error
        }

        let file_length= file_length as i32;
        let shapetype = *&shapes[0].shapetype();
        let header = header::Header {
            file_length,
            point_min: [0.0, 0.0, 0.0],
            point_max: [0.0, 0.0, 0.0],
            m_range: [0.0, 0.0],
            shape_type:shapetype,
            version: 1000,
        };

        header.write_to(&mut self.dest)?;

        for (i, shape) in shapes.into_iter().enumerate() {

            //TODO Check record size < i32_max ?
            let record_size = ((shape.size_in_bytes() + std::mem::size_of::<i32>()) / 2);
            let rc_hdr = RecordHeader{
                record_number: i as i32,
                record_size: record_size as i32,
            };
            rc_hdr.write_to(&mut self.dest)?;
            shape.shapetype().write_to(&mut self.dest)?;
            shape.write_to(&mut self.dest)?;
        }
        Ok(())
    }
}