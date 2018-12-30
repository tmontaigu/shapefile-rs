use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

use super::{ShapeType, ShpError, PatchType};
use std::io::Read;

pub enum Shape {
    NullShape,
    Point(Point),
    Polyline(Polyline),
    Polygon(Polyline),
    Multipoint(Multipoint),
    Multipatch(Multipatch)
}

pub struct BBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl BBox {
    pub fn read_from<T: Read>(mut source: T) -> Result<BBox, std::io::Error> {
        let xmin = source.read_f64::<LittleEndian>()?;
        let ymin = source.read_f64::<LittleEndian>()?;
        let xmax = source.read_f64::<LittleEndian>()?;
        let ymax = source.read_f64::<LittleEndian>()?;
        Ok(BBox { xmin, ymin, xmax, ymax })
    }
}

pub struct RecordHeader {
    pub record_number: i32,
    pub record_size: i32,
}

impl RecordHeader {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<RecordHeader, ShpError> {
        let record_number = source.read_i32::<BigEndian>()?;
        let record_size = source.read_i32::<BigEndian>()?;
        Ok(RecordHeader { record_number, record_size })
    }
}

pub struct ZDimension {
    range: [f64; 2],
    values: Vec<f64>,
}

pub type MDimension = ZDimension;

pub struct Polyline {
    pub bbox: BBox,
    pub num_parts: i32,
    pub num_points: i32,
    pub parts: Vec<i32>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub z: Option<ZDimension>,
    pub m: Option<MDimension>,
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
    pub z: Option<f64>,
    pub m: Option<f64>,
}

pub struct Multipoint {
    pub bbox: BBox,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub z: Option<ZDimension>,
    pub m: Option<MDimension>,
}

fn read_points<T: Read>(mut source: &mut T, num_points: i32) -> Result<(Vec<f64>, Vec<f64>), std::io::Error> {
    let mut xs = Vec::<f64>::with_capacity(num_points as usize);
    let mut ys = Vec::<f64>::with_capacity(num_points as usize);

    for i in 0..num_points {
        xs.push(source.read_f64::<LittleEndian>()?);
        ys.push(source.read_f64::<LittleEndian>()?);
    }
    Ok((xs, ys))
}

fn read_z_dimension<T: Read>(mut source: &mut T, num_points: i32) -> Result<ZDimension, std::io::Error> {
    let mut zs = Vec::<f64>::with_capacity(num_points as usize);
    let mut range = [0.0; 2];
    range[0] = source.read_f64::<LittleEndian>()?;
    range[1] = source.read_f64::<LittleEndian>()?;
    for i in 0..num_points {
        zs.push(source.read_f64::<LittleEndian>()?);
    }
    Ok(ZDimension{range, values: zs})
}

fn read_m_dimension<T: Read>(mut source: &mut T, num_points: i32) -> Result<MDimension, std::io::Error> {
    let mut zs = Vec::<f64>::with_capacity(num_points as usize);
    let mut range = [0.0; 2];
    range[0] = source.read_f64::<LittleEndian>()?;
    range[1] = source.read_f64::<LittleEndian>()?;
    for i in 0..num_points {
        zs.push(source.read_f64::<LittleEndian>()?);
    }
    Ok(MDimension{range, values: zs})
}
pub fn read_poly_line_record<T: Read>(mut source: &mut T, shape_type: ShapeType) -> Result<Polyline, std::io::Error> {
    //TODO check that shape type is polygon/polyline type
    let bbox = BBox::read_from(&mut source)?;
    let num_parts = source.read_i32::<LittleEndian>()?;
    let num_points = source.read_i32::<LittleEndian>()?;
    let vec_size = num_points as usize;;

    let mut parts = Vec::<i32>::with_capacity(num_parts as usize);
    for i in 0..num_parts {
        parts.push(source.read_i32::<LittleEndian>()?);
    }

    let (xs, ys) = read_points(&mut source, num_points)?;

    let mut z_dim = None;
    if shape_type.has_z() {
       z_dim = Some(read_z_dimension(&mut source, num_points)?);
    }

    let mut m_dim = None;
    if shape_type.has_z() {
        m_dim = Some(read_m_dimension(&mut source, num_points)?);
    }

    Ok(Polyline{
        bbox,
        num_parts,
        num_points,
        parts,
        xs,
        ys,
        z: z_dim,
        m: m_dim
    })
}

pub fn read_multipatch_record<T: Read>(mut source: &mut T, shape_type: ShapeType) -> Result<Multipatch, ShpError> {
    //TODO check that shape type is polygon/polyline type
    let bbox = BBox::read_from(&mut source)?;
    let num_parts = source.read_i32::<LittleEndian>()?;
    let num_points = source.read_i32::<LittleEndian>()?;
    let vec_size = num_points as usize;;

    let mut parts = Vec::<i32>::with_capacity(num_parts as usize);
    for i in 0..num_parts {
        parts.push(source.read_i32::<LittleEndian>()?);
    }

    let mut part_types = Vec::<PatchType>::with_capacity(num_parts as usize);
    for i in 0..num_parts {
        part_types.push(PatchType::read_from(&mut source)?);
    }

    let (xs, ys) = read_points(&mut source, num_points)?;

    let mut z_dim = None;
    if shape_type.has_z() {
        z_dim = Some(read_z_dimension(&mut source, num_points)?);
    }

    let mut m_dim = None;
    if shape_type.has_z() {
        m_dim = Some(read_m_dimension(&mut source, num_points)?);
    }

    Ok(Multipatch{
        bbox,
        parts,
        part_types,
        xs,
        ys,
        z: z_dim,
        m: m_dim
    })
}



pub fn read_point_record<T: Read>(mut source: &mut T, shape_type: ShapeType) -> Result<Point, std::io::Error> {
    let x = source.read_f64::<LittleEndian>()?;
    let y = source.read_f64::<LittleEndian>()?;

    let mut z = None;
    if shape_type.has_z() {
        z = Some(source.read_f64::<LittleEndian>()?);
    }

    let mut m = None;
    if shape_type.has_m() {
        m = Some(source.read_f64::<LittleEndian>()?);
    }
    Ok(Point{x, y, z, m})
}

pub fn read_multipoint_record<T: Read>(mut source: &mut T, shape_type: ShapeType) -> Result<Multipoint, std::io::Error> {
    let bbox = BBox::read_from(&mut source)?;
    let num_points = source.read_i32::<LittleEndian>()?;

    let vec_size = num_points as usize;;

    let (xs, ys) = read_points(&mut source, num_points)?;

    let mut z_dim = None;
    if shape_type.has_z() {
        z_dim = Some(read_z_dimension(&mut source, num_points)?);
    }

    let mut m_dim = None;
    if shape_type.has_z() {
        m_dim = Some(read_m_dimension(&mut source, num_points)?);
    }

    Ok(Multipoint {bbox, xs, ys, z: z_dim, m: m_dim})
}

