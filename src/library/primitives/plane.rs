use std::ops::{Mul, MulAssign};

// используем все примитивы
use crate::library::primitives::*;

/// Плоскость в 3D пространстве.
///
/// Плоскость задаётся точкой, через которую проходит, и нормалью к этой точке.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    /// Точка, через которую проходит плоскость.
    pub origin: Point3,
    /// Нормаль плоскости в виде единичного вектора.
    pub normal: Vec3,
}

impl Plane {
    pub fn new(origin: Point3, normal: Vec3) -> Self {
        debug_assert!(
            normal.is_normalized(),
            "нормаль должа иметь длину 1.0, но она длины {}",
            normal.length()
        );

        Self { origin, normal }
    }

    /// Применить преобразование к текущей плоскости `Plane`. Эта операция **создаёт новую** плоскость.
    pub fn apply_transform(self, transform: Transform3D) -> Self {
        Self::new(
            transform.apply_to_hvec(self.origin.into()).into(),
            transform.apply_to_hvec(self.normal.into()).into(),
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
