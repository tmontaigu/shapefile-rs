use crate::header;
use crate::{ShpError, ShapeType};
use crate::record;

use std::io::Read;

use std::iter::{FusedIterator, IntoIterator, Iterator};

pub struct Reader<T: Read> {
    source: T,
    header: header::Header,
    pos: usize,
}

impl<T: Read> Reader<T> {
    pub fn new(mut source: T) -> Result<Reader<T>, ShpError> {
        let header = header::Header::read_from(&mut source)?;
        Ok(Reader { source, header, pos: header::SHP_HEADER_SIZE as usize })
    }

    pub fn read(&mut self) -> Result<Vec<record::Shape>, ShpError> {
        let mut shapes = Vec::<record::Shape>::new();
        for shape in self {
            shapes.push(shape?);
        }
        Ok(shapes)
    }
}

fn read_shape<T: Read>(mut source: &mut T, shapetype: ShapeType) -> Result<record::Shape, ShpError> {
    let shape = match shapetype {
        ShapeType::Polyline |
        ShapeType::PolylineZ |
        ShapeType::PolylineM |
        ShapeType::Polygon |
        ShapeType::PolygonZ |
        ShapeType::PolygonM => record::Shape::Polyline(record::read_poly_line_record(&mut source, shapetype)?),

        ShapeType::Point |
        ShapeType::PointZ |
        ShapeType::PointM => record::Shape::Point(record::read_point_record(&mut source, shapetype)?),

        ShapeType::Multipoint |
        ShapeType::MultipointZ |
        ShapeType::MultipointM => record::Shape::Multipoint(record::read_multipoint_record(&mut source, shapetype)?),

        ShapeType::Multipatch => record::Shape::Multipatch(record::read_multipatch_record(&mut source, shapetype)?),

        ShapeType::NullShape => record::Shape::NullShape
    };
    Ok(shape)
}

impl<T: Read> Iterator for Reader<T> {
    type Item = Result<record::Shape, ShpError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= (self.header.file_length * 2) as usize {
            return None;
        }


        let hdr = match record::RecordHeader::read_from(&mut self.source) {
            Ok(hdr) => hdr,
            Err(e) => return Some(Err(e)),
        };
        self.pos += (std::mem::size_of::<i32>() * 2);

        let shapetype = match ShapeType::read_from(&mut self.source) {
            Ok(shapetype) => shapetype,
            Err(e) => return Some(Err(e)),
        };

        if shapetype != ShapeType::NullShape && shapetype != self.header.shape_type {
            println!("Mixing shape types, this is not allowed");
        }

        let pos_diff = (hdr.record_size as usize + std::mem::size_of::<i32>()) * 2;
        self.pos += pos_diff;

        Some(read_shape(&mut self.source, shapetype))
    }
}