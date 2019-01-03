use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

mod io;
mod poly;
mod point;
mod multipoint;


use super::{ShapeType, Error};

pub use record::poly::{Polyline, PolylineM, PolylineZ, Polygon, PolygonM, PolygonZ};
pub use record::point::{Point, PointM, PointZ};
pub use record::multipoint::{Multipoint, MultipointZ};

pub const NO_DATA: f64 = -10e38;

fn is_no_data(val: f64) -> bool {
    return val <= NO_DATA;
}


pub fn min_and_max_of_f64_slice(slice: &[f64]) -> (f64, f64) {
    slice.iter().fold(
        (std::f64::NAN, std::f64::NAN),
        |(min, max), val| {
            (f64::min(min, *val), f64::max(max, *val))
        })
}

pub trait EsriShape {
    fn shapetype(&self) -> ShapeType;
    fn size_in_bytes(&self) -> usize;
    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error>;
}

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
    MultipointZ(MultipointZ),
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
            ShapeType::MultipointZ => Shape::MultipointZ(MultipointZ::read_from(&mut source)?),
            _ => { unimplemented!() }
        };
        Ok(shape)
    }
}


pub struct BBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl BBox {
    pub fn from_xys(xs: &Vec<f64>, ys: &Vec<f64>) -> Self {
        let (xmin, xmax) = min_and_max_of_f64_slice(&xs);
        let (ymin, ymax) = min_and_max_of_f64_slice(&ys);
        Self { xmin, ymin, xmax, ymax }
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
    ($funcname:ident, $shapestruct: ty, $pat:pat, $shp:ident) => {
        pub fn $funcname(shapes: Vec<Shape>) -> Result<Vec<$shapestruct>, Error> {
            let mut shape_structs = Vec::<$shapestruct>::with_capacity(shapes.len());
            for shape in shapes {
                match shape {
                    Shape::NullShape => {},
                    $pat => shape_structs.push($shp),
                    _ => {
                        return Err(Error::MixedShapeType);
                    },
                }
            }
            Ok(shape_structs)
        }
    }
}

shape_vector_conversion!(to_vec_of_polyline, Polyline, Shape::Polyline(shp), shp);
shape_vector_conversion!(to_vec_of_polylinem, PolylineM, Shape::PolylineM(shp), shp);
shape_vector_conversion!(to_vec_of_polylinez, PolylineZ, Shape::PolylineZ(shp), shp);

shape_vector_conversion!(to_vec_of_point, Point, Shape::Point(shp), shp);
shape_vector_conversion!(to_vec_of_pointm, PointM, Shape::PointM(shp), shp);
shape_vector_conversion!(to_vec_of_pointz, PointZ, Shape::PointZ(shp), shp);

shape_vector_conversion!(to_vec_of_polygon, Polygon, Shape::Polygon(shp), shp);
shape_vector_conversion!(to_vec_of_polygonm, PolygonM, Shape::PolygonM(shp), shp);
shape_vector_conversion!(to_vec_of_polygonz, PolygonZ, Shape::PolygonZ(shp), shp);

shape_vector_conversion!(to_vec_of_multipoint, Multipoint, Shape::Multipoint(shp), shp);
shape_vector_conversion!(to_vec_of_multipointz, MultipointZ, Shape::MultipointZ(shp), shp);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_vec_of_poly_err() {
        let shapes = vec!(Shape::Point(Point::default()), Shape::Polyline(Polyline::default()));
        assert!(to_vec_of_polyline(shapes).is_err());
    }

    #[test]
    fn convert_to_vec_of_point_err() {
        let shapes = vec!(Shape::Point(Point::default()), Shape::Polyline(Polyline::default()));
        assert!(to_vec_of_point(shapes).is_err());
    }

    #[test]
    fn convert_to_vec_of_poly_ok() {
        let shapes = vec!(Shape::Polyline(Polyline::default()), Shape::Polyline(Polyline::default()));
        assert!(to_vec_of_polyline(shapes).is_ok());
    }

    #[test]
    fn convert_to_vec_of_point_ok() {
        let shapes = vec!(Shape::Point(Point::default()), Shape::Point(Point::default()));
        assert!(to_vec_of_point(shapes).is_ok());
    }

    #[test]
    fn test_poly_from_polym() {
        let polym = PolylineM {
            bbox: BBox { xmin: 0.0, ymin: 0.0, xmax: 0.0, ymax: 0.0 },
            xs: Vec::<f64>::new(),
            ys: Vec::<f64>::new(),
            ms: Vec::<f64>::new(),
            parts: Vec::<i32>::new(),
            m_range: [0.0, 0.0],
        };

        let _poly = Polyline::from(polym);
    }
}
