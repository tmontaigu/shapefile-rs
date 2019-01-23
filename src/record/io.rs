use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use record::{PointZ, is_no_data, NO_DATA};
use record::traits::{HasXY, HasMutXY, HasM};

pub(crate) fn read_xy_in_vec_of<PointType, T>(source: &mut T, num_points: i32) -> Result<Vec<PointType>, std::io::Error>
    where PointType: HasMutXY + Default,
          T: Read
{
    let mut points = Vec::<PointType>::with_capacity(num_points as usize);
    for _ in 0..num_points {
        let mut p = PointType::default();
        *p.x_mut() = source.read_f64::<LittleEndian>()?;
        *p.y_mut() = source.read_f64::<LittleEndian>()?;
        points.push(p);
    }
    Ok(points)
}

pub(crate) fn read_ms_into<T: Read, D: HasM>(
    source: &mut T,
    points: &mut Vec<D>,
) -> Result<(), std::io::Error> {
    for point in points {
        *point.m_mut() = f64::max(source.read_f64::<LittleEndian>()?, NO_DATA);
    }
    Ok(())
}

pub(crate) fn read_zs_into<T: Read>(
    source: &mut T,
    points: &mut Vec<PointZ>,
) -> Result<(), std::io::Error> {
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

pub(crate) fn read_parts<T: Read>(
    source: &mut T,
    num_parts: i32,
) -> Result<Vec<i32>, std::io::Error> {
    let mut parts = Vec::<i32>::with_capacity(num_parts as usize);
    for _ in 0..num_parts {
        parts.push(source.read_i32::<LittleEndian>()?);
    }
    Ok(parts)
}

pub(crate) fn write_points<T: Write, PointType: HasXY>(
    dest: &mut T,
    points: &[PointType],
) -> Result<(), std::io::Error> {
    for point in points {
        dest.write_f64::<LittleEndian>(point.x())?;
        dest.write_f64::<LittleEndian>(point.y())?;
    }
    Ok(())
}

pub(crate) fn write_ms<T: Write, PointType: HasM>(
    dest: &mut T,
    points: &[PointType],
) -> Result<(), std::io::Error> {
    for point in points {
        dest.write_f64::<LittleEndian>(point.m())?;
    }
    Ok(())
}

pub(crate) fn write_zs<T: Write>(dest: &mut T, points: &[PointZ]) -> Result<(), std::io::Error> {
    for point in points {
        dest.write_f64::<LittleEndian>(point.z)?;
    }
    Ok(())
}

pub(crate) fn write_parts<T: Write>(dest: &mut T, parts: &[i32]) -> Result<(), std::io::Error> {
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

pub(crate) fn calc_m_range<PointType: HasM>(points: &[PointType]) -> [f64; 2] {
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

pub(crate) fn calc_z_range(points: &[PointZ]) -> [f64; 2] {
    let mut range = [std::f64::MAX, std::f64::MIN];
    for point in points {
        range[0] = f64::min(range[0], point.z);
        range[1] = f64::max(range[1], point.z);
    }
    range
}
