//! Объявление и реализация структуры `Plane`.

use super::{Point3, UVec3};

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
}
