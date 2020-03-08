use record::{Point, PointM, PointZ};
use writer::{f64_max, f64_min};

/// Trait to access the x, and y values of a point
pub trait HasXY {
    /// Returns the value of the x dimension
    fn x(&self) -> f64;
    /// Returns the value of the y dimension
    fn y(&self) -> f64;
}

/// Trait to access the m value of a point
pub trait HasM {
    fn m(&self) -> f64;
}

/// Trait to access the z value of a point
pub trait HasZ {
    fn z(&self) -> f64;
}

pub(crate) trait HasMutXY {
    fn x_mut(&mut self) -> &mut f64;
    fn y_mut(&mut self) -> &mut f64;
}

pub(crate) trait HasMutM {
    fn m_mut(&mut self) -> &mut f64;
}

pub(crate) trait HasMutZ {
    fn z_mut(&mut self) -> &mut f64;
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
