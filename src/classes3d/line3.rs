use crate::{Line3, Point3, Vec3};

impl Line3 {
    pub fn new(origin: Point3, mut direction: Vec3) -> Self {
        debug_assert_ne!(
            direction,
            Vec3::zero(),
            "Попытка создать линию из нулевого вектора {:?}",
            direction
        );

        direction = direction.normalize();
        Self { origin, direction }
    }

    /// Получить прямую из 2-х точек.
    pub fn from_points(p1: Point3, p2: Point3) -> Self {
        debug_assert_ne!(
            p1, p2,
            "Попытка создать линию из равных точек {:?} и {:?}",
            p1, p2
        );

        let direction = p2 - p1;
        Self::new(p1, direction)
    }
}
