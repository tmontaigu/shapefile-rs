use std::io::Write;

use Error;
use record::{EsriShape, RecordHeader};
use header;


pub struct Writer<T: Write> {
    pub dest: T,
}

fn f64_min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    }
    else {
        b
    }
}

fn f64_max(a: f64, b: f64) -> f64 {
    if a > b {
        a
    }
    else {
        b
    }
}

impl<T: Write> Writer<T> {
    pub fn new(dest: T) -> Self {
        Self{dest}
    }

    //TODO This method should move (take mut self) as calling it twice would produce a shitty file
    pub fn write_shapes<S: EsriShape>(&mut self, shapes: Vec<S>) -> Result<(), Error> {
        let mut file_length = header::SHP_HEADER_SIZE as usize;
        for shape in &shapes {
            file_length += 2 * std::mem::size_of::<i32>(); // record_header
            file_length += std::mem::size_of::<i32>(); // shape_type
            file_length += shape.size_in_bytes();
        }
        file_length /= 2; // file size is in 16bit words

        if file_length > i32::max_value() as usize {
            panic!("To big"); //TODO convert in proper error
        }

        let mut point_min= [std::f64::MAX, std::f64::MAX, std::f64::MAX];
        let mut point_max= [std::f64::MIN, std::f64::MIN, std::f64::MIN];
        let mut m_range = [0.0, 0.0];
        for shape in &shapes {
            let bbox = shape.bbox();
            let z_range = shape.z_range();
            point_min[0] = f64_min(point_min[0], bbox.xmin);
            point_min[1] = f64_min(point_min[1], bbox.ymin);
            point_min[2] = f64_min(point_min[2], z_range[0]);

            point_max[0] = f64_max(point_max[0], bbox.xmax);
            point_max[1] = f64_max(point_max[1], bbox.ymax);
            point_max[2] = f64_max(point_max[2], z_range[1]);

            let s_m_range = shape.m_range();
            m_range[0] = f64_min(m_range[0], s_m_range[0]);
            m_range[1] = f64_max(m_range[1], s_m_range[1]);
        }

        if point_min[2] == std::f64::MAX {
            point_min[2] = 0.0;
        }

        if point_max[2] == std::f64::MIN {
            point_max[2] = 0.0;
        }

        let file_length= file_length as i32;
        let shapetype = *&shapes[0].shapetype();
        let header = header::Header {
            file_length,
            point_min,
            point_max,
            m_range,
            shape_type:shapetype,
            version: 1000,
        };

        header.write_to(&mut self.dest)?;

        for (i, shape) in shapes.into_iter().enumerate() {

            //TODO Check record size < i32_max ?
            let record_size = (shape.size_in_bytes() + std::mem::size_of::<i32>()) / 2;
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