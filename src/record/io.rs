use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use record::traits::{HasM, HasMutM, HasMutXY, HasMutZ, HasXY, HasZ};
use record::{GenericBBox, PointZ, NO_DATA};

pub(crate) fn bbox_read_xy_from<PointType: HasMutXY, R: Read>(
    bbox: &mut GenericBBox<PointType>,
    src: &mut R,
) -> std::io::Result<()> {
    *bbox.min.x_mut() = src.read_f64::<LittleEndian>()?;
    *bbox.min.y_mut() = src.read_f64::<LittleEndian>()?;
    *bbox.max.x_mut() = src.read_f64::<LittleEndian>()?;
    *bbox.max.y_mut() = src.read_f64::<LittleEndian>()?;
    Ok(())
}

pub(crate) fn bbox_write_xy_to<PointType: HasXY, W: Write>(
    bbox: &GenericBBox<PointType>,
    dst: &mut W,
) -> std::io::Result<()> {
    dst.write_f64::<LittleEndian>(bbox.min.x())?;
    dst.write_f64::<LittleEndian>(bbox.min.y())?;
    dst.write_f64::<LittleEndian>(bbox.max.x())?;
    dst.write_f64::<LittleEndian>(bbox.max.y())?;
    Ok(())
}

pub(crate) fn bbox_read_m_range_from<PointType: HasMutM, R: Read>(
    bbox: &mut GenericBBox<PointType>,
    src: &mut R,
) -> std::io::Result<()> {
    *bbox.min.m_mut() = src.read_f64::<LittleEndian>()?;
    *bbox.max.m_mut() = src.read_f64::<LittleEndian>()?;
    Ok(())
}

pub(crate) fn bbox_read_z_range_from<PointType: HasMutZ, R: Read>(
    bbox: &mut GenericBBox<PointType>,
    src: &mut R,
) -> std::io::Result<()> {
    *bbox.min.z_mut() = src.read_f64::<LittleEndian>()?;
    *bbox.max.z_mut() = src.read_f64::<LittleEndian>()?;
    Ok(())
}

pub(crate) fn bbox_write_m_range_to<PointType: HasM, W: Write>(
    bbox: &GenericBBox<PointType>,
    dst: &mut W,
) -> std::io::Result<()> {
    dst.write_f64::<LittleEndian>(bbox.min.m())?;
    dst.write_f64::<LittleEndian>(bbox.max.m())?;
    Ok(())
}

pub(crate) fn bbox_write_z_range_to<PointType: HasZ, W: Write>(
    bbox: &GenericBBox<PointType>,
    dst: &mut W,
) -> std::io::Result<()> {
    dst.write_f64::<LittleEndian>(bbox.min.z())?;
    dst.write_f64::<LittleEndian>(bbox.max.z())?;
    Ok(())
}

pub(crate) fn read_xy_in_vec_of<PointType, T>(
    source: &mut T,
    num_points: i32,
) -> Result<Vec<PointType>, std::io::Error>
where
    PointType: HasMutXY + Default,
    T: Read,
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

pub(crate) fn read_ms_into<T: Read, D: HasMutM>(
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
