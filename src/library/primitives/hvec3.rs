//! Объявление и реализация структуры `HVec3`.

use super::{Point3, Transform3D, UVec3, Vec3};
use std::{
    fmt::Display,
    ops::{Mul, MulAssign},
};

/// Однородный (homogeneous) вектор в 3D пространстве.
///
/// По сути, этот вектор является 4D вектором с координатами `(x, y, z, w)`, но каждый такой вектор соответсвует
/// либо обычному 3D вектору `(x, y, z)`, если `w = 0`, либо 3D точке `(x / w, y / w, z / w)`. 4-ая координата
/// необходима для проецирования и позволяет использовать матрицы преобразования 4x4 над этим вектором.
/// Поскольку это 4D вектор, отвечающий за 3D пространство, у этого вектора отсутсвуют базовые операции по типу длины,
/// сложения и им подобным, ибо за это отвечает обычный `Vec3` или `Point3`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// ========================================
// Различные конструкторы вектора
// ========================================

impl HVec3 {
    /// Создать вектор по всем 4-м координатам.
    ///
    /// Координаты `x`, `y`, `z` аналогичны 3D пространству, если же `w = 0`, то вектор
    /// задаёт какое-то направление, в противном случае задаёт какое-то положение в 3D пространстве.
    ///
    /// # Examples
    /// ```rust
    /// let hvec = HVec3::new(1.0, 2.0, 3.0, 1.0);
    /// assert_eq!(hvec.x, 1.0);
    /// assert_eq!(hvec.y, 2.0);
    /// assert_eq!(hvec.z, 3.0);
    /// assert_eq!(hvec.w, 1.0);
    /// ```
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Задать положение в 3D пространстве через 4D вектор.
    ///
    /// Координаты `x`, `y`, `z` задаются как есть, а `w = 1.0`.
    ///
    /// # Examples
    /// ```rust
    /// let hvec = HVec3::new_position(1.0, 2.0, 3.0);
    /// assert_eq!(hvec.x, 1.0);
    /// assert_eq!(hvec.y, 2.0);
    /// assert_eq!(hvec.z, 3.0);
    /// assert_eq!(hvec.w, 1.0);
    /// ```
    pub fn new_position(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, w: 1.0 }
    }

    /// Задать направление в 3D пространстве через 4D вектор.
    ///
    /// Координаты `x`, `y`, `z` задаются как есть, а `w = 0.0`.
    ///
    /// # Examples
    /// ```rust
    /// let hvec = HVec3::new_direction(1.0, 2.0, 3.0);
    /// assert_eq!(hvec.x, 1.0);
    /// assert_eq!(hvec.y, 2.0);
    /// assert_eq!(hvec.z, 3.0);
    /// assert_eq!(hvec.w, 0.0);
    /// ```
    pub fn new_direction(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, w: 1.0 }
    }
}

// ========================================
// Операции над 4D вектором
// ========================================

impl HVec3 {
    /// Приблизительное сравнение векторов на равенство.
    ///
    /// # Arguments
    /// - `other` - другой вектор, с которым происходит сравнение;
    /// - `tolerance` - допустимая погрешность. Если разница между координатами >=`tolerance`, то координаты считаются разными.
    pub fn approx_equal(self, other: Self, tolerance: f32) -> bool {
        (self.x - other.x).abs() < tolerance
            && (self.y - other.y).abs() < tolerance
            && (self.z - other.z).abs() < tolerance
    }

    /// Применить преобразование к текущему однородному вектору `HVec3`. Эта операция **создаёт новый** вектор.
    ///
    /// Операция выполняет произведение **строчного** вектора `HVec3` на матрицу преобразования:
    /// ```txt
    ///                 | m11 m12 m13 m14 |
    /// (x, y, z, w) x  | m21 m22 m23 m24 | = (x_new, y_new, z_new, w_new)
    ///                 | m31 m32 m33 m34 |
    ///                 | m41 m42 m43 m44 |
    /// ```
    /// `HVec3` не содержит в себе готовых методов по типу поворота по той причине, что одна и та же операция
    /// преобразования может быть применена сразу к большому количеству векторов, поэтому лучше переиспользовать
    /// одну и ту же матрицу.
    pub fn apply_transform(self, transform: Transform3D) -> Self {
        transform.apply_to_hvec(self)
    }
}

impl Display for HVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HVec3(x: {}, y: {}, z: {}, w: {})",
            self.x, self.y, self.z, self.w
        )
    }
}

impl Mul<Transform3D> for HVec3 {
    type Output = HVec3;

    /// Применить преобразование `Transform3D` к однородному вектору `HVec3`.
    ///
    /// Операция выполняет произведение **строчного** вектора `HVec3` на матрицу преобразования:
    /// ```txt
    ///                 | m11 m12 m13 m14 |
    /// (x, y, z, w) x  | m21 m22 m23 m24 | = (x_new, y_new, z_new, w_new)
    ///                 | m31 m32 m33 m34 |
    ///                 | m41 m42 m43 m44 |
    /// ```
    fn mul(self, rhs: Transform3D) -> Self::Output {
        rhs.apply_to_hvec(self)
    }
}

impl MulAssign<Transform3D> for HVec3 {
    /// Применить преобразование `Transform3D` к однородному вектору `HVec3`.
    ///
    /// Операция выполняет произведение **строчного** вектора `HVec3` на матрицу преобразования:
    /// ```txt
    ///                 | m11 m12 m13 m14 |
    /// (x, y, z, w) x  | m21 m22 m23 m24 | = (x_new, y_new, z_new, w_new)
    ///                 | m31 m32 m33 m34 |
    ///                 | m41 m42 m43 m44 |
    /// ```
    fn mul_assign(&mut self, rhs: Transform3D) {
        *self = rhs.apply_to_hvec(*self);
    }
}

impl From<Vec3> for HVec3 {
    /// Создать `HVec3` из `Vec3`. Однородная компонента `w = 0.0`.
    fn from(value: Vec3) -> Self {
        Self::new_direction(value.x, value.y, value.z)
    }
}

impl From<UVec3> for HVec3 {
    /// Создать `HVec3` из `UVec3`. Однородная компонента `w = 0.0`.
    fn from(value: UVec3) -> Self {
        Self::new_direction(value.x, value.y, value.z)
    }
}

impl From<Point3> for HVec3 {
    /// Создать `HVec3` из `Point3`. Однородная компонента `w = 1.0`.
    fn from(value: Point3) -> Self {
        Self::new_position(value.x, value.y, value.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 1e-8;

    fn assert_hvecs(got: HVec3, expected: HVec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    #[test]
    fn test_apply_identity_transform() {
        let vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        let transform = Transform3D::identity();
        let vec_transformed = vec.apply_transform(transform);

        assert_hvecs(vec_transformed, vec, TOLERANCE);
    }

    #[test]
    fn test_apply_translate_transform() {
        let mut vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        let transform = Transform3D::translation_uniform(1.0);
        let mut vec_transformed = vec.apply_transform(transform);

        let expected = HVec3::new(2.0, 3.0, 4.0, 1.0);
        assert_hvecs(vec_transformed, expected, TOLERANCE);

        // через умножение
        vec_transformed = vec * transform;
        assert_hvecs(vec_transformed, expected, TOLERANCE);

        // через присваивающее умножение
        vec *= transform;
        assert_hvecs(vec, expected, TOLERANCE);
    }
}
