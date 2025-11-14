use crate::{Plane, Point3, Vec3};

impl Plane {
    pub fn new(origin: Point3, normal: Vec3) -> Self {
        debug_assert!(
            normal.is_normalized(),
            "нормаль должа иметь длину 1.0, но она длины {}",
            normal.length()
        );

        Self { origin, normal }
    }
}
