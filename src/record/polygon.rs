//! Module with the definition of Polygon, PolygonM, PolygonZ
use super::io::MultiPartShapeWriter;
use super::polyline::GenericPolyline;
use super::traits::{GrowablePoint, HasXY, ShrinkablePoint};
use super::{
    close_points_if_not_already, ring_type_from_points_ordering, ConcreteReadableShape, EsriShape,
    GenericBBox, RingType, WritableShape,
};
use super::{Error, ShapeType};
use super::{HasShapeType, Point};
use super::{PointM, PointZ};
use super::{Polyline, PolylineM, PolylineZ};
use core::fmt;
use std::io::{Read, Write};
use std::mem::size_of;

#[cfg(feature = "geo-types")]
use geo_types::{Coord, LineString};
use std::ops::Index;
use std::slice::SliceIndex;

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
/// assert_eq!(ring[0], Point::new(-12.0, 6.0));
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
    Inner(Vec<PointType>),
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
    #[inline]
    pub fn len(&self) -> usize {
        self.points().len()
    }

    /// Returns whether the rings contains any points
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.points().is_empty()
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
    #[inline]
    pub fn points(&self) -> &[PointType] {
        match self {
            PolygonRing::Outer(points) => points,
            PolygonRing::Inner(points) => points,
        }
    }

    /// Consumes the ring and returns its points
    #[inline]
    pub fn into_inner(self) -> Vec<PointType> {
        match self {
            PolygonRing::Outer(points) => points,
            PolygonRing::Inner(points) => points,
        }
    }

    #[inline]
    fn points_vec_mut(&mut self) -> &mut Vec<PointType> {
        match self {
            PolygonRing::Outer(points) => points,
            PolygonRing::Inner(points) => points,
        }
    }
}

impl<PointType> AsRef<[PointType]> for PolygonRing<PointType> {
    fn as_ref(&self) -> &[PointType] {
        self.points()
    }
}

impl<PointType> PolygonRing<PointType>
where
    PointType: Copy + PartialEq + HasXY,
{
    fn close_and_reorder(&mut self) {
        self.close_if_not_already_closed();
        self.correctly_order_points();
    }

    fn close_if_not_already_closed(&mut self) {
        close_points_if_not_already(self.points_vec_mut())
    }

    fn correctly_order_points(&mut self) {
        let points = self.points_vec_mut();
        let actual_ring_type = super::ring_type_from_points_ordering(points);
        match (self, actual_ring_type) {
            (PolygonRing::Outer(points), RingType::InnerRing)
            | (PolygonRing::Inner(points), RingType::OuterRing) => {
                points.reverse();
            }
            _ => {}
        }
    }
}

impl<PointType, I: SliceIndex<[PointType]>> Index<I> for PolygonRing<PointType> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.points(), index)
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
/// To create a polygon with multiple rings use [`with_rings`].
///
/// # Notes
/// - A Polygon ring is a connected sequence of 4 or more points
///   **(this is not checked)**
/// - Polygon's rings MUST be closed (the first and last points MUST be the same) (p 13/34)
///   **(this is done by the constructors if you do not do it yourself)**
/// - The order of rings is not significant (p 13/34)
/// - A polygon may have multiple [`Outer`] rings (p12/34)
///
///
/// # geo-types
///
/// shapefile's Polygons can be converted to geo-types's `MultiPolygon<f64>`,
/// but not geo-types's Polygon<f64> as they only allow polygons with one outer ring.
///
/// geo-types's `Polygon<f64>` and `MultiPolygon<f64>` can be converted to shapefile's Polygon
///
/// ```
/// # #[cfg(feature = "geo-types")]
/// # fn main() -> Result<(), shapefile::Error>{
/// let mut polygons = shapefile::read_shapes_as::<_, shapefile::PolygonM>("tests/data/polygonm.shp")?;
/// let geo_polygon: geo_types::MultiPolygon<f64> = polygons.pop().unwrap().into();
/// let polygon = shapefile::PolygonZ::from(geo_polygon);
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "geo-types"))]
/// # fn main() {}
/// ```
///
/// [`new`]: #method.new
/// [`with_rings`]: #method.with_rings
/// [`Outer`]: enum.PolygonRing.html#variant.Outer
#[derive(Debug, Clone, PartialEq)]
pub struct GenericPolygon<PointType> {
    bbox: GenericBBox<PointType>,
    rings: Vec<PolygonRing<PointType>>,
}

impl<PointType> GenericPolygon<PointType>
where
    PointType: ShrinkablePoint + GrowablePoint + PartialEq + HasXY + Copy,
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
        Self::with_rings(vec![ring])
    }
}

impl<PointType> GenericPolygon<PointType>
where
    PointType: GrowablePoint + ShrinkablePoint + PartialEq + HasXY + Copy,
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
    /// # Example
    ///
    /// ```
    /// use shapefile::{PolygonRing, Point, Polygon};
    /// let polygon = Polygon::with_rings(vec![
    ///     PolygonRing::Outer(vec![
    ///         Point::new(-120.0, 60.0),
    ///         Point::new(-120.0, -60.0),
    ///         Point::new(120.0, -60.0),
    ///         Point::new(120.0, 60.0),
    ///         Point::new(-120.0, 60.0),
    ///     ]),
    ///     PolygonRing::Inner(vec![
    ///          Point::new(-60.0, 30.0),
    ///          Point::new(60.0, 30.0),
    ///          Point::new(60.0, -30.0),
    ///          Point::new(-60.0, -30.0),
    ///          Point::new(-60.0, 30.0),
    ///     ]),
    /// ]);
    ///
    /// assert_eq!(polygon.rings().len(), 2);
    /// ```
    ///
    /// [`PolygonRing`]: enum.PolygonRing.html
    pub fn with_rings(mut rings: Vec<PolygonRing<PointType>>) -> Self {
        rings.iter_mut().for_each(PolygonRing::close_and_reorder);
        let mut bbox = GenericBBox::<PointType>::from_points(rings[0].points());
        for ring in &rings[1..] {
            bbox.grow_from_points(ring.points());
        }
        Self { bbox, rings }
    }
}

impl<PointType> GenericPolygon<PointType> {
    /// Returns the bounding box associated to the polygon
    #[inline]
    pub fn bbox(&self) -> &GenericBBox<PointType> {
        &self.bbox
    }

    /// Returns the rings of the polygon
    #[inline]
    pub fn rings(&self) -> &[PolygonRing<PointType>] {
        &self.rings
    }

    /// Returns the ring as index
    ///
    /// # Example
    ///
    /// ```
    /// use shapefile::{polygon, NO_DATA};
    ///
    /// let polygon = polygon!{
    ///     Outer(
    ///         (0.0, 0.0, 0.0, NO_DATA),
    ///         (0.0, 1.0, 0.0, NO_DATA),
    ///         (1.0, 1.0, 0.0, NO_DATA),
    ///         (1.0, 0.0, 0.0, NO_DATA),
    ///     )
    /// };
    ///
    /// assert_eq!( polygon.ring(0).is_some(), true);
    /// assert_eq!(polygon.ring(1), None);
    /// ```
    #[inline]
    pub fn ring(&self, index: usize) -> Option<&PolygonRing<PointType>> {
        self.rings.get(index)
    }

    /// Consumes the shape and returns the rings
    #[inline]
    pub fn into_inner(self) -> Vec<PolygonRing<PointType>> {
        self.rings
    }

    /// Returns the sum of points of all the rings
    #[inline]
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
/// Specialization of the `GenericPolygon` struct to represent a `Polygon` shape
/// ( collection of [Point](../point/struct.Point.html))
pub type Polygon = GenericPolygon<Point>;

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Polygon({} rings)", self.rings.len())
    }
}

impl HasShapeType for Polygon {
    fn shapetype() -> ShapeType {
        ShapeType::Polygon
    }
}

impl ConcreteReadableShape for Polygon {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        Polyline::read_shape_content(source, record_size).map(Polygon::from)
    }
}

impl WritableShape for Polygon {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0_usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.rings.len();
        size += 2 * size_of::<f64>() * self.total_point_count();
        size
    }

    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error> {
        let parts_iter = self.rings().iter().map(|ring| ring.points());
        let writer = MultiPartShapeWriter::new(&self.bbox, parts_iter, dest);
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

/// Specialization of the `GenericPolygon` struct to represent a `PolygonM` shape
/// ( collection of [PointM](../point/struct.PointM.html))
pub type PolygonM = GenericPolygon<PointM>;

impl fmt::Display for PolygonM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PolygonM({} rings)", self.rings.len())
    }
}

impl HasShapeType for PolygonM {
    fn shapetype() -> ShapeType {
        ShapeType::PolygonM
    }
}

impl ConcreteReadableShape for PolygonM {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        PolylineM::read_shape_content(source, record_size).map(PolygonM::from)
    }
}

impl WritableShape for PolygonM {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0_usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.rings.len();
        size += 3 * size_of::<f64>() * self.total_point_count();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error> {
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

/// Specialization of the `GenericPolygon` struct to represent a `PolygonZ` shape
/// ( collection of [PointZ](../point/struct.PointZ.html))
pub type PolygonZ = GenericPolygon<PointZ>;

impl fmt::Display for PolygonZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PolygonZ({} rings)", self.rings.len())
    }
}

impl HasShapeType for PolygonZ {
    fn shapetype() -> ShapeType {
        ShapeType::PolygonZ
    }
}

impl ConcreteReadableShape for PolygonZ {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
        PolylineZ::read_shape_content(source, record_size).map(PolygonZ::from)
    }
}

impl WritableShape for PolygonZ {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0_usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.rings.len();
        size += 4 * size_of::<f64>() * self.total_point_count();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error> {
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

#[cfg(feature = "geo-types")]
impl<PointType> From<GenericPolygon<PointType>> for geo_types::MultiPolygon<f64>
where
    PointType: ShrinkablePoint + GrowablePoint + Copy,
    geo_types::Coord<f64>: From<PointType>,
{
    fn from(p: GenericPolygon<PointType>) -> Self {
        let mut last_poly = None;
        let mut polygons = Vec::<geo_types::Polygon<f64>>::new();
        for ring in p.rings {
            match ring {
                PolygonRing::Outer(points) => {
                    let exterior = points
                        .into_iter()
                        .map(Coord::<f64>::from)
                        .collect::<Vec<Coord<f64>>>();

                    if let Some(poly) = last_poly.take() {
                        polygons.push(poly);
                    }
                    last_poly = Some(geo_types::Polygon::new(LineString::from(exterior), vec![]))
                }
                PolygonRing::Inner(points) => {
                    let interior = points
                        .into_iter()
                        .map(Coord::<f64>::from)
                        .collect::<Vec<Coord<f64>>>();

                    if let Some(poly) = last_poly.as_mut() {
                        poly.interiors_push(interior);
                    } else {
                        // This is the strange (?) case: inner ring without a previous outer ring
                        polygons.push(geo_types::Polygon::<f64>::new(
                            LineString::<f64>::from(Vec::<Coord<f64>>::new()),
                            vec![LineString::from(interior)],
                        ));
                    }
                }
            }
        }
        if let Some(poly) = last_poly.take() {
            polygons.push(poly);
        }
        polygons.into()
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::Polygon<f64>> for GenericPolygon<PointType>
where
    PointType:
        From<geo_types::Coord<f64>> + GrowablePoint + ShrinkablePoint + PartialEq + HasXY + Copy,
{
    fn from(polygon: geo_types::Polygon<f64>) -> Self {
        let (outer, inners) = polygon.into_inner();
        let mut rings = Vec::<PolygonRing<PointType>>::with_capacity(inners.len() + 1);

        rings.push(PolygonRing::Outer(
            outer.0.into_iter().map(PointType::from).collect(),
        ));
        for inner in inners {
            rings.push(PolygonRing::Inner(
                inner.0.into_iter().map(PointType::from).collect(),
            ));
        }
        Self::with_rings(rings)
    }
}

#[cfg(feature = "geo-types")]
impl<PointType> From<geo_types::MultiPolygon<f64>> for GenericPolygon<PointType>
where
    PointType: HasXY
        + From<geo_types::Coord<f64>>
        + GrowablePoint
        + ShrinkablePoint
        + PartialEq
        + HasXY
        + Copy,
{
    fn from(multi_polygon: geo_types::MultiPolygon<f64>) -> Self {
        let mut all_rings = Vec::<PolygonRing<PointType>>::new();
        for polygon in multi_polygon {
            let mut rings = GenericPolygon::<PointType>::from(polygon).into_inner();
            all_rings.append(&mut rings);
        }
        Self::with_rings(all_rings)
    }
}

#[cfg(test)]
#[cfg(feature = "geo-types")]
mod test_geo_types {
    use super::*;
    #[test]
    fn shapefile_polygon_to_geotypes_polygon() {
        let simple_polygon = Polygon::new(PolygonRing::Outer(vec![
            Point::new(-1.1, -1.01),
            Point::new(-1.2, 1.02),
            Point::new(1.3, 1.03),
            Point::new(1.4, -1.04),
            Point::new(-1.1, -1.01),
        ]));

        let converted_multipolygon = geo_types::MultiPolygon::<f64>::from(simple_polygon);

        let converted_polygon = converted_multipolygon.into_iter().next().unwrap();

        let expected_geotypes_polygon = geo_types::Polygon::new(
            LineString::from(vec![
                (-1.1, -1.01),
                (-1.2, 1.02),
                (1.3, 1.03),
                (1.4, -1.04),
                (-1.1, -1.01),
            ]),
            vec![],
        );

        assert_eq!(converted_polygon, expected_geotypes_polygon);
    }

    #[test]
    fn shapefile_polygon_to_geotypes_polygon_auto_close() {
        let simple_polygon = Polygon::new(PolygonRing::Outer(vec![
            Point::new(-1.1, -1.01),
            Point::new(-1.2, 1.02),
            Point::new(1.3, 1.03),
            Point::new(1.4, -1.04),
        ]));

        let converted_polygon = geo_types::MultiPolygon::<f64>::from(simple_polygon);

        let converted_polygon = converted_polygon.into_iter().next().unwrap();

        let (geotypes_exterior, _) = converted_polygon.into_inner();

        assert_eq!(
            geotypes_exterior,
            LineString::from(vec![
                (-1.1, -1.01),
                (-1.2, 1.02),
                (1.3, 1.03),
                (1.4, -1.04),
                (-1.1, -1.01)
            ])
        );
    }

    #[test]
    fn geotypes_polygon_to_shapefile_polygon() {
        let geotypes_polygon = geo_types::Polygon::new(
            LineString::from(vec![
                (-1.1, -1.01),
                (-1.2, 1.02),
                (1.3, 1.03),
                (1.4, -1.04),
                (-1.1, -1.01),
            ]),
            vec![],
        );

        let converted_polygon = Polygon::from(geotypes_polygon);

        let expected_polygon = Polygon::new(PolygonRing::Outer(vec![
            Point::new(-1.1, -1.01),
            Point::new(-1.2, 1.02),
            Point::new(1.3, 1.03),
            Point::new(1.4, -1.04),
            Point::new(-1.1, -1.01),
        ]));

        assert_eq!(converted_polygon, expected_polygon);
    }

    #[test]
    fn shapefile_polygon_to_geotypes_polygon_with_inner_ring() {
        let one_ring_polygon = Polygon::with_rings(vec![
            PolygonRing::Outer(vec![
                Point::new(-1.1, -1.01),
                Point::new(-1.2, 1.02),
                Point::new(1.3, 1.03),
                Point::new(1.4, -1.04),
                Point::new(-1.1, -1.01),
            ]),
            PolygonRing::Inner(vec![
                Point::new(-0.51, -0.501),
                Point::new(0.54, -0.504),
                Point::new(0.53, 0.503),
                Point::new(-0.52, 0.502),
                Point::new(-0.51, -0.501),
            ]),
        ]);

        let converted_multipolygon = geo_types::MultiPolygon::<f64>::from(one_ring_polygon);

        let converted_polygon = converted_multipolygon.into_iter().next().unwrap();

        let expected_geotypes_polygon = geo_types::Polygon::new(
            LineString::from(vec![
                (-1.1, -1.01),
                (-1.2, 1.02),
                (1.3, 1.03),
                (1.4, -1.04),
                (-1.1, -1.01),
            ]),
            vec![LineString::from(vec![
                (-0.51, -0.501),
                (0.54, -0.504),
                (0.53, 0.503),
                (-0.52, 0.502),
                (-0.51, -0.501),
            ])],
        );

        assert_eq!(converted_polygon, expected_geotypes_polygon);
    }

    #[test]
    fn geotypes_polygon_to_shapefile_polygon_inner_ring() {
        let geotypes_polygon = geo_types::Polygon::new(
            LineString::from(vec![
                (-1.1, -1.01),
                (-1.2, 1.02),
                (1.3, 1.03),
                (1.4, -1.04),
                (-1.1, -1.01),
            ]),
            vec![LineString::from(vec![
                (-0.51, -0.501),
                (-0.52, 0.502),
                (0.53, 0.503),
                (0.54, -0.504),
                (-0.51, -0.501),
            ])],
        );

        let converted_polygon = Polygon::from(geotypes_polygon);

        let expected_polygon = Polygon::with_rings(vec![
            PolygonRing::Outer(vec![
                Point::new(-1.1, -1.01),
                Point::new(-1.2, 1.02),
                Point::new(1.3, 1.03),
                Point::new(1.4, -1.04),
                Point::new(-1.1, -1.01),
            ]),
            PolygonRing::Inner(vec![
                Point::new(-0.51, -0.501),
                Point::new(0.54, -0.504),
                Point::new(0.53, 0.503),
                Point::new(-0.52, 0.502),
                Point::new(-0.51, -0.501),
            ]),
        ]);

        assert_eq!(converted_polygon, expected_polygon);
    }
}
