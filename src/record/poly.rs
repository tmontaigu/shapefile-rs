//! Module with the definition of Polyline(M,Z) and Polygon(M,Z)

use std::fmt;
use std::io::{Read, Write};
use std::mem::size_of;
use std::slice::SliceIndex;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use record::io::*;
use record::{is_parts_array_valid, close_points_if_not_already};
use record::traits::HasXY;
use record::traits::{MultipartShape, MultipointShape};
use record::ConcreteReadableShape;
use record::{BBox, EsriShape, HasShapeType, WritableShape};
use record::{Point, PointM, PointZ};
use {Error, ShapeType};

#[cfg(feature = "geo-types")]
use geo_types;
#[cfg(feature = "geo-types")]
use std::convert::TryFrom;

pub struct GenericPolyline<PointType> {
    pub bbox: BBox,
    pub points: Vec<PointType>,
    pub parts: Vec<i32>,
}

/// Creating a Polyline
impl<PointType: HasXY> GenericPolyline<PointType> {
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
    ///
    /// assert_eq!(poly.points.len(), 2);
    /// assert_eq!(poly.parts, vec![0]);
    /// ```
    ///
    pub fn new(points: Vec<PointType>) -> Self {
        let bbox = BBox::from_points(&points);
        Self {
            bbox,
            points,
            parts: vec![0],
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
    /// let second_part = vec![
    ///     Point::new(3.0, 1.0),
    ///     Point::new(5.0, 6.0),
    /// ];
    /// let poly = Polyline::with_parts(vec![first_part, second_part]);
    ///
    /// assert_eq!(poly.points.len(), 4);
    /// assert_eq!(poly.parts, vec![0, 2]);
    /// ```
    ///
    pub fn with_parts(parts: Vec<Vec<PointType>>) -> Self {
        let mut parts_indices = Vec::<i32>::with_capacity(parts.len());
        parts_indices.push(0);
        if let Some(following_parts) = parts.get(1..) {
            for points in  following_parts {
                parts_indices.push(points.len() as i32)
            }
        }

        let num_points = parts.iter().map(|pts| pts.len()).sum();
        let mut all_points = Vec::<PointType>::with_capacity(num_points);
        for mut points in parts {
            all_points.append(&mut points);
        }

        let bbox = BBox::from_points(&all_points);
        Self {
            bbox,
            points: all_points,
            parts: parts_indices,
        }
    }
}


impl<PointType> From<GenericPolygon<PointType>> for GenericPolyline<PointType> {
    fn from(p: GenericPolygon<PointType>) -> Self {
        Self {
            bbox: p.bbox,
            points: p.points,
            parts: p.parts,
        }
    }
}

impl<PointType> From<GenericPolyline<PointType>> for GenericPolygon<PointType> {
    fn from(p: GenericPolyline<PointType>) -> Self {
        Self {
            bbox: p.bbox,
            points: p.points,
            parts: p.parts,
        }
    }
}

impl<PointType> MultipointShape<PointType> for GenericPolyline<PointType> {
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

impl<PointType> MultipartShape<PointType> for GenericPolyline<PointType> {
    fn parts_indices(&self) -> &[i32] {
        &self.parts
    }
}


#[cfg(feature = "geo-types")]
impl<PointType> From<GenericPolyline<PointType>> for geo_types::MultiLineString<f64>
    where PointType: Copy,
         geo_types::Coordinate<f64>: From<PointType>
    {
    fn from(polyline: GenericPolyline<PointType>) -> Self {
        use std::iter::FromIterator;
        let mut lines = Vec::<geo_types::LineString<f64>>::with_capacity(polyline.parts_indices().len());
        for parts in polyline.parts() {
            let line: Vec<geo_types::Coordinate<f64>> =
                parts.iter()
                    .map(|point| geo_types::Coordinate::<f64>::from(*point))
                    .collect();
            lines.push(line.into());
        }
        geo_types::MultiLineString::<f64>::from_iter(lines.into_iter())
    }
}


#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::Line<f64>> for GenericPolyline<PointType>
    where PointType: From<geo_types::Point<f64>> + HasXY
{
    fn from(line: geo_types::Line<f64>) -> Self {
        let (p1, p2) = line.points();
        Self::new(vec![PointType::from(p1), PointType::from(p2)])
    }
}


#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::LineString<f64>> for GenericPolyline<PointType>
    where PointType: From<geo_types::Coordinate<f64>> + HasXY
{
    fn from(line: geo_types::LineString<f64>) -> Self {
        let points: Vec<PointType> = line
            .into_iter()
            .map(|p| PointType::from(p))
            .collect();
        Self::new(points)
    }
}



#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::MultiLineString<f64>> for GenericPolyline<PointType>
    where PointType: From<geo_types::Coordinate<f64>> + HasXY
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
            parts
        }
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
            "Polyline({} points, {} parts)",
            self.points.len(),
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
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        if record_size != Self::size_of_record(num_points, num_parts) as i32 {
            Err(Error::InvalidShapeRecordSize)
        } else {
            let parts = read_parts(&mut source, num_parts)?;
            let points = read_xy_in_vec_of::<Point, T>(&mut source, num_points)?;

            Ok(Self {
                bbox,
                parts,
                points,
            })
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
        size += 2 * size_of::<f64>() * self.points.len();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        if !is_parts_array_valid(&self) {
            return Err(Error::MalformedShape);
        }
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.parts.len() as i32)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;
        write_parts(&mut dest, &self.parts)?;
        write_points(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for Polyline {
    fn bbox(&self) -> BBox {
        self.bbox
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
            "PolylineM({} points, {} parts)",
            self.points.len(),
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
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let parts = read_parts(&mut source, num_parts)?;

        let record_size_with_m = Self::size_of_record(num_points, num_parts, true) as i32;
        let record_size_without_m = Self::size_of_record(num_points, num_parts, false) as i32;

        if (record_size != record_size_with_m) & (record_size != record_size_without_m) {
            return Err(Error::InvalidShapeRecordSize);
        } else {
            let is_m_used = record_size == record_size_with_m;
            let mut points = read_xy_in_vec_of::<PointM, T>(&mut source, num_points)?;

            if is_m_used {
                let _m_range = read_range(&mut source)?;
                read_ms_into(&mut source, &mut points)?;
            }

            Ok(Self {
                bbox,
                parts,
                points,
            })
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
        size += 3 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        if !is_parts_array_valid(&self) {
            return Err(Error::MalformedShape);
        }
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.parts.len() as i32)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;
        write_parts(&mut dest, &self.parts)?;
        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for PolylineM {
    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn m_range(&self) -> [f64; 2] {
        calc_m_range(&self.points)
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
            "PolylineZ({} points, {} parts)",
            self.points.len(),
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
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let record_size_with_m = Self::size_of_record(num_points, num_parts, true) as i32;
        let record_size_without_m = Self::size_of_record(num_points, num_parts, false) as i32;

        if (record_size != record_size_with_m) & (record_size != record_size_without_m) {
            return Err(Error::InvalidShapeRecordSize);
        } else {
            let is_m_used = record_size == record_size_with_m;
            let parts = read_parts(&mut source, num_parts)?;

            let mut points = read_xy_in_vec_of::<PointZ, T>(&mut source, num_points)?;

            let _z_range = read_range(&mut source)?;
            read_zs_into(&mut source, &mut points)?;

            if is_m_used {
                let _m_range = read_range(&mut source)?;
                read_ms_into(&mut source, &mut points)?;
            }

            Ok(Self {
                bbox,
                parts,
                points,
            })
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
        size += 4 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        if !is_parts_array_valid(&self) {
            return Err(Error::MalformedShape);
        }
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.parts.len() as i32)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;
        write_parts(&mut dest, &self.parts)?;

        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.z_range())?;
        write_zs(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for PolylineZ {
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

/*
 * Polygon
 */


pub struct GenericPolygon<PointType> {
    pub bbox: BBox,
    pub points: Vec<PointType>,
    pub parts: Vec<i32>,
}

impl<PointType: HasXY + PartialEq + Copy> GenericPolygon<PointType> {
    /// # Examples
    ///
    /// Creating a PolygonZ with one ring,
    /// The constructor closes the polygon
    /// ```
    /// use shapefile::{PointZ, PolygonZ, NO_DATA};
    /// let points = vec![
    ///     PointZ::new(0.0, 0.0, 0.0, NO_DATA),
    ///     PointZ::new(0.0, 1.0, 0.0, NO_DATA),
    ///     PointZ::new(1.0, 0.0, 0.0, NO_DATA),
    /// ];
    /// let poly = PolygonZ::new(points);
    ///
    /// // The polygon gets closed 'explicitly'
    /// assert_eq!(poly.points.len(), 4);
    /// assert_eq!(poly.points.last(), poly.points.first());
    /// ```
    ///
    pub fn new(mut points: Vec<PointType>) -> Self {
        close_points_if_not_already(&mut points);
        if super::ring_type_from_points_ordering(&points) == super::RingType::InnerRing {
            points.reverse();
        }
        Self::from(GenericPolyline::<PointType>::new(points))
    }
}

impl<PointType: HasXY + PartialEq + Copy> GenericPolygon<PointType> {
    pub fn with_parts(mut parts_points: Vec<Vec<PointType>>) -> Self {
        parts_points.iter_mut().for_each(close_points_if_not_already);
        let mut has_outer_ring = false;
        for points in &mut parts_points {
            if super::ring_type_from_points_ordering(&points) == super::RingType::InnerRing && !has_outer_ring {
                points.reverse();
                has_outer_ring = true;
            }
        }
        Self::from(GenericPolyline::<PointType>::with_parts(parts_points))
    }
}

impl<PointType> MultipointShape<PointType> for GenericPolygon<PointType> {
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

impl<PointType> MultipartShape<PointType> for GenericPolygon<PointType> {
    fn parts_indices(&self) -> &[i32] {
        &self.parts
    }
}

/// Converts a shapefile polygon into a geo_types MultiPolygon
///
/// Because in a shapefile `A Polygon may contain multiple outer rings`
/// which are really just multiple polygons
///
/// Vertices of rings defining holes in polygons are in a counterclockwise direction
#[cfg(feature = "geo-types")]
impl<PointType> TryFrom<GenericPolygon<PointType>> for geo_types::MultiPolygon<f64>
    where PointType: HasXY + Copy,
          geo_types::Point<f64>: From<PointType>{
    type Error = Error;
    fn try_from(p: GenericPolygon<PointType>) -> Result<Self, Self::Error> {
        let mut last_poly = None;
        let mut polygons = Vec::<geo_types::Polygon<f64>>::new();
        for points_slc in p.parts() {
            let points = points_slc
                .iter()
                .map(|p| geo_types::Point::<f64>::from(*p))
                .collect::<Vec<geo_types::Point<f64>>>();
            if super::ring_type_from_points_ordering(points_slc) == super::RingType::OuterRing {
                let new_poly = geo_types::Polygon::new(points.into(), vec![]);
                if last_poly.is_some() {
                    polygons.push(last_poly.replace(new_poly).unwrap());
                } else {
                    last_poly = Some(new_poly);
                }
            } else {
                if let Some(ref mut polygon) = last_poly {
                    polygon.interiors_push(points);
                } else {
                    return Err(Error::OrphanInnerRing)
                }
            }
        }
        if let Some(poly) = last_poly {
            polygons.push(poly);
        }
        Ok(polygons.into())
    }

}

#[cfg(feature = "geo-types")]
/// geo_types guarantees that Polygons exterior and interiors are closed
impl<PointType> From<geo_types::Polygon<f64>> for GenericPolygon<PointType>
    where  PointType: HasXY + From<geo_types::Coordinate<f64>> + PartialEq + Copy {
    fn from(polygon: geo_types::Polygon<f64>) -> Self {
        if polygon.exterior(). num_coords() == 0 {
            return Self::new(vec![]);
        }

        let mut total_num_points = polygon.exterior().num_coords();
        total_num_points += polygon.interiors().iter().map(|ls| ls.num_coords()).sum::<usize>();

        let mut all_points = Vec::<PointType>::with_capacity(total_num_points);

        let (outer_ls, inners_ls) = polygon.into_inner();
        let mut outer_points = outer_ls
            .into_iter()
            .map(|c| PointType::from(c))
            .collect::<Vec<PointType>>();

        if super::ring_type_from_points_ordering(&outer_points) == super::RingType::InnerRing {
            outer_points.reverse();
        }
        all_points.append(&mut outer_points);
        let mut num_inner = inners_ls.len();
        let mut parts = Vec::<i32>::with_capacity(1 + num_inner);
        parts.push(0);
        for (i, inner_ls) in inners_ls.into_iter().enumerate() {
            if i != num_inner - 1 {
                parts.push(i as i32);
            }
            let mut inner_points = inner_ls
                .into_iter()
                .map(|c| PointType::from(c))
                .collect::<Vec<PointType>>();

            if super::ring_type_from_points_ordering(&inner_points) == super::RingType::OuterRing {
                inner_points.reverse();
            }
            all_points.append(&mut inner_points);
        }

        let bbox = BBox::from_points(&all_points);
        Self {
            bbox,
            points: all_points,
            parts
        }
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::MultiPolygon<f64>> for GenericPolygon<PointType>
    where  PointType: HasXY + From<geo_types::Coordinate<f64>> + PartialEq + Copy {
    fn from(multi_polygon: geo_types::MultiPolygon<f64>) -> Self {
        let polygons = multi_polygon
            .into_iter()
            .map(|polyg| GenericPolygon::<PointType>::from(polyg))
            .collect::<Vec<GenericPolygon<PointType>>>();

        let total_points_count = polygons
            .iter()
            .fold(0usize, |count, polygon| count + polygon.points.len());

        let mut all_points = Vec::<PointType>::with_capacity(total_points_count);
        let num_parts = polygons.len();
        let mut parts = Vec::<i32>::with_capacity(num_parts);
        parts.push(0);
        for (i, mut polygon) in polygons.into_iter().enumerate() {
            all_points.append(&mut polygon.points);
            if i != num_parts - 1 {
                parts.push(all_points.len() as i32)
            }
        }

        let bbox = BBox::from_points(&all_points);
        Self {
            bbox,
            points: all_points,
            parts
        }
    }
}

pub type Polygon = GenericPolygon<Point>;

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Polygon({} points, {} parts)",
            self.points.len(),
            self.parts.len()
        )
    }
}

impl HasShapeType for Polygon {
    fn shapetype() -> ShapeType {
        ShapeType::Polygon
    }
}

impl ConcreteReadableShape for Polygon {
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let poly = Polyline::read_shape_content(&mut source, record_size)?;
        Ok(poly.into())
    }
}

impl WritableShape for Polygon {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
        size += 2 * size_of::<f64>() * self.points.len();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        let poly: Polyline = self.into();
        poly.write_to(&mut dest)
    }
}

impl EsriShape for Polygon {
    fn bbox(&self) -> BBox {
        self.bbox
    }
}

/*
 * PolygonM
 */

pub type PolygonM = GenericPolygon<PointM>;

impl fmt::Display for PolygonM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PolygonM({} points, {} parts)",
            self.points.len(),
            self.parts.len()
        )
    }
}

impl HasShapeType for PolygonM {
    fn shapetype() -> ShapeType {
        ShapeType::PolygonM
    }
}

impl ConcreteReadableShape for PolygonM {
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let poly = PolylineM::read_shape_content(&mut source, record_size)?;
        Ok(Self::from(poly))
    }
}

impl WritableShape for PolygonM {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
        size += 3 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        PolylineM::from(self).write_to(&mut dest)
    }
}

impl EsriShape for PolygonM {
    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn m_range(&self) -> [f64; 2] {
        calc_m_range(&self.points)
    }
}

/*
 * PolygonZ
 */

pub type PolygonZ = GenericPolygon<PointZ>;

impl fmt::Display for PolygonZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PolygonZ({} points, {} parts)",
            self.points.len(),
            self.parts.len()
        )
    }
}

impl HasShapeType for PolygonZ {
    fn shapetype() -> ShapeType {
        ShapeType::PolygonZ
    }
}

impl ConcreteReadableShape for PolygonZ {
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let poly = PolylineZ::read_shape_content(&mut source, record_size)?;
        Ok(poly.into())
    }
}

impl WritableShape for PolygonZ {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
        size += 4 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        PolylineZ::from(self).write_to(&mut dest)
    }
}

impl EsriShape for PolygonZ {
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

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_of_polyline_z() {
        assert_eq!(PolylineZ::size_of_record(10, 3), 404);
    }
}
*/
