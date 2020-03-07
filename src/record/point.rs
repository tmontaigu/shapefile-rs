//! Module with the definition of Point, PointM and PointZ

use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use record::EsriShape;
use std::mem::size_of;
use {ShapeType, NO_DATA};

use super::Error;
use record::ConcreteReadableShape;
use record::{is_no_data, HasShapeType, WritableShape};
use std::fmt;

#[cfg(feature = "geo-types")]
use geo_types;


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
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        if record_size == 2 * size_of::<f64>() as i32 {
            let x = source.read_f64::<LittleEndian>()?;
            let y = source.read_f64::<LittleEndian>()?;
            Ok(Self { x, y })
        } else {
            Err(Error::InvalidShapeRecordSize)
        }
    }
}

impl WritableShape for Point {
    fn size_in_bytes(&self) -> usize {
        2 * size_of::<f64>()
    }

    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error> {
        dest.write_f64::<LittleEndian>(self.x)?;
        dest.write_f64::<LittleEndian>(self.y)?;
        Ok(())
    }
}

impl EsriShape for Point {
    fn x_range(&self) -> [f64; 2] {
        [self.x, self.x]
    }

    fn y_range(&self) -> [f64; 2] {
        [self.y, self.y]
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point(x: {}, y: {})", self.x, self.y)
    }
}


#[cfg(feature = "geo-types")]
impl From<Point> for geo_types::Point<f64> {
    fn from(p: Point) -> Self {
        geo_types::Point::new(p.x, p.y)
    }
}

#[cfg(feature = "geo-types")]
impl From<geo_types::Point<f64>> for Point {
    fn from(p: geo_types::Point<f64>) -> Self {
        Point::new(p.x(), p.y())
    }
}

#[cfg(feature = "geo-types")]
impl From<geo_types::Coordinate<f64>> for Point {
    fn from(c: geo_types::Coordinate<f64>) -> Self {
        Point::new(c.x, c.y)
    }
}

#[cfg(feature = "geo-types")]
impl From<Point> for geo_types::Coordinate<f64> {
    fn from(p: Point) -> Self {
        geo_types::Coordinate { x: p.x, y: p.y }
    }
}

/*
 * PointM
 */

/// Point with `x`, `y`, `m`
#[derive(PartialEq, Debug, Copy, Clone)]
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
    /// use shapefile::PointM;
    /// let point = PointM::new(1.0, 42.0, 13.37);
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 42.0);
    /// assert_eq!(point.m, 13.37);
    /// ```
    ///
    /// ```
    /// use shapefile::{PointM, NO_DATA};
    /// let point = PointM::default();
    /// assert_eq!(point.x, 0.0);
    /// assert_eq!(point.y, 0.0);
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
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        if record_size == 3 * size_of::<f64>() as i32 {
            let x = source.read_f64::<LittleEndian>()?;
            let y = source.read_f64::<LittleEndian>()?;
            let m = source.read_f64::<LittleEndian>()?;
            Ok(Self { x, y, m })
        } else {
            Err(Error::InvalidShapeRecordSize)
        }
    }
}

impl WritableShape for PointM {
    fn size_in_bytes(&self) -> usize {
        3 * size_of::<f64>()
    }

    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error> {
        dest.write_f64::<LittleEndian>(self.x)?;
        dest.write_f64::<LittleEndian>(self.y)?;
        dest.write_f64::<LittleEndian>(self.m)?;
        Ok(())
    }
}

impl EsriShape for PointM {
    fn x_range(&self) -> [f64; 2] {
        [self.x, self.x]
    }

    fn y_range(&self) -> [f64; 2] {
        [self.y, self.y]
    }

    fn m_range(&self) -> [f64; 2] {
        if is_no_data(self.m) {
            [0.0, 0.0]
        } else {
            [self.m, self.m]
        }
    }
}

impl fmt::Display for PointM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if is_no_data(self.m) {
            write!(f, "Point(x: {}, y: {}, m: NO_DATA)", self.x, self.y)
        } else {
            write!(f, "Point(x: {}, y: {}, m: {})", self.x, self.y, self.m)
        }
    }
}

impl Default for PointM {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            m: NO_DATA,
        }
    }
}


#[cfg(feature = "geo-types")]
impl From<PointM> for geo_types::Point<f64> {
    fn from(p: PointM) -> Self {
        geo_types::Point::new(p.x, p.y)
    }
}

#[cfg(feature = "geo-types")]
impl From<geo_types::Point<f64>> for PointM {
    fn from(p: geo_types::Point<f64>) -> Self {
        PointM {
            x: p.x(),
            y: p.y(),
            ..Default::default()
        }
    }
}

#[cfg(feature = "geo-types")]
impl From<geo_types::Coordinate<f64>> for PointM {
    fn from(c: geo_types::Coordinate<f64>) -> Self {
        PointM::new(c.x, c.y, NO_DATA)
    }
}

#[cfg(feature = "geo-types")]
impl From<PointM> for geo_types::Coordinate<f64> {
    fn from(p: PointM) -> Self {
        geo_types::Coordinate { x: p.x, y: p.y }
    }
}

/*
 * PointZ
 */

/// Point with `x`, `y`, `m`, `z`
#[derive(PartialEq, Debug, Copy, Clone)]
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

    fn read_xyz<R: Read>(source: &mut R) -> std::io::Result<Self> {
        let x = source.read_f64::<LittleEndian>()?;
        let y = source.read_f64::<LittleEndian>()?;
        let z = source.read_f64::<LittleEndian>()?;
        Ok(Self {
            x,
            y,
            z,
            m: NO_DATA,
        })
    }
}

impl HasShapeType for PointZ {
    fn shapetype() -> ShapeType {
        ShapeType::PointZ
    }
}

impl ConcreteReadableShape for PointZ {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        if record_size == 3 * size_of::<f64>() as i32 {
            let point = Self::read_xyz(source)?;
            Ok(point)
        } else if record_size == 4 * size_of::<f64>() as i32 {
            let mut point = Self::read_xyz(source)?;
            point.m = source.read_f64::<LittleEndian>()?;
            Ok(point)
        } else {
            Err(Error::InvalidShapeRecordSize)
        }
    }
}

impl WritableShape for PointZ {
    fn size_in_bytes(&self) -> usize {
        4 * size_of::<f64>()
    }

    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error> {
        dest.write_f64::<LittleEndian>(self.x)?;
        dest.write_f64::<LittleEndian>(self.y)?;
        dest.write_f64::<LittleEndian>(self.z)?;
        dest.write_f64::<LittleEndian>(self.m)?;
        Ok(())
    }
}

impl EsriShape for PointZ {
    fn x_range(&self) -> [f64; 2] {
        [self.x, self.x]
    }

    fn y_range(&self) -> [f64; 2] {
        [self.y, self.y]
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

impl Default for PointZ {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            m: NO_DATA,
        }
    }
}

impl fmt::Display for PointZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if is_no_data(self.m) {
            write!(
                f,
                "Point(x: {}, y: {}, z: {}, m: NO_DATA)",
                self.x, self.y, self.z
            )
        } else {
            write!(
                f,
                "Point(x: {}, y: {}, z: {}, m: {})",
                self.x, self.y, self.z, self.m
            )
        }
    }
}


#[cfg(feature = "geo-types")]
impl From<PointZ> for geo_types::Point<f64> {
    fn from(p: PointZ) -> Self {
        geo_types::Point::new(p.x, p.y)
    }
}

#[cfg(feature = "geo-types")]
impl From<geo_types::Point<f64>> for PointZ {
    fn from(p: geo_types::Point<f64>) -> Self {
        PointZ {
            x: p.x(),
            y: p.y(),
            ..Default::default()
        }
    }
}

#[cfg(feature = "geo-types")]
impl From<geo_types::Coordinate<f64>> for PointZ {
    fn from(c: geo_types::Coordinate<f64>) -> Self {
        PointZ::new(c.x, c.y, 0.0, NO_DATA)
    }
}

#[cfg(feature = "geo-types")]
impl From<PointZ> for geo_types::Coordinate<f64> {
    fn from(p: PointZ) -> Self {
        geo_types::Coordinate { x: p.x, y: p.y }
    }
}

#[cfg(test)]
#[cfg(feature = "geo-types")]
mod test_geo_types {
    use super::*;
    #[test]
    fn geo_types_point_conversion() {
        let p = Point::new(14.0, 42.65);
        let gp: geo_types::Point<f64> = p.into();

        assert_eq!(gp.x(), 14.0);
        assert_eq!(gp.y(), 42.65);

        let p: Point = gp.into();
        assert_eq!(p.x, 14.0);
        assert_eq!(p.y, 42.65);
    }

    #[test]
    fn geo_types_point_m_conversion() {
        let p = PointM::new(14.0, 42.65, 652.3);
        let gp: geo_types::Point<f64> = p.into();

        assert_eq!(gp.x(), 14.0);
        assert_eq!(gp.y(), 42.65);

        let p: PointM = gp.into();
        assert_eq!(p.x, 14.0);
        assert_eq!(p.y, 42.65);
        assert_eq!(p.m, NO_DATA);
    }

    #[test]
    fn geo_types_point_z_conversion() {
        let p = PointZ::new(14.0, 42.65, 111.0, 652.3);
        let gp: geo_types::Point<f64> = p.into();

        assert_eq!(gp.x(), 14.0);
        assert_eq!(gp.y(), 42.65);

        let p: PointZ = gp.into();
        assert_eq!(p.x, 14.0);
        assert_eq!(p.y, 42.65);
        assert_eq!(p.z, 0.0);
        assert_eq!(p.m, NO_DATA);
    }
}
