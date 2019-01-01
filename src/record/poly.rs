use crate::record::BBox;
use std::io::{Read, Write};

use crate::Error;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use crate::ShapeType;
use crate::record::EsriShape;
use crate::record::is_no_data;
use crate::NO_DATA;

use std::mem::size_of;

pub struct ZDimension {
    pub range: [f64; 2],
    pub values: Vec<f64>,
}

pub type MDimension = ZDimension;


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
        size += size_of::<f64>() * 4;
        size += size_of::<i32>();
        size += size_of::<i32>();
        size += size_of::<i32>() * num_parts;
        size += size_of::<f64>() * num_points;
        size += size_of::<f64>() * num_points;

        size
    }
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
        size += size_of::<f64>() * 2;
        size += size_of::<f64>() * num_points;
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
        size += size_of::<f64>() * 2;
        size += size_of::<f64>() * num_points;
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
        let m_range = std::mem::replace(&mut self.m_range, [0.0, 0.0]);
        let ms = std::mem::replace(&mut self.ms, Vec::<f64>::new());
        let z_range = std::mem::replace(&mut self.z_range, [0.0, 0.0]);
        let zs = std::mem::replace(&mut self.zs, Vec::<f64>::new());

        let poly = Polyline::from(PolylineM::from(self)); //FIXME
        poly.write_to(&mut dest)?;

        write_measures(&mut dest, &z_range, &zs)?;
        write_measures(&mut dest, &m_range, &ms)?;

        Ok(())
    }
}
