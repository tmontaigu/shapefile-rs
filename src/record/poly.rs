use std::io::{Read, Write};

use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

use record::{BBox, EsriShape, ReadableShape, MultipartShape, MultipointShape, HasShapeType, WritableShape};
use record::{Point, PointM, PointZ};
use {ShapeType, Error};
use record::io::*;

use std::mem::size_of;
use std::fmt;
use record::is_parts_array_valid;


pub struct GenericPolyline<PointType> {
    pub bbox: BBox,
    points: Vec<PointType>,
    parts: Vec<i32>,
}

impl<PointType: HasXY> GenericPolyline<PointType> {
    pub fn new(points: Vec<PointType>, parts: Vec<i32>) -> Self {
        let bbox = BBox::from_points(&points);
        Self{bbox, points, parts}
    }
}

impl<PointType> From<GenericPolygon<PointType>> for GenericPolyline<PointType> {
    fn from(p: GenericPolygon<PointType>) -> Self {
        Self {
            bbox: p.bbox,
            points: p.points,
            parts: p.parts,
        }
    }
}

impl<PointType> From<GenericPolyline<PointType>> for GenericPolygon<PointType> {
    fn from(p: GenericPolyline<PointType>) -> Self {
        Self {
            bbox: p.bbox,
            points: p.points,
            parts: p.parts,
        }
    }
}


impl<PointType> MultipointShape<PointType> for GenericPolyline<PointType> {
    fn points(&self) -> &[PointType] {
        &self.points
    }
}

impl<PointType> MultipartShape<PointType> for GenericPolyline<PointType> {
    fn parts(&self) -> &[i32] {
        &self.parts
    }
}


pub type Polyline = GenericPolyline<Point>;

impl fmt::Display for Polyline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Polyline({} points, {} parts)", self.points.len(), self.parts.len())
    }
}

impl HasShapeType for Polyline {
    fn shapetype() -> ShapeType {
        ShapeType::Polyline
    }
}

impl ReadableShape for Polyline {
    type ActualShape = Self;

    fn read_from<T: Read>(mut source: &mut T) -> Result<<Self as ReadableShape>::ActualShape, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let parts = read_parts(&mut source, num_parts)?;
        let points = read_xys_into_point_vec(&mut source, num_points)?;

        Ok(Self { bbox, parts, points })
    }
}

impl WritableShape for Polyline {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += size_of::<i32>();
        size += size_of::<i32>() * self.parts.len();
        size += 2 * size_of::<f64>() * self.points.len();
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
        write_points(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for Polyline {
    fn bbox(&self) -> BBox {
        self.bbox
    }
}


/*
 * PolylineM
 */

pub type PolylineM = GenericPolyline<PointM>;

impl fmt::Display for PolylineM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PolylineM({} points, {} parts)", self.points.len(), self.parts.len())
    }
}

impl HasShapeType for PolylineM {
    fn shapetype() -> ShapeType {
        ShapeType::PolylineM
    }
}

impl ReadableShape for PolylineM {
    type ActualShape = Self;

    fn read_from<T: Read>(mut source: &mut T) -> Result<<Self as ReadableShape>::ActualShape, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let parts = read_parts(&mut source, num_parts)?;

        let mut points = read_xys_into_pointm_vec(&mut source, num_points)?;

        let _m_range = read_range(&mut source)?;
        read_ms_into(&mut source, &mut points)?;

        Ok(Self { bbox, parts, points })
    }
}

impl WritableShape for PolylineM {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
        size += 3 * size_of::<f64>() * self.points.len();
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
        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for PolylineM {
    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn m_range(&self) -> [f64; 2] {
        calc_m_range(&self.points)
    }
}


/*
 * PolylineZ
 */


pub type PolylineZ = GenericPolyline<PointZ>;

impl fmt::Display for PolylineZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PolylineZ({} points, {} parts)", self.points.len(), self.parts.len())
    }
}

impl HasShapeType for PolylineZ {
    fn shapetype() -> ShapeType {
        ShapeType::PolylineZ
    }
}

impl ReadableShape for PolylineZ {
    type ActualShape = Self;

    fn read_from<T: Read>(mut source: &mut T) -> Result<<Self as ReadableShape>::ActualShape, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let parts = read_parts(&mut source, num_parts)?;

        let mut points = read_xys_into_pointz_vec(&mut source, num_points)?;

        let _z_range = read_range(&mut source)?;
        read_zs_into(&mut source, &mut points)?;

        let _m_range = read_range(&mut source)?;
        read_ms_into(&mut source, &mut points)?;

        Ok(Self { bbox, parts, points })
    }
}

impl WritableShape for PolylineZ {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
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

        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.z_range())?;
        write_zs(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;
        Ok(())
    }
}


impl EsriShape for PolylineZ {
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


/*
 * Polygon
 */

pub struct GenericPolygon<PointType> {
    pub bbox: BBox,
    points: Vec<PointType>,
    parts: Vec<i32>,
}

impl<PointType: HasXY> GenericPolygon<PointType> {
    pub fn new(points: Vec<PointType>, parts: Vec<i32>) -> Self {
        Self::from(GenericPolyline::<PointType>::new(points, parts))
    }
}

impl<PointType> MultipointShape<PointType> for GenericPolygon<PointType> {
    fn points(&self) -> &[PointType] {
        &self.points
    }
}

impl<PointType> MultipartShape<PointType> for GenericPolygon<PointType> {
    fn parts(&self) -> &[i32] {
        &self.parts
    }
}


pub type Polygon = GenericPolygon<Point>;

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Polygon({} points, {} parts)", self.points.len(), self.parts.len())
    }
}

impl HasShapeType for Polygon {
    fn shapetype() -> ShapeType {
        ShapeType::Polygon
    }
}

impl ReadableShape for Polygon {
    type ActualShape = Self;

    fn read_from<T: Read>(mut source: &mut T) -> Result<<Self as ReadableShape>::ActualShape, Error> {
        let poly = Polyline::read_from(&mut source)?;
        Ok(poly.into())
    }
}

impl WritableShape for Polygon {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
        size += 2 * size_of::<f64>() * self.points.len();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        let poly: Polyline = self.into();
        poly.write_to(&mut dest)
    }
}


impl EsriShape for Polygon {
    fn bbox(&self) -> BBox {
        self.bbox
    }
}

/*
 * PolygonM
 */

pub type PolygonM = GenericPolygon<PointM>;

impl fmt::Display for PolygonM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PolygonM({} points, {} parts)", self.points.len(), self.parts.len())
    }
}

impl HasShapeType for PolygonM {
    fn shapetype() -> ShapeType {
        ShapeType::PolygonM
    }
}


impl ReadableShape for PolygonM {
    type ActualShape = Self;

    fn read_from<T: Read>(mut source: &mut T) -> Result<<Self as ReadableShape>::ActualShape, Error> {
        let poly = PolylineM::read_from(&mut source)?;
        Ok(Self::from(poly))
    }
}

impl WritableShape for PolygonM {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
        size += 3 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        PolylineM::from(self).write_to(&mut dest)
    }
}

impl EsriShape for PolygonM {
    fn bbox(&self) -> BBox {
        self.bbox
    }

    fn m_range(&self) -> [f64; 2] {
        calc_m_range(&self.points)
    }
}

/*
 * PolygonZ
 */

pub type PolygonZ = GenericPolygon<PointZ>;

impl fmt::Display for PolygonZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PolygonZ({} points, {} parts)", self.points.len(), self.parts.len())
    }
}

impl HasShapeType for PolygonZ {
    fn shapetype() -> ShapeType {
        ShapeType::PolygonZ
    }
}


impl ReadableShape for PolygonZ {
    type ActualShape = Self;

    fn read_from<T: Read>(mut source: &mut T) -> Result<<Self as ReadableShape>::ActualShape, Error> {
        let poly = PolylineZ::read_from(&mut source)?;
        Ok(poly.into())
    }
}

impl WritableShape for PolygonZ {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0 as usize;
        size += size_of::<f64>() * 4;
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); //num points
        size += size_of::<i32>() * self.parts.len();
        size += 4 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        PolylineZ::from(self).write_to(&mut dest)
    }
}

impl EsriShape for PolygonZ {
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


/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_of_polyline_z() {
        assert_eq!(PolylineZ::size_of_record(10, 3), 404);
    }
}
*/