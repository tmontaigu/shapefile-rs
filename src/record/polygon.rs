//! Module with the definition of Polygon, PolygonM, PolygonZ
use record::{RingType, ring_type_from_points_ordering, GenericBBox, ConcreteReadableShape, WritableShape, EsriShape, close_points_if_not_already};
use record::traits::{GrowablePoint, ShrinkablePoint, HasXY};
use std::io::{Write, Read};
use record::io::MultiPartShapeWriter;
use core::fmt;
use ::{Point, HasShapeType};
use ::{ShapeType, Error};
use std::mem::size_of;
use ::{PointM, PointZ};
use super::{Polyline, PolylineM, PolylineZ};
use record::polyline::GenericPolyline;

/// Rings composing a Polygon
///
/// `Inner` rings define holes in polygons.
///
/// In shapefile, the point ordering is what is used to know if
/// a ring is an outer or inner one:
/// - **Outer** ring => points in clockwise order
/// - **Inner** ring => points in counter-clockwise order
///
/// # Note
///
/// Rings you get access from a [`GenericPolygon`] will always have its points ordered
/// according to its type (outer, inner).
///
/// But `PolygonRing`s you create won't be reordered until you move them into
/// a [`GenericPolygon`].
///
///
/// # Example
///
/// ```
/// use shapefile::{PolygonRing, Polygon, Point};
/// // Here the points are not in the correct order to be an Outer ring for a shapefile
/// let mut points = vec![
///     Point::new(-12.0, 6.0),
///     Point::new(-12.0, -6.0),
///     Point::new(12.0, -6.0),
///     Point::new(12.0, 6.0),
///     Point::new(-12.0, 6.0),
/// ];
///
/// let mut reversed_points = points.clone();
/// reversed_points.reverse();
///
/// let ring = PolygonRing::Outer(points);
/// assert_ne!(ring.points(), reversed_points.as_slice());
///
/// // Now the points will be reversed
/// let polygon = Polygon::new(ring);
/// assert_eq!(polygon.rings()[0].points(), reversed_points.as_slice());
/// ```
///
/// [`GenericPolygon`]: struct.GenericPolygon.html
#[derive(Debug, Clone, PartialEq)]
pub enum PolygonRing<PointType> {
    /// The outer ring of a polygon.
    Outer(Vec<PointType>),
    /// Defines a hole in a polygon
    Inner(Vec<PointType>)
}

impl<PointType> PolygonRing<PointType> {
    /// Returns the number of points inside the ring
    ///
    /// # Example
    ///
    /// ```
    /// use shapefile::{PolygonRing, Point};
    /// let ring = PolygonRing::Inner(vec![
    ///     Point::new(-12.0, 6.0),
    ///     Point::new(-12.0, -6.0),
    ///     Point::new(12.0, -6.0),
    ///     Point::new(12.0, 6.0),
    ///     Point::new(-12.0, 6.0),
    /// ]);
    /// assert_eq!(ring.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.points().len()
    }

    /// Returns a non-mutable slice to the points inside the ring
    ///
    /// ```
    /// use shapefile::{PolygonRing, Point};
    /// let ring = PolygonRing::Inner(vec![
    ///     Point::new(-12.0, 6.0),
    ///     Point::new(-12.0, -6.0),
    ///     Point::new(12.0, -6.0),
    ///     Point::new(12.0, 6.0),
    ///     Point::new(-12.0, 6.0),
    /// ]);
    /// assert_eq!(ring.points()[2], Point::new(12.0, -6.0));
    /// ```
    pub fn points(&self) -> &[PointType] {
        match self {
            PolygonRing::Outer(points) => &points,
            PolygonRing::Inner(points) => &points,
        }
    }

    fn points_vec_mut(&mut self) -> &mut Vec<PointType> {
        match self {
            PolygonRing::Outer(points) => points,
            PolygonRing::Inner(points) => points,
        }
    }
}


impl<PointType> PolygonRing<PointType>
    where PointType: Copy + PartialEq + HasXY {
    fn close_and_reorder(&mut self) {
        self.close_if_not_already_closed();
        self.correctly_order_points();
    }

    fn close_if_not_already_closed(&mut self) {
        close_points_if_not_already(self.points_vec_mut())
    }

    fn correctly_order_points(&mut self) {
        let points = self.points_vec_mut();
        let actual_ring_type = super::ring_type_from_points_ordering(&points);
        match (self, actual_ring_type) {
            (PolygonRing::Outer(points), RingType::InnerRing) |
            (PolygonRing::Inner(points), RingType::OuterRing) => {
                points.reverse();
            }
            _ => {}
        }
    }
}

impl<PointType: HasXY> From<Vec<PointType>> for PolygonRing<PointType> {
    fn from(p: Vec<PointType>) -> Self {
        match ring_type_from_points_ordering(&p) {
            RingType::OuterRing => PolygonRing::Outer(p),
            RingType::InnerRing => PolygonRing::Inner(p),
        }
    }
}

// TODO a Polygon is a connected sequence of 4 or more points
/// Generic struct to create Polygon; PolygonM, PolygonZ
///
/// Polygons can have multiple parts (or rings)
///
/// To create a polygon with only one part use [`new`].
///
/// To create a polygon with multiple parts use [`with_parts`].
///
/// # Notes
/// - A Polygon ring is a connected sequence of 4 or more points
///   **(this is not checked)**
/// - Polygon's rings MUST be closed (the first and last points MUST be the same) (p 13/34)
///   **(this is done by the constructors if you do not do it yourself)**
/// - The order of rings is not significant (p 13/34)
/// - A polygon may have multiple [`Outer`] rings (p12/34)
///
/// [`new`]: #method.new
/// [`with_parts`]: #method.with_parts
/// [`Outer`]: enum.PolygonRing.html#variant.Outer
#[derive(Debug, Clone, PartialEq)]
pub struct GenericPolygon<PointType> {
    bbox: GenericBBox<PointType>,
    rings: Vec<PolygonRing<PointType>>,
}

impl<PointType> GenericPolygon<PointType>
    where PointType: ShrinkablePoint + GrowablePoint + PartialEq + HasXY + Copy
{
    /// Creates a polygon with only one ring
    ///
    /// The ring will be closed if it is not
    /// (shapefiles requires the first and last point to be equal)
    ///
    /// The ring points may be reordered to match their type
    /// (see [`PolygonRing`])
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::{PolygonRing, PointZ, PolygonZ, NO_DATA};
    /// let ring = PolygonRing::Outer(vec![
    ///     PointZ::new(0.0, 0.0, 0.0, NO_DATA),
    ///     PointZ::new(0.0, 1.0, 0.0, NO_DATA),
    ///     PointZ::new(1.0, 1.0, 0.0, NO_DATA),
    ///     PointZ::new(1.0, 0.0, 0.0, NO_DATA),
    /// ]);
    /// let poly = PolygonZ::new(ring);
    /// assert_eq!(poly.rings()[0].points().first(), poly.rings()[0].points().last());
    /// ```
    ///
    /// [`PolygonRing`]: enum.PolygonRing.html
    pub fn new(mut ring: PolygonRing<PointType>) -> Self {
        ring.close_and_reorder();
        Self::with_parts(vec![ring])
    }
}

impl<PointType> GenericPolygon<PointType>
    where PointType: GrowablePoint + ShrinkablePoint + PartialEq + HasXY + Copy
{
    /// Creates a polygon with multiple rings
    ///
    /// The ring will be closed if it is not
    /// (shapefiles requires the first and last point to be equal)
    ///
    /// The ring points may be reordered to match their type
    /// (see [`PolygonRing`])
    ///
    ///
    /// [`PolygonRing`]: enum.PolygonRing.html
    pub fn with_parts(mut rings: Vec<PolygonRing<PointType>>) -> Self {
        rings.iter_mut()
            .for_each(PolygonRing::close_and_reorder);
        let mut bbox = GenericBBox::<PointType>::from_points(rings[0].points());
        for ring in &rings[1..] {
            bbox.grow_from_points(ring.points());
        }
        Self {
            bbox,
            rings,
        }
    }
}


impl<PointType> GenericPolygon<PointType> {
    /// Returns the bounding box associated to the polygon
    pub fn bbox(&self) -> &GenericBBox<PointType> {
        &self.bbox
    }

    /// Returns the rings of the polygon
    pub fn rings(&self) -> &[PolygonRing<PointType>] {
        &self.rings
    }

    pub fn ring(&self, index: usize) -> Option<&PolygonRing<PointType>> {
        self.rings.get(index)
    }

    /// Returns the sum of points of all the rings
    pub fn total_point_count(&self) -> usize {
        self.rings.iter().map(|ring| ring.len()).sum()
    }
}


impl<PointType: HasXY> From<GenericPolyline<PointType>> for GenericPolygon<PointType> {
    fn from(polyline: GenericPolyline<PointType>) -> Self {
        let mut rings = Vec::<PolygonRing<PointType>>::with_capacity(polyline.parts.len());
        for part in polyline.parts {
            rings.push(PolygonRing::from(part))
        }
        Self {
            bbox: polyline.bbox,
            rings,
        }
    }
}


/*
 * Polygon
*/

pub type Polygon = GenericPolygon<Point>;

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Polygon({} rings)",
            self.rings.len()
        )
    }
}

impl HasShapeType for Polygon {
    fn shapetype() -> ShapeType {
        ShapeType::Polygon
    }
}

impl ConcreteReadableShape for Polygon {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        Polyline::read_shape_content(source, record_size)
            .map(|polyline| Polygon::from(polyline))
    }

}

impl WritableShape for Polygon {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.rings.len();
        size += 2 * size_of::<f64>() * self.total_point_count();
        size
    }

    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error> {
        let parts_iter = self.rings().iter().map(|ring| ring.points());
        let writer = MultiPartShapeWriter::new(&self.bbox,parts_iter, dest);
        writer.write_point_shape()?;
        Ok(())
    }
}

impl EsriShape for Polygon {
    fn x_range(&self) -> [f64; 2] {
        self.bbox.x_range()
    }

    fn y_range(&self) -> [f64; 2] {
        self.bbox.y_range()
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
            "PolygonM({} rings)",
            self.rings.len()
        )
    }
}

impl HasShapeType for PolygonM {
    fn shapetype() -> ShapeType {
        ShapeType::PolygonM
    }
}

impl ConcreteReadableShape for PolygonM {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        PolylineM::read_shape_content(source, record_size)
            .map(|polyline| PolygonM::from(polyline))
    }
}

impl WritableShape for PolygonM {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.rings.len();
        size += 3 * size_of::<f64>() * self.total_point_count();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error> {
        let parts_iter = self.rings().iter().map(|ring| ring.points());
        let writer = MultiPartShapeWriter::new(&self.bbox, parts_iter, dest);
        writer.write_point_m_shape()?;
        Ok(())
    }
}

impl EsriShape for PolygonM {
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
 * PolygonZ
 */

pub type PolygonZ = GenericPolygon<PointZ>;

impl fmt::Display for PolygonZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PolygonZ({} rings)",
            self.rings.len()
        )
    }
}

impl HasShapeType for PolygonZ {
    fn shapetype() -> ShapeType {
        ShapeType::PolygonZ
    }
}

impl ConcreteReadableShape for PolygonZ {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        PolylineZ::read_shape_content(source, record_size)
            .map(|polyline| PolygonZ::from(polyline))
    }
}

impl WritableShape for PolygonZ {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.rings.len();
        size += 4 * size_of::<f64>() * self.total_point_count();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error> {
        let parts_iter = self.rings().iter().map(|ring| ring.points());
        let writer = MultiPartShapeWriter::new(&self.bbox, parts_iter, dest);
        writer.write_point_z_shape()?;
        Ok(())
    }
}

impl EsriShape for PolygonZ {
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


/// Converts a shapefile polygon into a geo_types MultiPolygon
///
/// Because in a shapefile `A Polygon may contain multiple outer rings`
/// which are really just multiple polygons
///
/// Vertices of rings defining holes in polygons are in a counterclockwise direction
#[cfg(feature = "geo-types")]
impl<PointType> TryFrom<GenericPolygon<PointType>> for geo_types::MultiPolygon<f64>
    where
        PointType: HasXY + Copy,
        geo_types::Point<f64>: From<PointType>,
{
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
                    return Err(Error::OrphanInnerRing);
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
    where
        PointType: HasXY + From<geo_types::Coordinate<f64>> + PartialEq + Copy,
{
    fn from(polygon: geo_types::Polygon<f64>) -> Self {
        if polygon.exterior().num_coords() == 0 {
            return Self::new(vec![]);
        }

        let mut total_num_points = polygon.exterior().num_coords();
        total_num_points += polygon
            .interiors()
            .iter()
            .map(|ls| ls.num_coords())
            .sum::<usize>();

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
        let num_inner = inners_ls.len();
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
            parts,
        }
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::MultiPolygon<f64>> for GenericPolygon<PointType>
    where
        PointType: HasXY + From<geo_types::Coordinate<f64>> + PartialEq + Copy,
{
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
            parts,
        }
    }
}
