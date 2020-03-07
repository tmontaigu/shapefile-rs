//! Module for the Multipatch shape
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fmt;
use std::io::{Read, Write};
use std::mem::size_of;

use record::io::*;
use record::ConcreteReadableShape;
use record::{close_points_if_not_already, GenericBBox};
use record::{EsriShape, HasShapeType, Point, PointZ, WritableShape};
use {Error, ShapeType};

#[cfg(feature = "geo-types")]
use geo_types;
#[cfg(feature = "geo-types")]
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, PartialEq)]
enum PatchType {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Patch {
    /// A linked strip of triangles, where every vertex
    /// (after the first two)completes a new triangle.
    ///
    /// A new triangle is always formed by connecting
    /// the new vertex with its two immediate predecessors
    TriangleStrip(Vec<PointZ>),
    /// A linked fan of triangles,
    /// where every vertex (after the first two) completes a new triangle.
    ///
    ///  A new triangle is always formed by connecting
    /// the new vertex with its immediate predecessor
    /// and the first vertex of the part.
    TriangleFan(Vec<PointZ>),
    /// The outer ring of a polygon.
    OuterRing(Vec<PointZ>),
    /// A hole of a polygon
    InnerRing(Vec<PointZ>),
    /// The first ring of a polygon of an unspecified type
    FirstRing(Vec<PointZ>),
    /// A ring of a polygon of an unspecified type
    Ring(Vec<PointZ>),
}

impl Patch {
    pub fn points(&self) -> &[PointZ] {
        match self {
            Patch::TriangleStrip(points) => points,
            Patch::TriangleFan(points) => points,
            Patch::OuterRing(points) => points,
            Patch::InnerRing(points) => points,
            Patch::FirstRing(points) => points,
            Patch::Ring(points) => points,
        }
    }
}

// TODO all the checks described at page 24/34
/// Shapefile's Multipatch shape (p 24/34)
///
/// The following things are important with Multipatch shape:
/// 1) Ring types must be closed
///    **(the various constructors will close the rings if you did not close them yourself)**
/// 2) InnerRings must follow their OuterRings (**this is not checked**)
/// 3) Parts must not intersects or penetrate each others (**this is not checked**)
/// 4) The points organization of [`TriangleStrip`] and [`TriangleFan`] is **not checked**
///
/// [`TriangleStrip`]: enum.Patch.html#variant.TriangleStrip
/// [`TriangleFan`]: enum.Patch.html#variant.TriangleFan
#[derive(Debug, PartialEq, Clone)]
pub struct Multipatch {
    bbox: GenericBBox<PointZ>,
    patches: Vec<Patch>,
}

impl Multipatch {
    /// Creates a Multipatch with one patch
    ///
    /// The constructor closes rings patch
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::{PointZ, Multipatch, NO_DATA, Patch};
    /// let points = vec![
    ///     PointZ::new(0.0, 0.0, 0.0, NO_DATA),
    ///     PointZ::new(0.0, 1.0, 0.0, NO_DATA),
    ///     PointZ::new(1.0, 1.0, 0.0, NO_DATA),
    ///     PointZ::new(1.0, 0.0, 0.0, NO_DATA),
    /// ];
    /// let multip = Multipatch::new(Patch::OuterRing(points));
    /// ```
    pub fn new(patch: Patch) -> Self {
        Self::with_parts(vec![patch])
    }

    pub fn with_parts(mut patches: Vec<Patch>) -> Self {
        for patch in patches.iter_mut() {
            match patch {
                Patch::TriangleStrip(_) => {},
                Patch::TriangleFan(_) => {},
                Patch::OuterRing( points) => {
                    close_points_if_not_already(points)
                },
                Patch::InnerRing(points) => {
                    close_points_if_not_already(points)
                },
                Patch::FirstRing(points) => {
                    close_points_if_not_already(points)
                },
                Patch::Ring(points) => {
                    close_points_if_not_already(points)
                },
            }
        }
        let mut bbox = GenericBBox::<PointZ>::from_points(patches[0].points());
        for patch in &patches[1..] {
            bbox.grow_from_points(patch.points());
        }

        Self {
            bbox,
            patches
        }
    }

    pub fn patches(&self) -> &Vec<Patch> {
        &self.patches
    }
    pub fn patch(&self, index: usize) -> Option<&Patch> {
        self.patches.get(index)
    }

    pub fn bbox(&self) -> &GenericBBox<PointZ> {
        &self.bbox
    }

    pub fn total_point_count(&self) -> usize {
        self.patches.iter().map(|patch| patch.points().len()).sum()
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
            "Multipatch({} patches)",
            self.patches.len()
        )
    }
}



impl HasShapeType for Multipatch {
    fn shapetype() -> ShapeType {
        ShapeType::Multipatch
    }
}

impl ConcreteReadableShape for Multipatch {
    fn read_shape_content<T: Read>(source: &mut T, record_size: i32) -> Result<Self, Error> {
       let reader = MultiPartShapeReader::<PointZ, T>::new(source)?;

        let record_size_with_m = Self::size_of_record(reader.num_points, reader.num_parts, true) as i32;
        let record_size_without_m = Self::size_of_record(reader.num_points, reader.num_parts, false) as i32;

        if (record_size != record_size_with_m) & (record_size != record_size_without_m) {
           Err(Error::InvalidShapeRecordSize)
        } else {
            let mut patch_types = vec![PatchType::Ring; reader.num_parts as usize];
            let mut patches = Vec::<Patch>::with_capacity(reader.num_parts as usize);
            for i in 0..reader.num_parts {
                patch_types[i as usize] = PatchType::read_from(reader.source)?;
            }
            let (bbox, patches_points) = reader
                .read_xy()
                .and_then(|rdr| rdr.read_zs())
                .and_then(|rdr| rdr.read_ms_if(record_size == record_size_with_m))
                .map_err(|err| Error::IoError(err))
                .map(|rdr| (rdr.bbox, rdr.parts))?;

            for (patch_type, points) in patch_types.iter().zip(patches_points) {
                let patch = match patch_type {
                    PatchType::TriangleStrip => Patch::TriangleStrip(points),
                    PatchType::TriangleFan => Patch::TriangleFan(points),
                    PatchType::OuterRing => Patch::OuterRing(points),
                    PatchType::InnerRing => Patch::InnerRing(points),
                    PatchType::FirstRing => Patch::FirstRing(points),
                    PatchType::Ring => Patch::Ring(points),
                };
                patches.push(patch);
            }
            Ok(Self { bbox, patches })
        }
    }
}

impl WritableShape for Multipatch {
    fn size_in_bytes(&self) -> usize {
        let mut size = 0usize;
        size += 4 * size_of::<f64>();
        size += size_of::<i32>();
        size += size_of::<i32>();
        size += size_of::<i32>() * self.patches.len();
        size += size_of::<i32>() * self.patches.len();
        size += 4 * size_of::<f64>() * self.total_point_count();
        size += 2 * size_of::<f64>();
        size += 2 * size_of::<f64>();
        size
    }

    fn write_to<T: Write>(self, dest: &mut T) -> Result<(), Error> {
        let parts_iter = self.patches.iter().map(|patch| patch.points());
        let writer = MultiPartShapeWriter::new(&self.bbox, parts_iter, dest);
        writer
            .write_bbox_xy()
            .and_then(|wrt| wrt.write_num_parts())
            .and_then(|wrt| wrt.write_num_points())
            .and_then(|wrt| wrt.write_parts_array())
            .and_then(|wrt| {
                for patch in self.patches.iter() {
                    match patch {
                        Patch::TriangleStrip(_) => wrt.dst.write_i32::<LittleEndian>(0)?,
                        Patch::TriangleFan(_) => wrt.dst.write_i32::<LittleEndian>(1)?,
                        Patch::OuterRing(_) => wrt.dst.write_i32::<LittleEndian>(2)?,
                        Patch::InnerRing(_) => wrt.dst.write_i32::<LittleEndian>(3)?,
                        Patch::FirstRing(_) => wrt.dst.write_i32::<LittleEndian>(4)?,
                        Patch::Ring(_) => wrt.dst.write_i32::<LittleEndian>(5)?,
                    }
                }
                Ok(wrt)
            })
            .and_then(|wrt| wrt.write_xy())
            .and_then(|wrt| wrt.write_bbox_z_range())
            .and_then(|wrt| wrt.write_zs())
            .and_then(|wrt| wrt.write_bbox_m_range())
            .and_then(|wrt| wrt.write_ms())
            .map_err(|err| Error::IoError(err))
            .map(|_wrt| {})
    }
}

impl EsriShape for Multipatch {
    fn x_range(&self) -> [f64; 2] {
        self.bbox.x_range()
    }

    fn y_range(&self) -> [f64; 2] {
        self.bbox.y_range()
    }

    fn z_range(&self) -> [f64; 2] {
        self.bbox.z_range()
    }

    fn m_range(&self) -> [f64; 2] {
        self.bbox.m_range()
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
                }
                PatchType::OuterRing => {
                    last_poly = Some(geo_types::Polygon::new(points.into(), vec![]));
                }
                PatchType::InnerRing => {
                    if let Some(ref mut polygon) = last_poly {
                        polygon.interiors_push(points);
                    } else {
                        return Err(Error::OrphanInnerRing);
                    }
                }
                PatchType::FirstRing => {
                    if last_poly.is_some() {
                        polygons.push(last_poly.take().unwrap());
                    } else {
                        last_poly = Some(geo_types::Polygon::new(points.into(), vec![]));
                    }
                }
                PatchType::Ring => {
                    if let Some(ref mut polygon) = last_poly {
                        polygon.interiors_push(points);
                    } else {
                        // treat as sequence of outer ring without hole -> a simple polygon
                        polygons.push(geo_types::Polygon::new(points.into(), vec![]));
                    }
                }
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
        if polygon.exterior().num_coords() == 0 {
            return Self::empty();
        }

        let mut total_num_points = polygon.exterior().num_coords();
        total_num_points += polygon
            .interiors()
            .iter()
            .map(|ls| ls.num_coords())
            .sum::<usize>();

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
            m_range,
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
            multipatch
                .parts
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
            m_range,
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::{Multipatch, PatchType, PointZ};
//
//     #[test]
//     fn test_multipatch_creation() {
//         let mp = Multipatch::with_parts(vec![
//             (
//                 vec![
//                     PointZ::new(0.0, 1.0, 0.0, 0.0),
//                     PointZ::new(0.0, 2.0, 0.0, 0.0),
//                     PointZ::new(0.0, 1.0, 0.0, 0.0),
//                 ],
//                 PatchType::OuterRing,
//             ),
//             (
//                 vec![
//                     PointZ::new(0.0, 17.0, 5.0, 0.0),
//                     PointZ::new(0.0, 28.0, 0.0, 0.0),
//                     PointZ::new(0.0, 17.0, 5.0, 0.0),
//                 ],
//                 PatchType::OuterRing,
//             ),
//         ]);
//
//         assert_eq!(mp.parts, vec![0, 3]);
//         assert_eq!(
//             mp.parts_type,
//             vec![PatchType::OuterRing, PatchType::OuterRing]
//         );
//     }
// }
