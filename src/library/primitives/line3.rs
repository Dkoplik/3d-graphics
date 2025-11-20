use std::ops::{Mul, MulAssign};

// используем все примитивы
use crate::library::primitives::*;

/// Линия в 3D пространстве.
///
/// Линия задаётся точкой, через которую проходит, и направлением.
#[derive(Debug, Clone, Copy)]
pub struct Line3 {
    /// Точка, через которую проходит прямая.
    pub origin: Point3,
    /// Направление прямой в виде единичного вектора.
    pub direction: Vec3,
}

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

    /// Применить преобразование к текущей линии `Line3`. Эта операция **создаёт новую** линию.
    pub fn apply_transform(self, transform: Transform3D) -> Self {
        Self::new(
            transform.apply_to_hvec(self.origin.into()).into(),
            transform.apply_to_hvec(self.direction.into()).into(),
        )
    }
}

impl Mul<Transform3D> for Line3 {
    type Output = Line3;

    /// Применить преобразование `Transform3D` к `Line3`.
    fn mul(self, rhs: Transform3D) -> Self::Output {
        self.apply_transform(rhs)
    }
}

impl MulAssign<Transform3D> for Line3 {
    /// Применить преобразование `Transform3D` к `Line3`.
    fn mul_assign(&mut self, rhs: Transform3D) {
        *self = *self * rhs;
    }
}
