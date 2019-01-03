use record::{BBox, EsriShape};
use std::io::{Read, Write};

use record::io::*;
use std::mem::size_of;


use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
use {ShapeType, Error};

pub struct Multipoint {
    pub bbox: BBox,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
}

impl Multipoint {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Multipoint, std::io::Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_points = source.read_i32::<LittleEndian>()?;
        let (xs, ys) = read_points(&mut source, num_points)?;

        Ok(Multipoint { bbox, xs, ys })
    }
}

impl EsriShape for Multipoint {
    fn shapetype(&self) -> ShapeType {
        ShapeType::Multipoint
    }

    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += 2 * size_of::<f64>() * self.xs.len();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.xs.len() as i32)?;
        write_points(&mut dest, &self.xs, &self.ys)?;
        Ok(())
    }
}

pub struct MultipointZ {
    pub bbox: BBox,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub z_range: [f64; 2],
    pub zs: Vec<f64>,
    pub m_range: [f64; 2],
    pub ms: Vec<f64>,
}

impl MultipointZ {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Self, std::io::Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_points = source.read_i32::<LittleEndian>()?;
        let (xs, ys) = read_points(&mut source, num_points)?;
        let (z_range, zs) = read_z_dimension(&mut source, num_points)?;
        let (m_range, ms) = read_m_dimension(&mut source, num_points)?;
        Ok(Self { bbox, xs, ys, m_range, ms, z_range, zs })
    }
}

impl EsriShape for MultipointZ {
    fn shapetype(&self) -> ShapeType {
        ShapeType::MultipointZ
    }

    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += 2 * size_of::<f64>() * self.xs.len();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>() * self.xs.len();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>() * self.xs.len();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.xs.len() as i32)?;
        write_points(&mut dest, &self.xs, &self.ys)?;
        write_range_and_vec(&mut dest, &self.z_range, &self.zs)?;
        write_range_and_vec(&mut dest, &self.m_range, &self.ms)?;
        Ok(())
    }
}