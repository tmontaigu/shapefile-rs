use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::fmt;

mod io;
pub mod poly;
pub mod point;
pub mod multipoint;
pub mod multipatch;

use super::{ShapeType, Error};
pub use record::point::{Point, PointM, PointZ};
pub use record::poly::{Polyline, PolylineM, PolylineZ};
pub use record::poly::{Polygon, PolygonM, PolygonZ};
pub use record::multipoint::{Multipoint, MultipointM, MultipointZ};
pub use record::multipatch::{Multipatch, PatchType};

use record::io::HasXY;

/// Value inferior to this are considered as NO_DATA
pub const NO_DATA: f64 = -10e38;

fn is_no_data(val: f64) -> bool {
    return val <= NO_DATA;
}


/// Traits to be able to retrieve the ShapeType corresponding to the type
pub trait HasShapeType {
    /// Returns the ShapeType
    fn shapetype() -> ShapeType;
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
    /// `An array of length NumParts. Stores, for each PolyLine, the index of its`
    /// `first point in the points array. Array indexes are with respect to 0`
    fn parts(&self) -> &[i32];

    /// Returns the slice of points corresponding to part n°`ìndex` if the shape
    /// actually has multiple parts
    fn part(&self, index: usize) -> Option<&[PointType]> {
        let parts = self.parts();
        if parts.len() < 2 {
            Some(self.points())
        } else {
            let first_index = *parts.get(index)? as usize;
            let last_index = *parts.get(index + 1)? as usize;
            self.points().get(first_index..last_index)
        }
    }
}


/// Trait implemented by all the Shapes that can be read
pub trait ReadableShape: HasShapeType {
    /// The type of shapes that will be returned when read
    type ActualShape;

    /// Function that actually reads the `ActualShape` from the source
    ///and returns it
    fn read_from<T: Read>(source: &mut T) -> Result<Self::ActualShape, Error>;
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
    let num_points = shape.points().len() as i32;
    shape.parts().iter().all(|p| (*p >= 0) & (*p < num_points))
}

/// enum of Shapes that can be read of written to a shapefile
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

impl Shape {
    pub fn read_from<T: Read>(mut source: &mut T, shapetype: ShapeType) -> Result<Shape, Error> {
        let shape = match shapetype {
            ShapeType::Polyline => Shape::Polyline(Polyline::read_from(&mut source)?),
            ShapeType::PolylineM => Shape::PolylineM(PolylineM::read_from(&mut source)?),
            ShapeType::PolylineZ => Shape::PolylineZ(PolylineZ::read_from(&mut source)?),
            ShapeType::Point => Shape::Point(Point::read_from(&mut source)?),
            ShapeType::PointM => Shape::PointM(PointM::read_from(&mut source)?),
            ShapeType::PointZ => Shape::PointZ(PointZ::read_from(&mut source)?),
            ShapeType::Polygon => Shape::Polygon(Polygon::read_from(&mut source)?),
            ShapeType::PolygonM => Shape::PolygonM(PolygonM::read_from(&mut source)?),
            ShapeType::PolygonZ => Shape::PolygonZ(PolygonZ::read_from(&mut source)?),
            ShapeType::Multipoint => Shape::Multipoint(Multipoint::read_from(&mut source)?),
            ShapeType::MultipointM => Shape::MultipointM(MultipointM::read_from(&mut source)?),
            ShapeType::MultipointZ => Shape::MultipointZ(MultipointZ::read_from(&mut source)?),
            ShapeType::Multipatch => Shape::Multipatch(Multipatch::read_from(&mut source)?),
            ShapeType::NullShape => Shape::NullShape,
        };
        Ok(shape)
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

macro_rules! impl_from_concrete_shape {
    (Shape::$ShapeEnumVariant:ident, $ConcreteShape:ident) => {
        impl From<$ConcreteShape> for Shape {
            fn from(concrete: $ConcreteShape) -> Self {
                Shape::$ShapeEnumVariant(concrete)
            }
        }
    };
}

impl_from_concrete_shape!(Shape::Polyline, Polyline);
impl_from_concrete_shape!(Shape::PolylineM, PolylineM);
impl_from_concrete_shape!(Shape::PolylineZ, PolylineZ);
impl_from_concrete_shape!(Shape::Polygon, Polygon);
impl_from_concrete_shape!(Shape::PolygonM, PolygonM);
impl_from_concrete_shape!(Shape::PolygonZ, PolygonZ);
impl_from_concrete_shape!(Shape::Multipoint, Multipoint);
impl_from_concrete_shape!(Shape::MultipointM, MultipointM);
impl_from_concrete_shape!(Shape::MultipointZ, MultipointZ);
impl_from_concrete_shape!(Shape::Multipatch, Multipatch);


#[derive(Copy, Clone)]
pub struct BBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl BBox {
    pub fn from_points<PointType: HasXY>(points: &Vec<PointType>) -> Self {
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
        Self{xmin, ymin, xmax, ymax}
    }

    pub fn read_from<T: Read>(mut source: T) -> Result<BBox, std::io::Error> {
        let xmin = source.read_f64::<LittleEndian>()?;
        let ymin = source.read_f64::<LittleEndian>()?;
        let xmax = source.read_f64::<LittleEndian>()?;
        let ymax = source.read_f64::<LittleEndian>()?;
        Ok(BBox { xmin, ymin, xmax, ymax })
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
    pub fn read_from<T: Read>(source: &mut T) -> Result<RecordHeader, Error> {
        let record_number = source.read_i32::<BigEndian>()?;
        let record_size = source.read_i32::<BigEndian>()?;
        Ok(RecordHeader { record_number, record_size })
    }

    pub fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), std::io::Error> {
        dest.write_i32::<BigEndian>(self.record_number)?;
        dest.write_i32::<BigEndian>(self.record_size)?;
        Ok(())
    }
}

macro_rules! shape_vector_conversion {
     ($funcname:ident, $ConcreteShapeStruct: ty, Shape::$EnumVariant:ident) => (
            pub fn $funcname(shapes: Vec<Shape>) -> Result<Vec<$ConcreteShapeStruct>, Error> {
                let mut shape_structs = Vec::<$ConcreteShapeStruct>::with_capacity(shapes.len());
                for shape in shapes {
                    match shape {
                        Shape::NullShape => {},
                        Shape::$EnumVariant(shp) => shape_structs.push(shp),
                        _ => {
                            return Err(Error::MixedShapeType);
                        },
                    }
                }
                Ok(shape_structs)
            }
    )
}

shape_vector_conversion!(to_vec_of_polyline, Polyline, Shape::Polyline);
shape_vector_conversion!(to_vec_of_polylinem, PolylineM, Shape::PolylineM);
shape_vector_conversion!(to_vec_of_polylinez, PolylineZ, Shape::PolylineZ);

shape_vector_conversion!(to_vec_of_point, Point, Shape::Point);
shape_vector_conversion!(to_vec_of_pointm, PointM, Shape::PointM);
shape_vector_conversion!(to_vec_of_pointz, PointZ, Shape::PointZ);

shape_vector_conversion!(to_vec_of_polygon, Polygon, Shape::Polygon);
shape_vector_conversion!(to_vec_of_polygonm, PolygonM, Shape::PolygonM);
shape_vector_conversion!(to_vec_of_polygonz, PolygonZ, Shape::PolygonZ);

shape_vector_conversion!(to_vec_of_multipoint, Multipoint, Shape::Multipoint);
shape_vector_conversion!(to_vec_of_multipointm, MultipointM, Shape::MultipointM);
shape_vector_conversion!(to_vec_of_multipointz, MultipointZ, Shape::MultipointZ);

shape_vector_conversion!(to_vec_of_multipatch, Multipatch, Shape::Multipatch);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_vec_of_poly_err() {
        let points = vec![Point::default(), Point::default()];
        let parts = Vec::<i32>::new();
        let shapes = vec![Shape::Point(Point::default()), Shape::Polyline(Polyline::new(points, parts))];
        assert!(to_vec_of_polyline(shapes).is_err());
    }

    #[test]
    fn convert_to_vec_of_point_err() {
        let points = vec![Point::default(), Point::default()];
        let parts = Vec::<i32>::new();
        let shapes = vec![Shape::Point(Point::default()), Shape::Polyline(Polyline::new(points, parts))];
        assert!(to_vec_of_point(shapes).is_err());
    }


    #[test]
    fn convert_to_vec_of_poly_ok() {
        let points = vec![Point::default(), Point::default()];
        let parts = Vec::<i32>::new();

        let shapes = vec![Shape::from(Polyline::new(points.clone(),parts.clone())), Shape::from(Polyline::new(points, parts))];

        assert!(to_vec_of_polyline(shapes).is_ok());
    }

    #[test]
    fn convert_to_vec_of_point_ok() {
        let shapes = vec!(Shape::Point(Point::default()), Shape::Point(Point::default()));
        assert!(to_vec_of_point(shapes).is_ok());
    }
}
