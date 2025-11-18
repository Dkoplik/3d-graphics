//! Реализация структуры `Point3`.

use std::ops::{Add, Mul, MulAssign, Sub};

use crate::{HVec3, Point3, Transform3D, Vec3};

impl Point3 {
    /// Создать новую точку по 3-м координатам.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Создать точку (0.0, 0.0, 0.0).
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Приблизительное сравнение точек на равенство.
    ///
    /// # Arguments
    /// - `other` - другая точка, с которой происходит сравнение;
    /// - `tolerance` - допустимая погрешность. Если разница между координатами >=`tolerance`, то координаты считаются разными.
    pub fn approx_equal(self, other: Self, tolerance: f32) -> bool {
        (self.x - other.x).abs() < tolerance
            && (self.y - other.y).abs() < tolerance
            && (self.z - other.z).abs() < tolerance
    }

    /// Применить преобразование к точке `Point3`. Эта операция **создаёт новую** точку.
    pub fn apply_transform(self, transform: Transform3D) -> Self {
        transform.apply_to_hvec(self.into()).into()
    }
}

impl Mul<Transform3D> for Point3 {
    type Output = Point3;

    /// Применить преобразование `Transform3D` к точке `Point3`.
    fn mul(self, rhs: Transform3D) -> Self::Output {
        self.apply_transform(rhs)
    }
}

impl MulAssign<Transform3D> for Point3 {
    /// Применить преобразование `Transform3D` к точке `Point3`.
    fn mul_assign(&mut self, rhs: Transform3D) {
        *self = *self * rhs;
    }
}

impl Sub for Point3 {
    type Output = Vec3;

    /// Производит операцию `-` между двумя точками, получая вектор.
    ///
    /// # Returns
    /// Возвращает вектор, направленный из вычитаемой (правой) точки в уменьшаемую (левую) точку.
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Add<Vec3> for Point3 {
    type Output = Self;

    /// Выполняет операцию `+` между точкой и вектором.
    ///
    /// По смыслу операция представляет собой смещение текущей точки на заданный вектор.
    fn add(self, rhs: Vec3) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl From<Vec3> for Point3 {
    /// Получить точку из `Vec3`.
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<HVec3> for Point3 {
    /// Получить точку из `HVec3`.
    fn from(value: HVec3) -> Self {
        Self::from(Vec3::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec3;

    const TOLERANCE: f32 = 1e-8;

    fn assert_vectors(got: Vec3, expected: Vec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn assert_points(got: Point3, expected: Point3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидалась точка {:?}, но получена точка {:?}, одна из координат которой отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    #[test]
    fn test_sub() {
        let begin = Point3::new(1.0, 2.0, 3.0);
        let end = Point3::new(5.0, 6.0, 7.0);

        let vec = end - begin;
        let expected = Vec3::new(5.0 - 1.0, 6.0 - 2.0, 7.0 - 3.0);
        assert_vectors(vec, expected, TOLERANCE);
    }

    #[test]
    fn test_add() {
        let point = Point3::new(1.0, 2.0, 3.0);
        let vec = Vec3::new(5.0, 6.0, 7.0);

        let moved_point = point + vec;
        let expected = Point3::new(1.0 + 5.0, 2.0 + 6.0, 3.0 + 7.0);
        assert_points(moved_point, expected, TOLERANCE);
    }
}
