use std::io::{Read, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use record::is_no_data;
use NO_DATA;
use record::{Point, PointM, PointZ};

pub trait HasXY {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
}

pub(crate) trait HasM {
    fn m(&self) -> f64;
    fn m_mut(&mut self) -> &mut f64;
}

macro_rules! impl_has_xy_for {
    ($PointType:ty) => {
        impl HasXY for $PointType {
            fn x(&self) -> f64 {
                self.x
            }
            fn y(&self) -> f64 {
                self.y
            }
        }
    };
}

macro_rules! impl_has_m_for {
    ($PointType:ty) => {
        impl HasM for $PointType {
             fn m(&self) -> f64 {
                self.m
             }

            fn m_mut(&mut self) -> &mut f64 {
                &mut self.m
            }
         }
    };
}

impl_has_xy_for!(Point);
impl_has_xy_for!(PointM);
impl_has_xy_for!(PointZ);

impl_has_m_for!(PointM);
impl_has_m_for!(PointZ);

macro_rules! define_read_xy_func {
    ($func:ident, $PointType:ident) => {
        pub(crate) fn $func<T: Read>(source: &mut T, num_points: i32) -> Result<Vec<$PointType>, std::io::Error> {
            let mut points = Vec::<$PointType>::with_capacity(num_points as usize);

            for _ in 0..num_points {
                let x = source.read_f64::<LittleEndian>()?;
                let y = source.read_f64::<LittleEndian>()?;
                let p = $PointType{x, y, ..Default::default()};
                points.push(p);
            }
            Ok(points)
        }
    };
}


define_read_xy_func!(read_xys_into_point_vec, Point);
define_read_xy_func!(read_xys_into_pointm_vec, PointM);
define_read_xy_func!(read_xys_into_pointz_vec, PointZ);


pub(crate) fn read_ms_into<T: Read, D: HasM>(source: &mut T, points: &mut Vec<D>) -> Result<(), std::io::Error> {
    for point in points {
        *point.m_mut() = f64::max(source.read_f64::<LittleEndian>()?, NO_DATA);
    }
    Ok(())
}


pub(crate) fn read_zs_into<T: Read>(source: &mut T, points: &mut Vec<PointZ>) -> Result<(), std::io::Error> {
    for point in points.iter_mut() {
        point.z = source.read_f64::<LittleEndian>()?;
    }
    Ok(())
}

pub(crate) fn read_range<T: Read>(source: &mut T) -> Result<[f64; 2], std::io::Error> {
    let mut range = [0.0, 0.0];
    range[0] = source.read_f64::<LittleEndian>()?;
    range[1] = source.read_f64::<LittleEndian>()?;
    Ok(range)
}

pub(crate) fn read_parts<T: Read>(source: &mut T, num_parts: i32) -> Result<Vec<i32>, std::io::Error> {
    let mut parts = Vec::<i32>::with_capacity(num_parts as usize);
    for _ in 0..num_parts {
        parts.push(source.read_i32::<LittleEndian>()?);
    }
    Ok(parts)
}


pub(crate) fn write_points<T: Write, PointType: HasXY>(dest: &mut T, points: &Vec<PointType>) -> Result<(), std::io::Error> {
    for point in points {
        dest.write_f64::<LittleEndian>(point.x())?;
        dest.write_f64::<LittleEndian>(point.y())?;
    }
    Ok(())
}

pub(crate) fn write_ms<T: Write, PointType: HasM>(dest: &mut T, points: &Vec<PointType>) -> Result<(), std::io::Error> {
    for point in points {
        dest.write_f64::<LittleEndian>(point.m())?;
    }
    Ok(())
}

pub(crate) fn write_zs<T: Write>(dest: &mut T, points: &Vec<PointZ>) -> Result<(), std::io::Error> {
    for point in points {
        dest.write_f64::<LittleEndian>(point.z)?;
    }
    Ok(())
}

pub(crate) fn write_parts<T: Write>(dest: &mut T, parts: &Vec<i32>) -> Result<(), std::io::Error> {
    for p in parts {
        dest.write_i32::<LittleEndian>(*p)?;
    }
    Ok(())
}

pub(crate) fn write_range<T: Write>(dest: &mut T, range: [f64; 2]) -> Result<(), std::io::Error> {
    dest.write_f64::<LittleEndian>(range[0])?;
    dest.write_f64::<LittleEndian>(range[1])?;
    Ok(())
}

pub(crate) fn calc_m_range<PointType: HasM>(points: &Vec<PointType>) -> [f64; 2] {
    let mut range = [std::f64::MAX, std::f64::MIN];
    for point in points {
        range[0] = f64::min(range[0], point.m());
        range[1] = f64::max(range[1], point.m());
    }

    if is_no_data(range[0]) {
        range[0] = 0.0;
    }

    if is_no_data(range[1]) {
        range[1] = 0.0;
    }

    range
}

pub(crate) fn calc_z_range(points: &Vec<PointZ>) -> [f64; 2] {
    let mut range = [std::f64::MAX, std::f64::MIN];
    for point in points {
        range[0] = f64::min(range[0], point.z);
        range[1] = f64::max(range[1], point.z);
    }
    range
}