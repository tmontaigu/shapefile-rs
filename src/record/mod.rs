use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io::{Read, Write};

pub mod io;
pub mod multipatch;
pub mod multipoint;
pub mod point;
pub mod poly;

use super::{Error, ShapeType};
pub use record::multipatch::{Multipatch, PatchType};
pub use record::multipoint::{Multipoint, MultipointM, MultipointZ};
pub use record::point::{Point, PointM, PointZ};
pub use record::poly::{Polygon, PolygonM, PolygonZ};
pub use record::poly::{Polyline, PolylineM, PolylineZ};

use record::io::HasXY;

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

pub trait ConcreteShape {
    type ActualShape;
}

// Basically we need Stabilized TryFrom trait
pub trait ConcreteShapeFromShape: ConcreteShape {
    fn try_from(shape: Shape) -> Result<Self::ActualShape, Error>;
}

/// Trait implemented by all the Shapes that are a collections of points
pub trait MultipointShape<PointType> {
    /// Returns a non mutable slice to the points
    fn points(&self) -> &[PointType];
    /*fn get<I: SliceIndex<[PointType]>>(&self, index: I) -> Option<&<I as SliceIndex<[PointType]>>::Output> {
        self.points().get(index)
    }*/
}

/// Trait for the Shapes that may have multiple parts
pub trait MultipartShape<PointType>: MultipointShape<PointType> {
    /// Returns a non mutable slice of the parts as written in the file:
    ///
    /// > An array of length NumParts. Stores, for each PolyLine, the index of its
    /// > first point in the points array. Array indexes are with respect to 0
    ///
    ///  # Examples
    ///
    /// ```
    /// use shapefile::record::MultipartShape;
    /// let filepath = "tests/data/linez.shp";
    /// let polylines_z = shapefile::read_as::<&str, shapefile::PolylineZ>(filepath).unwrap();
    ///
    /// let poly_z = &polylines_z[0];
    /// assert_eq!(poly_z.parts_indices(), &[0, 5, 7]);
    /// ```
    fn parts_indices(&self) -> &[i32];

    /// Returns the slice of points corresponding to part n°`ìndex` if the shape
    /// actually has multiple parts
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::record::MultipartShape;
    /// let filepath = "tests/data/linez.shp";
    /// let polylines_z = shapefile::read_as::<&str, shapefile::PolylineZ>(filepath).unwrap();
    ///
    /// let poly_z = &polylines_z[0];
    /// for points in poly_z.parts() {
    ///     println!("{} points", points.len());
    /// }
    /// ```
    fn part(&self, index: usize) -> Option<&[PointType]> {
        let parts = self.parts_indices();
        if parts.len() < 2 {
            Some(self.points())
        } else {
            let first_index = *parts.get(index)? as usize;
            let last_index = if index == parts.len() - 1 { self.points().len() } else { *parts.get(index + 1)? as usize };
            self.points().get(first_index..last_index)
        }
    }

    /// Returns an iterator over the parts of a MultipartShape
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::record::MultipartShape;
    /// let filepath = "tests/data/linez.shp";
    /// let polylines_z = shapefile::read_as::<&str, shapefile::PolylineZ>(filepath).unwrap();
    ///
    /// let poly_z = &polylines_z[0];
    /// let poly_z_parts: Vec<&[shapefile::PointZ]> = poly_z.parts().collect();
    /// assert_eq!(poly_z_parts.len(), 3);
    /// ```
    fn parts(&self) -> PartIterator<PointType, Self> {
        PartIterator {
            phantom: std::marker::PhantomData,
            shape: &self,
            current_part: 0,
        }
    }
}

/// Iterator over the parts of a Multipart shape
///
/// Each iteration yields a non-mutable slice of points of the current part
pub struct PartIterator<'a, PointType, Shape: 'a + MultipartShape<PointType> + ?Sized> {
    phantom: std::marker::PhantomData<PointType>,
    shape: &'a Shape,
    current_part: usize,
}

impl<'a, PointType: 'a, Shape: 'a + MultipartShape<PointType>> Iterator
for PartIterator<'a, PointType, Shape>
{
    type Item = &'a [PointType];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_part > self.shape.parts_indices().len() {
            None
        } else {
            self.current_part += 1;
            self.shape.part(self.current_part - 1)
        }
    }
}

pub trait ConcreteReadableShape: ConcreteShape + HasShapeType {
    /// Function that actually reads the `ActualShape` from the source
    ///and returns it
    fn read_shape_content<T: Read>(source: &mut T) -> Result<Self::ActualShape, Error>;
}

/// Trait implemented by all the Shapes that can be read
pub trait ReadableShape {
    type ReadShape;
    fn read_from<T: Read>(source: &mut T) -> Result<Self::ReadShape, Error>;
}

impl<S: ConcreteReadableShape> ReadableShape for S {
    type ReadShape = S::ActualShape;
    fn read_from<T: Read>(mut source: &mut T) -> Result<Self::ReadShape, Error> {
        let shapetype = ShapeType::read_from(&mut source)?;
        if shapetype == Self::shapetype() {
            S::read_shape_content(&mut source)
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
    fn z_range(&self) -> [f64; 2] {
        [0.0, 0.0]
    }
    fn m_range(&self) -> [f64; 2] {
        [0.0, 0.0]
    }
}

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
    fn read_from<T: Read>(mut source: &mut T) -> Result<Self::ReadShape, Error> {
        let shapetype = ShapeType::read_from(&mut source)?;
        let shape = match shapetype {
            ShapeType::Polyline => Shape::Polyline(Polyline::read_shape_content(&mut source)?),
            ShapeType::PolylineM => Shape::PolylineM(PolylineM::read_shape_content(&mut source)?),
            ShapeType::PolylineZ => Shape::PolylineZ(PolylineZ::read_shape_content(&mut source)?),
            ShapeType::Point => Shape::Point(Point::read_shape_content(&mut source)?),
            ShapeType::PointM => Shape::PointM(PointM::read_shape_content(&mut source)?),
            ShapeType::PointZ => Shape::PointZ(PointZ::read_shape_content(&mut source)?),
            ShapeType::Polygon => Shape::Polygon(Polygon::read_shape_content(&mut source)?),
            ShapeType::PolygonM => Shape::PolygonM(PolygonM::read_shape_content(&mut source)?),
            ShapeType::PolygonZ => Shape::PolygonZ(PolygonZ::read_shape_content(&mut source)?),
            ShapeType::Multipoint => {
                Shape::Multipoint(Multipoint::read_shape_content(&mut source)?)
            }
            ShapeType::MultipointM => {
                Shape::MultipointM(MultipointM::read_shape_content(&mut source)?)
            }
            ShapeType::MultipointZ => {
                Shape::MultipointZ(MultipointZ::read_shape_content(&mut source)?)
            }
            ShapeType::Multipatch => {
                Shape::Multipatch(Multipatch::read_shape_content(&mut source)?)
            }
            ShapeType::NullShape => Shape::NullShape,
        };
        Ok(shape)
    }
}

impl Shape {
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

#[derive(Copy, Clone)]
pub struct BBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl BBox {
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

pub struct RecordHeader {
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

pub fn convert_shapes_to_vec_of<S: ConcreteShapeFromShape>(
    shapes: Vec<Shape>,
) -> Result<Vec<S::ActualShape>, Error> {
    let mut concrete_shapes = Vec::<S::ActualShape>::with_capacity(shapes.len());
    for shape in shapes {
        let concrete = S::try_from(shape)?;
        concrete_shapes.push(concrete);
    }
    Ok(concrete_shapes)
}

macro_rules! impl_concrete_shape_for {
    ($ConcreteType:ident) => {
        impl ConcreteShape for $ConcreteType {
            type ActualShape = $ConcreteType;
        }
    };
}

macro_rules! impl_from_concrete_shape {
    ($ConcreteShape:ident=>Shape::$ShapeEnumVariant:ident) => {
        impl From<$ConcreteShape> for Shape {
            fn from(concrete: $ConcreteShape) -> Self {
                Shape::$ShapeEnumVariant(concrete)
            }
        }
    };
}

macro_rules! impl_to_concrete_shape {
    (Shape::$ShapeEnumVariant:ident=>$ConcreteShape:ident) => {
        impl ConcreteShapeFromShape for $ConcreteShape {
            fn try_from(shape: Shape) -> Result<Self::ActualShape, Error> {
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
        impl_to_concrete_shape!(Shape::$ShapeEnumVariant => $ConcreteShape);
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
