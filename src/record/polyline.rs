//! Module with the definition of Polyline, PolylineM, PolylineZ

use std::fmt;
use std::io::{Read, Write};
use std::mem::size_of;

use record::io::*;
use record::traits::{GrowablePoint, ShrinkablePoint};
use record::ConcreteReadableShape;
use record::GenericBBox;
use record::{EsriShape, HasShapeType, WritableShape};
use record::{Point, PointM, PointZ};
use {Error, ShapeType};

#[cfg(feature = "geo-types")]
use geo_types;

/// Generic struct to create Polyline; PolylineM, PolylineZ
///
/// Polylines can have multiple parts.
///
/// Polylines parts must have 2 at least 2 points
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
    /// # panic
    ///
    /// This will panic if the vec has less than 2 points
    pub fn new(points: Vec<PointType>) -> Self {
        assert!(
            points.len() >= 2,
            "Polylines parts must have at least 2 points"
        );
        Self {
            bbox: GenericBBox::<PointType>::from_points(&points),
            parts: vec![points],
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
    /// # panic
    ///
    /// This will panic if any of the parts are less than 2 points
    pub fn with_parts(parts: Vec<Vec<PointType>>) -> Self {
        assert!(
            parts.iter().all(|p| p.len() >= 2),
            "Polylines parts must have at least 2 points"
        );
        Self {
            bbox: GenericBBox::<PointType>::from_parts(&parts),
            parts,
        }
    }
}

impl<PointType> GenericPolyline<PointType> {
    /// Returns the bounding box associated to the polyline
    #[inline]
    pub fn bbox(&self) -> &GenericBBox<PointType> {
        &self.bbox
    }

    /// Returns a reference to all the parts
    #[inline]
    pub fn parts(&self) -> &Vec<Vec<PointType>> {
        &self.parts
    }

    /// Returns a reference to a part
    #[inline]
    pub fn part(&self, index: usize) -> Option<&Vec<PointType>> {
        self.parts.get(index)
    }

    /// Consumes the polyline and returns the parts
    #[inline]
    pub fn into_inner(self) -> Vec<Vec<PointType>> {
        self.parts
    }

    /// Returns the number of points contained in all the parts
    #[inline]
    pub fn total_point_count(&self) -> usize {
        self.parts.iter().map(|part| part.len()).sum()
    }
}

/// Specialization of the `GenericPolyline` struct to represent a `Polyline` shape
/// ( collection of [Point](../point/struct.Point.html))
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
        write!(f, "Polyline({} parts)", self.parts.len())
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
            rdr.read_xy().map_err(Error::IoError).and_then(|rdr| {
                Ok(Self {
                    bbox: rdr.bbox,
                    parts: rdr.parts,
                })
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

/// Specialization of the `GenericPolyline` struct to represent a `PolylineM` shape
/// ( collection of [PointM](../point/struct.PointM.html))
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
        write!(f, "PolylineM({} parts)", self.parts.len())
    }
}

impl HasShapeType for PolylineM {
    fn shapetype() -> ShapeType {
        ShapeType::PolylineM
    }
}

impl ConcreteReadableShape for PolylineM {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        let rdr = MultiPartShapeReader::<PointM, T>::new(source)?;

        let record_size_with_m = Self::size_of_record(rdr.num_points, rdr.num_parts, true) as i32;
        let record_size_without_m =
            Self::size_of_record(rdr.num_points, rdr.num_parts, false) as i32;

        if (record_size != record_size_with_m) && (record_size != record_size_without_m) {
            Err(Error::InvalidShapeRecordSize)
        } else {
            rdr.read_xy()
                .and_then(|rdr| rdr.read_ms_if(record_size == record_size_with_m))
                .map_err(Error::IoError)
                .and_then(|rdr| {
                    Ok(Self {
                        bbox: rdr.bbox,
                        parts: rdr.parts,
                    })
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

/// Specialization of the `GenericPolyline` struct to represent a `PolylineZ` shape
/// ( collection of [PointZ](../point/struct.PointZ.html))
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
        write!(f, "PolylineZ({} parts)", self.parts.len())
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
        let record_size_without_m =
            Self::size_of_record(rdr.num_points, rdr.num_parts, false) as i32;

        if (record_size != record_size_with_m) && (record_size != record_size_without_m) {
            Err(Error::InvalidShapeRecordSize)
        } else {
            rdr.read_xy()
                .and_then(|rdr| rdr.read_zs())
                .and_then(|rdr| rdr.read_ms_if(record_size == record_size_with_m))
                .map_err(Error::IoError)
                .and_then(|rdr| {
                    Ok(Self {
                        bbox: rdr.bbox,
                        parts: rdr.parts,
                    })
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
        let mut lines = Vec::<geo_types::LineString<f64>>::with_capacity(polyline.parts().len());

        for points in polyline.parts {
            let line: Vec<geo_types::Coordinate<f64>> = points
                .into_iter()
                .map(geo_types::Coordinate::<f64>::from)
                .collect();
            lines.push(line.into());
        }
        geo_types::MultiLineString::<f64>::from_iter(lines.into_iter())
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::Line<f64>> for GenericPolyline<PointType>
where
    PointType: From<geo_types::Point<f64>> + ShrinkablePoint + GrowablePoint + Copy,
{
    fn from(line: geo_types::Line<f64>) -> Self {
        let (p1, p2) = line.points();
        Self::new(vec![PointType::from(p1), PointType::from(p2)])
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::LineString<f64>> for GenericPolyline<PointType>
where
    PointType: From<geo_types::Coordinate<f64>> + ShrinkablePoint + GrowablePoint + Copy,
{
    fn from(line: geo_types::LineString<f64>) -> Self {
        let points: Vec<PointType> = line.into_iter().map(PointType::from).collect();
        Self::new(points)
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::MultiLineString<f64>> for GenericPolyline<PointType>
where
    PointType: From<geo_types::Coordinate<f64>> + ShrinkablePoint + GrowablePoint + Copy,
{
    fn from(mls: geo_types::MultiLineString<f64>) -> Self {
        let mut parts = Vec::<Vec<PointType>>::with_capacity(mls.0.len());
        for linestring in mls.0.into_iter() {
            parts.push(linestring.into_iter().map(PointType::from).collect());
        }
        Self::with_parts(parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Polylines parts must have at least 2 points")]
    fn test_polyline_new_less_than_2_points() {
        let _polyline = Polyline::new(vec![Point::new(1.0, 1.0)]);
    }

    #[test]
    #[should_panic(expected = "Polylines parts must have at least 2 points")]
    fn test_polyline_with_parts_less_than_2_points() {
        let _polyline = Polyline::with_parts(vec![
            vec![Point::new(1.0, 1.0), Point::new(2.0, 2.0)],
            vec![Point::new(1.0, 1.0)],
        ]);
    }
}

#[cfg(test)]
#[cfg(feature = "geo-types")]
mod test_geo_types_conversions {
    use super::*;
    use geo_types::{Coordinate, LineString, MultiLineString};
    use NO_DATA;
    use {PointM, PolylineM};

    #[test]
    fn test_polyline_into_multiline_string() {
        let polyline_m = PolylineM::with_parts(vec![
            vec![
                PointM::new(1.0, 5.0, 0.0),
                PointM::new(5.0, 5.0, NO_DATA),
                PointM::new(5.0, 1.0, 3.0),
            ],
            vec![PointM::new(1.0, 5.0, 0.0), PointM::new(1.0, 1.0, 0.0)],
        ]);

        let multiline_string: MultiLineString<f64> = polyline_m.into();

        let expected_multiline = geo_types::MultiLineString(vec![
            LineString::<f64>(vec![
                Coordinate { x: 1.0, y: 5.0 },
                Coordinate { x: 5.0, y: 5.0 },
                Coordinate { x: 5.0, y: 1.0 },
            ]),
            LineString::<f64>(vec![
                Coordinate { x: 1.0, y: 5.0 },
                Coordinate { x: 1.0, y: 1.0 },
            ]),
        ]);
        assert_eq!(multiline_string, expected_multiline);
    }

    #[test]
    fn test_line_into_polyline() {
        let line = geo_types::Line::new(
            Coordinate { x: 2.0, y: 3.0 },
            Coordinate { x: 6.0, y: -6.0 },
        );
        let polyline: PolylineZ = line.into();

        assert_eq!(
            polyline.parts,
            vec![vec![
                PointZ::new(2.0, 3.0, 0.0, NO_DATA),
                PointZ::new(6.0, -6.0, 0.0, NO_DATA)
            ]]
        );
    }

    #[test]
    fn test_linestring_into_polyline() {
        let linestring = LineString::from(vec![
            Coordinate { x: 1.0, y: 5.0 },
            Coordinate { x: 5.0, y: 5.0 },
            Coordinate { x: 5.0, y: 1.0 },
        ]);

        let polyline: Polyline = linestring.into();
        assert_eq!(
            polyline.parts,
            vec![vec![
                Point::new(1.0, 5.0),
                Point::new(5.0, 5.0),
                Point::new(5.0, 1.0),
            ]]
        )
    }

    #[test]
    fn test_multi_line_string_into_polyline() {
        let multiline_string = geo_types::MultiLineString(vec![
            LineString::<f64>(vec![
                Coordinate { x: 1.0, y: 5.0 },
                Coordinate { x: 5.0, y: 5.0 },
                Coordinate { x: 5.0, y: 1.0 },
            ]),
            LineString::<f64>(vec![
                Coordinate { x: 1.0, y: 5.0 },
                Coordinate { x: 1.0, y: 1.0 },
            ]),
        ]);

        let expected_polyline_z = PolylineZ::with_parts(vec![
            vec![
                PointZ::new(1.0, 5.0, 0.0, NO_DATA),
                PointZ::new(5.0, 5.0, 0.0, NO_DATA),
                PointZ::new(5.0, 1.0, 0.0, NO_DATA),
            ],
            vec![
                PointZ::new(1.0, 5.0, 0.0, NO_DATA),
                PointZ::new(1.0, 1.0, 0.0, NO_DATA),
            ],
        ]);

        let polyline_z: PolylineZ = multiline_string.into();
        assert_eq!(polyline_z, expected_polyline_z);
    }
}
