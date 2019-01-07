use record;
use header;
use {Error, ShapeType, Shape};


use std::fs::File;
use std::io::{BufReader, Read, SeekFrom, Seek};
use byteorder::{BigEndian, ReadBytesExt};
use record::ReadableShape;
use std::path::{PathBuf, Path};

const INDEX_RECORD_SIZE: usize = 2 * std::mem::size_of::<i32>();

pub struct ShapeIndex {
    pub offset: i32,
    pub record_size: i32,
}


fn read_index_file<T: Read>(mut source: T) -> Result<Vec<ShapeIndex>, Error> {
    let header = header::Header::read_from(&mut source)?;

    let num_shapes = ((header.file_length * 2) - header::HEADER_SIZE) / INDEX_RECORD_SIZE as i32;
    let mut shapes_index = Vec::<ShapeIndex>::with_capacity(num_shapes as usize);
    for _ in 0..num_shapes {
        let offset = source.read_i32::<BigEndian>()?;
        let record_size = source.read_i32::<BigEndian>()?;
        shapes_index.push(ShapeIndex { offset, record_size });
    }
    Ok(shapes_index)
}


pub struct Reader<T: Read> {
    source: T,
    header: header::Header,
    pos: usize,
    shapes_index: Vec<ShapeIndex>,
}


impl<T: Read> Reader<T> {
    pub fn new(mut source: T) -> Result<Reader<T>, Error> {
        let header = header::Header::read_from(&mut source)?;

        Ok(Reader { source, header, pos: header::HEADER_SIZE as usize, shapes_index: Vec::<ShapeIndex>::new() })
    }

    pub fn add_index_source(&mut self, source: T) -> Result<(), Error> {
        self.shapes_index = read_index_file(source)?;
        Ok(())
    }

    pub fn read(self) -> Result<Vec<Shape>, Error> {
        let mut shapes = Vec::<record::Shape>::new();
        for shape in self {
            shapes.push(shape?);
        }
        Ok(shapes)
    }

    pub fn read_as<S: ReadableShape>(mut self) -> Result<Vec<S::ActualShape>, Error> {
        let requested_shapetype = S::shapetype();
        if self.header.shape_type != requested_shapetype {
            let error = Error::MismatchShapeType {
                requested: requested_shapetype, actual: self.header.shape_type };
            return Err(error);
        }

        let mut shapes = Vec::<S::ActualShape>::new();
        while self.pos < (self.header.file_length * 2) as usize {
            let (record_size, shapetype) = self.read_record_size_and_shapetype()?;

            if shapetype != ShapeType::NullShape && shapetype != self.header.shape_type {
                return Err(Error::MixedShapeType);
            }

            if shapetype != ShapeType::NullShape && shapetype != requested_shapetype {
                let error = Error::MismatchShapeType {
                    requested: requested_shapetype, actual: shapetype };
                return Err(error);
            }

            self.pos += record_size as usize * 2;
            shapes.push(S::read_from(&mut self.source)?);
        }
        Ok(shapes)
    }

    pub fn header(&self) -> &header::Header {
        &self.header
    }

    fn read_record_size_and_shapetype(&mut self) -> Result<(i32, ShapeType), Error> {
        let hdr = record::RecordHeader::read_from(&mut self.source)?;
        self.pos += std::mem::size_of::<i32>() * 2;

        let shapetype = ShapeType::read_from(&mut self.source)?;

        Ok((hdr.record_size, shapetype))
    }
}

impl Reader<BufReader<File>> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path)?;
        let source = BufReader::new(file);
        Self::new(source)
    }
}


impl<T: Read + Seek> Reader<T> {
    pub fn read_nth_shape(&mut self, index: usize) -> Option<Result<Shape, Error>> {
        let offset =
            {
                let shape_idx = self.shapes_index.get(index)?;
                (shape_idx.offset * 2) as u64
            };

        match self.source.seek(SeekFrom::Start(offset)) {
            Err(e) => return Some(Err(Error::IoError(e))),
            Ok(_) => {}
        }
        self.into_iter().next()
    }
}


impl<T: Read> Iterator for Reader<T> {
    type Item = Result<Shape, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= (self.header.file_length * 2) as usize {
            return None;
        }

        let (record_size, shapetype) =  match self.read_record_size_and_shapetype() {
            Err(e) => return Some(Err(e)),
            Ok(t) => t
        };

        if shapetype != ShapeType::NullShape && shapetype != self.header.shape_type {
            println!("Mixing shape types, this is not allowed");
        }

        self.pos += record_size as usize * 2;
        Some(Shape::read_from(&mut self.source, shapetype))
    }
}

/*
#[allow(dead_code)]
enum SourceType<T: Read> {
    Path(PathBuf),
    Stream(T),
}

struct ReaderBuilder {
    shape: PathBuf,
    index_path: Option<PathBuf>,
}*/

pub struct FileReaderBuilder {
    shape_path: PathBuf,
    index_path: Option<PathBuf>,
}


impl FileReaderBuilder {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let shape_path = path.as_ref().to_path_buf();
        Self { shape_path, index_path: None }
    }

    pub fn with_index(mut self) -> Self {
        self.index_path = Some(self.shape_path.with_extension("shx"));
        self
    }

    pub fn build(self) -> Result<Reader<BufReader<File>>, Error> {
        let mut reader = Reader::from_path(self.shape_path)?;
        if let Some(p) = self.index_path {
            let index_source = BufReader::new(File::open(p)?);
            reader.add_index_source(index_source)?;
        }
        Ok(reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let reader = FileReaderBuilder::new("mdr.shp")
            .with_index()
            .build();
        assert_eq!(reader.is_err(), true);
    }
}