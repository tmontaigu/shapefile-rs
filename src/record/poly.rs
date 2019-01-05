use std::io::{Read, Write};

use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

use ShapeType;
use record::{EsriShape, min_and_max_of_f64_slice};
use record::BBox;
use Error;
use record::io::*;

use std::mem::size_of;
use NO_DATA;


pub struct Polyline {
    pub bbox: BBox,
    pub parts: Vec<i32>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
}

impl Polyline {
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>) -> Self {
        Self {
            bbox: BBox::from_xys(&xs, &ys),
            parts,
            xs,
            ys,
        }
    }

    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Polyline, std::io::Error> {
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
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
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

impl From<Polygon> for Polyline {
    fn from(p: Polygon) -> Self {
        Self {
            bbox: p.bbox,
            xs: p.xs,
            ys: p.ys,
            parts: p.parts,
        }
    }
}

impl From<PolylineZ> for Polyline {
    fn from(polyz: PolylineZ) -> Self {
        Self {
            bbox: polyz.bbox,
            xs: polyz.xs,
            ys: polyz.ys,
            parts: polyz.parts,
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

    fn bbox(&self) -> BBox {
       self.bbox
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
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>) -> Self {
        let ms = (0..xs.len()).map(|_| NO_DATA).collect();
        Self {
            bbox: BBox::from_xys(&xs, &ys),
            parts,
            xs,
            ys,
            m_range: [0.0, 0.0],
            ms,
        }
    }

    pub fn new_with_ms(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>, ms: Vec<f64>) -> Self {
        let (m_min, m_max) = min_and_max_of_f64_slice(&ms);
        Self {
            bbox: BBox::from_xys(&xs, &ys),
            parts,
            xs,
            ys,
            m_range: [m_min, m_max],
            ms,
        }
    }

    pub fn read_from<T: Read>(mut source: &mut T) -> Result<PolylineM, std::io::Error> {
        let poly = Polyline::read_from(&mut source)?;
        let (m_range, ms) = read_m_dimension(&mut source, poly.xs.len() as i32)?;
        Ok(Self {
            bbox: poly.bbox,
            parts: poly.parts,
            xs: poly.xs,
            ys: poly.ys,
            m_range,
            ms,
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

impl From<PolygonM> for PolylineM {
    fn from(p: PolygonM) -> Self {
        Self {
            bbox: p.bbox,
            xs: p.xs,
            ys: p.ys,
            ms: p.ms,
            m_range: p.m_range,
            parts: p.parts,
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

        write_range_and_vec(&mut dest, &m_range, &ms)?;

        Ok(())
    }

    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn m_range(&self) -> [f64; 2] {
        self.m_range
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
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>, zs: Vec<f64>) -> Self {
        let ms = (0..xs.len()).map(|_| NO_DATA).collect();
        let (z_min, z_max) = min_and_max_of_f64_slice(&zs);
        Self {
            bbox: BBox::from_xys(&xs, &ys),
            parts,
            xs,
            ys,
            z_range: [z_min, z_max],
            zs,
            m_range: [0.0, 0.0],
            ms,
        }
    }

    pub fn new_with_ms(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>, zs: Vec<f64>, ms: Vec<f64>) -> Self {
        let (m_min, m_max) = min_and_max_of_f64_slice(&ms);
        let (z_min, z_max) = min_and_max_of_f64_slice(&zs);
        Self {
            bbox: BBox::from_xys(&xs, &ys),
            parts,
            xs,
            ys,
            z_range: [z_min, z_max],
            zs,
            m_range: [m_min, m_max],
            ms,
        }
    }

    pub fn size_of_record(num_points: usize, num_parts: usize) -> usize {
        let mut size = PolylineM::size_of_record(num_points, num_parts);
        size += size_of::<f64>() * 2;
        size += size_of::<f64>() * num_points;
        size
    }

    pub fn read_from<T: Read>(mut source: &mut T) -> Result<PolylineZ, Error> {
        let poly = Polyline::read_from(&mut source)?;
        let (z_range, zs) = read_z_dimension(&mut source, poly.xs.len() as i32)?;
        let (m_range, ms) = read_m_dimension(&mut source, poly.xs.len() as i32)?;
        Ok(Self {
            bbox: poly.bbox,
            parts: poly.parts,
            xs: poly.xs,
            ys: poly.ys,
            z_range,
            zs,
            m_range,
            ms,
        })
    }
}

impl EsriShape for PolylineZ {
    fn shapetype(&self) -> ShapeType {
        ShapeType::PolylineZ
    }

    fn size_in_bytes(&self) -> usize {
        Self::size_of_record(self.xs.len(), self.parts.len())
    }

    fn write_to<T: Write>(mut self, mut dest: &mut T) -> Result<(), Error> {
        let m_range = std::mem::replace(&mut self.m_range, [0.0, 0.0]);
        let ms = std::mem::replace(&mut self.ms, Vec::<f64>::new());
        let z_range = std::mem::replace(&mut self.z_range, [0.0, 0.0]);
        let zs = std::mem::replace(&mut self.zs, Vec::<f64>::new());

        let poly = Polyline::from(PolylineM::from(self)); //FIXME
        poly.write_to(&mut dest)?;

        write_range_and_vec(&mut dest, &z_range, &zs)?;
        write_range_and_vec(&mut dest, &m_range, &ms)?;

        Ok(())
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

impl From<PolygonZ> for PolylineZ {
    fn from(p: PolygonZ) -> Self {
        Self {
            bbox: p.bbox,
            xs: p.xs,
            ys: p.ys,
            zs: p.zs,
            ms: p.ms,
            m_range: p.m_range,
            z_range: p.z_range,
            parts: p.parts,
        }
    }
}

pub struct Polygon {
    pub bbox: BBox,
    pub parts: Vec<i32>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
}

impl Polygon {
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>) -> Self {
        Polyline::new(xs, ys, parts).into()
    }

    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Polygon, Error> {
        let poly = Polyline::read_from(&mut source)?;
        Ok(poly.into())
    }
}

impl From<Polyline> for Polygon {
    fn from(p: Polyline) -> Self {
        Self {
            bbox: p.bbox,
            xs: p.xs,
            ys: p.ys,
            parts: p.parts,
        }
    }
}

impl EsriShape for Polygon {
    fn shapetype(&self) -> ShapeType {
        ShapeType::Polygon
    }

    fn size_in_bytes(&self) -> usize {
        let poly: &Polyline = self.as_ref();
        poly.size_in_bytes()
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        let polyline: Polyline = self.into();
        polyline.write_to(&mut dest)?;
        Ok(())
    }

    fn bbox(&self) -> BBox {
       self.bbox
    }
}

impl AsRef<Polyline> for Polygon {
    fn as_ref(&self) -> &Polyline {
        unsafe {
            std::mem::transmute::<&Polygon, &Polyline>(self)
        }
    }
}

pub struct PolygonM {
    pub bbox: BBox,
    pub parts: Vec<i32>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub m_range: [f64; 2],
    pub ms: Vec<f64>,
}

impl AsRef<PolylineM> for PolygonM {
    fn as_ref(&self) -> &PolylineM {
        unsafe { std::mem::transmute::<&PolygonM, &PolylineM>(self) }
    }
}

impl From<PolylineM> for PolygonM {
    fn from(p: PolylineM) -> Self {
        Self {
            bbox: p.bbox,
            xs: p.xs,
            ys: p.ys,
            ms: p.ms,
            parts: p.parts,
            m_range: p.m_range,
        }
    }
}

impl PolygonM {
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>) -> Self {
        PolylineM::new(xs, ys, parts).into()
    }

    pub fn new_with_ms(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>, ms: Vec<f64>) -> Self {
        PolylineM::new_with_ms(xs, ys, parts, ms).into()
    }

    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Self, std::io::Error> {
        let poly = PolylineM::read_from(&mut source)?;
        Ok(Self::from(poly))
    }
}

impl EsriShape for PolygonM {
    fn shapetype(&self) -> ShapeType {
        ShapeType::PolygonM
    }

    fn size_in_bytes(&self) -> usize {
        self.as_ref().size_in_bytes()
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        let polym: PolylineM = self.into();
        polym.write_to(&mut dest)?;
        Ok(())
    }

    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn m_range(&self) -> [f64; 2] {
        self.m_range
    }
}

pub struct PolygonZ {
    pub bbox: BBox,
    pub parts: Vec<i32>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub z_range: [f64; 2],
    pub zs: Vec<f64>,
    pub m_range: [f64; 2],
    pub ms: Vec<f64>,
}

impl AsRef<PolylineZ> for PolygonZ {
    fn as_ref(&self) -> &PolylineZ {
        unsafe { std::mem::transmute::<&PolygonZ, &PolylineZ>(&self) }
    }
}

impl From<PolylineZ> for PolygonZ {
    fn from(p: PolylineZ) -> Self {
        Self {
            bbox: p.bbox,
            xs: p.xs,
            ys: p.ys,
            zs: p.zs,
            ms: p.ms,
            m_range: p.m_range,
            z_range: p.z_range,
            parts: p.parts,
        }
    }
}

impl PolygonZ {
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, zs: Vec<f64>, parts: Vec<i32>) -> Self {
       PolylineZ::new(xs, ys, parts, zs).into()
    }

    pub fn new_with_ms(xs: Vec<f64>, ys: Vec<f64>, parts: Vec<i32>, zs: Vec<f64>, ms: Vec<f64>) -> Self {
        PolylineZ::new_with_ms(xs, ys, parts, zs, ms).into()
    }


    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Self, Error> {
        let poly = PolylineZ::read_from(&mut source)?;
        Ok(poly.into())
    }
}

impl EsriShape for PolygonZ {
    fn shapetype(&self) -> ShapeType {
        ShapeType::PolygonZ
    }

    fn size_in_bytes(&self) -> usize {
        self.as_ref().size_in_bytes()
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        let polyz: PolylineZ = self.into();
        polyz.write_to(&mut dest)?;
        Ok(())
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_of_polyline_z() {
        assert_eq!(PolylineZ::size_of_record(10, 3), 404);
    }
}