//! Reader module, contains the definitions of the types that a user should use to read a file
//!
//!
//! The [Reader](struct.Reader.html) is the struct that actually reads the file from any source as long as it implements the
//! `Read` Trait (`std::fs::File`, and `std::io::Cursor` for example).
//!
//! It is recommended to create a `Reader` using its
//! [from_path](struct.Reader.html#method.from_path) method as this constructor
//! will take care of opening the .shx and .dbf files corresponding to the .shp (if they exists).
//!
//! If you want to read a shapefile that is not storred in a file (e.g the shp data is in a buffer),
//! you will have to construct the `Reader` "by hand" with its [new](struct.Reader.html#method.new) method.
//!
//! If you want the "manually" constructed `Reader` to also read the *shx* and *dbf* file content
//! you will have to use [add_index_source](struct.Reader.html#method.add_index_source) and/or
//! [add_dbf_source](struct.Reader.html#method.add_dbf_source)
//!
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
//! If you know beforehand the exact type that the .shp file is made of,
//! you can use the different `*_as::<S>()` functions.:
//! - [read_as](struct.Reader.html#method.read_as) To read all the shapes as the specified type
//! - [iter_shapes_as](struct.Reader.html#method.iter_shapes_as) To iterate over the shapes as shapes
//! of the specified type
//! - [iter_shapes_and_records_as](struct.Reader.html#method.iter_shapes_and_records_as) To iterate
//! over both the shapes and records
//!
//! Otherwise use the functions that return [Shape](../record/enum.Shape.html)s and do a `match`
//!
//! - [read](struct.Reader.html#method.read)
//! - [iter_shapes](struct.Reader.html#method.iter_shapes)
//! - [iter_shapes_and_records](struct.Reader.html#method.iter_shapes_and_records)
//!
//!
//! Two functions ([read](fn.read.html) and [read_as](fn.read_as.html)) are provided to read
//! files with one function call (thus not having to build a `Reader`)

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::iter::FusedIterator;
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};

use header;
use record;
use {Error, Shape};

use record::ReadableShape;

const INDEX_RECORD_SIZE: usize = 2 * std::mem::size_of::<i32>();

pub(crate) struct ShapeIndex {
    pub offset: i32,
    pub record_size: i32,
}

/// Read the content of a .shx file
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
) -> Result<(record::RecordHeader, S), Error> {
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
    type Item = Result<S, Error>;

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

pub struct ShapeRecordIterator<T: Read, S: ReadableShape> {
    shape_iter: ShapeIterator<T, S>,
    dbf_reader: dbase::Reader<T>,
}

impl<T: Read, S: ReadableShape> Iterator for ShapeRecordIterator<T, S> {
    type Item = Result<(S, dbase::Record), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let shape = match self.shape_iter.next()? {
            Err(e) => return Some(Err(e)),
            Ok(shp) => shp,
        };

        let record = match self.dbf_reader.next()? {
            Err(e) => return Some(Err(Error::DbaseError(e))),
            Ok(rcd) => rcd,
        };

        Some(Ok((shape, record)))
    }
}

impl<T: Read, S: ReadableShape> FusedIterator for ShapeRecordIterator<T, S> {}

//TODO Make it possible for the dbf source to be of a different dtype ?
/// struct that reads the content of a shapefile
pub struct Reader<T: Read> {
    source: T,
    header: header::Header,
    shapes_index: Option<Vec<ShapeIndex>>,
    dbf_reader: Option<dbase::Reader<T>>,
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
    ///
    /// # Example
    ///
    /// ```
    /// use std::fs::File;
    /// let file = File::open("tests/data/line.shp").unwrap();
    /// let reader = shapefile::Reader::new(file).unwrap();
    /// ```
    pub fn new(mut source: T) -> Result<Reader<T>, Error> {
        let header = header::Header::read_from(&mut source)?;

        Ok(Reader {
            source,
            header,
            shapes_index: None,
            dbf_reader: None,
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
    pub fn read_as<S: ReadableShape>(self) -> Result<Vec<S>, Error> {
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

    /// Read and return _only_ the records contained in the *.dbf* file
    pub fn read_records(self) -> Result<Vec<dbase::Record>, Error> {
        let dbf_reader = self.dbf_reader.ok_or(Error::MissingDbf)?;
        dbf_reader.read().or_else(|e| Err(Error::DbaseError(e)))
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

    /// Returns an iterator over the Shapes and their Records
    ///
    /// # Errors
    ///
    /// The `Result` will be an error if the .dbf wasn't found
    ///
    /// # Example
    /// ```
    /// use shapefile::{Reader, Multipatch};
    /// let reader = Reader::from_path("tests/data/multipatch.shp").unwrap();
    /// for result in reader.iter_shapes_and_records_as::<Multipatch>().unwrap() {
    ///     let (shape, record) = result.unwrap();
    ///     // ...
    /// }
    /// ```
    pub fn iter_shapes_and_records_as<S: ReadableShape>(
        mut self,
    ) -> Result<ShapeRecordIterator<T, S>, Error> {
        let maybe_dbf_reader = self.dbf_reader.take();
        if let Some(dbf_reader) = maybe_dbf_reader {
            let shape_iter = self.iter_shapes_as::<S>();
            Ok(ShapeRecordIterator {
                shape_iter,
                dbf_reader,
            })
        } else {
            Err(Error::MissingDbf)
        }
    }

    /// Returns an iterator over the Shapes and their Records
    ///
    /// # Errors
    ///
    /// The `Result` will be an error if the .dbf wasn't found
    ///
    /// # Example
    /// ```
    /// use shapefile::{Reader, Shape};
    /// let reader = Reader::from_path("tests/data/multipatch.shp").unwrap();
    /// for result in reader.iter_shapes_and_records().unwrap() {
    ///     let (shape, record) = result.unwrap();
    ///     match shape {
    ///         Shape::Multipatch(multip) => {/*...*/ },
    ///         //..
    ///         _ => { /*...*/ }
    ///     }
    ///     // ...
    /// }
    /// ```
    pub fn iter_shapes_and_records(self) -> Result<ShapeRecordIterator<T, Shape>, Error> {
        self.iter_shapes_and_records_as::<Shape>()
    }

    /// Reads the index file from the source
    /// This allows to later read shapes by giving their index without reading the whole file
    ///
    /// (see [read_nth_shape()](struct.Reader.html#method.read_nth_shape))
    pub fn add_index_source(&mut self, source: T) -> Result<(), Error> {
        self.shapes_index = Some(read_index_file(source)?);
        Ok(())
    }

    /// Adds the `source` as the source where the dbf record will be read from
    pub fn add_dbf_source(&mut self, source: T) -> Result<(), Error> {
        let dbf_reader = dbase::Reader::new(source)?;
        self.dbf_reader = Some(dbf_reader);
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
    /// Will attempt to read both the .shx and .dbf associated with the file,
    /// if they do not exists the function will not fail, and you will get an error later
    /// if you try to use a function that requires the file to be present.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// assert_eq!(Path::new("tests/data/linem.dbf").exists(), false);
    /// assert_eq!(Path::new("tests/data/linem.shx").exists(), false);
    ///
    /// // both .shx and .dbf does not exists, but creation does not fail
    /// let mut reader = shapefile::Reader::from_path("tests/data/linem.shp").unwrap();
    /// let result = reader.iter_shapes_and_records();
    /// assert_eq!(result.is_err(),  true);
    ///
    ///
    /// assert_eq!(Path::new("tests/data/multipatch.dbf").exists(), true);
    ///
    /// let mut reader = shapefile::Reader::from_path("tests/data/multipatch.shp").unwrap();
    /// let result = reader.iter_shapes_and_records();
    /// assert_eq!(result.is_err(),  false);
    /// ```
    ///
    ///
    /// ```
    /// use shapefile::Reader;
    /// let mut reader = Reader::from_path("tests/data/line.shp").unwrap();
    /// let polylines = reader.read_as::<shapefile::Polyline>().unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let shape_path = path.as_ref().to_path_buf();
        let shx_path = shape_path.with_extension("shx");
        let dbf_path = shape_path.with_extension("dbf");

        let source = BufReader::new(File::open(shape_path)?);
        let mut reader = Self::new(source)?;

        if shx_path.exists() {
            let index_source = BufReader::new(File::open(shx_path)?);
            reader.add_index_source(index_source)?;
        }

        if dbf_path.exists() {
            let dbf_source = BufReader::new(File::open(dbf_path)?);
            reader.add_dbf_source(dbf_source)?;
        }
        Ok(reader)
    }
}

/// Sources that implements `Seek` have access to
/// a few more methods that uses the *index file(.shx)*
impl<T: Read + Seek> Reader<T> {
    /// Reads the `n`th shape of the shapefile
    ///
    /// # Important
    ///
    /// Even though in shapefiles, shapes are indexed starting from '1'.
    /// this method expects indexes starting from 0.
    ///
    /// # Returns
    ///
    /// `None` if the index is out of range
    ///
    /// # Errors
    ///
    /// This method will return an `Error::MissingIndexFile` if you use it
    /// but no *.shx* was found when opening the shapefile.
    pub fn read_nth_shape_as<S: ReadableShape>(
        &mut self,
        index: usize,
    ) -> Option<Result<S, Error>> {
        if let Some(ref shapes_index) = self.shapes_index {
            let offset = {
                let shape_idx = shapes_index.get(index)?;
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
        } else {
            Some(Err(Error::MissingIndexFile))
        }
    }

    /// Reads the `n`th shape of the shapefile
    pub fn read_nth_shape(&mut self, index: usize) -> Option<Result<Shape, Error>> {
        self.read_nth_shape_as::<Shape>(index)
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
/// let polylines = shapefile::read_as::<_, shapefile::PolylineZ>("tests/data/polygon.shp");
/// assert_eq!(polylines.is_err(), true);
///
/// let polygons = shapefile::read_as::<_, shapefile::Polygon>("tests/data/polygon.shp");
/// assert_eq!(polygons.is_ok(), true);
/// ```
///
/// If the reading is successful, the returned `Vec<S:ReadShape>>`is a vector of actual structs
/// Useful if you know in at compile time which kind of shape you expect the file to have
pub fn read_as<T: AsRef<Path>, S: ReadableShape>(path: T) -> Result<Vec<S>, Error> {
    let reader = Reader::from_path(path)?;
    reader.read_as::<S>()
}

#[cfg(test)]
mod tests {}
