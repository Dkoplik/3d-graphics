use crate::{Plane, Point3, Transformable3, Vec3};

impl Plane {
    pub fn new(origin: Point3, mut normal: Vec3) -> Self {
        normal = normal.normalize(); // на случай если вектор был не нормализован
        Self { origin, normal }
    }
}

impl Transformable3 for Plane {
    fn transform(self, transform: crate::Transform3D) -> Self {
        let origin = self.origin.transform(transform);
        let normal = self.normal.transform(transform);
        Self { origin, normal }
    }

    fn apply_transform(&mut self, transform: crate::Transform3D) {
        self.origin.apply_transform(transform);
        self.normal.apply_transform(transform);
    }
}