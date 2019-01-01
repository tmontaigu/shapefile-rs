use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};

use super::{ShapeType, Error, PatchType};
use std::io::Read;
use std::mem::size_of;
use std::io::Write;

use std::convert::Into;

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
    fn write_to<T: Write>(mut self, dest: &mut T) -> Result<(), Error>;
}

pub enum Shape {
    NullShape,
    Point(Point),
    PointM(PointM),
    PointZ(PointZ),
    Polyline(Polyline),
    PolylineM(PolylineM),
    PolylineZ(PolylineZ),
    Polygon(Polyline),
    Multipoint(Multipoint),
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

pub struct ZDimension {
    pub range: [f64; 2],
    pub values: Vec<f64>,
}

pub type MDimension = ZDimension;

fn write_parts<T: Write>(dest: &mut T, parts: &Vec<i32>) -> Result<(), std::io::Error> {
    for p in parts {
        dest.write_i32::<LittleEndian>(*p)?;
    }
    Ok(())
}

fn write_points<T: Write>(dest: &mut T, xs: &Vec<f64>, ys: &Vec<f64>) -> Result<(), std::io::Error> {
    assert_eq!(xs.len(), ys.len());

    for (x, y) in xs.into_iter().zip(ys) {
        dest.write_f64::<LittleEndian>(*x)?;
        dest.write_f64::<LittleEndian>(*y)?;
    }

    Ok(())
}

fn write_measures<T: Write>(dest: &mut T, m_range: &[f64; 2], ms: &Vec<f64>) -> Result<(), std::io::Error> {
    dest.write_f64::<LittleEndian>(m_range[0])?;
    dest.write_f64::<LittleEndian>(m_range[1])?;

    for m in ms {
        dest.write_f64::<LittleEndian>(*m)?;
    }

    Ok(())
}

pub struct Polyline {
    pub bbox: BBox,
    pub parts: Vec<i32>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
}

impl Polyline {
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>) -> Self {
        Polyline {
            bbox: BBox::from_xys(&xs, &ys),
            parts,
            xs,
            ys,
        }
    }

    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Polyline, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let mut parts = Vec::<i32>::with_capacity(num_parts as usize);
        for _i in 0..num_parts {
            parts.push(source.read_i32::<LittleEndian>()?);
        }

        let (xs, ys) = read_points(&mut source, num_points)?;
        Ok(Self {
            bbox,
            parts,
            xs,
            ys,
        })
    }

    fn size_of_record(num_points: usize, num_parts: usize) -> usize {
        let mut size = 0 as usize;
        size += size_of::<i32>();
        size += (size_of::<f64>() * 4);
        size += size_of::<i32>();
        size += size_of::<i32>();
        size += (size_of::<i32>() * num_parts);
        size += (size_of::<f64>() * num_points);
        size += (size_of::<f64>() * num_points);

        size
    }
}

impl From<PolylineM> for Polyline {
    fn from(polym: PolylineM) -> Self {
        Self {
            bbox: polym.bbox,
            xs: polym.xs,
            ys: polym.ys,
            parts: polym.parts,
        }
    }
}

impl EsriShape for Polyline {
    fn shapetype(&self) -> ShapeType {
        ShapeType::Polyline
    }

    fn size_in_bytes(&self) -> usize {
        Self::size_of_record(self.xs.len(), self.parts.len())
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        self.shapetype().write_to(&mut dest)?;
        self.bbox.write_to(&mut dest)?;

        dest.write_i32::<LittleEndian>(self.parts.len() as i32)?;
        dest.write_i32::<LittleEndian>(self.xs.len() as i32)?;

        write_parts(&mut dest, &self.parts)?;
        write_points(&mut dest, &self.xs, &self.ys)?;

        Ok(())
    }
}


pub struct PolylineM {
    pub bbox: BBox,
    pub parts: Vec<i32>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub m_range: [f64; 2],
    pub ms: Vec<f64>,
}

impl PolylineM {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<PolylineM, Error> {
        let poly = Polyline::read_from(&mut source)?;
        let m_dim = read_m_dimension(&mut source, poly.xs.len() as i32)?;
        Ok(Self {
            bbox: poly.bbox,
            parts: poly.parts,
            xs: poly.xs,
            ys: poly.ys,
            m_range: m_dim.range,
            ms: m_dim.values,
        })
    }

    pub fn size_of_record(num_points: usize, num_parts: usize) -> usize {
        let mut size = Polyline::size_of_record(num_points, num_parts);
        size += (size_of::<f64>() * 2);
        size += (size_of::<f64>() * num_points);
        size
    }
}

impl From<PolylineZ> for PolylineM {
    fn from(polyz: PolylineZ) -> Self {
        Self {
            bbox: polyz.bbox,
            xs: polyz.xs,
            ys: polyz.ys,
            ms: polyz.ms,
            parts: polyz.parts,
            m_range: polyz.m_range,
        }
    }
}

impl EsriShape for PolylineM {
    fn shapetype(&self) -> ShapeType {
        ShapeType::PolylineM
    }

    fn size_in_bytes(&self) -> usize {
        Self::size_of_record(self.xs.len(), self.parts.len())
    }

    fn write_to<T: Write>(mut self, mut dest: &mut T) -> Result<(), Error> {
        let m_range = std::mem::replace(&mut self.m_range, [0.0, 0.0]);
        let ms = std::mem::replace(&mut self.ms, Vec::<f64>::new());
        let poly = Polyline::from(self);
        poly.write_to(&mut dest)?;

        write_measures(&mut dest, &m_range, &ms)?;

        Ok(())
    }
}

pub struct PolylineZ {
    pub bbox: BBox,
    pub parts: Vec<i32>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub z_range: [f64; 2],
    pub zs: Vec<f64>,
    pub m_range: [f64; 2],
    pub ms: Vec<f64>,
}

impl PolylineZ {
    pub fn size_of_record(num_points: usize, num_parts: usize) -> usize {
        let mut size = PolylineM::size_of_record(num_points, num_parts);
        size += (size_of::<f64>() * 2);
        size += (size_of::<f64>() * num_points);
        size
    }

    pub fn read_from<T: Read>(mut source: &mut T) -> Result<PolylineZ, Error> {
        let poly = Polyline::read_from(&mut source)?;
        let z_dim = read_z_dimension(&mut source, poly.xs.len() as i32)?;
        let m_dim = read_m_dimension(&mut source, poly.xs.len() as i32)?;
        Ok(Self {
            bbox: poly.bbox,
            parts: poly.parts,
            xs: poly.xs,
            ys: poly.ys,
            z_range: z_dim.range,
            zs: z_dim.values,
            m_range: m_dim.range,
            ms: m_dim.values,
        })
    }
}

impl EsriShape for PolylineZ {
    fn shapetype(&self) -> ShapeType {
        ShapeType::PolylineZ
    }

    fn size_in_bytes(&self) -> usize {
        Self::size_of_record(self.xs.len(), self.xs.len())
    }

    fn write_to<T: Write>(mut self, mut dest: &mut T) -> Result<(), Error> {
        let z_range = std::mem::replace(&mut self.z_range, [0.0, 0.0]);
        let zs = std::mem::replace(&mut self.zs, Vec::<f64>::new());
        let poly = PolylineM::from(self);
        poly.write_to(&mut dest)?;

        write_measures(&mut dest, &z_range, &zs)?;

        Ok(())
    }
}

pub struct Multipatch {
    pub bbox: BBox,
    pub parts: Vec<i32>,
    pub part_types: Vec<PatchType>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub z: Option<ZDimension>,
    pub m: Option<MDimension>,
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn read_from<T: Read>(source: &mut T) -> Result<Self, std::io::Error> {
        let x = source.read_f64::<LittleEndian>()?;
        let y = source.read_f64::<LittleEndian>()?;
        Ok(Self { x, y })
    }
}

pub struct PointM {
    pub x: f64,
    pub y: f64,
    pub m: f64,
}

impl PointM {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Self, std::io::Error> {
        let point = Point::read_from(&mut source)?;
        let m = source.read_f64::<LittleEndian>()?;
        Ok(Self {
            x: point.x,
            y: point.y,
            m,
        })
    }
}


pub struct PointZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: f64,
}

impl PointZ {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Self, std::io::Error> {
        let point = Point::read_from(&mut source)?;
        let z = source.read_f64::<LittleEndian>()?;
        let m = source.read_f64::<LittleEndian>()?;
        Ok(Self {
            x: point.x,
            y: point.y,
            z,
            m,
        })
    }
}

pub struct Multipoint {
    pub bbox: BBox,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub z_range: Option<[f64; 2]>,
    pub zs: Option<Vec<f64>>,
    pub m_range: Option<[f64; 2]>,
    pub ms: Option<Vec<f64>>,
}

impl Default for Polyline {
    fn default() -> Self {
        Polyline {
            bbox: BBox { xmin: 0.0, ymin: 0.0, xmax: 0.0, ymax: 0.0 },
            parts: Vec::<i32>::new(),
            xs: Vec::<f64>::new(),
            ys: Vec::<f64>::new(),
        }
    }
}

impl Default for Point {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
    }
}

fn read_points<T: Read>(source: &mut T, num_points: i32) -> Result<(Vec<f64>, Vec<f64>), std::io::Error> {
    let mut xs = Vec::<f64>::with_capacity(num_points as usize);
    let mut ys = Vec::<f64>::with_capacity(num_points as usize);

    for _i in 0..num_points {
        xs.push(source.read_f64::<LittleEndian>()?);
        ys.push(source.read_f64::<LittleEndian>()?);
    }
    Ok((xs, ys))
}

fn read_z_dimension<T: Read>(source: &mut T, num_points: i32) -> Result<ZDimension, std::io::Error> {
    let mut zs = Vec::<f64>::with_capacity(num_points as usize);
    let mut range = [0.0; 2];
    range[0] = source.read_f64::<LittleEndian>()?;
    range[1] = source.read_f64::<LittleEndian>()?;
    for _i in 0..num_points {
        zs.push(source.read_f64::<LittleEndian>()?);
    }
    Ok(ZDimension { range, values: zs })
}

fn read_m_dimension<T: Read>(source: &mut T, num_points: i32) -> Result<MDimension, std::io::Error> {
    let mut zs = Vec::<f64>::with_capacity(num_points as usize);
    let mut range = [0.0; 2];
    range[0] = source.read_f64::<LittleEndian>()?;
    range[1] = source.read_f64::<LittleEndian>()?;
    for _i in 0..num_points {
        let value = source.read_f64::<LittleEndian>()?;
        if is_no_data(value) {
            zs.push(NO_DATA);
        } else {
            zs.push(value);
        }
    }
    Ok(MDimension { range, values: zs })
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
shape_vector_conversion!(to_vec_of_point, Point, Shape::Point(shp), shp);
shape_vector_conversion!(to_vec_of_multipoint, Multipoint, Shape::Multipoint(shp), shp);
shape_vector_conversion!(to_vec_of_multipatch, Multipatch, Shape::Multipatch(shp), shp);


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

        let poly = Polyline::from(polym);
    }
}
