use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use record::traits::{HasM, HasMutM, HasMutXY, HasMutZ, HasXY, HasZ};
use record::{GenericBBox, PointZ, NO_DATA};
use ::{Point, PointM};

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


struct PartIndexIter<'a> {
    parts_indices: &'a Vec<i32>,
    current_part_index: usize,
    num_points: i32,
}

impl<'a> PartIndexIter<'a> {
    fn new(parts_indices: &'a Vec<i32>, num_points: i32) -> Self {
        Self {
            parts_indices,
            current_part_index: 0,
            num_points
        }
    }
}

impl<'a> Iterator for PartIndexIter<'a> {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_part_index < self.parts_indices.len() {
            let start_of_part_index = self.parts_indices[self.current_part_index];
            let end_of_part_index = self.parts_indices
                .get(self.current_part_index + 1).copied().unwrap_or(self.num_points);
            self.current_part_index += 1;
            debug_assert!(end_of_part_index >= start_of_part_index);
            Some((start_of_part_index, end_of_part_index))
        } else {
            None
        }
    }
}


pub(crate) struct MultiPartShapeReader<'a, PointType, R: Read> {
    pub(crate) num_points: i32,
    pub(crate) num_parts: i32,
    pub(crate) parts: Vec<Vec<PointType>>,
    pub(crate) bbox: GenericBBox<PointType>,
    pub(crate) source: &'a mut R,
    parts_array: Vec<i32>,
}

impl<'a, PointType: Default + HasMutXY, R: Read> MultiPartShapeReader<'a, PointType, R> {
    pub(crate) fn new(source: &'a mut R) -> std::io::Result<Self> {
        let  mut bbox = GenericBBox::<PointType>::default();
        bbox_read_xy_from(&mut bbox, source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;
        let parts_array = read_parts(source, num_parts)?;
        let parts = Vec::<Vec<PointType>>::with_capacity(num_parts as usize);
        Ok(Self {
            num_points,
            num_parts,
            parts_array,
            parts,
            source,
            bbox,
        })
    }

    pub(crate) fn read_xy(mut self) -> std::io::Result<Self> {
        for (start_index, end_index) in PartIndexIter::new(&self.parts_array, self.num_points) {
            let num_points_in_part = end_index - start_index;
            self.parts.push(read_xy_in_vec_of(self.source, num_points_in_part)?);
        }
        Ok(self)
    }
}

impl<'a, PointType: HasMutM, R: Read> MultiPartShapeReader<'a, PointType, R> {
    pub(crate) fn read_ms(mut self) -> std::io::Result<Self> {
        bbox_read_m_range_from(&mut self.bbox, &mut self.source)?;
        for part_points in self.parts.iter_mut() {
            read_ms_into(self.source, part_points)?;
        }
        Ok(self)
    }

    pub(crate) fn read_ms_if(self, condition: bool) -> std::io::Result<Self> {
        if condition {
            self.read_ms()
        } else {
            Ok(self)
        }
    }
}

impl<'a, R: Read> MultiPartShapeReader<'a, PointZ, R> {
    pub(crate) fn read_zs(mut self) -> std::io::Result<Self> {
        bbox_read_z_range_from(&mut self.bbox, &mut self.source)?;
        for part_points in self.parts.iter_mut() {
            read_zs_into(self.source, part_points)?;
        }
        Ok(self)
    }
}


pub(crate) struct MultiPartShapeWriter<'a, PointType, T, W>
where T: Iterator<Item=&'a [PointType]> + Clone,
      W: Write {
    pub(crate) dst: &'a mut W,
    parts_iter: T,
    bbox: &'a GenericBBox<PointType>,
}

impl<'a, PointType, T, W> MultiPartShapeWriter<'a, PointType, T, W>
    where T: Iterator<Item=&'a [PointType]> + Clone,
          W: Write {

    pub(crate) fn new(bbox: &'a GenericBBox<PointType>, parts_iter: T, dst: &'a mut W) -> Self {
        Self {
            parts_iter,
            bbox,
            dst,
        }
    }

    pub(crate) fn write_num_points(self) -> std::io::Result<Self> {
        let point_count: usize = self.parts_iter.clone().map(|points| points.len()).sum();
        self.dst.write_i32::<LittleEndian>(point_count as i32)?;
        Ok(self)
    }

    pub(crate) fn write_num_parts(self) -> std::io::Result<Self> {
        let num_parts = self.parts_iter.clone().count();
        self.dst.write_i32::<LittleEndian>(num_parts as i32)?;
        Ok(self)
    }

    pub(crate) fn write_parts_array(self) -> std::io::Result<Self> {
        let mut sum = 0;
        for i in self.parts_iter.clone().map(|points| points.len() as i32) {
            self.dst.write_i32::<LittleEndian>(sum)?;
            sum += i;
        }
        Ok(self)
    }
}

impl<'a, PointType, T, W> MultiPartShapeWriter<'a, PointType, T, W>
    where T: Iterator<Item=&'a [PointType]> + Clone,
          W: Write,
          PointType: HasXY
{
    pub(crate) fn write_bbox_xy(self) -> std::io::Result<Self> {
        bbox_write_xy_to(&self.bbox, self.dst)?;
        Ok(self)
    }

    pub(crate) fn write_xy(self) -> std::io::Result<Self> {
        for points in self.parts_iter.clone() {
            write_points(self.dst, points)?;
        }
        Ok(self)
    }
}

impl<'a, PointType, T, W> MultiPartShapeWriter<'a, PointType, T, W>
    where T: Iterator<Item=&'a [PointType]> + Clone,
          W: Write,
          PointType: HasM
{
    pub(crate) fn write_bbox_m_range(self) -> std::io::Result<Self> {
        bbox_write_m_range_to(&self.bbox, self.dst)?;
        Ok(self)
    }

    pub(crate) fn write_ms(self) -> std::io::Result<Self> {
        for points in self.parts_iter.clone() {
            write_ms(self.dst, points)?;
        }
        Ok(self)
    }
}

impl<'a, T, W> MultiPartShapeWriter<'a, PointZ, T, W>
    where T: Iterator<Item=&'a [PointZ]> + Clone,
          W: Write

{
    pub(crate) fn write_bbox_z_range(self) -> std::io::Result<Self> {
        bbox_write_z_range_to(&self.bbox, self.dst)?;
        Ok(self)
    }

    pub(crate) fn write_zs(self) -> std::io::Result<Self> {
        for points in self.parts_iter.clone() {
            write_zs(self.dst, points)?;
        }
        Ok(self)
    }
}

impl<'a, T, W> MultiPartShapeWriter<'a, Point, T, W>
    where T: Iterator<Item=&'a [Point]> + Clone,
          W: Write {

    pub(crate) fn write_point_shape(self) -> std::io::Result<Self> {
        self.write_bbox_xy()
            .and_then(|wrt| wrt.write_num_parts())
            .and_then(|wrt| wrt.write_num_points())
            .and_then(|wrt| wrt.write_parts_array())
            .and_then(|wrt| wrt.write_xy())
    }
}

impl<'a, T, W> MultiPartShapeWriter<'a, PointM, T, W>
    where T: Iterator<Item=&'a [PointM]> + Clone,
          W: Write {
    pub(crate) fn write_point_m_shape(self) -> std::io::Result<Self> {
        self.write_bbox_xy()
            .and_then(|wrt| wrt.write_num_parts())
            .and_then(|wrt| wrt.write_num_points())
            .and_then(|wrt| wrt.write_parts_array())
            .and_then(|wrt| wrt.write_xy())
            .and_then(|wrt| wrt.write_bbox_m_range())
            .and_then(|wrt| wrt.write_ms())
    }
}

impl<'a, T, W> MultiPartShapeWriter<'a, PointZ, T, W>
    where T: Iterator<Item=&'a [PointZ]> + Clone,
          W: Write {
    pub(crate) fn write_point_z_shape(self) -> std::io::Result<Self> {
        self.write_bbox_xy()
            .and_then(|wrt| wrt.write_num_parts())
            .and_then(|wrt| wrt.write_num_points())
            .and_then(|wrt| wrt.write_parts_array())
            .and_then(|wrt| wrt.write_xy())
            .and_then(|wrt| wrt.write_bbox_z_range())
            .and_then(|wrt| wrt.write_zs())
            .and_then(|wrt| wrt.write_bbox_m_range())
            .and_then(|wrt| wrt.write_ms())
    }
}
