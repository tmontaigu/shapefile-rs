use std::io::{Read};

use byteorder::{ReadBytesExt, LittleEndian};

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
