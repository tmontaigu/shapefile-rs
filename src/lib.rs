//! Read & Write [Shapefile](http://downloads.esri.com/support/whitepapers/mo_/shapefile.pdf) in Rust
//!
//! _.dbf_ can be read & written to (but the support for this is still alpha-ish)
//!
//! As different shapefiles can store different type of shapes
//! (but one shapefile can only store the same type of shapes)
//! This library provide two ways of reading the shapes:
//!
//! 1) Reading as [Shape](record/enum.Shape.html) and then do a `match` to handle the different shapes
//! 2) Reading directly as concrete shapes (ie Polyline, PolylineZ, Point, etc) this of course only
//! works if the file actually contains shapes that matches the requested type
//!
//! # Shapefiles shapes
//!
//! The [`Point`], [`PointM`] and [`PointZ`] are the base data types of shapefiles,
//! the other shapes (`Polyline, Multipoint`, ...) are collections of these type of points
//! with different semantics (multiple parts or no, closed parts or no, ...)
//!
//! With the exception of the [`Multipatch`] shape, each shape as a variant for each type
//! of point. ([`Multipatch`] always uses [`PointZ`])
//! Eg: For the polyline, there is [`Polyline`], [`PolylineM`], [`PolylineZ`]
//!
//! # Reading
//! For more details see the [reader](reader/index.html) module
//!
//! # Writing
//!
//! To write a file see the [writer](writer/index.html) module
//!
//!
//! # Features
//!
//! The `geo-types` feature can be enabled to have access to `From` and `TryFrom`
//! implementations allowing to convert (or try to) back and forth between shapefile's type and
//! the one in `geo_types`
extern crate byteorder;
pub extern crate dbase;

pub mod header;
pub mod reader;
pub mod record;
pub mod writer;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::convert::From;
use std::fmt;
use std::io::{Read, Write};

pub use reader::{read, read_as, Reader};
pub use record::traits::{MultipartShape, MultipointShape};
pub use record::Multipatch;
pub use record::{convert_shapes_to_vec_of, HasShapeType, ReadableShape};
pub use record::{Multipoint, MultipointM, MultipointZ};
pub use record::{PatchType, Shape, NO_DATA};
pub use record::{Point, PointM, PointZ};
pub use record::{Polygon, PolygonM, PolygonZ};
pub use record::{Polyline, PolylineM, PolylineZ};
pub use writer::Writer;

#[cfg(feature = "geo-types")]
extern crate geo_types;

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
    MalformedShape,
    /// Error returned when trying to read the shape records as a certain shape type
    /// but the actual shape type does not correspond to the one asked
    MismatchShapeType {
        /// The requested ShapeType
        requested: ShapeType,
        /// The actual type of the shape
        actual: ShapeType,
    },
    InvalidShapeRecordSize,

    DbaseError(dbase::Error),
    MissingDbf,
    MissingIndexFile,
    /// This error can happen when trying to convert a multipatch or polgyon into
    /// geo_types::Multipolygon, this error happen when during such conversion,
    /// an inner ring has no corresponding outer ring.
    OrphanInnerRing,
    NullShapeConversion,
    GeometryCollectionConversion,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IoError(error)
    }
}

impl From<dbase::Error> for Error {
    fn from(e: dbase::Error) -> Error {
        Error::DbaseError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "{}", e),
            Error::InvalidFileCode(code) => write!(
                f,
                "The file code ' {} ' is invalid, is this a Shapefile ?",
                code
            ),
            Error::InvalidShapeType(code) => write!(
                f,
                "The code ' {} ' does not correspond to any of the ShapeType code defined by ESRI",
                code
            ),
            Error::MismatchShapeType { requested, actual } => write!(
                f,
                "The requested type: '{}' does not correspond to the actual shape type: '{}'",
                requested, actual
            ),
            e => write!(f, "{:?}", e),
        }
    }
}

impl std::error::Error for Error {}

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
