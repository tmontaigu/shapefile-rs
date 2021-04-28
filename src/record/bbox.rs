///! Bounding Boxes
use record::traits::{GrowablePoint, HasM, HasXY, HasZ, ShrinkablePoint};
use record::EsriShape;
use writer::{f64_max, f64_min};
use PointZ;

/// The Bounding Box type used in this crate.
///
/// Each shape that is a collection of points have a bounding box
/// associated to it generally accessible using the `bbox()` method.
///
/// # Example
///
/// ```
/// use shapefile::{PointM, PolylineM};
/// let poly = PolylineM::new(vec![
///     PointM::new(1.0, 2.0, 13.42),
///     PointM::new(2.0, 1.0, 42.3713),
/// ]);
///
/// let bbox = poly.bbox();
/// assert_eq!(bbox.min, PointM::new(1.0, 1.0, 13.42));
/// assert_eq!(bbox.max, PointM::new(2.0, 2.0, 42.3713));
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GenericBBox<PointType> {
    pub max: PointType,
    pub min: PointType,
}

impl<PointType> GenericBBox<PointType> {
    pub(crate) fn from_points(points: &[PointType]) -> Self
    where
        PointType: Copy + ShrinkablePoint + GrowablePoint,
    {
        let mut min_point = points[0];
        let mut max_point = points[0];

        for point in &points[1..] {
            min_point.shrink(point);
            max_point.grow(point);
        }

        Self {
            max: max_point,
            min: min_point,
        }
    }

    pub(crate) fn grow_from_points(&mut self, points: &[PointType])
    where
        PointType: ShrinkablePoint + GrowablePoint,
    {
        for point in points {
            self.min.shrink(point);
            self.max.grow(point);
        }
    }

    pub(crate) fn from_parts(parts: &Vec<Vec<PointType>>) -> Self
    where
        PointType: ShrinkablePoint + GrowablePoint + Copy,
    {
        let mut bbox = Self::from_points(&parts[0]);
        for part in &parts[1..] {
            bbox.grow_from_points(part);
        }
        bbox
    }
}

impl<PointType: HasXY> GenericBBox<PointType> {
    pub fn x_range(&self) -> [f64; 2] {
        [self.min.x(), self.max.x()]
    }

    pub fn y_range(&self) -> [f64; 2] {
        [self.min.y(), self.max.y()]
    }
}

impl<PointType: HasZ> GenericBBox<PointType> {
    pub fn z_range(&self) -> [f64; 2] {
        [self.min.z(), self.max.z()]
    }
}

impl<PointType: HasM> GenericBBox<PointType> {
    pub fn m_range(&self) -> [f64; 2] {
        [self.min.m(), self.max.m()]
    }
}

impl<PointType: Default> Default for GenericBBox<PointType> {
    fn default() -> Self {
        Self {
            max: PointType::default(),
            min: PointType::default(),
        }
    }
}

pub type BBoxZ = GenericBBox<PointZ>;

impl BBoxZ {
    pub(crate) fn grow_from_shape<S: EsriShape>(&mut self, shape: &S) {
        let x_range = shape.x_range();
        let y_range = shape.y_range();
        let z_range = shape.z_range();
        let m_range = shape.m_range();

        self.min.x = f64_min(x_range[0], self.min.x);
        self.max.x = f64_max(x_range[1], self.max.x);
        self.min.y = f64_min(y_range[0], self.min.y);
        self.max.y = f64_max(y_range[1], self.max.y);

        if S::shapetype().has_m() {
            self.min.m = f64_min(m_range[0], self.min.m);
            self.max.m = f64_max(m_range[1], self.max.m);
        }

        if S::shapetype().has_z() {
            self.min.z = f64_min(z_range[0], self.min.z);
            self.max.z = f64_max(z_range[1], self.max.z);
        }
    }
}
