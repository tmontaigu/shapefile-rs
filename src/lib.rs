extern crate byteorder;

pub mod header;
mod record;

use std::io::Read;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::convert::{From};
use std::fmt;


#[derive(Debug)]
pub enum ShpError {
    InvalidFileCode(i32),
    IoError(std::io::Error),
    InvalidShapeType(i32),
}

impl From<std::io::Error> for ShpError {
    fn from(error: std::io::Error) -> ShpError {
        ShpError::IoError(error)
    }
}

#[derive(Debug, PartialEq)]
pub enum ShapeType {
    NullShape,
    Point,
    Polyline,
    Polygon,
    Multipoint,

    PointZ,
    PolylineZ,
    PolygonZ,
    MultipointZ,

    PointM,
    PolylineM,
    PolygonM,
    MultipointM,

    Multipatch
}

impl ShapeType {
    pub fn read_from<T: Read>(source: &mut T) -> Result<ShapeType, ShpError> {
        let code = source.read_i32::<LittleEndian>()?;
        match ShapeType::from(code) {
            Some(t) => Ok(t),
            None => Err(ShpError::InvalidShapeType(code))
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shape_type_from_wrong_code() {
        assert!(ShapeType::from(128).is_none());
    }
}
