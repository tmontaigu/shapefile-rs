//! Module for the Multipatch shape
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fmt;
use std::io::{Read, Write};
use std::mem::size_of;
use std::slice::SliceIndex;

use record::io::*;
use record::{is_parts_array_valid, close_points_if_not_already};
use record::traits::{MultipartShape, MultipointShape};
use record::BBox;
use record::ConcreteReadableShape;
use record::{EsriShape, HasShapeType, Point, PointZ, WritableShape};
use {Error, ShapeType};

#[cfg(feature = "geo-types")]
use geo_types;
#[cfg(feature = "geo-types")]
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PatchType {
    TriangleStrip,
    TriangleFan,
    OuterRing,
    InnerRing,
    FirstRing,
    Ring,
}

impl PatchType {
    pub fn read_from<T: Read>(source: &mut T) -> Result<PatchType, Error> {
        let code = source.read_i32::<LittleEndian>()?;
        Self::from(code).ok_or_else(|| Error::InvalidPatchType(code))
    }

    pub fn from(code: i32) -> Option<PatchType> {
        match code {
            0 => Some(PatchType::TriangleStrip),
            1 => Some(PatchType::TriangleFan),
            2 => Some(PatchType::OuterRing),
            3 => Some(PatchType::InnerRing),
            4 => Some(PatchType::FirstRing),
            5 => Some(PatchType::Ring),
            _ => None,
        }
    }
}

/// Following things are important with Multipatch shape:
/// 1) Ring types must be closed
/// 2) InnerRings must follow their OuterRings
/// 3) Parts must not intersects or penetrate each others
#[derive(Debug, PartialEq, Clone)]
pub struct Multipatch {
    pub bbox: BBox,
    pub points: Vec<PointZ>,
    pub parts: Vec<i32>,
    pub parts_type: Vec<PatchType>,
    pub z_range: [f64; 2],
    pub m_range: [f64; 2],
}

impl Multipatch {
    /// # Examples
    ///
    /// Creating a Multipatch with one outer ring,
    /// The constructor closes rings
    /// ```
    /// use shapefile::{PointZ, Multipatch, NO_DATA, PatchType};
    /// let points = vec![
    ///     PointZ::new(0.0, 0.0, 0.0, NO_DATA),
    ///     PointZ::new(0.0, 1.0, 0.0, NO_DATA),
    ///     PointZ::new(1.0, 0.0, 0.0, NO_DATA),
    /// ];
    /// let multip = Multipatch::new(points, PatchType::OuterRing);
    ///
    /// // The polygon gets closed 'explicitly'
    /// assert_eq!(multip.points.len(), 4);
    /// assert_eq!(multip.points.last(), multip.points.first());
    /// ```
    ///
    pub fn new(mut points: Vec<PointZ>, part_type: PatchType) -> Self {
        match part_type {
            PatchType::OuterRing | PatchType::InnerRing | PatchType::FirstRing => {
                close_points_if_not_already(&mut points);
            },
            _ => {},
        }
        let bbox = BBox::from_points(&points);
        let m_range = calc_m_range(&points);
        let z_range = calc_z_range(&points);
        Self {
            bbox,
            points,
            parts: vec![0],
            parts_type: vec![part_type],
            z_range,
            m_range,
        }
    }

    pub fn with_parts(parts_points: Vec<(Vec<PointZ>, PatchType)>) -> Self {
        let num_parts = parts_points.len();
        let num_points = parts_points.iter().map(|pts| pts.0.len()).sum();

        let mut all_points = Vec::<PointZ>::with_capacity(num_points);
        let mut parts = Vec::<i32>::with_capacity(num_parts);
        let mut parts_type = Vec::<PatchType>::with_capacity(num_parts);

        for (i, (mut points, patch_type)) in parts_points.into_iter().enumerate() {
            if i != num_parts - 1 {
                parts.push(all_points.len() as i32);
            }
            match patch_type {
                PatchType::OuterRing | PatchType::InnerRing | PatchType::FirstRing => {
                    close_points_if_not_already(&mut points);
                },
                _ => {},
            }
            all_points.append(&mut points);
            parts_type.push(patch_type);
        }

        let bbox = BBox::from_points(&all_points);
        let m_range = calc_m_range(&all_points);
        let z_range = calc_z_range(&all_points);
        Self {
            bbox,
            points: all_points,
            parts,
            parts_type,
            z_range,
            m_range
        }
    }

    pub fn empty() -> Self {
        Self {
            bbox: BBox::new(0.0, 0.0, 0.0, 0.0),
            points: vec![],
            parts: vec![],
            parts_type: vec![],
            z_range: [0.0,0.0],
            m_range: [0.0, 0.0]
        }
    }

    pub(crate) fn size_of_record(num_points: i32, num_parts: i32, is_m_used: bool) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>(); // BBOX
        size += size_of::<i32>(); // num parts
        size += size_of::<i32>(); // num points
        size += size_of::<i32>() * num_parts as usize; // parts
        size += size_of::<i32>() * num_parts as usize; // parts type
        size += size_of::<Point>() * num_points as usize;
        size += 2 * size_of::<f64>(); // mandatory Z Range
        size += size_of::<f64>() * num_points as usize; // mandatory Z

        if is_m_used {
            size += 2 * size_of::<f64>(); // Optional M range
            size += size_of::<f64>() * num_points as usize; // Optional M
        }
        size
    }
}

impl fmt::Display for Multipatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Multipatch({} points, {} parts)",
            self.points.len(),
            self.parts.len()
        )
    }
}

impl MultipointShape<PointZ> for Multipatch {
    fn point<I: SliceIndex<[PointZ]>>(
        &self,
        index: I,
    ) -> Option<&<I as SliceIndex<[PointZ]>>::Output> {
        self.points.get(index)
    }
    fn points(&self) -> &[PointZ] {
        &self.points
    }
}

impl MultipartShape<PointZ> for Multipatch {
    fn parts_indices(&self) -> &[i32] {
        &self.parts
    }
}

impl HasShapeType for Multipatch {
    fn shapetype() -> ShapeType {
        ShapeType::Multipatch
    }
}

impl ConcreteReadableShape for Multipatch {
    fn read_shape_content<T: Read>(mut source: &mut T, record_size: i32) -> Result<Self, Error> {
        let bbox = BBox::read_from(&mut source)?;
        let num_parts = source.read_i32::<LittleEndian>()?;
        let num_points = source.read_i32::<LittleEndian>()?;

        let record_size_with_m = Self::size_of_record(num_points, num_parts, true) as i32;
        let record_size_without_m = Self::size_of_record(num_points, num_parts, false) as i32;

        if (record_size != record_size_with_m) & (record_size != record_size_without_m) {
            return Err(Error::InvalidShapeRecordSize);
        }

        let is_m_used = record_size == record_size_with_m;

        let parts = read_parts(&mut source, num_parts)?;

        let mut parts_type = Vec::<PatchType>::with_capacity(num_parts as usize);
        for _ in 0..num_parts {
            let code = source.read_i32::<LittleEndian>()?;
            match PatchType::from(code) {
                Some(t) => parts_type.push(t),
                None => return Err(Error::InvalidPatchType(code)),
            }
        }
        let mut points = read_xy_in_vec_of::<PointZ, T>(&mut source, num_points)?;

        let z_range = read_range(&mut source)?;
        read_zs_into(&mut source, &mut points)?;

        let mut m_range = [0.0, 0.0];
        if is_m_used {
            m_range = read_range(&mut source)?;
            read_ms_into(&mut source, &mut points)?;
        }

        Ok(Self {
            bbox,
            parts,
            parts_type,
            points,
            z_range,
            m_range,
        })
    }
}

impl WritableShape for Multipatch {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += size_of::<i32>();
        size += size_of::<i32>() * self.parts.len();
        size += size_of::<i32>() * self.parts_type.len();
        size += 4 * size_of::<f64>() * self.points.len();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, mut dest: &mut T) -> Result<(), Error> {
        if !is_parts_array_valid(&self) {
            return Err(Error::MalformedShape);
        }
        self.bbox.write_to(&mut dest)?;
        dest.write_i32::<LittleEndian>(self.parts.len() as i32)?;
        dest.write_i32::<LittleEndian>(self.points.len() as i32)?;
        write_parts(&mut dest, &self.parts)?;
        let part_types: Vec<i32> = self.parts_type.iter().map(|t| *t as i32).collect();
        write_parts(&mut dest, &part_types)?;

        write_points(&mut dest, &self.points)?;

        write_range(&mut dest, self.z_range())?;
        write_zs(&mut dest, &self.points)?;

        write_range(&mut dest, self.m_range())?;
        write_ms(&mut dest, &self.points)?;
        Ok(())
    }
}

impl EsriShape for Multipatch {
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

/// Converts a Multipatch to Multipolygon
///
/// For simplicity,reasons, Triangle Fan & Triangle Strip are considered
/// to be valid polygons
/// `
/// When the individual types of rings in a collection of rings representing a polygonal patch with holes
/// are unknown, the sequence must start with First Ring,
/// followed by a number of Rings. A sequence of Rings not preceded by an First Ring
/// is treated as a sequence of Outer Rings without holes.
/// `
#[cfg(feature = "geo-types")]
impl TryFrom<Multipatch> for geo_types::MultiPolygon<f64> {
    type Error = Error;

    fn try_from(mp: Multipatch) -> Result<Self, Self::Error> {
       let mut polygons = Vec::<geo_types::Polygon<f64>>::new();
        let mut last_poly = None;
       for (points, part_type) in mp.parts().zip(&mp.parts_type) {
           let points = points
               .iter()
               .map(|p| geo_types::Point::<f64>::from(*p))
               .collect::<Vec<geo_types::Point<f64>>>();
            match part_type {
                PatchType::TriangleStrip | PatchType::TriangleFan => {
                    polygons.push(geo_types::Polygon::new(points.into(), vec![]));
                },
                PatchType::OuterRing => {
                    last_poly = Some(geo_types::Polygon::new(points.into(), vec![]));
                },
                PatchType::InnerRing => {
                    if let Some(ref mut polygon) = last_poly {
                        polygon.interiors_push(points);
                    } else {
                        return Err(Error::OrphanInnerRing)
                    }
                },
                PatchType::FirstRing => {
                    if last_poly.is_some() {
                        polygons.push(last_poly.take().unwrap());
                    } else {
                        last_poly = Some(geo_types::Polygon::new(points.into(), vec![]));
                    }
                },
                PatchType::Ring => {
                    if let Some(ref mut polygon) = last_poly {
                        polygon.interiors_push(points);
                    } else {
                        // treat as sequence of outer ring without hole -> a simple polygon
                        polygons.push(geo_types::Polygon::new(points.into(), vec![]));
                    }
                },
            }
       }
        if let Some(poly) = last_poly {
            polygons.push(poly);
        }
        Ok(polygons.into())
    }
}


#[cfg(feature = "geo-types")]
impl From<geo_types::Polygon<f64>> for Multipatch {
    fn from(polygon: geo_types::Polygon<f64>) -> Self {
        if polygon.exterior(). num_coords() == 0 {
            return Self::empty();
        }

        let mut total_num_points = polygon.exterior().num_coords();
        total_num_points += polygon.interiors().iter().map(|ls| ls.num_coords()).sum::<usize>();

        let mut parts = vec![0i32];
        let mut parts_type = vec![PatchType::OuterRing];
        let mut all_points = Vec::<PointZ>::with_capacity(total_num_points);

        let (outer_ls, inners_ls) = polygon.into_inner();
        let mut outer_points = outer_ls
            .into_iter()
            .map(|c| PointZ::from(c))
            .collect::<Vec<PointZ>>();

        if super::ring_type_from_points_ordering(&outer_points) == super::RingType::InnerRing {
            outer_points.reverse();
        }
        all_points.append(&mut outer_points);

        for inner_ls in inners_ls {
            parts.push((all_points.len() - 1) as i32);
            let mut inner_points = inner_ls
                .into_iter()
                .map(|c| PointZ::from(c))
                .collect::<Vec<PointZ>>();

            if super::ring_type_from_points_ordering(&inner_points) == super::RingType::OuterRing {
                inner_points.reverse();
            }
            all_points.append(&mut inner_points);
            parts_type.push(PatchType::InnerRing);
        }

        let m_range = calc_m_range(&all_points);
        let z_range = calc_z_range(&all_points);
        Self {
            bbox: BBox::from_points(&all_points),
            points: all_points,
            parts,
            parts_type,
            z_range,
            m_range
        }
    }
}


#[cfg(feature = "geo-types")]
impl From<geo_types::MultiPolygon<f64>> for Multipatch {
    fn from(multi_polygon: geo_types::MultiPolygon<f64>) -> Self {
        let multipatches = multi_polygon
            .into_iter()
            .map(|polyg| Multipatch::from(polyg))
            .collect::<Vec<Multipatch>>();

        let total_points_count = multipatches
            .iter()
            .fold(0usize, |count, multipatch| count + multipatch.points.len());

        let total_part_count = multipatches
            .iter()
            .fold(0usize, |count, multipatch| count + multipatch.parts.len());

        let mut all_points = Vec::<PointZ>::with_capacity(total_points_count);
        let mut all_parts = Vec::<i32>::with_capacity(total_part_count);
        let mut all_parts_type = Vec::<PatchType>::with_capacity(total_part_count);

        for mut multipatch in multipatches {
            multipatch.parts
                .into_iter()
                .map(|index| index + (all_points.len() as i32))
                .for_each(|index| all_parts.push(index));
            all_points.append(&mut multipatch.points);
            all_parts_type.append(&mut multipatch.parts_type);

        }
        let m_range = calc_m_range(&all_points);
        let z_range = calc_z_range(&all_points);
        Self {
            bbox: BBox::from_points(&all_points),
            points: all_points,
            parts: all_parts,
            parts_type: all_parts_type,
            z_range,
            m_range
        }
    }
}