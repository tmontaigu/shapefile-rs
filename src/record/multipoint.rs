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
use record::traits::{GrowablePoint, MultipointShape, ShrinkablePoint};
use record::EsriShape;
use record::{ConcreteReadableShape, GenericBBox};
use record::{HasShapeType, WritableShape};
use record::{Point, PointM, PointZ};
use {Error, ShapeType};

#[cfg(feature = "geo-types")]
use geo_types;

/// Generic struct to create the Multipoint, MultipointM, MultipointZ types
///
/// Multipoints are a collection of... multiple points,
/// they can be created from [`Vec`] of points using the [`From`] trait
/// or using the [`new`] method.
///
/// `Multipoint` shapes only offers non-mutable access to the points data,
/// to be able to mutate it you have to move the points data out of the struct.
///
/// ```
/// use shapefile::{Multipoint, Point};
/// let multipoint = Multipoint::from(vec![
///     Point::new(1.0, 1.0),
///     Point::new(2.0, 2.0),
/// ]);
///
/// let points: Vec<Point> = multipoint.into();
/// assert_eq!(points.len(), 2);
///
/// ```
///
/// [`new`]: #method.new
#[derive(Debug, Clone, PartialEq)]
pub struct GenericMultipoint<PointType> {
    pub(crate) bbox: GenericBBox<PointType>,
    pub(crate) points: Vec<PointType>,
}

impl<PointType: ShrinkablePoint + GrowablePoint + Copy> GenericMultipoint<PointType> {
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
        let bbox = GenericBBox::<PointType>::from_points(&points);
        Self { bbox, points }
    }
}

impl<PointType> GenericMultipoint<PointType> {
    /// Returns the bbox
    ///
    /// # Example
    ///
    ///
    /// ```
    /// use shapefile::{MultipointZ, PointZ, NO_DATA};
    /// let multipointz = MultipointZ::new(vec![
    ///     PointZ::new(1.0, 4.0, 1.2, 4.2),
    ///     PointZ::new(2.0, 6.0, 4.0, 13.37),
    /// ]);
    ///
    /// let bbox = multipointz.bbox();
    /// assert_eq!(bbox.min.x, 1.0);
    /// assert_eq!(bbox.max.x, 2.0);
    /// assert_eq!(bbox.m_range(), [4.2, 13.37])
    /// ```
    pub fn bbox(&self) -> &GenericBBox<PointType> {
        &self.bbox
    }
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

impl<PointType> From<Vec<PointType>> for GenericMultipoint<PointType>
where
    PointType: ShrinkablePoint + GrowablePoint + Copy,
{
    fn from(points: Vec<PointType>) -> Self {
        Self::new(points)
    }
}

// We do this because we can't use generics:
// error[E0210]: type parameter `PointType` must be used as the type parameter for some local type
// (e.g., `MyStruct<PointType>`)
macro_rules! impl_from_multipoint_to_vec_for_point_type {
    ($PointType:ty) => {
        impl From<GenericMultipoint<$PointType>> for Vec<$PointType> {
            fn from(multipoints: GenericMultipoint<$PointType>) -> Self {
                multipoints.points
            }
        }
    };
}

impl_from_multipoint_to_vec_for_point_type!(Point);
impl_from_multipoint_to_vec_for_point_type!(PointM);
impl_from_multipoint_to_vec_for_point_type!(PointZ);

#[cfg(feature = "geo-types")]
impl<PointType> From<GenericMultipoint<PointType>> for geo_types::MultiPoint<f64>
where
    geo_types::Point<f64>: From<PointType>,
{
    fn from(multi_points: GenericMultipoint<PointType>) -> Self {
        multi_points
            .points
            .into_iter()
            .map(|p| geo_types::Point::from(p))
            .collect::<Vec<geo_types::Point<f64>>>()
            .into()
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::MultiPoint<f64>> for GenericMultipoint<PointType>
where
    PointType: From<geo_types::Point<f64>> + HasXY,
{
    fn from(mp: geo_types::MultiPoint<f64>) -> Self {
        let points = mp.into_iter().map(|p| p.into()).collect();
        Self::new(points)
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
        let mut bbox = GenericBBox::<Point>::default();
        bbox_read_xy_from(&mut bbox, source)?;

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

    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error> {
        bbox_write_xy_to(&self.bbox, dest)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;
        for point in self.points {
            dest.write_f64::<LittleEndian>(point.x)?;
            dest.write_f64::<LittleEndian>(point.y)?;
        }
        Ok(())
    }
}

impl EsriShape for Multipoint {
    fn x_range(&self) -> [f64; 2] {
        self.bbox.x_range()
    }

    fn y_range(&self) -> [f64; 2] {
        self.bbox.y_range()
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
        let mut bbox = GenericBBox::<PointM>::default();
        bbox_read_xy_from(&mut bbox, source)?;

        let num_points = source.read_i32::<LittleEndian>()?;

        let size_with_m = Self::size_of_record(num_points, true) as i32;
        let size_without_m = Self::size_of_record(num_points, false) as i32;

        if (record_size != size_with_m) & (record_size != size_without_m) {
            Err(Error::InvalidShapeRecordSize)
        } else {
            let m_is_used = size_with_m == record_size;
            let mut points = read_xy_in_vec_of::<PointM, T>(&mut source, num_points)?;

            if m_is_used {
                bbox_read_m_range_from(&mut bbox, source)?;
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
        bbox_write_xy_to(&self.bbox, dest)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;

        write_points(&mut dest, &self.points)?;

        bbox_write_m_range_to(&self.bbox, dest)?;
        write_ms(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for MultipointM {
    fn x_range(&self) -> [f64; 2] {
        self.bbox.x_range()
    }

    fn y_range(&self) -> [f64; 2] {
        self.bbox.y_range()
    }

    fn m_range(&self) -> [f64; 2] {
        self.bbox.m_range()
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
        let mut bbox = GenericBBox::<PointZ>::default();
        bbox_read_xy_from(&mut bbox, source)?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let size_with_m = Self::size_of_record(num_points, true) as i32;
        let size_without_m = Self::size_of_record(num_points, false) as i32;

        if (record_size != size_with_m) & (record_size != size_without_m) {
            Err(Error::InvalidShapeRecordSize)
        } else {
            let m_is_used = size_with_m == record_size;
            let mut points = read_xy_in_vec_of::<PointZ, T>(&mut source, num_points)?;

            bbox_read_z_range_from(&mut bbox, source)?;
            read_zs_into(&mut source, &mut points)?;

            if m_is_used {
                bbox_read_m_range_from(&mut bbox, source)?;
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
        bbox_write_xy_to(&self.bbox, dest)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;

        write_points(&mut dest, &self.points)?;

        bbox_write_z_range_to(&self.bbox, dest)?;
        write_zs(&mut dest, &self.points)?;

        bbox_write_m_range_to(&self.bbox, dest)?;
        write_ms(&mut dest, &self.points)?;

        Ok(())
    }
}

impl EsriShape for MultipointZ {
    fn x_range(&self) -> [f64; 2] {
        self.bbox.x_range()
    }

    fn y_range(&self) -> [f64; 2] {
        self.bbox.y_range()
    }

    fn z_range(&self) -> [f64; 2] {
        self.bbox.z_range()
    }

    fn m_range(&self) -> [f64; 2] {
        self.bbox.m_range()
    }
}

#[cfg(test)]
#[cfg(feature = "geo-types")]
mod tests {
    use super::*;
    use {geo_types, NO_DATA};

    #[test]
    fn test_multipoint_to_geo_types_multipoint() {
        let points = vec![Point::new(1.0, 1.0), Point::new(2.0, 2.0)];
        let shapefile_multipoint = Multipoint::new(points);
        let geo_types_multipoint = geo_types::MultiPoint::from(shapefile_multipoint);

        let mut iter = geo_types_multipoint.into_iter();
        let p1 = iter.next().unwrap();
        let p2 = iter.next().unwrap();
        assert_eq!(p1.x(), 1.0);
        assert_eq!(p1.y(), 1.0);

        assert_eq!(p2.x(), 2.0);
        assert_eq!(p2.y(), 2.0);
    }

    #[test]
    fn test_multipoint_m_to_geo_types_multipoint() {
        let points = vec![
            PointM::new(120.0, 56.0, 42.2),
            PointM::new(6.0, 18.7, 462.54),
        ];
        let shapefile_multipoint = MultipointM::new(points);
        let geo_types_multipoint = geo_types::MultiPoint::from(shapefile_multipoint);

        let mut iter = geo_types_multipoint.into_iter();
        let p1 = iter.next().unwrap();
        let p2 = iter.next().unwrap();
        assert_eq!(p1.x(), 120.0);
        assert_eq!(p1.y(), 56.0);

        assert_eq!(p2.x(), 6.0);
        assert_eq!(p2.y(), 18.7);

        let geo_types_multipoint: geo_types::MultiPoint<_> = vec![p1, p2].into();
        let shapefile_multipoint = MultipointM::from(geo_types_multipoint);

        assert_eq!(shapefile_multipoint.points[0].x, 120.0);
        assert_eq!(shapefile_multipoint.points[0].y, 56.0);
        assert_eq!(shapefile_multipoint.points[0].m, NO_DATA);

        assert_eq!(shapefile_multipoint.points[1].x, 6.0);
        assert_eq!(shapefile_multipoint.points[1].y, 18.7);
        assert_eq!(shapefile_multipoint.points[0].m, NO_DATA);
    }

    #[test]
    fn test_multipoint_z_to_geo_types_multipoint() {
        let points = vec![
            PointZ::new(1.0, 1.0, 17.0, 18.0),
            PointZ::new(2.0, 2.0, 15.0, 16.0),
        ];
        let shapefile_multipoint = MultipointZ::new(points);
        let geo_types_multipoint = geo_types::MultiPoint::from(shapefile_multipoint);

        let mut iter = geo_types_multipoint.into_iter();
        let p1 = iter.next().unwrap();
        let p2 = iter.next().unwrap();
        assert_eq!(p1.x(), 1.0);
        assert_eq!(p1.y(), 1.0);

        assert_eq!(p2.x(), 2.0);
        assert_eq!(p2.y(), 2.0);

        let geo_types_multipoint: geo_types::MultiPoint<_> = vec![p1, p2].into();
        let shapefile_multipoint = MultipointZ::from(geo_types_multipoint);

        assert_eq!(shapefile_multipoint.points[0].x, 1.0);
        assert_eq!(shapefile_multipoint.points[0].y, 1.0);
        assert_eq!(shapefile_multipoint.points[0].z, 0.0);
        assert_eq!(shapefile_multipoint.points[0].m, NO_DATA);

        assert_eq!(shapefile_multipoint.points[1].x, 2.0);
        assert_eq!(shapefile_multipoint.points[1].y, 2.0);
        assert_eq!(shapefile_multipoint.points[0].z, 0.0);
        assert_eq!(shapefile_multipoint.points[0].m, NO_DATA);
    }
}
