//! Reader module, contains the definitions of the types that a user should use to read a file
//!
//! # Reader
//!
//! The [Reader] is the struct that actually reads the different files
//! that make up a _shapefile_.
//!
//! ## Examples
//!
//! When reading from a file:
//!
//! Creates a reader from a path, then iterate over its `Shapes`, reading one shape each iteration
//! ```
//! # fn main() -> Result<(), shapefile::Error> {
//! let mut reader = shapefile::Reader::from_path("tests/data/multipatch.shp")?;
//! for shape_record in reader.iter_shapes_and_records() {
//!     let (shape, record) = shape_record?;
//!     println!("{}", shape);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # ShapeReader
//!
//! If you only care about the geometries stored in the _.shp_ file, whether or not the _.dbf_ file
//! actually exists, you can use the [ShapeReader].
//!
//! # Extra
//!
//! If you know beforehand the exact type that the .shp file is made of,
//! you can use the different `*_as::<S>()` methods.:
//! - [Reader::read_as] To read all the shapes and records as the specified types using a Reader
//! - [Reader::iter_shapes_and_records_as] To iterate over both the shapes and records of the Reader
//!
//! - [ShapeReader::read_as] To read all the shapes (only) as the specified type using ShapeReader
//! - [ShapeReader::iter_shapes_as] To iterate over the shapes as shapes
//!   of the specified type using ShapeReader
//!
//!
//! Otherwise use the functions that return [Shapes](../record/enum.Shape.html) and do a `match`
//!
//! - [Reader::read]
//! - [Reader::iter_shapes_and_records]
//! - [ShapeReader::read]
//! - [ShapeReader::iter_shapes]
//!
//! ## One liners
//!
//! Some 'one liner' functions are provided to read the content
//! of a shapefile with one line of code
//!
//! - [read]
//! - [read_as]
//! - [read_shapes]
//! - [read_shapes_as]

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use header;
use record;
use record::ReadableShape;
use {Error, Shape};

const INDEX_RECORD_SIZE: usize = 2 * std::mem::size_of::<i32>();

#[derive(Copy, Clone)]
pub(crate) struct ShapeIndex {
    pub offset: i32,
    pub record_size: i32,
}

impl ShapeIndex {
    pub(crate) fn write_to<W: Write>(self, dest: &mut W) -> std::io::Result<()> {
        dest.write_i32::<BigEndian>(self.offset)?;
        dest.write_i32::<BigEndian>(self.record_size)?;
        Ok(())
    }
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
pub struct ShapeIterator<'a, T: Read, S: ReadableShape> {
    _shape: std::marker::PhantomData<S>,
    source: &'a mut T,
    current_pos: usize,
    file_length: usize,
}

impl<'a, T: Read, S: ReadableShape> Iterator for ShapeIterator<'a, T, S> {
    type Item = Result<S, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos >= self.file_length {
            None
        } else {
            let (hdr, shape) = match read_one_shape_as::<T, S>(self.source) {
                Err(e) => return Some(Err(e)),
                Ok(hdr_and_shape) => hdr_and_shape,
            };
            self.current_pos += record::RecordHeader::SIZE;
            self.current_pos += hdr.record_size as usize * 2;
            Some(Ok(shape))
        }
    }
}

pub struct ShapeRecordIterator<'a, T: Read + Seek, S: ReadableShape, R: dbase::ReadableRecord> {
    shape_iter: ShapeIterator<'a, T, S>,
    record_iter: dbase::RecordIterator<'a, T, R>,
}

impl<'a, T: Read + Seek, S: ReadableShape, R: dbase::ReadableRecord> Iterator
    for ShapeRecordIterator<'a, T, S, R>
{
    type Item = Result<(S, R), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let shape = match self.shape_iter.next()? {
            Err(e) => return Some(Err(e)),
            Ok(shp) => shp,
        };

        let record = match self.record_iter.next()? {
            Err(e) => return Some(Err(Error::DbaseError(e))),
            Ok(rcd) => rcd,
        };

        Some(Ok((shape, record)))
    }
}

/// This reader only reads the `.shp` and optionally the (`.shx`) files
/// of a shapefile.
pub struct ShapeReader<T: Read> {
    source: T,
    header: header::Header,
    shapes_index: Option<Vec<ShapeIndex>>,
}

impl<T: Read> ShapeReader<T> {
    /// Creates a new ShapeReader from a source that implements the `Read` trait
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
    /// # fn main() -> Result<(), shapefile::Error> {
    /// use std::fs::File;
    /// let file = File::open("tests/data/line.shp")?;
    /// let reader = shapefile::ShapeReader::new(file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(mut source: T) -> Result<Self, Error> {
        let header = header::Header::read_from(&mut source)?;

        Ok(Self {
            source,
            header,
            shapes_index: None,
        })
    }

    /// Creates a new ShapeReader using 2 sources, one for the _.shp_
    /// the other for the _.shx_
    ///
    /// The _.shp_ header is read upon creation
    /// and the whole _.shx_ file is read upon creation.
    ///
    /// # Example
    /// ```no_run
    /// # fn main() -> Result<(), shapefile::Error> {
    /// use std::fs::File;
    /// let shp_file = File::open("cities.shp")?;
    /// let shx_file = File::open("cities.shx:")?;
    /// let reader = shapefile::ShapeReader::with_shx(shp_file, shx_file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_shx(mut source: T, shx_source: T) -> Result<Self, Error> {
        let shapes_index = Some(read_index_file(shx_source)?);
        let header = header::Header::read_from(&mut source)?;

        Ok(Self {
            source,
            header,
            shapes_index,
        })
    }

    /// Returns a non-mutable reference to the header read
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// let reader = shapefile::ShapeReader::from_path("tests/data/pointz.shp")?;
    /// let header = reader.header();
    /// assert_eq!(header.shape_type, shapefile::ShapeType::PointZ);
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), shapefile::Error> {
    /// use shapefile::ShapeReader;
    /// let mut reader = ShapeReader::from_path("tests/data/linem.shp")?;
    /// let polylines_m = reader.read_as::<shapefile::PolylineM>(); // we ask for the correct type
    /// assert_eq!(polylines_m.is_ok(), true);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// use shapefile::ShapeReader;
    /// let mut reader = ShapeReader::from_path("tests/data/linem.shp")?;
    /// let polylines = reader.read_as::<shapefile::Polyline>(); // we ask for the wrong type
    /// assert_eq!(polylines.is_err(), true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_as<S: ReadableShape>(mut self) -> Result<Vec<S>, Error> {
        self.iter_shapes_as::<S>().collect()
    }

    /// Reads all the shapes and returns them
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// let mut reader = shapefile::ShapeReader::from_path("tests/data/multipoint.shp")?;
    /// let shapes = reader.read()?;
    /// for shape in shapes {
    ///     match shape {
    ///         shapefile::Shape::Multipoint(pts) => println!(" Yay Multipoints: {}", pts),
    ///         _ => panic!("ups, not multipoints"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn read(mut self) -> Result<Vec<Shape>, Error> {
        self.iter_shapes_as::<Shape>().collect()
    }

    /// Returns an iterator that tries to read the shapes as the specified type
    /// Will return an error of the type `S` does not match the actual type in the file
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// let mut reader = shapefile::ShapeReader::from_path("tests/data/multipoint.shp")?;
    /// for multipoints in reader.iter_shapes_as::<shapefile::Multipoint>() {
    ///     let points = multipoints?;
    ///     println!("{}", points);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter_shapes_as<S: ReadableShape>(&mut self) -> ShapeIterator<'_, T, S> {
        ShapeIterator {
            _shape: std::marker::PhantomData,
            source: &mut self.source,
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
    /// # fn main() -> Result<(), shapefile::Error> {
    /// let mut reader = shapefile::ShapeReader::from_path("tests/data/multipoint.shp")?;
    /// for shape in reader.iter_shapes() {
    ///     match shape? {
    ///         shapefile::Shape::Multipatch(shp) => println!("Multipoint!"),
    ///         _ => println!("Other type of shape"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// let mut reader = shapefile::ShapeReader::from_path("tests/data/multipoint.shp")?;
    /// for shape in reader.iter_shapes() {
    ///     match shape? {
    ///         shapefile::Shape::Multipatch(shp) => println!("Multipoint!"),
    ///         _ => println!("Other type of shape"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter_shapes(&mut self) -> ShapeIterator<'_, T, Shape> {
        self.iter_shapes_as::<Shape>()
    }
}

/// Sources that implements `Seek` have access to
/// a few more methods that uses the *index file(.shx)*
impl<T: Read + Seek> ShapeReader<T> {
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
            if index >= shapes_index.len() {
                return None;
            }

            if let Err(e) = self.seek(index) {
                return Some(Err(e));
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

    /// Seek to the start of the shape at `index`
    ///
    /// # Error
    ///
    /// Returns [Error::MissingIndexFile] if the _shx_ file
    /// was not found by [ShapeReader::from_path] or the reader
    /// was not constructed with [ShapeReader::with_shx]
    pub fn seek(&mut self, index: usize) -> Result<(), Error> {
        if let Some(ref shapes_index) = self.shapes_index {
            let offset = shapes_index
                .get(index)
                .map(|shape_idx| (shape_idx.offset * 2) as u64);

            match offset {
                Some(n) => self.source.seek(SeekFrom::Start(n)),
                None => self.source.seek(SeekFrom::End(0)),
            }?;
            Ok(())
        } else {
            Err(Error::MissingIndexFile)
        }
    }

    /// Returns the number of shapes in the shapefile
    ///
    /// # Error
    ///
    /// Returns [Error::MissingIndexFile] if the _shx_ file
    /// was not found by [ShapeReader::from_path] or the reader
    /// was not constructed with [ShapeReader::with_shx]
    ///
    /// # Example
    ///
    /// ```
    /// let reader = shapefile::ShapeReader::from_path("tests/data/point.shp").unwrap();
    /// // point.shp has a .shx file next to it, so we can read the count data
    /// assert_eq!(1, reader.shape_count().unwrap());
    ///
    /// let reader = shapefile::ShapeReader::from_path("tests/data/pointm.shp").unwrap();
    /// // There is no pointm.shx, so the shape_count() method returns error
    /// assert!(reader.shape_count().is_err(), "Should return error if no index file");
    /// ```
    pub fn shape_count(&self) -> Result<usize, Error> {
        if let Some(ref shapes_index) = self.shapes_index {
            Ok(shapes_index.len())
        } else {
            Err(Error::MissingIndexFile)
        }
    }
}

impl ShapeReader<BufReader<File>> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let shape_path = path.as_ref().to_path_buf();
        let shx_path = shape_path.with_extension("shx");

        let source = BufReader::new(File::open(shape_path)?);

        if shx_path.exists() {
            let index_source = BufReader::new(File::open(shx_path)?);
            Self::with_shx(source, index_source)
        } else {
            Self::new(source)
        }
    }
}

/// Reader that reads a _shapefile_.
///
/// The recommended way to create a _Reader_ is by using its
/// [Reader::from_path] associated function as it
/// will take care of opening the _.shx_ and _.dbf_ files corresponding to the _.shp_
/// and return an error in case some mandatory files are missing.
///
///
/// If you want to read a shapefile that is not stored in a file
/// (e.g the shp data is in a buffer), you will have to construct
/// the *Reader* "by hand" with its [Reader::new] associated function.
pub struct Reader<T: Read + Seek> {
    shape_reader: ShapeReader<T>,
    dbase_reader: dbase::Reader<T>,
}

impl<T: Read + Seek> Reader<T> {
    /// Creates a new Reader from both a ShapeReader (.shp, .shx) and dbase::Reader (.dbf)
    pub fn new(shape_reader: ShapeReader<T>, dbase_reader: dbase::Reader<T>) -> Self {
        Self {
            shape_reader,
            dbase_reader,
        }
    }

    /// Returns the header of the .shp file
    pub fn header(&self) -> &header::Header {
        self.shape_reader.header()
    }

    pub fn iter_shapes_and_records_as<S: ReadableShape, R: dbase::ReadableRecord>(
        &mut self,
    ) -> ShapeRecordIterator<'_, T, S, R> {
        ShapeRecordIterator {
            shape_iter: self.shape_reader.iter_shapes_as::<S>(),
            record_iter: self.dbase_reader.iter_records_as::<R>(),
        }
    }

    /// Returns an iterator that returns both the shape and the record
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// let mut reader = shapefile::Reader::from_path("tests/data/multipatch.shp")?;
    /// for shape_record in reader.iter_shapes_and_records() {
    ///     let (shape, record) = shape_record?;
    ///     println!("Geometry: {}, Properties {:?}", shape, record);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter_shapes_and_records(&mut self) -> ShapeRecordIterator<'_, T, Shape, dbase::Record> {
        self.iter_shapes_and_records_as::<Shape, dbase::Record>()
    }

    pub fn read_as<S: ReadableShape, R: dbase::ReadableRecord>(
        &mut self,
    ) -> Result<Vec<(S, R)>, Error> {
        self.iter_shapes_and_records_as::<S, R>().collect()
    }

    /// Read all the shape and record and returns them in a [Vec]
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// let mut reader = shapefile::Reader::from_path("tests/data/multipatch.shp")?;
    /// let data = reader.read()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn read(&mut self) -> Result<Vec<(Shape, dbase::Record)>, Error> {
        self.read_as::<Shape, dbase::Record>()
    }

    /// Seeks to the start of the shape at `index`
    pub fn seek(&mut self, index: usize) -> Result<(), Error> {
        self.shape_reader.seek(index)?;
        self.dbase_reader.seek(index)?;
        Ok(())
    }

    /// Returns the number of shapes in the shapefile
    ///
    /// See [ShapeReader::shape_count]
    pub fn shape_count(&self) -> Result<usize, Error> {
        self.shape_reader.shape_count()
    }

    /// Consumes the self and returns the dbase table info
    /// which can be given to [TableWriterBuild](dbase::TableWriterBuilder) or
    /// [crate::Writer::from_path_with_info] to create a shapefile where the .dbf file has the
    /// same structure as the .dbf read by this reader
    pub fn into_table_info(self) -> dbase::TableInfo {
        self.dbase_reader.into_table_info()
    }
}

impl Reader<BufReader<File>> {
    /// Creates a reader from a path the .shp file
    ///
    /// Will attempt to read both the `.shx` and `.dbf` associated with the file,
    ///
    /// If the `.shx` is not found, no error will be returned,
    /// however [seek] will fail is used.
    ///
    /// If the `.dbf` is not found [Error::MissingDbf] will be return as the error.
    ///
    /// [seek]: ShapeReader::seek
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// # fn main() -> Result<(), shapefile::Error> {
    /// assert_eq!(Path::new("tests/data/linem.dbf").exists(), false);
    /// assert_eq!(Path::new("tests/data/linem.shx").exists(), false);
    ///
    /// // .dbf file not found, the reader can't be created
    /// let mut reader = shapefile::Reader::from_path("tests/data/linem.shp");
    /// assert_eq!(reader.is_err(), true);
    ///
    /// assert_eq!(Path::new("tests/data/multipatch.dbf").exists(), true);
    ///
    /// // The dbf file exists, the reader can be created
    /// let mut reader = shapefile::Reader::from_path("tests/data/multipatch.shp");
    /// assert_eq!(reader.is_err(),  false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let shape_path = path.as_ref().to_path_buf();
        let dbf_path = shape_path.with_extension("dbf");

        if dbf_path.exists() {
            let shape_reader = ShapeReader::from_path(path)?;
            let dbf_source = BufReader::new(File::open(dbf_path)?);
            let dbf_reader = dbase::Reader::new(dbf_source)?;
            Ok(Self {
                shape_reader,
                dbase_reader: dbf_reader,
            })
        } else {
            return Err(Error::MissingDbf);
        }
    }
}

pub fn read<T: AsRef<Path>>(path: T) -> Result<Vec<(Shape, dbase::Record)>, Error> {
    read_as::<T, Shape, dbase::Record>(path)
}

pub fn read_as<T: AsRef<Path>, S: ReadableShape, R: dbase::ReadableRecord>(
    path: T,
) -> Result<Vec<(S, R)>, Error> {
    Reader::from_path(path).and_then(|mut rdr| rdr.read_as::<S, R>())
}

/// Function to read all the Shapes in a file as a certain type
///
/// It does not open the .dbf file.
///
/// Fails and return `Err(Error:MismatchShapeType)`
///
///  # Examples
///
/// ```
/// let polylines = shapefile::read_shapes_as::<_, shapefile::PolylineZ>("tests/data/polygon.shp");
/// assert_eq!(polylines.is_err(), true);
///
/// let polygons = shapefile::read_shapes_as::<_, shapefile::Polygon>("tests/data/polygon.shp");
/// assert_eq!(polygons.is_ok(), true);
/// ```
///
/// If the reading is successful, the returned `Vec<S:ReadShape>>`is a vector of actual structs
/// Useful if you know in at compile time which kind of shape you expect the file to have
pub fn read_shapes_as<T: AsRef<Path>, S: ReadableShape>(path: T) -> Result<Vec<S>, Error> {
    ShapeReader::from_path(path).and_then(|rdr| rdr.read_as::<S>())
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
/// # fn main() -> Result<(), shapefile::Error> {
/// let shapes = shapefile::read_shapes("tests/data/multipatch.shp")?;
/// assert_eq!(shapes.len(), 1);
/// # Ok(())
/// # }
/// ```
pub fn read_shapes<T: AsRef<Path>>(path: T) -> Result<Vec<Shape>, Error> {
    read_shapes_as::<T, Shape>(path)
}

#[cfg(test)]
mod tests {}
