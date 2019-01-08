use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

use std::mem::size_of;

use record::io::*;
use record::BBox;
use record::{EsriShape, ReadableShape, MultipointShape, MultipartShape, WritableShape, HasShapeType, PointZ};
use {ShapeType, Error};

use std::fmt;
use record::is_parts_array_valid;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PatchType {
    TriangleStrip,
    TriangleFan,
    OuterRing,
    InnerRing,
    FirstRing,
    Ring,
}

impl PatchType {
    pub fn read_from<T: Read>(source: &mut T) -> Result<PatchType, Error> {
        let code = source.read_i32::<LittleEndian>()?;
        Self::from(code).ok_or(Error::InvalidPatchType(code))
    }

    pub fn from(code: i32) -> Option<PatchType> {
        match code {
            0 => Some(PatchType::TriangleStrip),
            1 => Some(PatchType::TriangleFan),
            2 => Some(PatchType::OuterRing),
            3 => Some(PatchType::InnerRing),
            4 => Some(PatchType::FirstRing),
            5 => Some(PatchType::Ring),
            _ => None
        }
    }
}


pub struct Multipatch {
    pub bbox: BBox,
    points: Vec<PointZ>,
    parts: Vec<i32>,
    pub parts_type: Vec<PatchType>,
    pub z_range: [f64; 2],
    pub m_range: [f64; 2],
}

impl Multipatch {
    pub fn new(points: Vec<PointZ>, parts: Vec<i32>, parts_type: Vec<PatchType>) -> Self {
        let bbox = BBox::from_points(&points);
        let m_range = calc_m_range(&points);
        let z_range = calc_z_range(&points);
        Self{bbox, points, parts, parts_type, z_range, m_range}
    }
}

impl fmt::Display for Multipatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Multipatch({} points, {} parts)", self.points.len(), self.parts.len())
    }
}

impl MultipointShape<PointZ> for Multipatch {
    fn points(&self) -> &[PointZ] {
        &self.points
    }
}

impl MultipartShape<PointZ> for Multipatch {
    fn parts(&self) -> &[i32] {
        &self.parts
    }
}

impl HasShapeType for Multipatch {
    fn shapetype() -> ShapeType {
        ShapeType::Multipatch
    }
}

impl ReadableShape for Multipatch {
    type ActualShape = Self;

    fn read_from<T: Read>(mut source: &mut T) -> Result<Self::ActualShape, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let parts = read_parts(&mut source, num_parts)?;

        let mut parts_type = Vec::<PatchType>::with_capacity(num_parts as usize);
        for _ in 0..num_parts {
            let code = source.read_i32::<LittleEndian>()?;
            match PatchType::from(code) {
                Some(t) => parts_type.push(t),
                None => return Err(Error::InvalidPatchType(code)),
            }
        }
        let mut points = read_xys_into_pointz_vec(&mut source, num_points)?;

        let z_range = read_range(&mut source)?;
        read_zs_into(&mut source, &mut points)?;

        let m_range = read_range(&mut source)?;
        read_ms_into(&mut source, &mut points)?;

        Ok(Self { bbox, parts, parts_type, points, z_range, m_range })
    }
}

impl WritableShape for Multipatch {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += size_of::<i32>();
        size += size_of::<i32>() * self.parts.len();
        size += size_of::<i32>() * self.parts_type.len();
        size += 4 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        if !is_parts_array_valid(&self) {
            return Err(Error::MalformedShape)
        }
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.parts.len() as i32)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;
        write_parts(&mut dest, &self.parts)?;
        let part_types: Vec<i32> = self.parts_type.iter().map(|t| *t as i32).collect();
        write_parts(&mut dest, &part_types)?;

        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.z_range())?;
        write_zs(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for Multipatch {
    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn z_range(&self) -> [f64; 2] {
        self.z_range
    }

    fn m_range(&self) -> [f64; 2] {
        self.m_range
    }
}
