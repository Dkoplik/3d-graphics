use crate::{Line3, Point3, Vec3};

impl Line3 {
    pub fn new(origin: Point3, mut direction: Vec3) -> Self {
        direction = direction.normalize();
        Self { origin, direction }
    }

    /// Получить прямую из 2-х точек.
    pub fn from_points(p1: Point3, p2: Point3) -> Self {
        let direction = p2 - p1;
        Self::new(p1, direction)
    }
}
