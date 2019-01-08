extern crate byteorder;

pub mod header;
pub mod reader;
pub mod record;
pub mod writer;

use std::io::{Read, Write};
use std::convert::From;
use std::fmt;
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub use record::{NO_DATA, Shape, PatchType};
pub use record::{HasShapeType, ReadableShape, MultipartShape, MultipointShape};
pub use record::{Point, PointM, PointZ};
pub use record::{Polyline, PolylineM, PolylineZ};
pub use record::{Polygon, PolygonM, PolygonZ};
pub use record::{Multipoint, MultipointM, MultipointZ};
pub use record::Multipatch;
pub use reader::Reader;


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
        Self::from(code).ok_or(Error::InvalidShapeType(code))
    }

    pub(crate) fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), std::io::Error> {
        dest.write_i32::<LittleEndian>(*self as i32)?;
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
            00 => Some(ShapeType::NullShape),
            01 => Some(ShapeType::Point),
            03 => Some(ShapeType::Polyline),
            05 => Some(ShapeType::Polygon),
            08 => Some(ShapeType::Multipoint),
            11 => Some(ShapeType::PointZ),
            13 => Some(ShapeType::PolylineZ),
            15 => Some(ShapeType::PolygonZ),
            18 => Some(ShapeType::MultipointZ),
            21 => Some(ShapeType::PointM),
            23 => Some(ShapeType::PolylineM),
            25 => Some(ShapeType::PolygonM),
            28 => Some(ShapeType::MultipointM),
            31 => Some(ShapeType::Multipatch),
            _ => None
        }
    }

    /// Returns whether the ShapeType has the third dimension Z
    pub fn has_z(&self) -> bool {
        match self {
            ShapeType::PointZ |
            ShapeType::PolylineZ |
            ShapeType::PolygonZ |
            ShapeType::MultipointZ => true,
            _ => false
        }
    }

    /// Returns whether the ShapeType has the optional measure dimension
    pub fn has_m(&self) -> bool {
        match self {
            ShapeType::PointZ |
            ShapeType::PolylineZ |
            ShapeType::PolygonZ |
            ShapeType::MultipointZ |
            ShapeType::PointM |
            ShapeType::PolylineM |
            ShapeType::PolygonM |
            ShapeType::MultipointM => true,
            _ => false
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
            ShapeType::Multipatch => write!(f, "Multipatch")
        }
    }
}

#[macro_export]
macro_rules! have_same_len_as {
    ($val:expr, $y:expr) => (
        $val == $y.len()
    );
    ($val:expr, $x:expr, $($y:expr),+) => (
        ($x.len() == $val) &&  have_same_len_as!($val, $($y),+)
    );
}


#[macro_export]
macro_rules! all_have_same_len {
    ($x:expr, $($y:expr),+) => (
        have_same_len_as!($x.len(), $($y),+)
    )
}

/// Function to read all the Shapes in a file.
///
/// Returns a `Vec<Shape>` which means that you will have to `match`
/// the individual [Shape](enum.Shape.html) contained inside the `Vec` to get the actual `struct`
///
/// Useful if you don't know in advance (at compile time) which kind of
/// shape the file contains
pub fn read<T: AsRef<Path>>(path: T) -> Result<Vec<Shape>, Error> {
    let reader = Reader::from_path(path)?;
    reader.read()
}

/// Function to read all the Shapes in a file as a certain type
///
/// Fails and return `Err(Error:MismatchShapeType)`
///
/// For example if you try to `read_as::<shapefile::PolylineZ>("polylines.shp")`
/// when the "polylines.shp" file Only contains `Polyline` the function will fail.
///
/// If the reading is successful Returned `Vec<S:ActualShape>>`is a vector of actual structs
/// Useful if you know in at compile time which kind of shape you expect the file to have
pub fn read_as<T: AsRef<Path>, S: ReadableShape>(path: T) -> Result<Vec<S::ActualShape>, Error> {
    let reader = Reader::from_path(path)?;
    reader.read_as::<S>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_type_from_wrong_code() {
        assert_eq!(ShapeType::from(128), None);
    }

    #[test]
    fn they_do_not_have_same_len() {
        let x = vec![0, 0, 0, 0];
        let y = vec![0, 0, 0, 0];
        let z = vec![0, 0, 0, 0, 0];

        assert_eq!(all_have_same_len!(x, y, z), false);
    }

    #[test]
    fn they_have_same_len() {
        let x = vec![0, 0, 0, 0];
        let y = vec![0, 0, 0, 0];
        let z = vec![0, 0, 0, 0];

        assert_eq!(all_have_same_len!(x, y, z), true);
    }
}
