use crate::{Line3, Point3, Transformable3, Vec3};

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

impl Transformable3 for Line3 {
    fn transform(self, transform: crate::Transform3D) -> Self {
        let origin = self.origin.transform(transform);
        let direction = self.direction.transform(transform);
        Self { origin, direction }
    }

    fn apply_transform(&mut self, transform: crate::Transform3D) {
        self.origin.apply_transform(transform);
        self.direction.apply_transform(transform);
    }
}