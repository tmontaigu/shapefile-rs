use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

use std::mem::size_of;

use record::io::*;
use record::BBox;
use record::{EsriShape, ReadableShape};
use {ShapeType, Error, all_have_same_len, have_same_len_as};

use std::fmt;

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

impl fmt::Display for Multipatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Multipatch({} points, {} parts)", self.xs.len(), self.parts.len())
    }
}

impl ReadableShape for Multipatch {
    type ActualShape = Self;

    fn shapetype() -> ShapeType {
        ShapeType::Multipatch
    }

    fn read_from<T: Read>(mut source: &mut T) -> Result<Self::ActualShape, Error> {
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

impl EsriShape for Multipatch {
    fn shapetype(&self) -> ShapeType {
        ShapeType::Multipatch
    }

    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += size_of::<i32>();
        size += size_of::<i32>() * self.parts.len();
        size += size_of::<i32>() * self.parts_type.len();
        size += 2 * size_of::<f64>() * self.xs.len();
        size += 2 * size_of::<f64>();
        size += size_of::<f64>() * self.xs.len();
        size += 2 * size_of::<f64>();
        size += size_of::<f64>() * self.xs.len();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        if all_have_same_len!(self.xs, self.ys, self.zs, self.ms) &&
            all_have_same_len!(self.parts, self.parts_type)
        {
            self.bbox.write_to(&mut dest)?;
            dest.write_i32::<LittleEndian>(self.parts.len() as i32)?;
            dest.write_i32::<LittleEndian>(self.xs.len() as i32)?;
            write_parts(&mut dest, &self.parts)?;
            let part_types: Vec<i32> = self.parts_type.into_iter().map(|t| t as i32).collect();
            write_parts(&mut dest, &part_types)?;
            write_points(&mut dest, &self.xs, &self.ys)?;
            write_range_and_vec(&mut dest, &self.z_range, &self.zs)?;
            write_range_and_vec(&mut dest, &self.m_range, &self.ms)?;
            Ok(())
        } else {
            Err(Error::MalformedShape)
        }
    }

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
