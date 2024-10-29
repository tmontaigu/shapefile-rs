use std::hint::unreachable_unchecked;

use crate::{
    Multipoint, MultipointM, MultipointZ, Point, PointM, PointZ, PolygonRing, Polyline, PolylineM,
    PolylineZ, NO_DATA,
};
use geo_traits::{
    CoordTrait, LineStringTrait, MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait,
    PointTrait, PolygonTrait,
};

// Shapefile points can't be null, so we implement both traits on them
impl CoordTrait for Point {
    type T = f64;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("invalid dimension index"),
        }
    }

    unsafe fn nth_unchecked(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => unreachable_unchecked(),
        }
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl CoordTrait for &Point {
    type T = f64;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("invalid dimension index"),
        }
    }

    unsafe fn nth_unchecked(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => unreachable_unchecked(),
        }
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl PointTrait for Point {
    type T = f64;
    type CoordType<'a>
        = &'a Point
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(self)
    }
}

impl PointTrait for &Point {
    type T = f64;
    type CoordType<'a>
        = &'a Point
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(self)
    }
}

// Shapefile points can't be null, so we implement both traits on them
impl CoordTrait for PointM {
    type T = f64;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.m <= NO_DATA {
            geo_traits::Dimensions::Xy
        } else {
            geo_traits::Dimensions::Xym
        }
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            2 => self.m,
            _ => panic!("invalid dimension index"),
        }
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl CoordTrait for &PointM {
    type T = f64;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.m <= NO_DATA {
            geo_traits::Dimensions::Xy
        } else {
            geo_traits::Dimensions::Xym
        }
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            2 => self.m,
            _ => panic!("invalid dimension index"),
        }
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl PointTrait for PointM {
    type T = f64;
    type CoordType<'a>
        = &'a PointM
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.m <= NO_DATA {
            geo_traits::Dimensions::Xy
        } else {
            geo_traits::Dimensions::Xym
        }
    }

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(self)
    }
}

impl PointTrait for &PointM {
    type T = f64;
    type CoordType<'a>
        = &'a PointM
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.m <= NO_DATA {
            geo_traits::Dimensions::Xy
        } else {
            geo_traits::Dimensions::Xym
        }
    }

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(self)
    }
}

// Shapefile points can't be null, so we implement both traits on them
impl CoordTrait for PointZ {
    type T = f64;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.m <= NO_DATA {
            geo_traits::Dimensions::Xyz
        } else {
            geo_traits::Dimensions::Xyzm
        }
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            2 => self.z,
            3 => {
                if self.m > NO_DATA {
                    self.m
                } else {
                    panic!("asked for 4th item from coordinate but this coordinate does not have 4 dimensions.")
                }
            }
            _ => panic!("invalid dimension index"),
        }
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl CoordTrait for &PointZ {
    type T = f64;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.m <= NO_DATA {
            geo_traits::Dimensions::Xyz
        } else {
            geo_traits::Dimensions::Xyzm
        }
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            2 => self.z,
            3 => {
                if self.m > NO_DATA {
                    self.m
                } else {
                    panic!("asked for 4th item from coordinate but this coordinate does not have 4 dimensions.")
                }
            }
            _ => panic!("invalid dimension index"),
        }
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl PointTrait for PointZ {
    type T = f64;
    type CoordType<'a>
        = &'a PointZ
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.m <= NO_DATA {
            geo_traits::Dimensions::Xyz
        } else {
            geo_traits::Dimensions::Xyzm
        }
    }

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(self)
    }
}

impl PointTrait for &PointZ {
    type T = f64;
    type CoordType<'a>
        = &'a PointZ
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        if self.m <= NO_DATA {
            geo_traits::Dimensions::Xyz
        } else {
            geo_traits::Dimensions::Xyzm
        }
    }

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(self)
    }
}

pub struct LineString<'a>(&'a [Point]);

impl LineStringTrait for LineString<'_> {
    type T = f64;
    type CoordType<'b>
        = &'b Point
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.0.get_unchecked(i)
    }
}

pub struct LineStringM<'a>(&'a [PointM]);

impl LineStringTrait for LineStringM<'_> {
    type T = f64;
    type CoordType<'b>
        = &'b PointM
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xym
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.0.get_unchecked(i)
    }
}

pub struct LineStringZ<'a>(&'a [PointZ]);

impl LineStringTrait for LineStringZ<'_> {
    type T = f64;
    type CoordType<'b>
        = &'b PointZ
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        // Check the first underlying coordinate to check if it's XYZ or XYZM
        self.0
            .first()
            .map(CoordTrait::dim)
            .unwrap_or(geo_traits::Dimensions::Xyz)
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.0.get_unchecked(i)
    }
}

pub struct Polygon {
    outer: Vec<Point>,
    inner: Vec<Vec<Point>>,
}

impl<'a> PolygonTrait for &'a Polygon {
    type T = f64;
    type RingType<'b>
        = LineString<'a>
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn num_interiors(&self) -> usize {
        self.inner.len()
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        Some(LineString(&self.outer))
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        LineString(&self.inner[i])
    }
}

pub struct PolygonM {
    outer: Vec<PointM>,
    inner: Vec<Vec<PointM>>,
}

impl<'a> PolygonTrait for &'a PolygonM {
    type T = f64;
    type RingType<'b>
        = LineStringM<'a>
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xym
    }

    fn num_interiors(&self) -> usize {
        self.inner.len()
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        Some(LineStringM(&self.outer))
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        LineStringM(&self.inner[i])
    }
}

pub struct PolygonZ {
    outer: Vec<PointZ>,
    inner: Vec<Vec<PointZ>>,
}

impl<'a> PolygonTrait for &'a PolygonZ {
    type T = f64;
    type RingType<'b>
        = LineStringZ<'a>
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        // Check the first coord of the outer ring to check if it's XYZ or XYZM
        self.outer
            .first()
            .map(CoordTrait::dim)
            .unwrap_or(geo_traits::Dimensions::Xyz)
    }

    fn num_interiors(&self) -> usize {
        self.inner.len()
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        Some(LineStringZ(&self.outer))
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        LineStringZ(&self.inner[i])
    }
}

impl MultiPointTrait for Multipoint {
    type T = f64;
    type PointType<'b>
        = &'b Point
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn num_points(&self) -> usize {
        self.points().len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        self.point(i).unwrap()
    }
}

impl MultiPointTrait for MultipointM {
    type T = f64;
    type PointType<'b>
        = &'b PointM
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xym
    }

    fn num_points(&self) -> usize {
        self.points().len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        self.point(i).unwrap()
    }
}

impl MultiPointTrait for MultipointZ {
    type T = f64;
    type PointType<'b>
        = &'b PointZ
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        // Check the first point to check if it's XYZ or XYZM
        self.points
            .first()
            .map(CoordTrait::dim)
            .unwrap_or(geo_traits::Dimensions::Xyz)
    }

    fn num_points(&self) -> usize {
        self.points().len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        self.point(i).unwrap()
    }
}

impl MultiLineStringTrait for Polyline {
    type T = f64;
    type LineStringType<'a>
        = LineString<'a>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn num_line_strings(&self) -> usize {
        self.parts().len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        LineString(self.part(i).unwrap())
    }
}

impl MultiLineStringTrait for PolylineM {
    type T = f64;
    type LineStringType<'a>
        = LineStringM<'a>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xym
    }

    fn num_line_strings(&self) -> usize {
        self.parts().len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        LineStringM(self.part(i).unwrap())
    }
}

impl MultiLineStringTrait for PolylineZ {
    type T = f64;
    type LineStringType<'a>
        = LineStringZ<'a>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        // Check the first point to check if it's XYZ or XYZM
        self.parts
            .first()
            .and_then(|line_string| line_string.first().map(CoordTrait::dim))
            .unwrap_or(geo_traits::Dimensions::Xyz)
    }

    fn num_line_strings(&self) -> usize {
        self.parts().len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        LineStringZ(self.part(i).unwrap())
    }
}

pub struct MultiPolygon(Vec<Polygon>);

impl From<crate::Polygon> for MultiPolygon {
    fn from(geom: crate::Polygon) -> Self {
        let mut last_poly = None;
        let mut polygons = Vec::new();
        for ring in geom.into_inner() {
            match ring {
                PolygonRing::Outer(points) => {
                    if let Some(poly) = last_poly.take() {
                        polygons.push(poly);
                    }
                    last_poly = Some(Polygon {
                        outer: points,
                        inner: vec![],
                    });
                }
                PolygonRing::Inner(points) => {
                    if let Some(poly) = last_poly.as_mut() {
                        poly.inner.push(points);
                    } else {
                        panic!("inner ring without a previous outer ring");
                        // This is the strange (?) case: inner ring without a previous outer ring
                        // polygons.push(geo_types::Polygon::<f64>::new(
                        //     LineString::<f64>::from(Vec::<Coordinate<f64>>::new()),
                        //     vec![LineString::from(interior)],
                        // ));
                    }
                }
            }
        }

        if let Some(poly) = last_poly.take() {
            polygons.push(poly);
        }

        Self(polygons)
    }
}

impl MultiPolygonTrait for MultiPolygon {
    type T = f64;
    type PolygonType<'a> = &'a Polygon;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn num_polygons(&self) -> usize {
        self.0.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        &self.0[i]
    }
}

pub struct MultiPolygonM(Vec<Polygon>);

impl From<crate::Polygon> for MultiPolygonM {
    fn from(geom: crate::Polygon) -> Self {
        let mut last_poly = None;
        let mut polygons = Vec::new();
        for ring in geom.into_inner() {
            match ring {
                PolygonRing::Outer(points) => {
                    if let Some(poly) = last_poly.take() {
                        polygons.push(poly);
                    }
                    last_poly = Some(Polygon {
                        outer: points,
                        inner: vec![],
                    });
                }
                PolygonRing::Inner(points) => {
                    if let Some(poly) = last_poly.as_mut() {
                        poly.inner.push(points);
                    } else {
                        panic!("inner ring without a previous outer ring");
                        // This is the strange (?) case: inner ring without a previous outer ring
                        // polygons.push(geo_types::Polygon::<f64>::new(
                        //     LineString::<f64>::from(Vec::<Coordinate<f64>>::new()),
                        //     vec![LineString::from(interior)],
                        // ));
                    }
                }
            }
        }

        if let Some(poly) = last_poly.take() {
            polygons.push(poly);
        }

        Self(polygons)
    }
}

impl MultiPolygonTrait for MultiPolygonM {
    type T = f64;
    type PolygonType<'a> = &'a Polygon;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xym
    }

    fn num_polygons(&self) -> usize {
        self.0.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        &self.0[i]
    }
}

pub struct MultiPolygonZ(Vec<PolygonZ>);

impl From<crate::PolygonZ> for MultiPolygonZ {
    fn from(geom: crate::PolygonZ) -> Self {
        let mut last_poly = None;
        let mut polygons = Vec::new();
        for ring in geom.into_inner() {
            match ring {
                PolygonRing::Outer(points) => {
                    if let Some(poly) = last_poly.take() {
                        polygons.push(poly);
                    }
                    last_poly = Some(PolygonZ {
                        outer: points,
                        inner: vec![],
                    });
                }
                PolygonRing::Inner(points) => {
                    if let Some(poly) = last_poly.as_mut() {
                        poly.inner.push(points);
                    } else {
                        panic!("inner ring without a previous outer ring");
                        // This is the strange (?) case: inner ring without a previous outer ring
                        // polygons.push(geo_types::Polygon::<f64>::new(
                        //     LineString::<f64>::from(Vec::<Coordinate<f64>>::new()),
                        //     vec![LineString::from(interior)],
                        // ));
                    }
                }
            }
        }

        if let Some(poly) = last_poly.take() {
            polygons.push(poly);
        }

        Self(polygons)
    }
}

impl MultiPolygonTrait for MultiPolygonZ {
    type T = f64;
    type PolygonType<'a> = &'a PolygonZ;

    fn dim(&self) -> geo_traits::Dimensions {
        // Check the first polygon to check if it's XYZ or XYZM
        self.0
            .first()
            .map(|polygon| polygon.dim())
            .unwrap_or(geo_traits::Dimensions::Xyz)
    }

    fn num_polygons(&self) -> usize {
        self.0.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        &self.0[i]
    }
}
