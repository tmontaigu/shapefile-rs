use crate::header;
use crate::{Error, ShapeType, Shape};
use crate::record;

use std::io::Read;

use std::iter::{FusedIterator, IntoIterator, Iterator};

pub struct Reader<T: Read> {
    source: T,
    header: header::Header,
    pos: usize,
}

impl<T: Read> Reader<T> {
    pub fn new(mut source: T) -> Result<Reader<T>, Error> {
        let header = header::Header::read_from(&mut source)?;
        Ok(Reader { source, header, pos: header::SHP_HEADER_SIZE as usize })
    }

    pub fn read(&mut self) -> Result<Vec<Shape>, Error> {
        let mut shapes = Vec::<record::Shape>::new();
        for shape in self {
            shapes.push(shape?);
        }
        Ok(shapes)
    }
}

fn read_shape<T: Read>(mut source: &mut T, shapetype: ShapeType) -> Result<Shape, Error> {
    let shape = match shapetype {
        ShapeType::Polyline |
        ShapeType::PolylineZ |
        ShapeType::PolylineM |
        ShapeType::Polygon |
        ShapeType::PolygonZ |
        ShapeType::PolygonM => Shape::Polyline(record::read_poly_line_record(&mut source, shapetype)?),

        ShapeType::Point |
        ShapeType::PointZ |
        ShapeType::PointM => Shape::Point(record::read_point_record(&mut source, shapetype)?),

        ShapeType::Multipoint |
        ShapeType::MultipointZ |
        ShapeType::MultipointM => Shape::Multipoint(record::read_multipoint_record(&mut source, shapetype)?),

        ShapeType::Multipatch => Shape::Multipatch(record::read_multipatch_record(&mut source, shapetype)?),

        ShapeType::NullShape => Shape::NullShape
    };
    Ok(shape)
}

impl<T: Read> Iterator for Reader<T> {
    type Item = Result<Shape, Error>;

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