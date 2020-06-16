//! Module with the definition of the [Writer](struct.Writer.html) that allows writing shapefile
//!
//! It is recommended to create a `Writer` using its [from_path](struct.Writer.html#method.from_path) method
//! to ensure that both the .shp and .shx files are created.
//! Then use its [writes_shapes](struct.Writer.html#method.write_shapes) method to write the files.

use std::io::{BufWriter, Write};

use header;
use record::{BBoxZ, EsriShape, RecordHeader};
use std::fs::File;
use std::path::Path;
use Error;

use byteorder::{BigEndian, WriteBytesExt};
use reader::ShapeIndex;

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

/// struct that writes the shapes
pub struct Writer<T: Write> {
    pub dest: T,
    index_dest: Option<T>,
    dbase_dest: Option<T>,
}

impl<T: Write> Writer<T> {
    /// Creates a writer that can be used to write a new shapefile.
    ///
    /// The `dest` argument is only for the .shp
    pub fn new(dest: T) -> Self {
        Self {
            dest,
            index_dest: None,
            dbase_dest: None,
        }
    }

    //TODO This method should move as calling it twice would produce a shitty file
    /// Writes the shapes to the file
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), shapefile::Error> {
    /// use shapefile::Point;
    /// let mut writer = shapefile::Writer::from_path("points.shp")?;
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
    /// let mut writer = shapefile::Writer::from_path("polylines.shp")?;
    /// let points = vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(2.0, 0.0)];
    /// let polyline = Polyline::new(points);
    ///
    /// writer.write_shapes(&vec![polyline])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_shapes<S: EsriShape>(&mut self, shapes: &[S]) -> Result<(), Error> {
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
        header.write_to(&mut self.dest)?;
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

            rc_hdr.write_to(&mut self.dest)?;
            shapetype.write_to(&mut self.dest)?;
            shape.write_to(&mut self.dest)?;
            pos += record_size as i32 + RecordHeader::SIZE as i32 / 2;
        }

        if let Some(ref mut shx_dest) = &mut self.index_dest {
            write_index_file(shx_dest, &header, shapes_index)?;
        }

        Ok(())
    }

    pub fn write_shapes_and_records<S: EsriShape>(
        mut self,
        shapes: &[S],
        records: Vec<dbase::Record>,
    ) -> Result<(), Error> {
        if shapes.len() != records.len() {
            panic!("The shapes and records vectors must have the same len");
        }
        self.write_shapes(&shapes)?;
        if let Some(dbase_dest) = self.dbase_dest {
            let dbase_writer = dbase::Writer::new(dbase_dest);
            dbase_writer.write(&records)?;
        }
        Ok(())
    }

    /// Adds dest as the destination where the index file will be written
    pub fn add_index_dest(&mut self, dest: T) {
        self.index_dest = Some(dest);
    }

    /// Adds dest as the destination where the dbase content will be written
    pub fn add_dbase_dest(&mut self, dest: T) {
        self.dbase_dest = Some(dest);
    }
}

impl Writer<BufWriter<File>> {
    /// Creates a new writer from a path.
    /// Creates both a .shp and .shx files
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// let writer = shapefile::Writer::from_path("/dev/null");
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let shp_path = path.as_ref().to_path_buf();
        let shx_path = shp_path.with_extension("shx");
        let dbf_path = shp_path.with_extension("dbf");

        let shp_file = BufWriter::new(File::create(shp_path)?);
        let shx_file = BufWriter::new(File::create(shx_path)?);
        let dbf_file = BufWriter::new(File::create(dbf_path)?);

        let mut writer = Self::new(shp_file);
        writer.add_index_dest(shx_file);
        writer.add_dbase_dest(dbf_file);

        Ok(writer)
    }
}
