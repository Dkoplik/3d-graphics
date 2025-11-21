//! Объявление и реализация структуры `Line3`.

use super::{Point3, Transform3D, UVec3};
use std::ops::{Mul, MulAssign};

/// Линия в 3D пространстве.
///
/// Линия задаётся точкой, через которую проходит, и направлением.
#[derive(Debug, Clone, Copy)]
pub struct Line3 {
    /// Точка, через которую проходит прямая.
    pub origin: Point3,
    /// Направление прямой в виде единичного вектора.
    pub direction: UVec3,
}

impl Line3 {
    /// Создать прямую из начальной точки и направления прямой.
    ///
    /// # Examples
    /// ```rust
    /// let line = Line3::new(Point3::new(1.0, 2.0, 3.0), UVec3::new(1.0, 0.0, 0.0));
    /// assert!(line.origin.approx_equal(Point3::new(1.0, 2.0, 3.0)));
    /// assert!(line.direction.approx_equal(UVec3::new(1.0, 0.0, 0.0)));
    /// ```
    pub fn new(origin: Point3, direction: UVec3) -> Self {
        Self { origin, direction }
    }

    /// Получить прямую из 2-х точек.
    ///
    /// # Examples
    /// ```rust
    /// let p1 = Point3::new(1.0, 2.0, 3.0);
    /// let p2 = Point3::new(2.0, 2.0, 3.0);
    /// let line = Line3::from_points(p1, p2);
    /// assert!(line.origin.approx_equal(Point3::new(1.0, 2.0, 3.0)));
    /// assert!(line.direction.approx_equal(UVec3::new(1.0, 0.0, 0.0)));
    /// ```
    pub fn from_points(p1: Point3, p2: Point3) -> Self {
        debug_assert_ne!(
            p1, p2,
            "Попытка создать линию из равных точек {:?} и {:?}",
            p1, p2
        );

        let direction = (p2 - p1).normalize();
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
