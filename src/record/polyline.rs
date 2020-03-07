//! Module with the definition of Polyline, PolylineM, PolylineZ

use std::fmt;
use std::io::{Read, Write};
use std::mem::size_of;


use record::io::*;
use record::traits::{GrowablePoint, ShrinkablePoint};
use record::{ConcreteReadableShape};
use record::{GenericBBox};
use record::{EsriShape, HasShapeType, WritableShape};
use record::{Point, PointM, PointZ};
use {Error, ShapeType};

#[cfg(feature = "geo-types")]
use geo_types;
#[cfg(feature = "geo-types")]
use std::convert::TryFrom;

/// Generic struct to create Polyline; PolylineM, PolylineZ
///
/// Polylines can have multiple parts.
///
/// To create a polyline with only one part use [`new`],
/// to create a polyline with multiple parts use [`with_parts`]
///
/// [`new`]: #method.new
/// [`with_parts`]: #method.with_parts
#[derive(Debug, Clone, PartialEq)]
pub struct GenericPolyline<PointType> {
    pub(crate) bbox: GenericBBox<PointType>,
    pub(crate) parts: Vec<Vec<PointType>>,
}

/// Creating a Polyline
impl<PointType: ShrinkablePoint + GrowablePoint + Copy> GenericPolyline<PointType> {
    /// # Examples
    ///
    /// Polyline with single part
    /// ```
    /// use shapefile::{Point, Polyline};
    /// let points = vec![
    ///     Point::new(1.0, 1.0),
    ///     Point::new(2.0, 2.0),
    /// ];
    /// let poly = Polyline::new(points);
    /// ```
    ///
    pub fn new(points: Vec<PointType>) -> Self {
        Self {
            bbox: GenericBBox::<PointType>::from_points(&points),
            parts: vec![points]
        }
    }

    /// # Examples
    ///
    /// Polyline with multiple parts
    /// ```
    /// use shapefile::{Point, Polyline};
    /// let first_part = vec![
    ///     Point::new(1.0, 1.0),
    ///     Point::new(2.0, 2.0),
    /// ];
    ///
    /// let second_part = vec![
    ///     Point::new(3.0, 1.0),
    ///     Point::new(5.0, 6.0),
    /// ];
    ///
    /// let third_part = vec![
    ///     Point::new(17.0, 15.0),
    ///     Point::new(18.0, 19.0),
    ///     Point::new(20.0, 19.0),
    /// ];
    /// let poly = Polyline::with_parts(vec![first_part, second_part, third_part]);
    /// ```
    ///
    pub fn with_parts(parts: Vec<Vec<PointType>>) -> Self {
        Self {
            bbox: GenericBBox::<PointType>::from_parts(&parts),
            parts
        }
    }
}

impl<PointType> GenericPolyline<PointType> {
    /// Returns the bounding box associated to the polyline
    pub fn bbox(&self) -> &GenericBBox<PointType> {
        &self.bbox
    }

    pub fn parts(&self) -> &Vec<Vec<PointType>> {
        &self.parts
    }

    pub fn part(&self, index: usize) -> Option<&Vec<PointType>> {
        self.parts.get(index)
    }

    pub fn total_point_count(&self) -> usize {
        self.parts.iter().map(|part| part.len()).sum()
    }
}


pub type Polyline = GenericPolyline<Point>;

impl Polyline {
    pub(crate) fn size_of_record(num_points: i32, num_parts: i32) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>(); // BBOX
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); // num points
        size += size_of::<i32>() * num_parts as usize;
        size += size_of::<Point>() * num_points as usize;
        size
    }
}

impl fmt::Display for Polyline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Polyline({} parts)",
            self.parts.len()
        )
    }
}

impl HasShapeType for Polyline {
    fn shapetype() -> ShapeType {
        ShapeType::Polyline
    }
}

impl ConcreteReadableShape for Polyline {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        let rdr = MultiPartShapeReader::<Point, T>::new(source)?;
        if record_size != Self::size_of_record(rdr.num_points, rdr.num_parts) as i32 {
            Err(Error::InvalidShapeRecordSize)
        } else {
            rdr.read_xy()
                .map_err(|io_err| Error::IoError(io_err))
                .and_then(|rdr| Ok(Self{
                    bbox: rdr.bbox,
                    parts: rdr.parts
                }))
        }
    }
}

impl WritableShape for Polyline {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += size_of::<i32>();
        size += size_of::<i32>() * self.parts.len();
        size += 2 * size_of::<f64>() * self.total_point_count();
        size
    }

    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error> {
        let parts_iter = self.parts.iter().map(|part| part.as_slice());
        let writer = MultiPartShapeWriter::new(&self.bbox, parts_iter, dest);
        writer.write_point_shape()?;
        Ok(())
    }
}

impl EsriShape for Polyline {
    fn x_range(&self) -> [f64; 2] {
        self.bbox.x_range()
    }

    fn y_range(&self) -> [f64; 2] {
        self.bbox.y_range()
    }
}

/*
 * PolylineM
 */

pub type PolylineM = GenericPolyline<PointM>;

impl PolylineM {
    pub(crate) fn size_of_record(num_points: i32, num_parts: i32, is_m_used: bool) -> usize {
        let mut size = Polyline::size_of_record(num_points, num_parts);
        if is_m_used {
            size += 2 * size_of::<f64>(); // MRange
            size += num_points as usize * size_of::<f64>(); // M
        }
        size
    }
}

impl fmt::Display for PolylineM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PolylineM({} parts)",
            self.parts.len()
        )
    }
}

impl HasShapeType for PolylineM {
    fn shapetype() -> ShapeType {
        ShapeType::PolylineM
    }
}

impl ConcreteReadableShape for PolylineM {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        let rdr= MultiPartShapeReader::<PointM, T>::new(source)?;

        let record_size_with_m = Self::size_of_record(rdr.num_points, rdr.num_parts, true) as i32;
        let record_size_without_m = Self::size_of_record(rdr.num_points, rdr.num_parts, false) as i32;

        if (record_size != record_size_with_m) && (record_size != record_size_without_m) {
           Err(Error::InvalidShapeRecordSize)
        } else {
            rdr.read_xy()
                .and_then(|rdr| rdr.read_ms_if(record_size == record_size_with_m))
                .map_err(|io_err| Error::IoError(io_err))
                .and_then(|rdr| Ok(Self {bbox: rdr.bbox, parts: rdr.parts }))
        }
    }
}

impl WritableShape for PolylineM {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
        size += 3 * size_of::<f64>() * self.total_point_count();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error> {
        let parts_iter = self.parts.iter().map(|part| part.as_slice());
        let writer = MultiPartShapeWriter::new(&self.bbox, parts_iter, dest);
        writer.write_point_m_shape()?;
        Ok(())
    }
}

impl EsriShape for PolylineM {
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
 * PolylineZ
 */

pub type PolylineZ = GenericPolyline<PointZ>;

impl PolylineZ {
    pub(crate) fn size_of_record(num_points: i32, num_parts: i32, is_m_used: bool) -> usize {
        let mut size = Polyline::size_of_record(num_points, num_parts);
        size += 2 * size_of::<f64>(); // ZRange
        size += num_points as usize * size_of::<f64>(); // Z
        if is_m_used {
            size += 2 * size_of::<f64>(); // MRange
            size += num_points as usize * size_of::<f64>(); // M
        }
        size
    }
}

impl fmt::Display for PolylineZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PolylineZ({} parts)",
            self.parts.len()
        )
    }
}

impl HasShapeType for PolylineZ {
    fn shapetype() -> ShapeType {
        ShapeType::PolylineZ
    }
}

impl ConcreteReadableShape for PolylineZ {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        let rdr = MultiPartShapeReader::<PointZ, T>::new(source)?;

        let record_size_with_m = Self::size_of_record(rdr.num_points, rdr.num_parts, true) as i32;
        let record_size_without_m = Self::size_of_record(rdr.num_points, rdr.num_parts, false) as i32;

        if (record_size != record_size_with_m) && (record_size != record_size_without_m) {
            Err(Error::InvalidShapeRecordSize)
        } else {
            rdr
                .read_xy()
                .and_then(|rdr| rdr.read_zs())
                .and_then(|rdr| rdr.read_ms_if(record_size == record_size_with_m))
                .map_err(|err| Error::IoError(err))
                .and_then(|rdr| Ok(Self { bbox: rdr.bbox, parts: rdr.parts }))
        }
    }
}

impl WritableShape for PolylineZ {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
        size += 4 * size_of::<f64>() * self.total_point_count();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error> {
        let parts_iter = self.parts.iter().map(|part| part.as_slice());
        let writer = MultiPartShapeWriter::new(&self.bbox, parts_iter, dest);
        writer.write_point_z_shape()?;
        Ok(())
    }
}

impl EsriShape for PolylineZ {
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


#[cfg(feature = "geo-types")]
impl<PointType> From<GenericPolyline<PointType>> for geo_types::MultiLineString<f64>
    where
        PointType: Copy,
        geo_types::Coordinate<f64>: From<PointType>,
{
    fn from(polyline: GenericPolyline<PointType>) -> Self {
        use std::iter::FromIterator;
        let mut lines =
            Vec::<geo_types::LineString<f64>>::with_capacity(polyline.parts_indices().len());
        for parts in polyline.parts() {
            let line: Vec<geo_types::Coordinate<f64>> = parts
                .iter()
                .map(|point| geo_types::Coordinate::<f64>::from(*point))
                .collect();
            lines.push(line.into());
        }
        geo_types::MultiLineString::<f64>::from_iter(lines.into_iter())
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::Line<f64>> for GenericPolyline<PointType>
    where
        PointType: From<geo_types::Point<f64>> + HasXY,
{
    fn from(line: geo_types::Line<f64>) -> Self {
        let (p1, p2) = line.points();
        Self::new(vec![PointType::from(p1), PointType::from(p2)])
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::LineString<f64>> for GenericPolyline<PointType>
    where
        PointType: From<geo_types::Coordinate<f64>> + HasXY,
{
    fn from(line: geo_types::LineString<f64>) -> Self {
        let points: Vec<PointType> = line.into_iter().map(|p| PointType::from(p)).collect();
        Self::new(points)
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::MultiLineString<f64>> for GenericPolyline<PointType>
    where
        PointType: From<geo_types::Coordinate<f64>> + HasXY,
{
    fn from(mls: geo_types::MultiLineString<f64>) -> Self {
        let mut points = Vec::<PointType>::new();
        let mut point_index: i32 = 0;
        let mut parts = Vec::<i32>::new();
        for line_string in mls {
            parts.push(point_index);
            for point in line_string {
                points.push(point.into());
            }
            point_index += points.len() as i32;
        }
        let bbox = BBox::from_points(&points);
        Self {
            bbox,
            points,
            parts,
        }
    }
}
