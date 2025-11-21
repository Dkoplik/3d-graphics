//! Объявление и реализация структуры `Plane`.

use super::{Point3, Transform3D, UVec3};
use std::ops::{Mul, MulAssign};

/// Плоскость в 3D пространстве.
///
/// Плоскость задаётся точкой, через которую проходит, и нормалью к этой точке.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    /// Точка, через которую проходит плоскость.
    pub origin: Point3,
    /// Нормаль плоскости в виде единичного вектора.
    pub normal: UVec3,
}

impl Plane {
    /// Создать плоскость из начальной точки и нормали к плоскости.
    ///
    /// # Examples
    /// ```rust
    /// let plane = Plane::new(Point3::new(1.0, 2.0, 3.0), UVec3::new(1.0, 0.0, 0.0));
    /// assert!(plane.origin.approx_equal(Point3::new(1.0, 2.0, 3.0)));
    /// assert!(plane.normal.approx_equal(UVec3::new(1.0, 0.0, 0.0)));
    /// ```
    pub fn new(origin: Point3, normal: UVec3) -> Self {
        Self { origin, normal }
    }

    /// Применить преобразование к текущей плоскости `Plane`. Эта операция **создаёт новую** плоскость.
    pub fn apply_transform(self, transform: Transform3D) -> Self {
        Self::new(
            transform.apply_to_hvec(self.origin.into()).into(),
            transform.inverse().apply_to_hvec(self.normal.into()).into(),
        )
    }
}

impl Mul<Transform3D> for Plane {
    type Output = Plane;

    /// Применить преобразование `Transform3D` к `Plane`.
    fn mul(self, rhs: Transform3D) -> Self::Output {
        self.apply_transform(rhs)
    }
}

impl MulAssign<Transform3D> for Plane {
    /// Применить преобразование `Transform3D` к  `Plane`.
    fn mul_assign(&mut self, rhs: Transform3D) {
        *self = *self * rhs;
    }
}
