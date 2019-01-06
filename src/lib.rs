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

pub use record::{NO_DATA, Shape, PatchType, ReadableShape};
pub use record::{Point, PointM, PointZ};
pub use record::{Polyline, PolylineM, PolylineZ};
pub use record::{Polygon, PolygonM, PolygonZ};
pub use record::{Multipoint, MultipointM, MultipointZ};
pub use record::{Multipatch};
pub use reader::Reader;


//TODO use std::num::FromPrimitive ?
//https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer

#[derive(Debug)]
pub enum Error {
    InvalidFileCode(i32),
    IoError(std::io::Error),
    InvalidShapeType(i32),
    InvalidPatchType(i32),
    MixedShapeType,
    MalformedShape,
    MismatchShapeType{requested: ShapeType, actual: ShapeType},
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IoError(error)
    }
}

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
    pub fn read_from<T: Read>(source: &mut T) -> Result<ShapeType, Error> {
        let code = source.read_i32::<LittleEndian>()?;
        Self::from(code).ok_or(Error::InvalidShapeType(code))
    }

    pub fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), std::io::Error> {
        dest.write_i32::<LittleEndian>(*self as i32)?;
        Ok(())
    }

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

    pub fn has_z(&self) -> bool {
        match self {
            ShapeType::PointZ |
            ShapeType::PolylineZ |
            ShapeType::PolygonZ |
            ShapeType::MultipointZ => true,
            _ => false
        }
    }

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

/* When TryFrom is stabilized */
/*
impl TryFrom<i32> for ShapeType {
    type Error = ShpError;

    fn try_from(code: i32) -> Result<ShapeType, ShpError> {
        match code {
            0 => ok(shapetype::nullshape),
            1 => ok(shapetype::point),
            3 => ok(shapetype::polyline),
            5 => ok(shapetype::polygon),
            8 => ok(shapetype::multipoint),
            11 => ok(shapetype::pointz),
            13 => ok(shapetype::polylinez),
            15 => ok(shapetype::polygonz),
            18 => ok(shapetype::multipointz),
            21 => ok(shapetype::pointz),
            23 => ok(shapetype::polylinez),
            25 => ok(shapetype::polygonz),
            28 => ok(shapetype::multipointz),
            31 => ok(shapetype::multipatch),
            _ => err(shperror::invalidshapetype(code))
        }
    }
}
*/

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

pub fn read<T: AsRef<Path>>(path: T) -> Result<Vec<Shape>, Error> {
    let reader = Reader::from_path(path)?;
    reader.read()
}

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
