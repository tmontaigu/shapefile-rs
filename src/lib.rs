//! Read & Write [Shapefile](http://downloads.esri.com/support/whitepapers/mo_/shapefile.pdf) in Rust
//!
//! _.dbf_ files are not currently supported
//!
//! As different shapefiles can store different type of shapes
//! (but one shapefile can only store the same type of shapes)
//! This library provide two ways of reading the shapes:
//!
//! 1) Reading as [Shape](record/enum.Shape.html) and then do a `match` to handle the different shapes
//! 2) Reading directly as concrete shapes (ie Polyline, PolylineZ, Point, etc) this of course only
//! works if the file actually contains shapes that matches the requested type
//!
//! # Reading
//! For more details see the [reader](reader/index.html) module
//!
//!
//! # Writing
//!
//! To write a file use the [Writer](writer/struct.Writer.html)
extern crate byteorder;

pub mod header;
pub mod reader;
pub mod record;
pub mod writer;

use std::convert::From;
use std::fmt;
use std::io::{Read, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub use reader::{FileReaderBuilder, Reader};
pub use record::Multipatch;
pub use record::{HasShapeType, MultipartShape, MultipointShape, ReadableShape};
pub use record::{Multipoint, MultipointM, MultipointZ};
pub use record::{PatchType, Shape, NO_DATA};
pub use record::{Point, PointM, PointZ};
pub use record::{Polygon, PolygonM, PolygonZ};
pub use record::{Polyline, PolylineM, PolylineZ};
pub use writer::Writer;

/// All Errors that can happen when using this library
#[derive(Debug)]
pub enum Error {
    /// Wrapper around standard io::Error that might occur when reading/writing
    IoError(std::io::Error),
    /// The file read had an invalid File code (meaning it's not a Shapefile)
    InvalidFileCode(i32),
    /// The file read had an invalid [ShapeType](enum.ShapeType.html) code
    /// (either in the file header or any record type)
    InvalidShapeType(i32),
    /// The Multipatch shape read from the file had an invalid [PatchType](enum.PatchType.html) code
    InvalidPatchType(i32),
    /// Emitted when the file read mixes [ShapeType](enum.ShapeType.html)
    /// Which is not allowed by the specification (expect for NullShape)
    MixedShapeType,
    /// Error emitted when you try to write a malformed Shape
    /// For example: a mismatch between the number of x and z coordinates
    MalformedShape,
    /// Error returned when trying to read the shape records as a certain shape type
    /// but the actual shape type does not correspond to the one asked
    MismatchShapeType {
        /// The requested ShapeType
        requested: ShapeType,
        /// The actual type of the shape
        actual: ShapeType,
    },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IoError(error)
    }
}

/// The enum for the ShapeType as defined in the
/// specification
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShapeType {
    NullShape = 0,
    Point = 1,
    Polyline = 3,
    Polygon = 5,
    Multipoint = 8,

    PointZ = 11,
    PolylineZ = 13,
    PolygonZ = 15,
    MultipointZ = 18,

    PointM = 21,
    PolylineM = 23,
    PolygonM = 25,
    MultipointM = 28,

    Multipatch = 31,
}

impl ShapeType {
    pub(crate) fn read_from<T: Read>(source: &mut T) -> Result<ShapeType, Error> {
        let code = source.read_i32::<LittleEndian>()?;
        Self::from(code).ok_or_else(|| Error::InvalidShapeType(code))
    }

    pub(crate) fn write_to<T: Write>(self, dest: &mut T) -> Result<(), std::io::Error> {
        dest.write_i32::<LittleEndian>(self as i32)?;
        Ok(())
    }

    /// Returns the ShapeType corresponding to the input code
    /// if the code is valid
    /// ```
    /// use shapefile::ShapeType;
    ///
    /// assert_eq!(ShapeType::from(25), Some(ShapeType::PolygonM));
    /// assert_eq!(ShapeType::from(60), None);
    /// ```
    pub fn from(code: i32) -> Option<ShapeType> {
        match code {
            0 => Some(ShapeType::NullShape),
            1 => Some(ShapeType::Point),
            3 => Some(ShapeType::Polyline),
            5 => Some(ShapeType::Polygon),
            8 => Some(ShapeType::Multipoint),
            11 => Some(ShapeType::PointZ),
            13 => Some(ShapeType::PolylineZ),
            15 => Some(ShapeType::PolygonZ),
            18 => Some(ShapeType::MultipointZ),
            21 => Some(ShapeType::PointM),
            23 => Some(ShapeType::PolylineM),
            25 => Some(ShapeType::PolygonM),
            28 => Some(ShapeType::MultipointM),
            31 => Some(ShapeType::Multipatch),
            _ => None,
        }
    }

    /// Returns whether the ShapeType has the third dimension Z
    pub fn has_z(self) -> bool {
        match self {
            ShapeType::PointZ
            | ShapeType::PolylineZ
            | ShapeType::PolygonZ
            | ShapeType::MultipointZ => true,
            _ => false,
        }
    }

    /// Returns whether the ShapeType has the optional measure dimension
    pub fn has_m(self) -> bool {
        match self {
            ShapeType::PointZ
            | ShapeType::PolylineZ
            | ShapeType::PolygonZ
            | ShapeType::MultipointZ
            | ShapeType::PointM
            | ShapeType::PolylineM
            | ShapeType::PolygonM
            | ShapeType::MultipointM => true,
            _ => false,
        }
    }

    /// Returns true if the shape may have multiple parts
    pub fn is_multipart(self) -> bool {
        match self {
            ShapeType::Point
            | ShapeType::PointM
            | ShapeType::PointZ
            | ShapeType::Multipoint
            | ShapeType::MultipointM
            | ShapeType::MultipointZ => false,
            _ => true,
        }
    }
}

impl fmt::Display for ShapeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShapeType::NullShape => write!(f, "NullShape"),
            ShapeType::Point => write!(f, "Point"),
            ShapeType::Polyline => write!(f, "Polyline"),
            ShapeType::Polygon => write!(f, "Polygon"),
            ShapeType::Multipoint => write!(f, "Multipoint"),
            ShapeType::PointZ => write!(f, "PointZ"),
            ShapeType::PolylineZ => write!(f, "PolylineZ"),
            ShapeType::PolygonZ => write!(f, "PolygonZ"),
            ShapeType::MultipointZ => write!(f, "MultipointZ"),
            ShapeType::PointM => write!(f, "PointM"),
            ShapeType::PolylineM => write!(f, "PolylineM"),
            ShapeType::PolygonM => write!(f, "PolygonM"),
            ShapeType::MultipointM => write!(f, "MultipointM"),
            ShapeType::Multipatch => write!(f, "Multipatch"),
        }
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
/// If the reading is successful Returned `Vec<S:ActualShape>>`is a vector of actual structs
/// Useful if you know in at compile time which kind of shape you expect the file to have
pub fn read_as<T: AsRef<Path>, S: ReadableShape>(path: T) -> Result<Vec<S::ReadShape>, Error> {
    let reader = Reader::from_path(path)?;
    reader.read_as::<S>()
}
