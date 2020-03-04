use std::slice::SliceIndex;

use record::{Point, PointM, PointZ};
use writer::{f64_max, f64_min};

/// Trait to acces the x, and y values of a point
pub trait HasXY {
    /// Returns the value of the x dimension
    fn x(&self) -> f64;
    /// Returns the value of the y dimension
    fn y(&self) -> f64;
}

pub(crate) trait HasMutXY {
    fn x_mut(&mut self) -> &mut f64;
    fn y_mut(&mut self) -> &mut f64;
}

pub trait HasM {
    fn m(&self) -> f64;
}

pub(crate) trait HasMutM {
    fn m_mut(&mut self) -> &mut f64;
}

pub trait HasZ {
    fn z(&self) -> f64;
}

pub(crate) trait HasMutZ {
    fn z_mut(&mut self) -> &mut f64;
}

/// Trait that allows access to the slice of points of shapes that
/// have multiple points.
///
/// For convenience, even `Point`, `PointM`, `PointZ` implements this trait
/// meaning that all shapes are MultipointShape
pub trait MultipointShape<PointType> {
    //TODO Is this method useful since there is a .points() method
    //     that means users can do .points()[10] or .points()[..15]
    fn point<I: SliceIndex<[PointType]>>(
        &self,
        index: I,
    ) -> Option<&<I as SliceIndex<[PointType]>>::Output>;

    /// Returns a non mutable slice to the points
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::record::{MultipointShape, Polyline};
    /// let file_path = "tests/data/line.shp";
    /// let polylines = shapefile::read_as::<&str, Polyline>(file_path).unwrap();
    /// let first = &polylines[0];
    /// for point in first.points() {
    ///     println!("{}, {}", point.x, point.y);
    /// }
    /// ```
    ///
    /// ```
    /// use shapefile::record::{MultipointShape, PolylineZ};
    /// let file_path = "tests/data/linez.shp";
    /// let polylines = shapefile::read_as::<&str, PolylineZ>(file_path).unwrap();
    /// let first = &polylines[0];
    /// for point in first.points() {
    ///     println!("{} {} {}", point.x, point.y, point.z);
    /// }
    /// ```
    fn points(&self) -> &[PointType];
}

/// Trait for the Shapes that may have multiple parts
pub trait MultipartShape<PointType>: MultipointShape<PointType> {
    /// Returns a non mutable slice of the parts as written in the file:
    ///
    /// > An array of length NumParts. Stores, for each PolyLine, the index of its
    /// > first point in the points array. Array indexes are with respect to 0
    ///
    ///  # Examples
    ///
    /// ```
    /// use shapefile::record::MultipartShape;
    /// let filepath = "tests/data/linez.shp";
    /// let polylines_z = shapefile::read_as::<&str, shapefile::PolylineZ>(filepath).unwrap();
    ///
    /// let poly_z = &polylines_z[0];
    /// assert_eq!(poly_z.parts_indices(), &[0, 5, 7]);
    /// ```
    fn parts_indices(&self) -> &[i32];

    /// Returns the slice of points corresponding to part n°`ìndex` if the shape
    /// actually has multiple parts
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::record::MultipartShape;
    /// let filepath = "tests/data/linez.shp";
    /// let polylines_z = shapefile::read_as::<&str, shapefile::PolylineZ>(filepath).unwrap();
    ///
    /// let poly_z = &polylines_z[0];
    /// for points in poly_z.parts() {
    ///     println!("{} points", points.len());
    /// }
    /// ```
    fn part(&self, index: usize) -> Option<&[PointType]> {
        let parts = self.parts_indices();
        if parts.len() < 2 {
            Some(self.points())
        } else {
            let first_index = *parts.get(index)? as usize;
            let last_index = if index == parts.len() - 1 {
                self.points().len()
            } else {
                *parts.get(index + 1)? as usize
            };
            self.points().get(first_index..last_index)
        }
    }

    /// Returns an iterator over the parts of a MultipartShape
    ///
    /// # Examples
    ///
    /// ```
    /// use shapefile::record::MultipartShape;
    /// let filepath = "tests/data/linez.shp";
    /// let polylines_z = shapefile::read_as::<&str, shapefile::PolylineZ>(filepath).unwrap();
    ///
    /// let poly_z = &polylines_z[0];
    /// let poly_z_parts: Vec<&[shapefile::PointZ]> = poly_z.parts().collect();
    /// assert_eq!(poly_z_parts.len(), 3);
    /// ```
    fn parts(&self) -> PartIterator<PointType, Self> {
        PartIterator {
            phantom: std::marker::PhantomData,
            shape: &self,
            current_part: 0,
        }
    }
}

/// Iterator over the parts of a Multipart shape
///
/// Each iteration yields a non-mutable slice of points of the current part
pub struct PartIterator<'a, PointType, Shape: 'a + MultipartShape<PointType> + ?Sized> {
    phantom: std::marker::PhantomData<PointType>,
    shape: &'a Shape,
    current_part: usize,
}

impl<'a, PointType, Shape> Iterator for PartIterator<'a, PointType, Shape>
where
    PointType: 'a,
    Shape: 'a + MultipartShape<PointType>,
{
    type Item = &'a [PointType];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_part >= self.shape.parts_indices().len() {
            None
        } else {
            self.current_part += 1;
            self.shape.part(self.current_part - 1)
        }
    }
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

macro_rules! impl_has_mut_xy_for {
    ($PointType:ty) => {
        impl HasMutXY for $PointType {
            fn x_mut(&mut self) -> &mut f64 {
                &mut self.x
            }
            fn y_mut(&mut self) -> &mut f64 {
                &mut self.y
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
        }

        impl HasMutM for $PointType {
            fn m_mut(&mut self) -> &mut f64 {
                &mut self.m
            }
        }
    };
}

impl_has_xy_for!(Point);
impl_has_xy_for!(PointM);
impl_has_xy_for!(PointZ);

impl_has_mut_xy_for!(Point);
impl_has_mut_xy_for!(PointM);
impl_has_mut_xy_for!(PointZ);

impl_has_m_for!(PointM);
impl_has_m_for!(PointZ);

impl HasZ for PointZ {
    fn z(&self) -> f64 {
        self.z
    }
}

impl HasMutZ for PointZ {
    fn z_mut(&mut self) -> &mut f64 {
        &mut self.z
    }
}

pub trait ShrinkablePoint {
    fn shrink(&mut self, other: &Self);
}

pub trait GrowablePoint {
    fn grow(&mut self, other: &Self);
}

impl ShrinkablePoint for Point {
    fn shrink(&mut self, other: &Self) {
        self.x = f64_min(self.x, other.x);
        self.y = f64_min(self.y, other.y);
    }
}

impl ShrinkablePoint for PointM {
    fn shrink(&mut self, other: &Self) {
        self.x = f64_min(self.x, other.x);
        self.y = f64_min(self.y, other.y);
        self.m = f64_min(self.m, other.m);
    }
}

impl ShrinkablePoint for PointZ {
    fn shrink(&mut self, other: &Self) {
        self.x = f64_min(self.x, other.x);
        self.y = f64_min(self.y, other.y);
        self.z = f64_min(self.z, other.z);
        self.m = f64_min(self.m, other.m);
    }
}

impl GrowablePoint for Point {
    fn grow(&mut self, other: &Self) {
        self.x = f64_max(self.x, other.x);
        self.y = f64_max(self.y, other.y);
    }
}

impl GrowablePoint for PointM {
    fn grow(&mut self, other: &Self) {
        self.x = f64_max(self.x, other.x);
        self.y = f64_max(self.y, other.y);
        self.m = f64_max(self.m, other.m);
    }
}

impl GrowablePoint for PointZ {
    fn grow(&mut self, other: &Self) {
        self.x = f64_max(self.x, other.x);
        self.y = f64_max(self.y, other.y);
        self.z = f64_max(self.z, other.z);
        self.m = f64_max(self.m, other.m);
    }
}
