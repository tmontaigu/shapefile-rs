//! Module with the definition of Point, PointM and PointZ

use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use record::EsriShape;
use std::mem::size_of;
use ShapeType;

use super::Error;
use record::ConcreteReadableShape;
use record::{is_no_data, BBox, HasShapeType, WritableShape};
use std::fmt;

/// Point with only `x` and `y` coordinates
#[derive(PartialEq, Debug, Default, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Creates a new point
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::Point;
    /// let point = Point::new(1.0, 42.0);
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 42.0);
    /// ```
    ///
    /// ```
    /// use shapefile::Point;
    /// let point = Point::default();
    /// assert_eq!(point.x, 0.0);
    /// assert_eq!(point.y, 0.0);
    /// ```
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl HasShapeType for Point {
    fn shapetype() -> ShapeType {
        ShapeType::Point
    }
}

impl ConcreteReadableShape for Point {
    fn read_shape_content<T: Read>(source: &mut T) -> Result<Self::ActualShape, Error> {
        let x = source.read_f64::<LittleEndian>()?;
        let y = source.read_f64::<LittleEndian>()?;
        Ok(Self { x, y })
    }
}

impl WritableShape for Point {
    fn size_in_bytes(&self) -> usize {
        2 * size_of::<f64>()
    }

    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error> {
        dest.write_f64::<LittleEndian>(self.x)?;
        dest.write_f64::<LittleEndian>(self.y)?;
        Ok(())
    }
}

impl EsriShape for Point {
    fn bbox(&self) -> BBox {
        BBox {
            xmin: self.x,
            ymin: self.y,
            xmax: self.x,
            ymax: self.y,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point(x: {}, y: {})", self.x, self.y)
    }
}

/*
 * PointM
 */


/// Point with `x`, `y`, `m`
#[derive(PartialEq, Debug, Default, Copy, Clone)]
pub struct PointM {
    pub x: f64,
    pub y: f64,
    pub m: f64,
}

impl PointM {
    /// Creates a new pointM
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::{PointM, NO_DATA};
    /// let point = PointM::new(1.0, 42.0, NO_DATA);
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 42.0);
    /// assert_eq!(point.m, NO_DATA);
    /// ```
    pub fn new(x: f64, y: f64, m: f64) -> Self {
        Self { x, y, m }
    }
}

impl HasShapeType for PointM {
    fn shapetype() -> ShapeType {
        ShapeType::PointM
    }
}

impl ConcreteReadableShape for PointM {
    fn read_shape_content<T: Read>(mut source: &mut T) -> Result<Self::ActualShape, Error> {
        let point = Point::read_shape_content(&mut source)?;
        let m = source.read_f64::<LittleEndian>()?;
        Ok(Self {
            x: point.x,
            y: point.y,
            m,
        })
    }
}

impl WritableShape for PointM {
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

impl fmt::Display for PointM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point(x: {}, y: {}, m: {})", self.x, self.y, self.m)
    }
}

impl EsriShape for PointM {
    fn bbox(&self) -> BBox {
        BBox {
            xmin: self.x,
            ymin: self.y,
            xmax: self.x,
            ymax: self.y,
        }
    }

    fn m_range(&self) -> [f64; 2] {
        if is_no_data(self.m) {
            [0.0, 0.0]
        } else {
            [self.m, self.m]
        }
    }
}

impl fmt::Display for PointZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Point(x: {}, y: {}, z: {}, m: {})",
            self.x, self.y, self.z, self.m
        )
    }
}

/*
 * PointZ
 */

/// Point with `x`, `y`, `m`, `z`
#[derive(PartialEq, Debug, Default, Copy, Clone)]
pub struct PointZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: f64,
}

impl PointZ {
    /// Creates a new pointZ
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::{PointZ, NO_DATA};
    /// let point = PointZ::new(1.0, 42.0, 13.37, NO_DATA);
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 42.0);
    /// assert_eq!(point.z, 13.37);
    /// assert_eq!(point.m, NO_DATA);
    /// ```
    pub fn new(x: f64, y: f64, z: f64, m: f64) -> Self {
        Self { x, y, z, m }
    }
}

impl HasShapeType for PointZ {
    fn shapetype() -> ShapeType {
        ShapeType::PointZ
    }
}

impl ConcreteReadableShape for PointZ {
    fn read_shape_content<T: Read>(mut source: &mut T) -> Result<Self::ActualShape, Error> {
        let point = Point::read_shape_content(&mut source)?;
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

impl WritableShape for PointZ {
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

impl EsriShape for PointZ {
    fn bbox(&self) -> BBox {
        BBox {
            xmin: self.x,
            ymin: self.y,
            xmax: self.x,
            ymax: self.y,
        }
    }

    fn z_range(&self) -> [f64; 2] {
        [self.z, self.z]
    }

    fn m_range(&self) -> [f64; 2] {
        if is_no_data(self.m) {
            [0.0, 0.0]
        } else {
            [self.m, self.m]
        }
    }
}
