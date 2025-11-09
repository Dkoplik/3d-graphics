use crate::{Plane, Point3, Vec3};

impl Plane {
    pub fn new(origin: Point3, mut normal: Vec3) -> Self {
        normal = normal.normalize(); // на случай если вектор был не нормализован
        Self { origin, normal }
    }
}
