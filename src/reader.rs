use record;
use header;
use {Error, ShapeType, Shape};


use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read, SeekFrom, Seek};

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

    pub fn read(self) -> Result<Vec<Shape>, Error> {
        let mut shapes = Vec::<record::Shape>::new();
        for shape in self {
            shapes.push(shape?);
        }
        Ok(shapes)
    }
}

impl Reader<BufReader<File>> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path)?;
        let source = BufReader::new(file);

        Self::new( source)
    }
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
        self.pos += std::mem::size_of::<i32>() * 2;

        let shapetype = match ShapeType::read_from(&mut self.source) {
            Ok(shapetype) => shapetype,
            Err(e) => return Some(Err(e)),
        };

        if shapetype != ShapeType::NullShape && shapetype != self.header.shape_type {
            println!("Mixing shape types, this is not allowed");
        }

        let pos_diff = (hdr.record_size as usize + std::mem::size_of::<i32>()) * 2;
        self.pos += pos_diff;

        Some(Shape::read_from(&mut self.source, shapetype))
    }
}


