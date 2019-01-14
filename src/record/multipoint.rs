//! Module with the definition of Multipoint(M, Z)
use std::fmt;
use std::io::{Read, Write};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use record::io::*;
use record::ConcreteReadableShape;
use record::{BBox, EsriShape};
use record::{HasShapeType, MultipointShape, WritableShape};
use record::{Point, PointM, PointZ};
use {Error, ShapeType};

pub struct GenericMultipoint<PointType> {
    pub bbox: BBox,
    points: Vec<PointType>,
}

impl<PointType> MultipointShape<PointType> for GenericMultipoint<PointType> {
    fn points(&self) -> &[PointType] {
        &self.points
    }
}

impl<PointType: HasXY> GenericMultipoint<PointType> {
    pub fn new(points: Vec<PointType>) -> Self {
        let bbox = BBox::from_points(&points);
        Self { bbox, points }
    }
}

/*
 * Multipoint
 */

pub type Multipoint = GenericMultipoint<Point>;

impl fmt::Display for Multipoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Multipoint({} points)", self.points.len())
    }
}

impl HasShapeType for Multipoint {
    fn shapetype() -> ShapeType {
        ShapeType::Multipoint
    }
}

impl ConcreteReadableShape for Multipoint {
    fn read_shape_content<T: Read>(mut source: &mut T) -> Result<Self::ActualShape, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_points = source.read_i32::<LittleEndian>()?;
        let points = read_xys_into_point_vec(&mut source, num_points)?;
        Ok(Self { bbox, points })
    }
}

impl WritableShape for Multipoint {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += 2 * size_of::<f64>() * self.points.len();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;
        for point in self.points {
            dest.write_f64::<LittleEndian>(point.x)?;
            dest.write_f64::<LittleEndian>(point.y)?;
        }
        Ok(())
    }
}

impl EsriShape for Multipoint {
    fn bbox(&self) -> BBox {
        self.bbox
    }
}

/*
 * MultipointM
 */

pub type MultipointM = GenericMultipoint<PointM>;

impl fmt::Display for MultipointM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MultipointM({} points)", self.points.len())
    }
}

impl HasShapeType for MultipointM {
    fn shapetype() -> ShapeType {
        ShapeType::MultipointM
    }
}

impl ConcreteReadableShape for MultipointM {
    fn read_shape_content<T: Read>(mut source: &mut T) -> Result<Self::ActualShape, Error> {
        let bbox = BBox::read_from(&mut source)?;

        let num_points = source.read_i32::<LittleEndian>()?;
        let mut points = read_xys_into_pointm_vec(&mut source, num_points)?;

        let _m_range = read_range(&mut source)?;
        read_ms_into(&mut source, &mut points)?;

        Ok(Self { bbox, points })
    }
}

impl WritableShape for MultipointM {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += 3 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;

        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for MultipointM {
    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn m_range(&self) -> [f64; 2] {
        calc_m_range(&self.points)
    }
}

/*
 * MultipointZ
 */

pub type MultipointZ = GenericMultipoint<PointZ>;

impl fmt::Display for MultipointZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MultipointZ({} points)", self.points.len())
    }
}

impl HasShapeType for MultipointZ {
    fn shapetype() -> ShapeType {
        ShapeType::MultipointZ
    }
}

impl ConcreteReadableShape for MultipointZ {
    fn read_shape_content<T: Read>(mut source: &mut T) -> Result<Self::ActualShape, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_points = source.read_i32::<LittleEndian>()?;
        let mut points = read_xys_into_pointz_vec(&mut source, num_points)?;

        let _z_range = read_range(&mut source)?;
        read_zs_into(&mut source, &mut points)?;

        let _m_range = read_range(&mut source)?;
        read_ms_into(&mut source, &mut points)?;

        Ok(Self { bbox, points })
    }
}

impl WritableShape for MultipointZ {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += 4 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;

        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.z_range())?;
        write_zs(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;

        Ok(())
    }
}

impl EsriShape for MultipointZ {
    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn z_range(&self) -> [f64; 2] {
        calc_z_range(&self.points)
    }

    fn m_range(&self) -> [f64; 2] {
        calc_m_range(&self.points)
    }
}
