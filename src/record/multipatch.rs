use std::io::Read;
use Error;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

use record::io::*;
use record::BBox;

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
    pub parts: Vec<i32>,
    pub parts_type: Vec<PatchType>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub z_range: [f64; 2],
    pub zs: Vec<f64>,
    pub m_range: [f64; 2],
    pub ms: Vec<f64>,
}

impl Multipatch {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Self, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let mut parts = Vec::<i32>::with_capacity(num_parts as usize);
        for _ in 0..num_parts {
            parts.push(source.read_i32::<LittleEndian>()?);
        }

        let mut parts_type = Vec::<PatchType>::with_capacity(num_parts as usize);
        for _ in 0..num_parts {
            let code = source.read_i32::<LittleEndian>()?;
            match PatchType::from(code) {
                Some(t) => parts_type.push(t),
                None => return Err(Error::InvalidPatchType(code)),
            }
        }

        let (xs, ys) = read_points(&mut source, num_points)?;
        let (z_range, zs) = read_z_dimension(&mut source, num_points)?;
        let (m_range, ms) = read_m_dimension(&mut source, num_points)?;
        Ok(Self { bbox, parts, parts_type, xs, ys, z_range, zs, m_range, ms })
    }
}
