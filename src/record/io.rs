use std::io::{Read, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use record::is_no_data;
use NO_DATA;

pub(crate) fn read_points<T: Read>(source: &mut T, num_points: i32) -> Result<(Vec<f64>, Vec<f64>), std::io::Error> {
    let mut xs = Vec::<f64>::with_capacity(num_points as usize);
    let mut ys = Vec::<f64>::with_capacity(num_points as usize);

    for _i in 0..num_points {
        xs.push(source.read_f64::<LittleEndian>()?);
        ys.push(source.read_f64::<LittleEndian>()?);
    }
    Ok((xs, ys))
}

pub(crate) fn read_z_dimension<T: Read>(source: &mut T, num_points: i32) -> Result<([f64; 2], Vec<f64>), std::io::Error> {
    let mut zs = Vec::<f64>::with_capacity(num_points as usize);
    let mut range = [0.0; 2];
    range[0] = source.read_f64::<LittleEndian>()?;
    range[1] = source.read_f64::<LittleEndian>()?;
    for _i in 0..num_points {
        zs.push(source.read_f64::<LittleEndian>()?);
    }
    Ok((range, zs))
}

pub(crate) fn read_m_dimension<T: Read>(source: &mut T, num_points: i32) -> Result<([f64; 2], Vec<f64>), std::io::Error> {
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
    Ok((range, zs))
}

pub(crate) fn write_range_and_vec<T: Write>(dest: &mut T, range: &[f64; 2], vec: &Vec<f64>) -> Result<(), std::io::Error> {
    dest.write_f64::<LittleEndian>(range[0])?;
    dest.write_f64::<LittleEndian>(range[1])?;

    for m in vec {
        dest.write_f64::<LittleEndian>(*m)?;
    }

    Ok(())
}

pub(crate) fn write_points<T: Write>(dest: &mut T, xs: &Vec<f64>, ys: &Vec<f64>) -> Result<(), std::io::Error> {
    assert_eq!(xs.len(), ys.len());

    for (x, y) in xs.into_iter().zip(ys) {
        dest.write_f64::<LittleEndian>(*x)?;
        dest.write_f64::<LittleEndian>(*y)?;
    }

    Ok(())
}


pub(crate) fn write_parts<T: Write>(dest: &mut T, parts: &Vec<i32>) -> Result<(), std::io::Error> {
    for p in parts {
        dest.write_i32::<LittleEndian>(*p)?;
    }
    Ok(())
}
