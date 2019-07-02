//! Module with the definition of Multipoint(M, Z)
//!
//! All three variant of Multipoint Shape (Multipoint, MultipointM, MultipointZ)
//! are specialization of the `GenericMultipoint`
//!
//! The `GenericMultipoint` Shape implements the [MultipointShape](../trait.MultipointShape.html) trait
//! which means that to access the points of a multipoint you will have to use the
//! [points](../trait.MultipointShape.html#method.points) method
use std::fmt;
use std::io::{Read, Write};
use std::mem::size_of;
use std::slice::SliceIndex;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use record::io::*;
use record::traits::{HasXY, MultipointShape};
use record::ConcreteReadableShape;
use record::{BBox, EsriShape};
use record::{HasShapeType, WritableShape};
use record::{Point, PointM, PointZ};
use {Error, ShapeType};

/// Generic struct to create the Multipoint, MultipointM, MultipointZ types
pub struct GenericMultipoint<PointType> {
    /// The 2D bounding box
    pub bbox: BBox,
    pub points: Vec<PointType>,
}

impl<PointType> MultipointShape<PointType> for GenericMultipoint<PointType> {
    fn point<I: SliceIndex<[PointType]>>(
        &self,
        index: I,
    ) -> Option<&<I as SliceIndex<[PointType]>>::Output> {
        self.points.get(index)
    }
    fn points(&self) -> &[PointType] {
        &self.points
    }
}

impl<PointType: HasXY> GenericMultipoint<PointType> {
    /// Creates a new Multipoint shape
    ///
    /// # Examples
    ///
    /// Creating Multipoint
    /// ```
    /// use shapefile::{Multipoint, Point};
    /// let points = vec![
    ///     Point::new(1.0, 1.0),
    ///     Point::new(2.0, 2.0),
    /// ];
    /// let multipoint = Multipoint::new(points);
    /// ```
    ///
    /// Creating a MultipointM
    /// ```
    /// use shapefile::{MultipointM, PointM, NO_DATA};
    /// let points = vec![
    ///     PointM::new(1.0, 1.0, NO_DATA),
    ///     PointM::new(2.0, 2.0, NO_DATA),
    /// ];
    /// let multipointm = MultipointM::new(points);
    /// ```
    ///
    /// Creating a MultipointZ
    /// ```
    /// use shapefile::{MultipointZ, PointZ, NO_DATA};
    /// let points = vec![
    ///     PointZ::new(1.0, 1.0, 1.0, NO_DATA),
    ///     PointZ::new(2.0, 2.0, 2.0, NO_DATA),
    /// ];
    /// let multipointz = MultipointZ::new(points);
    /// ```

    pub fn new(points: Vec<PointType>) -> Self {
        let bbox = BBox::from_points(&points);
        Self { bbox, points }
    }
}

/*
 * Multipoint
 */

/// Specialization of the `GenericMultipoint` struct to represent a `Multipoint` shape
/// ( collection of [Point](../point/struct.Point.html))
pub type Multipoint = GenericMultipoint<Point>;

impl Multipoint {
    pub(crate) fn size_of_record(num_points: i32) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>(); // BBOX
        size += size_of::<i32>(); // num points
        size += size_of::<Point>() * num_points as usize;
        size
    }
}

impl fmt::Display for Multipoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Multipoint({} points)", self.points.len())
    }
}

impl HasShapeType for Multipoint {
    fn shapetype() -> ShapeType {
        ShapeType::Multipoint
    }
}

impl ConcreteReadableShape for Multipoint {
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_points = source.read_i32::<LittleEndian>()?;
        if record_size == Self::size_of_record(num_points) as i32 {
            let points = read_xy_in_vec_of::<Point, T>(&mut source, num_points)?;
            Ok(Self { bbox, points })
        } else {
            Err(Error::InvalidShapeRecordSize)
        }
    }
}

impl WritableShape for Multipoint {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>(); // BBOX
        size += size_of::<i32>(); // num points
        size += 2 * size_of::<f64>() * self.points.len();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;
        for point in self.points {
            dest.write_f64::<LittleEndian>(point.x)?;
            dest.write_f64::<LittleEndian>(point.y)?;
        }
        Ok(())
    }
}

impl EsriShape for Multipoint {
    fn bbox(&self) -> BBox {
        self.bbox
    }
}

/*
 * MultipointM
 */

/// Specialization of the `GenericMultipoint` struct to represent a `MultipointM` shape
/// ( collection of [PointM](../point/struct.PointM.html))
pub type MultipointM = GenericMultipoint<PointM>;

impl MultipointM {
    pub(crate) fn size_of_record(num_points: i32, is_m_used: bool) -> usize {
        let mut size = Multipoint::size_of_record(num_points);
        if is_m_used {
            size += 2 * size_of::<f64>(); // M Range
            size += size_of::<f64>() * num_points as usize; // M
        }
        size
    }
}

impl fmt::Display for MultipointM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MultipointM({} points)", self.points.len())
    }
}

impl HasShapeType for MultipointM {
    fn shapetype() -> ShapeType {
        ShapeType::MultipointM
    }
}

impl ConcreteReadableShape for MultipointM {
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let bbox = BBox::read_from(&mut source)?;

        let num_points = source.read_i32::<LittleEndian>()?;

        let size_with_m = Self::size_of_record(num_points, true) as i32;
        let size_without_m = Self::size_of_record(num_points, false) as i32;

        if (record_size != size_with_m) & (record_size != size_without_m) {
            Err(Error::InvalidShapeRecordSize)
        } else {
            let m_is_used = size_with_m == record_size;
            let mut points = read_xy_in_vec_of::<PointM, T>(&mut source, num_points)?;

            if m_is_used {
                let _m_range = read_range(&mut source)?;
                read_ms_into(&mut source, &mut points)?;
            }
            Ok(Self { bbox, points })
        }
    }
}

impl WritableShape for MultipointM {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += 3 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;

        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for MultipointM {
    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn m_range(&self) -> [f64; 2] {
        calc_m_range(&self.points)
    }
}

/*
 * MultipointZ
 */

/// Specialization of the `GenericMultipoint` struct to represent a `MultipointZ` shape
/// ( collection of [PointZ](../point/struct.PointZ.html))
pub type MultipointZ = GenericMultipoint<PointZ>;

impl fmt::Display for MultipointZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MultipointZ({} points)", self.points.len())
    }
}
impl MultipointZ {
    pub(crate) fn size_of_record(num_points: i32, is_m_used: bool) -> usize {
        let mut size = Multipoint::size_of_record(num_points);
        size += 2 * size_of::<f64>(); // Z Range
        size += size_of::<f64>() * num_points as usize; // Z

        if is_m_used {
            size += 2 * size_of::<f64>(); // M Range
            size += size_of::<f64>() * num_points as usize; // M
        }

        size
    }
}

impl HasShapeType for MultipointZ {
    fn shapetype() -> ShapeType {
        ShapeType::MultipointZ
    }
}

impl ConcreteReadableShape for MultipointZ {
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let size_with_m = Self::size_of_record(num_points, true) as i32;
        let size_without_m = Self::size_of_record(num_points, false) as i32;

        if (record_size != size_with_m) & (record_size != size_without_m) {
            Err(Error::InvalidShapeRecordSize)
        } else {
            let m_is_used = size_with_m == record_size;
            let mut points = read_xy_in_vec_of::<PointZ, T>(&mut source, num_points)?;

            let _z_range = read_range(&mut source)?;
            read_zs_into(&mut source, &mut points)?;

            if m_is_used {
                let _m_range = read_range(&mut source)?;
                read_ms_into(&mut source, &mut points)?;
            }

            Ok(Self { bbox, points })
        }
    }
}

impl WritableShape for MultipointZ {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += 4 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;

        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.z_range())?;
        write_zs(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;

        Ok(())
    }
}

impl EsriShape for MultipointZ {
    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn z_range(&self) -> [f64; 2] {
        calc_z_range(&self.points)
    }

    fn m_range(&self) -> [f64; 2] {
        calc_m_range(&self.points)
    }
}
