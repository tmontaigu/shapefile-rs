//! Reader module, contains the definitions of the types that a user should use to read a file
//!
//! This module proposes two type: a [Reader](struct.Reader.html) and a [FileReaderBuilder](struct.FileReaderBuilder.html)
//!
//! The Reader is the struct that actually reads the file from any source as long as it implements the
//! `Read` Trait (`std::fs::File`, and `std::io::Cursor` for example).
//!
//! Note that by default the Reader does not read the index file and so the methods
//! [read_nth_shape_as](struct.Reader.html#method.read_nth_shape_as) and [read_nth_shape](struct.Reader.html#method.read_nth_shape) will _not_ work.
//! If you wish to use them you will have to give to the reader a source for the index file via [add_index_source](struct.Reader.html#method.add_index_source)
//!
//! Or use the [FileReaderBuilder](struct.FileReaderBuilder.html) if you are reading from files (not buffers)
//!
//! # Examples
//!
//! When reading from a file:
//!
//! Creates a reader from a path, then iterate over its `Shapes`, reading one shape each iteration
//! ```
//! let reader = shapefile::Reader::from_path("tests/data/pointm.shp").unwrap();
//! for shape in reader {
//!     let shape = shape.unwrap();
//!     println!("{}", shape);
//! }
//! ```
//!
//! Creates a reader from a path, reads the whole file at once
//! ```
//! let reader = shapefile::Reader::from_path("tests/data/pointm.shp").unwrap();
//! let shapes = reader.read().unwrap();
//! ```
//!
//! ```
//! let mut reader = shapefile::FileReaderBuilder::new("tests/data/line.shp").with_index().build().unwrap();
//! ```
//!
//! If you know beforehand the exact type that the .shp file is made of,
//! you can use the different `*_as::<S>()` functions.:
//! - [read_as](struct.Reader.html#method.read_as) To read all the shapes as the specified type
//! - [iter_shapes_as](struct.Reader.html#method.iter_shapes_as) To iterate over the shapes as shapes
//! of the specified type
//!
//!
//! Two functions ([read](fn.read.html) and [read_as](fn.read_as.html)) are provided to read
//! files with one function call (thus not having to build a `Reader`)

use header;
use record;
use {Error, Shape};

use byteorder::{BigEndian, ReadBytesExt};
use record::ReadableShape;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::iter::FusedIterator;
use std::path::{Path, PathBuf};

const INDEX_RECORD_SIZE: usize = 2 * std::mem::size_of::<i32>();

pub(crate) struct ShapeIndex {
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
        shapes_index.push(ShapeIndex {
            offset,
            record_size,
        });
    }
    Ok(shapes_index)
}

/// Reads and returns one shape and its header from the source
fn read_one_shape_as<T: Read, S: ReadableShape>(
    mut source: &mut T,
) -> Result<(record::RecordHeader, S::ReadShape), Error> {
    let hdr = record::RecordHeader::read_from(&mut source)?;
    let record_size = hdr.record_size * 2;
    let shape = S::read_from(&mut source, record_size)?;
    Ok((hdr, shape))
}


/// Struct that handle iteration over the shapes of a .shp file
pub struct ShapeIterator<T: Read, S: ReadableShape> {
    _shape: std::marker::PhantomData<S>,
    source: T,
    current_pos: usize,
    file_length: usize,
}

impl<T: Read, S: ReadableShape> Iterator for ShapeIterator<T, S> {
    type Item = Result<S::ReadShape, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos >= self.file_length {
            None
        } else {
            let (hdr, shape) = match read_one_shape_as::<T, S>(&mut self.source) {
                Err(e) => return Some(Err(e)),
                Ok(hdr_and_shape) => hdr_and_shape,
            };
            self.current_pos += record::RecordHeader::SIZE;
            self.current_pos += hdr.record_size as usize * 2;
            Some(Ok(shape))
        }
    }
}

impl<T: Read, S: ReadableShape> FusedIterator for ShapeIterator<T, S> {}


/// struct that reads the content of a shapefile
pub struct Reader<T: Read> {
    source: T,
    header: header::Header,
    shapes_index: Vec<ShapeIndex>,
}

impl<T: Read> Reader<T> {
    /// Creates a new Reader from a source that implements the `Read` trait
    ///
    /// The Shapefile header is read upon creation (but no reading of the Shapes is done)
    ///
    /// # Errors
    ///
    /// Will forward any `std::io::Error`
    ///
    /// Will also return an error if the data is not a shapefile (Wrong file code)
    ///
    /// Will also return an error if the shapetype read from the input source is invalid
    pub fn new(mut source: T) -> Result<Reader<T>, Error> {
        let header = header::Header::read_from(&mut source)?;

        Ok(Reader {
            source,
            header,
            shapes_index: Vec::<ShapeIndex>::new(),
        })
    }

    /// Returns a non-mutable reference to the header read
    ///
    /// # Examples
    ///
    /// ```
    /// let reader = shapefile::Reader::from_path("tests/data/pointz.shp").unwrap();
    /// let header = reader.header();
    /// assert_eq!(header.shape_type, shapefile::ShapeType::PointZ);
    /// ```
    pub fn header(&self) -> &header::Header {
        &self.header
    }



    /// Reads all the shape as shape of a certain type.
    ///
    /// To be used if you know in advance which shape type the file contains.
    ///
    /// # Errors
    /// The function has an additional error that is returned if  the shape type you asked to be
    /// read does not match the actual shape type in the file.
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::Reader;
    /// let mut reader = Reader::from_path("tests/data/linem.shp").unwrap();
    /// let polylines_m = reader.read_as::<shapefile::PolylineM>().unwrap(); // we ask for the correct type
    /// ```
    ///
    /// ```
    /// use shapefile::Reader;
    /// let mut reader = Reader::from_path("tests/data/linem.shp").unwrap();
    /// let polylines = reader.read_as::<shapefile::Polyline>(); // we ask for the wrong type
    /// assert_eq!(polylines.is_err(), true);
    /// ```
    pub fn read_as<S: ReadableShape>(self) -> Result<Vec<S::ReadShape>, Error> {
        self.iter_shapes_as::<S>().collect()
    }

    /// Reads all the shapes and returns them
    ///
    /// # Examples
    /// ```
    /// use shapefile::Reader;
    /// let mut reader = Reader::from_path("tests/data/multipoint.shp").unwrap();
    /// let shapes = reader.read().unwrap();
    /// for shape in shapes {
    ///     match shape {
    ///         shapefile::Shape::Multipoint(pts) => println!(" Yay Multipoints: {}", pts),
    ///         _ => panic!("ups, not multipoints"),
    ///     }
    /// }
    /// ```
    ///
    pub fn read(self) -> Result<Vec<Shape>, Error> {
        self.into_iter().collect()
    }


    /// Returns an iterator that tries to read the shapes as the specified type
    /// Will return an error of the type `S` does not match the actual type in the file
    ///
    /// # Examples
    ///
    /// ```
    /// let reader = shapefile::Reader::from_path("tests/data/multipoint.shp").unwrap();
    /// for multipoints in reader.iter_shapes_as::<shapefile::Multipoint>() {
    ///     let points = multipoints.unwrap();
    ///     println!("{}", points);
    /// }
    /// ```
    pub fn iter_shapes_as<S: ReadableShape>(self) -> ShapeIterator<T, S> {
        ShapeIterator {
            _shape: std::marker::PhantomData,
            source: self.source,
            current_pos: header::HEADER_SIZE as usize,
            file_length: (self.header.file_length * 2) as usize,
        }
    }

    /// Returns an iterator that to reads the shapes wraps them in the enum [Shape](enum.Shape.html)
    /// You do not need to call this method and can iterate over the `Reader` directly
    ///
    /// # Examples
    ///
    /// ```
    /// let reader = shapefile::Reader::from_path("tests/data/multipoint.shp").unwrap();
    /// for shape in reader.iter_shapes() {
    ///     match shape.unwrap() {
    ///         shapefile::Shape::Multipatch(shp) => println!("Multipoint!"),
    ///         _ => println!("Other type of shape"),
    ///     }
    /// }
    /// ```
    ///
    /// ```
    /// let reader = shapefile::Reader::from_path("tests/data/multipoint.shp").unwrap();
    /// for shape in reader {
    ///     match shape.unwrap() {
    ///         shapefile::Shape::Multipatch(shp) => println!("Multipoint!"),
    ///         _ => println!("Other type of shape"),
    ///     }
    /// }
    /// ```
    pub fn iter_shapes(self) -> ShapeIterator<T, Shape> {
        ShapeIterator {
            _shape: std::marker::PhantomData,
            source: self.source,
            current_pos: header::HEADER_SIZE as usize,
            file_length: (self.header.file_length * 2) as usize,
        }
    }

    /// Reads the index file from the source
    /// This allows to later read shapes by giving their index without reading the whole file
    ///
    /// (see `Reader::read_nth_shape()`)
    pub fn add_index_source(&mut self, source: T) -> Result<(), Error> {
        self.shapes_index = read_index_file(source)?;
        Ok(())
    }
}

impl<T: Read> IntoIterator for Reader<T> {
    type Item = Result<Shape, Error>;
    type IntoIter = ShapeIterator<T, Shape>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_shapes()
    }
}

impl Reader<BufReader<File>> {
    /// Creates a reader from a path to a file
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::Reader;
    /// let mut reader = Reader::from_path("tests/data/line.shp").unwrap();
    /// let polylines = reader.read_as::<shapefile::Polyline>().unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path)?;
        let source = BufReader::new(file);
        Self::new(source)
    }
}

impl<T: Read + Seek> Reader<T> {
    /// Reads the `n`th shape of the shapefile
    ///
    /// # Returns
    ///
    /// `None` if the index is out of range or if no index file was added prior to
    /// calling this function
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::FileReaderBuilder;
    /// let mut reader = FileReaderBuilder::new("tests/data/line.shp").with_index().build().unwrap();
    /// let shape = reader.read_nth_shape_as::<shapefile::Polyline>(117);
    /// assert_eq!(shape.is_none(), true);
    ///
    /// let shape = reader.read_nth_shape_as::<shapefile::Polyline>(0);
    /// assert_eq!(shape.is_some(), true)
    /// ```
    pub fn read_nth_shape_as<S: ReadableShape>(
        &mut self,
        index: usize,
    ) -> Option<Result<S::ReadShape, Error>> {
        let offset = {
            let shape_idx = self.shapes_index.get(index)?;
            (shape_idx.offset * 2) as u64
        };

        if let Err(e) = self.source.seek(SeekFrom::Start(offset)) {
            return Some(Err(Error::IoError(e)));
        }

        let (_, shape) = match read_one_shape_as::<T, S>(&mut self.source) {
            Err(e) => return Some(Err(e)),
            Ok(hdr_and_shape) => hdr_and_shape,
        };

        if let Err(e) = self
            .source
            .seek(SeekFrom::Start(header::HEADER_SIZE as u64))
        {
            return Some(Err(Error::IoError(e)));
        }
        Some(Ok(shape))
    }

    /// Reads the `n`th shape of the shapefile
    ///
    /// # Returns
    ///
    /// `None` if the index is out of range or if no index file was added prior to
    /// calling this function
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::FileReaderBuilder;
    /// let mut reader = FileReaderBuilder::new("tests/data/line.shp").with_index().build().unwrap();
    /// let shape = reader.read_nth_shape(117);
    /// assert_eq!(shape.is_none(), true);
    ///
    /// let shape = reader.read_nth_shape(0);
    /// assert_eq!(shape.is_some(), true)
    /// ```
    ///
    /// ```
    /// use shapefile::Reader;
    /// let mut reader = Reader::from_path("tests/data/line.shp").unwrap();
    /// let shape = reader.read_nth_shape(0);
    /// assert_eq!(shape.is_none(), true); // We didn't give the shx file
    /// ```
    pub fn read_nth_shape(&mut self, index: usize) -> Option<Result<Shape, Error>> {
        self.read_nth_shape_as::<Shape>(index)
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
    /// Creates a new FileReaderBuilder, with path being the path to the .shp file
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let shape_path = path.as_ref().to_path_buf();
        Self {
            shape_path,
            index_path: None,
        }
    }

    /// Tells the builder to also open the .shx corresponding to the .shp given when
    /// creating the FileReaderBuilder
    pub fn with_index(mut self) -> Self {
        self.index_path = Some(self.shape_path.with_extension("shx"));
        self
    }

    /// Creates the Reader
    ///
    /// Forwards error that may happen when opening the files
    pub fn build(self) -> Result<Reader<BufReader<File>>, Error> {
        let mut reader = Reader::from_path(self.shape_path)?;
        if let Some(p) = self.index_path {
            let index_source = BufReader::new(File::open(p)?);
            reader.add_index_source(index_source)?;
        }
        Ok(reader)
    }
}

/// Function to read all the Shapes in a file.
///
/// Returns a `Vec<Shape>` which means that you will have to `match`
/// the individual [Shape](enum.Shape.html) contained inside the `Vec` to get the actual `struct`
///
/// Useful if you don't know in advance (at compile time) which kind of
/// shape the file contains
///
/// # Examples
///
/// ```
/// let shapes = shapefile::read("tests/data/multipatch.shp").unwrap();
/// assert_eq!(shapes.len(), 1);
/// ```
pub fn read<T: AsRef<Path>>(path: T) -> Result<Vec<Shape>, Error> {
    let reader = Reader::from_path(path)?;
    reader.read()
}

/// Function to read all the Shapes in a file as a certain type
///
/// Fails and return `Err(Error:MismatchShapeType)`
///
///  # Examples
///
/// ```
/// let polylines = shapefile::read_as::<&str, shapefile::PolylineZ>("tests/data/polygon.shp");
/// assert_eq!(polylines.is_err(), true);
///
/// let polygons = shapefile::read_as::<&str, shapefile::Polygon>("tests/data/polygon.shp");
/// assert_eq!(polygons.is_ok(), true);
/// ```
///
/// If the reading is successful, the returned `Vec<S:ReadShape>>`is a vector of actual structs
/// Useful if you know in at compile time which kind of shape you expect the file to have
pub fn read_as<T: AsRef<Path>, S: ReadableShape>(path: T) -> Result<Vec<S::ReadShape>, Error> {
    let reader = Reader::from_path(path)?;
    reader.read_as::<S>()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let reader = FileReaderBuilder::new("mdr.shp").with_index().build();
        assert_eq!(reader.is_err(), true);
    }
}
