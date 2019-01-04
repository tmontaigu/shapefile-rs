use std::io::{Read, Write};

use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
use record::EsriShape;
use ShapeType;
use std::mem::size_of;

use super::Error;

pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn read_from<T: Read>(source: &mut T) -> Result<Self, std::io::Error> {
        let x = source.read_f64::<LittleEndian>()?;
        let y = source.read_f64::<LittleEndian>()?;
        Ok(Self { x, y })
    }
}

impl EsriShape for Point {
    fn shapetype(&self) -> ShapeType {
        ShapeType::Point
    }

    fn size_in_bytes(&self) -> usize {
        2 * size_of::<f64>()
    }

    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error> {
        dest.write_f64::<LittleEndian>(self.x)?;
        dest.write_f64::<LittleEndian>(self.y)?;
        Ok(())
    }
}

impl Default for Point {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
    }
}

pub struct PointM {
    pub x: f64,
    pub y: f64,
    pub m: f64,
}

impl PointM {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Self, std::io::Error> {
        let point = Point::read_from(&mut source)?;
        let m = source.read_f64::<LittleEndian>()?;
        Ok(Self {
            x: point.x,
            y: point.y,
            m,
        })
    }
}

impl EsriShape for PointM {
    fn shapetype(&self) -> ShapeType {
        ShapeType::PointM
    }

    fn size_in_bytes(&self) -> usize {
        3 * size_of::<f64>()
    }

    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error> {
        dest.write_f64::<LittleEndian>(self.x)?;
        dest.write_f64::<LittleEndian>(self.y)?;
        dest.write_f64::<LittleEndian>(self.m)?;
        Ok(())
    }
}


pub struct PointZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: f64,
}

impl PointZ {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Self, std::io::Error> {
        let point = Point::read_from(&mut source)?;
        let z = source.read_f64::<LittleEndian>()?;
        let m = source.read_f64::<LittleEndian>()?;
        Ok(Self {
            x: point.x,
            y: point.y,
            z,
            m,
        })
    }
}

impl EsriShape for PointZ {
    fn shapetype(&self) -> ShapeType {
        ShapeType::PointZ
    }

    fn size_in_bytes(&self) -> usize {
        4 * size_of::<f64>()
    }

    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error> {
        dest.write_f64::<LittleEndian>(self.x)?;
        dest.write_f64::<LittleEndian>(self.y)?;
        dest.write_f64::<LittleEndian>(self.z)?;
        dest.write_f64::<LittleEndian>(self.m)?;
        Ok(())
    }
}