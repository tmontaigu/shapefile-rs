//! Shape records
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io::{Read, Write};

pub mod bbox;
pub(crate) mod io;
pub mod macros;
pub mod multipatch;
pub mod multipoint;
pub mod point;
pub mod polygon;
pub mod polyline;
pub mod traits;

use super::{Error, ShapeType};
pub use bbox::{BBoxZ, GenericBBox};
pub use multipatch::{Multipatch, Patch};
pub use multipoint::{Multipoint, MultipointM, MultipointZ};
pub use point::{Point, PointM, PointZ};
pub use polygon::{Polygon, PolygonM, PolygonRing, PolygonZ};
pub use polyline::{Polyline, PolylineM, PolylineZ};
use traits::HasXY;

#[cfg(feature = "geo-types")]
use geo_types;

/// Value inferior to this are considered as NO_DATA
pub const NO_DATA: f64 = -10e38;

fn is_no_data(val: f64) -> bool {
    val <= NO_DATA
}

/// Traits to be able to retrieve the ShapeType corresponding to the type
pub trait HasShapeType {
    /// Returns the ShapeType
    fn shapetype() -> ShapeType;
}

/// Simple Trait to store the type of the shape
pub trait ConcreteShape: Sized + HasShapeType {}

pub trait ConcreteReadableShape: ConcreteShape {
    /// Function that actually reads the `ActualShape` from the source
    /// and returns it
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error>;
}

/// Trait implemented by all the Shapes that can be read
pub trait ReadableShape: Sized {
    fn read_from<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error>;
}

impl<S: ConcreteReadableShape> ReadableShape for S {
    fn read_from<T: Read>(mut source: &mut T, mut record_size: i32) -> Result<S, Error> {
        let shapetype = ShapeType::read_from(&mut source)?;
        record_size -= std::mem::size_of::<i32>() as i32;
        if shapetype == Self::shapetype() {
            S::read_shape_content(&mut source, record_size)
        } else {
            Err(Error::MismatchShapeType {
                requested: Self::shapetype(),
                actual: shapetype,
            })
        }
    }
}

/// Trait implemented by all Shapes that can be written
pub trait WritableShape {
    /// Returns the size in bytes that the Shapes will take once written.
    /// Does _not_ include the shapetype
    fn size_in_bytes(&self) -> usize;

    /// Writes the shape to the dest
    fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), Error>;
}

pub trait EsriShape: HasShapeType + WritableShape {
    fn x_range(&self) -> [f64; 2];
    fn y_range(&self) -> [f64; 2];
    /// Should return the Z range of this shape
    fn z_range(&self) -> [f64; 2] {
        [0.0, 0.0]
    }
    /// Should return the M range of this shape
    fn m_range(&self) -> [f64; 2] {
        [0.0, 0.0]
    }
}

pub(crate) fn is_part_closed<PointType: PartialEq>(points: &[PointType]) -> bool {
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        first == last
    } else {
        false
    }
}

pub(crate) fn close_points_if_not_already<PointType: PartialEq + Copy>(
    points: &mut Vec<PointType>,
) {
    if !is_part_closed(points) {
        if let Some(point) = points.first().copied() {
            points.push(point)
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub(crate) enum RingType {
    OuterRing,
    InnerRing,
}

/// Given the points, check if they represent an outer ring of a polygon
///
/// As per ESRI's Shapefile 1998 whitepaper:
/// `
/// The order of vertices or orientation for a ring indicates which side of the ring
/// is the interior of the polygon.
/// The neighborhood to the right of an observer walking along
/// the ring in vertex order is the neighborhood inside the polygon.
/// Vertices of rings defining holes in polygons are in a counterclockwise direction.
/// Vertices for a single, ringed polygon are, therefore, always in clockwise order.
/// `
///
/// Inner Rings defines holes -> points are in counterclockwise order
/// Outer Rings's points are un clockwise order
///
/// https://stackoverflow.com/questions/1165647/how-to-determine-if-a-list-of-polygon-points-are-in-clockwise-order/1180256#1180256
pub(crate) fn ring_type_from_points_ordering<PointType: HasXY>(points: &[PointType]) -> RingType {
    let area = points
        .windows(2)
        .map(|pts| (pts[1].x() - pts[0].x()) * (pts[1].y() + pts[0].y()))
        .sum::<f64>()
        / 2.0f64;

    if area < 0.0 {
        RingType::InnerRing
    } else {
        RingType::OuterRing
    }
}

/// enum of Shapes that can be read or written to a shapefile
///
/// # geo-types
///
/// `shapefile::Shape` and `geo_types::Geometry<f64>` can be converted from one to another,
/// however this conversion is not infallible so it is done using `TryFrom`
///
/// ```
/// # #[cfg(feature = "geo-types")]
/// # fn main() -> Result<(), shapefile::Error>{
/// use std::convert::TryFrom;
/// use shapefile::Shape;
/// let mut shapes = shapefile::read_shapes("tests/data/line.shp")?;
/// let last_shape = shapes.pop().unwrap();
/// let geometry = geo_types::Geometry::<f64>::try_from(last_shape);
///
/// assert_eq!(geometry.is_ok(), true);
/// assert_eq!(geo_types::Geometry::<f64>::try_from(Shape::NullShape).is_err(), true);
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "geo-types"))]
/// # fn main() {}
/// ```
///
pub enum Shape {
    NullShape,
    Point(Point),
    PointM(PointM),
    PointZ(PointZ),
    Polyline(Polyline),
    PolylineM(PolylineM),
    PolylineZ(PolylineZ),
    Polygon(Polygon),
    PolygonM(PolygonM),
    PolygonZ(PolygonZ),
    Multipoint(Multipoint),
    MultipointM(MultipointM),
    MultipointZ(MultipointZ),
    Multipatch(Multipatch),
}

impl HasShapeType for Shape {
    fn shapetype() -> ShapeType {
        ShapeType::Point
    }
}

impl ReadableShape for Shape {
    fn read_from<T: Read>(mut source: &mut T, mut record_size: i32) -> Result<Self, Error> {
        let shapetype = ShapeType::read_from(&mut source)?;
        record_size -= std::mem::size_of::<i32>() as i32;
        let shape = match shapetype {
            ShapeType::Polyline => {
                Shape::Polyline(Polyline::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::PolylineM => {
                Shape::PolylineM(PolylineM::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::PolylineZ => {
                Shape::PolylineZ(PolylineZ::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::Point => Shape::Point(Point::read_shape_content(&mut source, record_size)?),
            ShapeType::PointM => {
                Shape::PointM(PointM::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::PointZ => {
                Shape::PointZ(PointZ::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::Polygon => {
                Shape::Polygon(Polygon::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::PolygonM => {
                Shape::PolygonM(PolygonM::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::PolygonZ => {
                Shape::PolygonZ(PolygonZ::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::Multipoint => {
                Shape::Multipoint(Multipoint::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::MultipointM => {
                Shape::MultipointM(MultipointM::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::MultipointZ => {
                Shape::MultipointZ(MultipointZ::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::Multipatch => {
                Shape::Multipatch(Multipatch::read_shape_content(&mut source, record_size)?)
            }
            ShapeType::NullShape => Shape::NullShape,
        };
        Ok(shape)
    }
}

impl Shape {
    /// Returns the shapetype
    pub fn shapetype(&self) -> ShapeType {
        match self {
            Shape::Polyline(_) => ShapeType::Polyline,
            Shape::PolylineM(_) => ShapeType::PolylineM,
            Shape::PolylineZ(_) => ShapeType::PolylineZ,
            Shape::Point(_) => ShapeType::Point,
            Shape::PointM(_) => ShapeType::PointM,
            Shape::PointZ(_) => ShapeType::PointZ,
            Shape::Polygon(_) => ShapeType::Polygon,
            Shape::PolygonM(_) => ShapeType::PolygonM,
            Shape::PolygonZ(_) => ShapeType::PolygonZ,
            Shape::Multipoint(_) => ShapeType::Multipoint,
            Shape::MultipointM(_) => ShapeType::Multipoint,
            Shape::MultipointZ(_) => ShapeType::Multipoint,
            Shape::Multipatch(_) => ShapeType::Multipatch,
            Shape::NullShape => ShapeType::NullShape,
        }
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Shape::")?;
        match self {
            Shape::Polyline(shp) => write!(f, "{}", shp),
            Shape::PolylineM(shp) => write!(f, "{}", shp),
            Shape::PolylineZ(shp) => write!(f, "{}", shp),
            Shape::Point(shp) => write!(f, "{}", shp),
            Shape::PointM(shp) => write!(f, "{}", shp),
            Shape::PointZ(shp) => write!(f, "{}", shp),
            Shape::Polygon(shp) => write!(f, "{}", shp),
            Shape::PolygonM(shp) => write!(f, "{}", shp),
            Shape::PolygonZ(shp) => write!(f, "{}", shp),
            Shape::Multipoint(shp) => write!(f, "{}", shp),
            Shape::MultipointM(shp) => write!(f, "{}", shp),
            Shape::MultipointZ(shp) => write!(f, "{}", shp),
            Shape::Multipatch(shp) => write!(f, "{}", shp),
            Shape::NullShape => write!(f, "NullShape"),
        }
    }
}

/// Header of a shape record, present before any shape record
#[derive(Debug, Copy, Clone)]
pub(crate) struct RecordHeader {
    pub record_number: i32,
    pub record_size: i32,
}

impl RecordHeader {
    pub(crate) const SIZE: usize = 2 * std::mem::size_of::<i32>();

    pub fn read_from<T: Read>(source: &mut T) -> Result<RecordHeader, Error> {
        let record_number = source.read_i32::<BigEndian>()?;
        let record_size = source.read_i32::<BigEndian>()?;
        Ok(RecordHeader {
            record_number,
            record_size,
        })
    }

    pub fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), std::io::Error> {
        dest.write_i32::<BigEndian>(self.record_number)?;
        dest.write_i32::<BigEndian>(self.record_size)?;
        Ok(())
    }
}

/// Function that can converts a `Vec<Shape>` to a vector of any real struct
/// (ie [Polyline](polyline/type.Polyline.html), [Multipatch](multipatch/struct.Multipatch.html), etc)
/// if all the `Shapes` in the `Vec` are of the correct corresponding variant.
///
/// # Examples
///
/// ```
/// use shapefile::{Polyline, Multipoint, Point, Shape};
/// use shapefile::convert_shapes_to_vec_of;
///
/// // Build a Vec<Shape> with only polylines in it
/// let points = vec![Point::default(), Point::default()];
/// let shapes = vec![
///     Shape::from(Polyline::new(points.clone())),
///     Shape::from(Polyline::new(points)),
/// ];
///
/// // try a conversion to the wrong type
/// assert_eq!(convert_shapes_to_vec_of::<Multipoint>(shapes).is_ok(), false);
/// ```
///
/// ```
/// # fn main() -> Result<(), shapefile::Error> {
/// use shapefile::{convert_shapes_to_vec_of, MultipointZ};
/// let shapes = shapefile::read_shapes("tests/data/multipointz.shp")?;
/// let multipoints = convert_shapes_to_vec_of::<MultipointZ>(shapes);
/// assert_eq!(multipoints.is_ok(), true);
/// # Ok(())
/// # }
/// ```
pub fn convert_shapes_to_vec_of<S>(shapes: Vec<Shape>) -> Result<Vec<S>, Error>
where
    S: TryFrom<Shape>,
    Error: From<<S as TryFrom<Shape>>::Error>,
{
    let mut concrete_shapes = Vec::<S>::with_capacity(shapes.len());
    for shape in shapes {
        let concrete = S::try_from(shape)?;
        concrete_shapes.push(concrete);
    }
    Ok(concrete_shapes)
}

/// Macro to have less boiler plate code to write just to implement
/// the ConcreteShape Trait
macro_rules! impl_concrete_shape_for {
    ($ConcreteType:ident) => {
        impl ConcreteShape for $ConcreteType {}
    };
}

/// macro that implements the From<T> Trait for the Shape enum
/// where T is any of the ConcreteShape
macro_rules! impl_from_concrete_shape {
    ($ConcreteShape:ident=>Shape::$ShapeEnumVariant:ident) => {
        impl From<$ConcreteShape> for Shape {
            fn from(concrete: $ConcreteShape) -> Self {
                Shape::$ShapeEnumVariant(concrete)
            }
        }
    };
}

/// macro to implement the TryFrom<Shape> trait
macro_rules! impl_try_from_shape {
    (Shape::$ShapeEnumVariant:ident=>$ConcreteShape:ident) => {
        impl TryFrom<Shape> for $ConcreteShape {
            type Error = Error;
            fn try_from(shape: Shape) -> Result<Self, Self::Error> {
                match shape {
                    Shape::$ShapeEnumVariant(shp) => Ok(shp),
                    _ => Err(Error::MismatchShapeType {
                        requested: Self::shapetype(),
                        actual: shape.shapetype(),
                    }),
                }
            }
        }
    };
}

macro_rules! impl_to_way_conversion {
    (Shape::$ShapeEnumVariant:ident<=>$ConcreteShape:ident) => {
        impl_try_from_shape!(Shape::$ShapeEnumVariant => $ConcreteShape);
        impl_from_concrete_shape!($ConcreteShape => Shape::$ShapeEnumVariant);
    };
}

impl_concrete_shape_for!(Point);
impl_concrete_shape_for!(PointM);
impl_concrete_shape_for!(PointZ);
impl_concrete_shape_for!(Polyline);
impl_concrete_shape_for!(PolylineM);
impl_concrete_shape_for!(PolylineZ);
impl_concrete_shape_for!(Polygon);
impl_concrete_shape_for!(PolygonM);
impl_concrete_shape_for!(PolygonZ);
impl_concrete_shape_for!(Multipoint);
impl_concrete_shape_for!(MultipointM);
impl_concrete_shape_for!(MultipointZ);
impl_concrete_shape_for!(Multipatch);

impl_to_way_conversion!(Shape::Point <=> Point);
impl_to_way_conversion!(Shape::PointM <=> PointM);
impl_to_way_conversion!(Shape::PointZ <=> PointZ);
impl_to_way_conversion!(Shape::Polyline <=> Polyline);
impl_to_way_conversion!(Shape::PolylineM <=> PolylineM);
impl_to_way_conversion!(Shape::PolylineZ <=> PolylineZ);
impl_to_way_conversion!(Shape::Polygon <=> Polygon);
impl_to_way_conversion!(Shape::PolygonM <=> PolygonM);
impl_to_way_conversion!(Shape::PolygonZ <=> PolygonZ);
impl_to_way_conversion!(Shape::Multipoint <=> Multipoint);
impl_to_way_conversion!(Shape::MultipointM <=> MultipointM);
impl_to_way_conversion!(Shape::MultipointZ <=> MultipointZ);
impl_to_way_conversion!(Shape::Multipatch <=> Multipatch);

/// Tries to convert a shapefile's Shape into a geo_types::Geometry
///
/// This conversion can fail because the conversion of shapefile's polygons & multipatch into
/// their geo_types counter parts can fail. And the NullShape has no equivalent Geometry;
#[cfg(feature = "geo-types")]
impl TryFrom<Shape> for geo_types::Geometry<f64> {
    type Error = &'static str;

    fn try_from(shape: Shape) -> Result<Self, Self::Error> {
        use geo_types::Geometry;
        match shape {
            Shape::NullShape => Err("Cannot convert NullShape into any geo_types Geometry"),
            Shape::Point(point) => Ok(Geometry::Point(geo_types::Point::from(point))),
            Shape::PointM(point) => Ok(Geometry::Point(geo_types::Point::from(point))),
            Shape::PointZ(point) => Ok(Geometry::Point(geo_types::Point::from(point))),
            Shape::Polyline(polyline) => Ok(Geometry::MultiLineString(
                geo_types::MultiLineString::<f64>::from(polyline),
            )),
            Shape::PolylineM(polyline) => Ok(Geometry::MultiLineString(
                geo_types::MultiLineString::<f64>::from(polyline),
            )),
            Shape::PolylineZ(polyline) => Ok(Geometry::MultiLineString(
                geo_types::MultiLineString::<f64>::from(polyline),
            )),
            Shape::Polygon(polygon) => Ok(Geometry::MultiPolygon(
                geo_types::MultiPolygon::<f64>::from(polygon),
            )),
            Shape::PolygonM(polygon) => Ok(Geometry::MultiPolygon(
                geo_types::MultiPolygon::<f64>::from(polygon),
            )),
            Shape::PolygonZ(polygon) => Ok(Geometry::MultiPolygon(
                geo_types::MultiPolygon::<f64>::from(polygon),
            )),
            Shape::Multipoint(multipoint) => Ok(Geometry::MultiPoint(
                geo_types::MultiPoint::<f64>::from(multipoint),
            )),
            Shape::MultipointM(multipoint) => Ok(Geometry::MultiPoint(
                geo_types::MultiPoint::<f64>::from(multipoint),
            )),
            Shape::MultipointZ(multipoint) => Ok(Geometry::MultiPoint(
                geo_types::MultiPoint::<f64>::from(multipoint),
            )),
            Shape::Multipatch(multipatch) => {
                geo_types::MultiPolygon::<f64>::try_from(multipatch).map(Geometry::MultiPolygon)
            }
        }
    }
}

/// Converts a Geometry to a Shape
///
/// Since all Geometries are in 2D, the resulting shape will be 2D
/// (Polygon, Polyline, etc and not PolylineM, PolylineZ, etc)
///
/// Fails if the geometry is a GeometryCollection, Rect, or Triangle
#[cfg(feature = "geo-types")]
impl TryFrom<geo_types::Geometry<f64>> for Shape {
    type Error = &'static str;
    fn try_from(geometry: geo_types::Geometry<f64>) -> Result<Self, Self::Error> {
        match geometry {
            geo_types::Geometry::Point(point) => Ok(Shape::Point(point.into())),
            geo_types::Geometry::Line(line) => Ok(Shape::Polyline(line.into())),
            geo_types::Geometry::LineString(polyline) => Ok(Shape::Polyline(polyline.into())),
            geo_types::Geometry::Polygon(polygon) => Ok(Shape::Polygon(polygon.into())),
            geo_types::Geometry::MultiPoint(multipoint) => Ok(Shape::Multipoint(multipoint.into())),
            geo_types::Geometry::MultiLineString(multi_linestring) => {
                Ok(Shape::Polyline(multi_linestring.into()))
            }
            geo_types::Geometry::MultiPolygon(multi_polygon) => {
                Ok(Shape::Polygon(multi_polygon.into()))
            }
            geo_types::Geometry::GeometryCollection(_) => {
                Err("Cannot convert geo_types::GeometryCollection into a Shape")
            }
            #[allow(unreachable_patterns)] // Unreachable before geo-types 0.6.0
            _ => {
                // New geometries Rect(_) and Triangle(_) added in 0.6.0
                Err("Cannot convert unrecognized Geometry type into a Shape")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_vec_of_poly_err() {
        let points = vec![Point::default(), Point::default()];
        let shapes = vec![
            Shape::Point(Point::default()),
            Shape::Polyline(Polyline::new(points)),
        ];
        assert!(convert_shapes_to_vec_of::<Polyline>(shapes).is_err());
    }

    #[test]
    fn convert_to_vec_of_point_err() {
        let points = vec![Point::default(), Point::default()];
        let shapes = vec![
            Shape::Point(Point::default()),
            Shape::Polyline(Polyline::new(points)),
        ];
        assert!(convert_shapes_to_vec_of::<Point>(shapes).is_err());
    }

    #[test]
    fn convert_to_vec_of_poly_ok() {
        let points = vec![Point::default(), Point::default()];

        let shapes = vec![
            Shape::from(Polyline::new(points.clone())),
            Shape::from(Polyline::new(points)),
        ];

        assert!(convert_shapes_to_vec_of::<Polyline>(shapes).is_ok());
    }

    #[test]
    fn convert_to_vec_of_point_ok() {
        let shapes = vec![
            Shape::Point(Point::default()),
            Shape::Point(Point::default()),
        ];
        assert!(convert_shapes_to_vec_of::<Point>(shapes).is_ok());
    }

    #[test]
    fn test_vertices_order() {
        let mut points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
        ];

        assert_eq!(ring_type_from_points_ordering(&points), RingType::InnerRing);
        points.reverse();
        assert_eq!(ring_type_from_points_ordering(&points), RingType::OuterRing);
    }
}
