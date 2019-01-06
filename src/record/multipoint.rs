use record::{BBox, EsriShape, min_and_max_of_f64_slice, NO_DATA};
use std::io::{Read, Write};

use record::io::*;
use std::mem::size_of;


use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
use {ShapeType, Error, all_have_same_len, have_same_len_as};

pub struct Multipoint {
    pub bbox: BBox,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
}

impl Multipoint {
    pub fn new(xs: Vec<f64>, ys: Vec<f64>) -> Self {
        let bbox = BBox::from_xys(&xs, &xs);
        Self { bbox, xs, ys }
    }

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
        if all_have_same_len!(self.xs, self.ys) {
            self.bbox.write_to(&mut dest)?;
            dest.write_i32::<LittleEndian>(self.xs.len() as i32)?;
            write_points(&mut dest, &self.xs, &self.ys)?;
            Ok(())
        }
        else {
            Err(Error::MalformedShape)
        }

    }

    fn bbox(&self) -> BBox {
        self.bbox
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
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, zs: Vec<f64>) -> Self {
        let bbox = BBox::from_xys(&xs, &ys);
        let (min, max) = min_and_max_of_f64_slice(&zs);
        let num_pts = xs.len();
        Self { bbox, xs, ys, z_range: [min, max], zs, m_range: [0.0, 0.0], ms: (0..num_pts).map(|_|NO_DATA).collect() }
    }

    pub fn new_with_m(xs: Vec<f64>, ys: Vec<f64>, zs: Vec<f64>, ms: Vec<f64>) -> Self {
        let bbox = BBox::from_xys(&xs, &ys);
        let (z_min, z_max) = min_and_max_of_f64_slice(&zs);
        let (m_min, m_max) = min_and_max_of_f64_slice(&ms);
        Self { bbox, xs, ys, z_range: [z_min, z_max], zs, m_range: [m_min, m_max], ms }
    }

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
        size += size_of::<f64>() * self.xs.len();
        size += 2 * size_of::<f64>();
        size += size_of::<f64>() * self.xs.len();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        if all_have_same_len!(self.xs, self.ys, self.zs, self.ms) {
            self.bbox.write_to(&mut dest)?;
            dest.write_i32::<LittleEndian>(self.xs.len() as i32)?;
            write_points(&mut dest, &self.xs, &self.ys)?;
            write_range_and_vec(&mut dest, &self.z_range, &self.zs)?;
            write_range_and_vec(&mut dest, &self.m_range, &self.ms)?;
            Ok(())
        }
        else {
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