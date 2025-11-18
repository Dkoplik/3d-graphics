//! Реализация структуры `HVec3`.

use std::ops::{Mul, MulAssign};

use crate::{HVec3, Point3, Transform3D, Vec3};

impl HVec3 {
    // ========================================
    // Различные конструкторы вектора
    // ========================================

    /// Создать вектор по 3-м координатам 3D пространства.
    ///
    /// Создаёт однородный вектор по 3-м заданным координатам,
    /// при этом компонента однородности равна `w = 1.0`.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, w: 1.0 }
    }

    /// Создать вектор по всем 4-м координатам.
    ///
    /// Аналогично конструктору `new`, но тут также можно указать
    /// значение для компоненты однородности `w`.
    pub fn new_full(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Получить нулевой вектор в **глобальных** координатах.
    ///
    /// Вектор имеет вид `(0.0, 0.0, 0.0, 1.0)`.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Получить единичный вектор (1.0, 0.0, 0.0, 1.0) в глобальных координатах.
    pub fn plus_x() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    /// Получить единичный вектор (-1.0, 0.0, 0.0, 1.0) в глобальных координатах.
    pub fn minus_x() -> Self {
        Self::new(-1.0, 0.0, 0.0)
    }

    /// Получить единичный вектор (0.0, 1.0, 0.0, 1.0) в глобальных координатах.
    pub fn plus_y() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    /// Получить единичный вектор (0.0, -1.0, 0.0, 1.0) в глобальных координатах.
    pub fn minus_y() -> Self {
        Self::new(0.0, -1.0, 0.0)
    }

    /// Получить единичный вектор (0.0, 0.0, 1.0, 1.0) в глобальных координатах.
    pub fn plus_z() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    /// Получить единичный вектор (0.0, 0.0, -1.0, 1.0) в глобальных координатах.
    pub fn minus_z() -> Self {
        Self::new(0.0, 0.0, -1.0)
    }

    /// Получить единичный вектор с направлением "вверх" в **глобальных** координатах.
    ///
    /// Направлением вверх считается `+y`, поэтому вектор имеет вид `(0.0, 1.0, 0.0, 1.0)`.
    pub fn up() -> Self {
        Self::plus_y()
    }

    /// Получить единичный вектор с направлением "вниз" в **глобальных** координатах.
    ///
    /// Направлением вниз считается `-y`, поэтому вектор имеет вид `(0.0, -1.0, 0.0, 1.0)`.
    pub fn down() -> Self {
        Self::minus_y()
    }

    /// Получить единичный вектор с направлением "влево" в **глобальных** координатах.
    ///
    /// Направлением влево считается `-x`, поэтому вектор имеет вид `(-1.0, 0.0, 0.0, 1.0)`.
    pub fn left() -> Self {
        Self::minus_x()
    }

    /// Получить единичный вектор с направлением "вправо" в **глобальных** координатах.
    ///
    /// Направлением вправо считается `+x`, поэтому вектор имеет вид `(1.0, 0.0, 0.0, 1.0)`.
    pub fn right() -> Self {
        Self::plus_x()
    }

    /// Получить единичный вектор с направлением "прямо" в **глобальных** координатах.
    ///
    /// Направлением прямо считается `+z`, поэтому вектор имеет вид `(0.0, 0.0, 1.0, 1.0)`.
    pub fn forward() -> Self {
        Self::plus_z()
    }

    /// Получить единичный вектор с направлением "назад" в **глобальных** координатах.
    ///
    /// Направлением назад считается `-z`, поэтому вектор имеет вид `(0.0, 0.0, -1.0, 1.0)`.
    pub fn backward() -> Self {
        Self::minus_z()
    }

    // ========================================
    // Операции над 4D вектором
    // ========================================

    /// Приблизительное сравнение векторов на равенство.
    ///
    /// # Arguments
    /// - `other` - другой вектор, с которым происходит сравнение;
    /// - `tolerance` - допустимая погрешность. Если разница между координатами >=`tolerance`, то координаты считаются разными.
    pub fn approx_equal(self, other: Self, tolerance: f32) -> bool {
        //                     ^
        // other лучше копировать, ибо вектор небольшой, а обращений несколько, ссылка будет замедлять
        (self.x - other.x).abs() < tolerance
            && (self.y - other.y).abs() < tolerance
            && (self.z - other.z).abs() < tolerance
    }

    /// Применить преобразование к текущемы однородному вектору `HVec3`. Эта операция **создаёт новый** вектор.
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
    /// Создать `HVec3` из `Vec3`. Однородная компонента `w = 1.0`.
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<Point3> for HVec3 {
    /// Создать `HVec3` из `Point3`. Однородная компонента `w = 1.0`.
    fn from(value: Point3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Transform3D;

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
        let vec = HVec3::new(1.0, 2.0, 3.0);

        let transform = Transform3D::identity();
        let vec_transformed = vec.apply_transform(transform);

        assert_hvecs(vec_transformed, vec, TOLERANCE);
    }

    #[test]
    fn test_apply_translate_transform() {
        let mut vec = HVec3::new(1.0, 2.0, 3.0);

        let transform = Transform3D::translation_uniform(1.0);
        let mut vec_transformed = vec.apply_transform(transform);

        let expected = HVec3::new(2.0, 3.0, 4.0);
        assert_hvecs(vec_transformed, expected, TOLERANCE);

        // через умножение
        vec_transformed = vec * transform;
        assert_hvecs(vec_transformed, expected, TOLERANCE);

        // через присваивающее умножение
        vec *= transform;
        assert_hvecs(vec, expected, TOLERANCE);
    }
}
