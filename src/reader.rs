use crate::header;
use crate::{ShpError, ShapeType};
use crate::record;

use std::io::{Read};

pub struct Reader<T: Read> {
   source: T ,
   header: header::Header,
}

impl<T: Read> Reader<T> {
   pub fn new(mut source: T) -> Result<Reader<T>, ShpError> {
      let header = header::Header::read_from(&mut source)?;
      Ok(Reader{source, header})
   }

   pub fn read(&mut self) -> Result<Vec<record::Shape>, ShpError> {
      let mut current_pos = header::SHP_HEADER_SIZE;
      println!("File Size: {}", self.header.file_length * 2);

      let mut shapes= Vec::<record::Shape>::new();
      while current_pos < self.header.file_length * 2 {
         println!("curr pos: {}", current_pos);
         let hdr = record::RecordHeader::read_from(&mut self.source)?;
         let shapetype = ShapeType::read_from(&mut self.source)?;

         if shapetype != ShapeType::NullShape && shapetype != self.header.shape_type {
            println!("Mixing shape types, this is not allowed");
         }

         let shape = match shapetype {
            ShapeType::Polyline |
            ShapeType::PolylineZ |
            ShapeType::PolylineM |
            ShapeType::Polygon |
            ShapeType::PolygonZ |
            ShapeType::PolygonM => record::Shape::Polyline(record::read_poly_line_record(&mut self.source, shapetype)?),

            ShapeType::Point |
            ShapeType::PointZ |
            ShapeType::PointM  => record::Shape::Point(record::read_point_record(&mut self.source, shapetype)?),

            ShapeType::Multipoint |
            ShapeType::MultipointZ |
            ShapeType::MultipointM  => record::Shape::Multipoint(record::read_multipoint_record(&mut self.source, shapetype)?),

            ShapeType::Multipatch => record::Shape::Multipatch(record::read_multipatch_record(&mut self.source, shapetype)?),

            ShapeType::NullShape => record::Shape::NullShape
         };

         if shapetype != ShapeType::NullShape {
            shapes.push(shape);
         }

         let pos_diff = (hdr.record_size + std::mem::size_of::<i32>() as i32) * 2;
         println!("Record: {}, {}, {} -> {}", hdr.record_number, hdr.record_size, shapetype, pos_diff);

         current_pos += pos_diff;
      }
      Ok(shapes)
   }

}
