use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io::{Read, Write};

pub mod io;
pub mod multipatch;
pub mod multipoint;
pub mod point;
pub mod poly;
pub mod traits;

use super::{Error, ShapeType};
pub use record::multipatch::{Multipatch, PatchType};
pub use record::multipoint::{Multipoint, MultipointM, MultipointZ};
pub use record::traits::{MultipointShape, MultipartShape};
pub use record::point::{Point, PointM, PointZ};
pub use record::poly::{Polygon, PolygonM, PolygonZ};
pub use record::poly::{Polyline, PolylineM, PolylineZ};
use record::traits::HasXY;
use std::convert::TryFrom;

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
pub trait ConcreteShape {
    type ActualShape;
}

pub trait ConcreteReadableShape: ConcreteShape + HasShapeType {
    /// Function that actually reads the `ActualShape` from the source
    /// and returns it
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self::ActualShape, Error>;
}

/// Trait implemented by all the Shapes that can be read
pub trait ReadableShape {
    type ReadShape;
    fn read_from<T: Read>(source: &mut T, record_size: i32) -> Result<Self::ReadShape, Error>;
}

impl<S: ConcreteReadableShape> ReadableShape for S {
    type ReadShape = S::ActualShape;
    fn read_from<T: Read>(mut source: &mut T, mut record_size: i32) -> Result<Self::ReadShape, Error> {
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
    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error>;
}

pub trait EsriShape: HasShapeType + WritableShape {
    fn bbox(&self) -> BBox;
    /// Should the Z range of this shape (maybe require computing it)
    fn z_range(&self) -> [f64; 2] {
        [0.0, 0.0]
    }
    /// Should the M range of this shape (maybe require computing it)
    fn m_range(&self) -> [f64; 2] {
        [0.0, 0.0]
    }
}

/// Validate the `parts array` of the any `MultipartShape`.
///
/// Requirements for a parts array to be valid are
///
/// 1) at least one part
/// 2) indices must be in range [0, num_points[
pub(crate) fn is_parts_array_valid<PointType, ST: MultipartShape<PointType>>(shape: &ST) -> bool {
    if shape.parts_indices().is_empty() {
        return false;
    }
    let num_points = shape.points().len() as i32;
    shape
        .parts_indices()
        .iter()
        .all(|p| (*p >= 0) & (*p < num_points))
}

/// enum of Shapes that can be read or written to a shapefile
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
    type ReadShape = Self;
    fn read_from<T: Read>(mut source: &mut T, mut record_size: i32) -> Result<Self::ReadShape, Error> {
        let shapetype = ShapeType::read_from(&mut source)?;
        record_size -= std::mem::size_of::<i32>() as i32;
        let shape = match shapetype {
            ShapeType::Polyline => Shape::Polyline(Polyline::read_shape_content(&mut source, record_size)?),
            ShapeType::PolylineM => Shape::PolylineM(PolylineM::read_shape_content(&mut source, record_size)?),
            ShapeType::PolylineZ => Shape::PolylineZ(PolylineZ::read_shape_content(&mut source, record_size)?),
            ShapeType::Point => Shape::Point(Point::read_shape_content(&mut source, record_size)?),
            ShapeType::PointM => Shape::PointM(PointM::read_shape_content(&mut source, record_size)?),
            ShapeType::PointZ => Shape::PointZ(PointZ::read_shape_content(&mut source, record_size)?),
            ShapeType::Polygon => Shape::Polygon(Polygon::read_shape_content(&mut source, record_size)?),
            ShapeType::PolygonM => Shape::PolygonM(PolygonM::read_shape_content(&mut source, record_size)?),
            ShapeType::PolygonZ => Shape::PolygonZ(PolygonZ::read_shape_content(&mut source, record_size)?),
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

/// 2D (x, y) Bounding box
#[derive(Copy, Clone)]
pub struct BBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl BBox {
    /// Creates a new bounding box by computing the extent from
    /// any slice of points that have a x and y coordinates
    pub fn from_points<PointType: HasXY>(points: &[PointType]) -> Self {
        let mut xmin = std::f64::MAX;
        let mut ymin = std::f64::MAX;
        let mut xmax = std::f64::MIN;
        let mut ymax = std::f64::MIN;

        for point in points {
            xmin = f64::min(xmin, point.x());
            ymin = f64::min(ymin, point.y());
            xmax = f64::max(xmax, point.x());
            ymax = f64::max(ymax, point.y());
        }
        Self {
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }

    pub fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Self {
        BBox{xmin, ymin, xmax, ymax}
    }

    pub fn read_from<T: Read>(mut source: T) -> Result<BBox, std::io::Error> {
        let xmin = source.read_f64::<LittleEndian>()?;
        let ymin = source.read_f64::<LittleEndian>()?;
        let xmax = source.read_f64::<LittleEndian>()?;
        let ymax = source.read_f64::<LittleEndian>()?;
        Ok(BBox {
            xmin,
            ymin,
            xmax,
            ymax,
        })
    }

    pub fn write_to<T: Write>(&self, mut dest: T) -> Result<(), std::io::Error> {
        dest.write_f64::<LittleEndian>(self.xmin)?;
        dest.write_f64::<LittleEndian>(self.ymin)?;
        dest.write_f64::<LittleEndian>(self.xmax)?;
        dest.write_f64::<LittleEndian>(self.ymax)?;
        Ok(())
    }
}

/// Header of a shape record, present before any shape record
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
/// (ie [Polyline](poly/type.Polyline.html), [Multipatch](multipatch/struct.Multipatch.html), etc)
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
/// let parts = Vec::<i32>::new();
/// let shapes = vec![
///     Shape::from(Polyline::new(points.clone(), parts.clone())),
///     Shape::from(Polyline::new(points, parts)),
/// ];
///
/// // try a conversion to the wrong type
/// assert_eq!(convert_shapes_to_vec_of::<Multipoint>(shapes).is_ok(), false);
/// ```
///
/// ```
/// use shapefile::{convert_shapes_to_vec_of, MultipointZ};
/// let shapes = shapefile::read("tests/data/multipointz.shp").unwrap();
/// let multipoints = convert_shapes_to_vec_of::<MultipointZ>(shapes);
/// assert_eq!(multipoints.is_ok(), true);
/// ```
pub fn convert_shapes_to_vec_of<S>(
    shapes: Vec<Shape>,
) -> Result<Vec<S>, Error>
    where S:TryFrom<Shape>,
        Error: From<<S as TryFrom<Shape>>::Error> {
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
        impl ConcreteShape for $ConcreteType {
            type ActualShape = $ConcreteType;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_vec_of_poly_err() {
        let points = vec![Point::default(), Point::default()];
        let parts = Vec::<i32>::new();
        let shapes = vec![
            Shape::Point(Point::default()),
            Shape::Polyline(Polyline::new(points, parts)),
        ];
        assert!(convert_shapes_to_vec_of::<Polyline>(shapes).is_err());
    }

    #[test]
    fn convert_to_vec_of_point_err() {
        let points = vec![Point::default(), Point::default()];
        let parts = Vec::<i32>::new();
        let shapes = vec![
            Shape::Point(Point::default()),
            Shape::Polyline(Polyline::new(points, parts)),
        ];
        assert!(convert_shapes_to_vec_of::<Point>(shapes).is_err());
    }

    #[test]
    fn convert_to_vec_of_poly_ok() {
        let points = vec![Point::default(), Point::default()];
        let parts = Vec::<i32>::new();

        let shapes = vec![
            Shape::from(Polyline::new(points.clone(), parts.clone())),
            Shape::from(Polyline::new(points, parts)),
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
}
