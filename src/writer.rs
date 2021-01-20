//! Module with the definition of the [Writer] that allows writing shapefile
//!
//! # Writer
//!
//! [Writer] is the struct that writes a complete shapefile (_.shp_, _.shx_, _.dbf_).
//!
//! # ShapeWriter
//!
//! The [ShapeWriter] can be used if you only want to write the .shp
//! and .shx files, however since it does not write the .dbf file, it is not recommended.
use std::io::{BufWriter, Write};

use header;
use record::{BBoxZ, EsriShape, RecordHeader};
use std::fs::File;
use std::path::Path;
use Error;

use byteorder::{BigEndian, WriteBytesExt};
use reader::ShapeIndex;
use dbase::{TableWriterBuilder};

pub(crate) fn f64_min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

pub(crate) fn f64_max(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

fn write_index_file<T: Write>(
    mut dest: &mut T,
    shapefile_header: &header::Header,
    shapes_index: Vec<ShapeIndex>,
) -> Result<(), std::io::Error> {
    let mut header = *shapefile_header;
    let content_len = shapes_index.len() * 2 * std::mem::size_of::<i32>();
    header.file_length = header::HEADER_SIZE + content_len as i32;
    header.file_length /= 2;

    header.write_to(&mut dest)?;
    for shape_index in shapes_index {
        dest.write_i32::<BigEndian>(shape_index.offset)?;
        dest.write_i32::<BigEndian>(shape_index.record_size)?;
    }
    Ok(())
}

/// struct that handles the writing of the .shp
/// and (optionally) the .idx
///
/// The recommended way to create a ShapeWriter by using [ShapeWriter::from_path]
///
/// # Important
///
/// As this writer does not write the _.dbf_, it does not write what is considered
/// a complete (thus valid) shapefile.
pub struct ShapeWriter<T: Write> {
    shp_dest: T,
    shx_dest: Option<T>,
}

impl<T: Write> ShapeWriter<T> {
    /// Creates a writer that can be used to write a new shapefile.
    ///
    /// The `dest` argument is only for the .shp
    pub fn new(shp_dest: T) -> Self {
        Self {
            shp_dest,
            shx_dest: None,
        }
    }

    pub fn with_shx(shp_dest: T, shx_dest: T) -> Self {
        Self {
            shp_dest,
            shx_dest: Some(shx_dest)
        }
    }

    /// Writes the shapes to the file
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// use shapefile::Point;
    /// let mut writer = shapefile::ShapeWriter::from_path("points.shp")?;
    /// let points = vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(2.0, 0.0)];
    ///
    /// writer.write_shapes(&points)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// use shapefile::{Point, Polyline};
    /// let mut writer = shapefile::ShapeWriter::from_path("polylines.shp")?;
    /// let points = vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(2.0, 0.0)];
    /// let polyline = Polyline::new(points);
    ///
    /// writer.write_shapes(&vec![polyline])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_shapes<S: EsriShape>(mut self, shapes: &[S]) -> Result<(), Error> {
        let mut file_length = header::HEADER_SIZE as usize;
        for shape in shapes {
            file_length += 2 * std::mem::size_of::<i32>(); // record_header
            file_length += std::mem::size_of::<i32>(); // shape_type
            file_length += shape.size_in_bytes();
        }
        file_length /= 2; // file size is in 16bit words

        assert!(file_length <= i32::max_value() as usize);

        let file_length = file_length as i32;
        let shapetype = S::shapetype();
        let header = header::Header {
            bbox: BBoxZ::from_shapes(shapes),
            file_length,
            shape_type: shapetype,
            version: 1000,
        };

        let mut pos = header::HEADER_SIZE / 2;
        header.write_to(&mut self.shp_dest)?;
        let mut shapes_index = Vec::<ShapeIndex>::with_capacity(shapes.len());
        for (i, shape) in (1..).zip(shapes) {
            //TODO Check record size < i32_max ?
            let record_size = (shape.size_in_bytes() + std::mem::size_of::<i32>()) / 2;
            let rc_hdr = RecordHeader {
                record_number: i,
                record_size: record_size as i32,
            };

            shapes_index.push(ShapeIndex {
                offset: pos,
                record_size: record_size as i32,
            });

            rc_hdr.write_to(&mut self.shp_dest)?;
            shapetype.write_to(&mut self.shp_dest)?;
            shape.write_to(&mut self.shp_dest)?;
            pos += record_size as i32 + RecordHeader::SIZE as i32 / 2;
        }

        if let Some(ref mut shx_dest) = &mut self.shx_dest {
            write_index_file(shx_dest, &header, shapes_index)?;
        }

        Ok(())
    }
}

impl ShapeWriter<BufWriter<File>> {
    /// Creates a new writer from a path.
    /// Creates both a .shp and .shx files
    ///
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let writer = shapefile::ShapeWriter::from_path("new_file.shp");
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let shp_path = path.as_ref().to_path_buf();
        let shx_path = shp_path.with_extension("shx");

        let shp_file = BufWriter::new(File::create(shp_path)?);
        let shx_file = BufWriter::new(File::create(shx_path)?);

        Ok(Self::with_shx(shp_file,shx_file))
    }
}

/// The Writer writes a complete shapefile that is, it
/// writes the 3 mandatory files (.shp, .shx, .dbf)
///
/// The recommended way to create a new shapefile is via the
/// [Writer::from_path] or [Writer::from_path_with_info] associated functions.
///
/// # Examples
///
/// To create a Writer that writes a .dbf file that has the same
/// structure as .dbf read earlier you will have to do:
///
/// ```
/// # fn main() -> Result<(), shapefile::Error> {
/// let mut reader = shapefile::Reader::from_path("tests/data/multipatch.shp")?;
/// let shape_records = reader.read()?;
/// let table_info = reader.into_table_info();
///
/// let writer = shapefile::Writer::from_path_with_info("new_multipatch.shp", table_info);
///
/// # std::fs::remove_file("new_multipatch.shp")?;
/// # std::fs::remove_file("new_multipatch.shx")?;
/// # std::fs::remove_file("new_multipatch.dbf")?;
/// # Ok(())
/// # }
/// ```
pub struct Writer<T: Write> {
    shape_writer: ShapeWriter<T>,
    dbase_writer: dbase::TableWriter<T>
}

impl<T: Write> Writer<T> {
    /// Creates a new writer using the provided ShapeWriter and TableWriter
    ///
    /// # Example
    ///
    /// Creating a Writer that writes to in memory buffers.
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// use std::convert::TryInto;
    /// let mut shp_dest = std::io::Cursor::new(Vec::<u8>::new());
    /// let mut shx_dest = std::io::Cursor::new(Vec::<u8>::new());
    /// let mut dbf_dest = std::io::Cursor::new(Vec::<u8>::new());
    ///
    /// let shape_writer = shapefile::ShapeWriter::with_shx(&mut shp_dest, &mut shx_dest);
    /// let dbase_writer = dbase::TableWriterBuilder::new()
    ///     .add_character_field("Name".try_into().unwrap(), 50)
    ///     .build_with_dest(&mut dbf_dest);
    ///
    /// let shape_writer = shapefile::Writer::new(shape_writer, dbase_writer);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(shape_writer: ShapeWriter<T>, dbase_writer: dbase::TableWriter<T>) -> Self {
        Self {
            shape_writer,
            dbase_writer
        }
    }

    // TODO once we get the ability to write shapes and records 'iteratively' the input to
    //      this function could be IntoIterator<Item=(S, R)>
    pub fn write_shapes_and_records<S: EsriShape, R: dbase::WritableRecord>(self, shapes: &[S], records: &[R]) -> Result<(), Error> {
        if shapes.len() != records.len() {
            panic!("There must be has many shapes as there are records");
        }
        self.shape_writer.write_shapes(shapes)?;
        self.dbase_writer.write(records)?;
        Ok(())
    }
}

impl Writer<BufWriter<File>> {
    /// Creates all the files needed for the shapefile to be complete (.shp, .shx, .dbf)
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// use std::convert::TryInto;
    /// let table_builder = dbase::TableWriterBuilder::new()
    ///     .add_character_field("name".try_into().unwrap(), 50);
    /// let writer = shapefile::Writer::from_path("new_cities.shp", table_builder)?;
    /// # std::fs::remove_file("new_cities.shp")?;
    /// # std::fs::remove_file("new_cities.shx")?;
    /// # std::fs::remove_file("new_cities.dbf")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P, table_builder: TableWriterBuilder) -> Result<Self, Error> {
        Ok(Self {
            shape_writer: ShapeWriter::from_path(path.as_ref())?,
            dbase_writer: table_builder.build_with_file_dest(
                path.as_ref().with_extension("dbf"))?
        })
    }

    pub fn from_path_with_info<P: AsRef<Path>>(path: P, table_info: dbase::TableInfo) -> Result<Self, Error> {
        Ok(Self {
            shape_writer: ShapeWriter::from_path(path.as_ref())?,
            dbase_writer: dbase::TableWriterBuilder::from_table_info(table_info)
                .build_with_file_dest(path.as_ref().with_extension("dbf"))?
        })
    }
}
