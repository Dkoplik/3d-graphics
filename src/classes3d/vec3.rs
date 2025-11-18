//! Реализация структуры `Vec3`.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::{HVec3, Point3, Vec3};

impl Vec3 {
    // ========================================
    // Различные конструкторы вектора
    // ========================================

    /// Создать вектор по 3-м координатам.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Получить нулевой вектор (0.0, 0.0, 0.0).
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Получить единичный вектор (1.0, 0.0, 0.0) в глобальных координатах.
    pub fn plus_x() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    /// Получить единичный вектор (-1.0, 0.0, 0.0) в глобальных координатах.
    pub fn minus_x() -> Self {
        Self::new(-1.0, 0.0, 0.0)
    }

    /// Получить единичный вектор (0.0, 1.0, 0.0) в глобальных координатах.
    pub fn plus_y() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    /// Получить единичный вектор (0.0, -1.0, 0.0) в глобальных координатах.
    pub fn minus_y() -> Self {
        Self::new(0.0, -1.0, 0.0)
    }

    /// Получить единичный вектор (0.0, 0.0, 1.0) в глобальных координатах.
    pub fn plus_z() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    /// Получить единичный вектор (0.0, 0.0, -1.0) в глобальных координатах.
    pub fn minus_z() -> Self {
        Self::new(0.0, 0.0, -1.0)
    }

    /// Получить единичный вектор с направлением "вверх".
    ///
    /// Направлением вверх считается `+y`, поэтому вектор имеет вид `(0.0, 1.0, 0.0)`.
    pub fn up() -> Self {
        Self::plus_y()
    }

    /// Получить единичный вектор с направлением "вниз".
    ///
    /// Направлением вниз считается `-y`, поэтому вектор имеет вид `(0.0, -1.0, 0.0)`.
    pub fn down() -> Self {
        Self::minus_y()
    }

    /// Получить единичный вектор с направлением "влево".
    ///
    /// Направлением влево считается `-x`, поэтому вектор имеет вид `(-1.0, 0.0, 0.0)`.
    pub fn left() -> Self {
        Self::minus_x()
    }

    /// Получить единичный вектор с направлением "вправо".
    ///
    /// Направлением вправо считается `+x`, поэтому вектор имеет вид `(1.0, 0.0, 0.0)`.
    pub fn right() -> Self {
        Self::plus_x()
    }

    /// Получить единичный вектор с направлением "прямо".
    ///
    /// Направлением прямо считается `+z`, поэтому вектор имеет вид `(0.0, 0.0, 1.0)`.
    pub fn forward() -> Self {
        Self::plus_z()
    }

    /// Получить единичный вектор с направлением "назад".
    ///
    /// Направлением назад считается `-z`, поэтому вектор имеет вид `(0.0, 0.0, -1.0)`.
    pub fn backward() -> Self {
        Self::minus_z()
    }

    /// Получить проекцию вектора `from` на плоскость XY в **глобальных** координатах.
    pub fn projection_xy(from: Self) -> Self {
        Self::new(from.x, from.y, 0.0)
    }

    /// Получить проекцию вектора `from` на плоскость XZ в **глобальных** координатах.
    pub fn projection_xz(from: Self) -> Self {
        Self::new(from.x, 0.0, from.z)
    }

    /// Получить проекцию вектора `from` на плоскость YZ в **глобальных** координатах.
    pub fn projection_yz(from: Self) -> Self {
        Self::new(0.0, from.y, from.z)
    }

    // ========================================
    // Операции над 3D вектором
    // ========================================

    /// Скалярное произведение векторов.
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Возвращает косинус угла между 2-мя векторами.
    pub fn cos(self, other: Self) -> f32 {
        self.dot(other) / (self.length() * other.length())
    }

    /// Возвращает угл в радианах между 2-мя векторами.
    pub fn angle_rad(self, other: Self) -> f32 {
        self.cos(other).acos()
    }

    /// Возвращает угл в градусах между 2-мя векторами.
    pub fn angle_deg(self, other: Self) -> f32 {
        self.cos(other).acos().to_degrees()
    }

    /// Векторное произведение векторов для **правой** координатной системы.
    pub fn cross_right(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn cross_left(self, other: Self) -> Self {
        -self.cross_right(other)
    }

    /// Получить квадрат длины вектора.
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Получить длину вектора.
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Привести вектор к единичной длине.
    pub fn normalize(self) -> Self {
        let len = self.length();
        debug_assert_ne!(len, 0.0, "попытка нормализовать нулевой вектор");
        self / len
    }

    /// Является ли вектор нормализованным.
    pub fn is_normalized(&self) -> bool {
        self.length_squared() - 1.0 <= 2.0 * f32::EPSILON
    }

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
}

impl Neg for Vec3 {
    type Output = Self;

    /// Создаёт из вектора `a` отрицательный вектор `-a`.
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    /// Находит сумму между двумя векторами по правилу параллелограмма.
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    /// Находит разность между векторами по правилу параллелограмма.
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    /// Умножение вектора на скаляр.
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl From<Point3> for Vec3 {
    /// Получить вектор из `Point3`.
    fn from(value: Point3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<HVec3> for Vec3 {
    /// Получить вектор из `HVec3`.
    ///
    /// 4D вектор `(x, y, z, w)` становится 3D вектором `(x/w, y/w, z/w)`.
    fn from(value: HVec3) -> Self {
        if value.w == 0.0 {
            Self {
                x: f32::INFINITY,
                y: f32::INFINITY,
                z: f32::INFINITY,
            }
        } else {
            Self {
                x: value.x / value.w,
                y: value.y / value.w,
                z: value.z / value.w,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn assert_floats(got: f32, expected: f32, tolerance: f32) {
        assert!(
            (got - expected).abs() <= tolerance,
            "ожидалось число {}, но получено {}, которое отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    #[test]
    fn test_dot_product() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        // (1*4) + (2*5) + (3*6) = 4 + 10 + 18 = 32
        assert_floats(v1.dot(v2), 32.0, TOLERANCE);

        // Тест с перпендикулярными векторами (должен быть 0)
        let v3 = Vec3::new(1.0, 0.0, 0.0);
        let v4 = Vec3::new(0.0, 1.0, 0.0);
        assert_floats(v3.dot(v4), 0.0, TOLERANCE);

        // Тест с отрицательными координатами
        let v5 = Vec3::new(-1.0, 2.0, -3.0);
        let v6 = Vec3::new(2.0, -1.0, 1.0);
        assert_floats(v5.dot(v6), -2.0 - 2.0 - 3.0, TOLERANCE); // -1*2 + 2*(-1) + (-3)*1
    }

    #[test]
    fn test_cross_product() {
        // Базовые векторы i, j, k
        let i = Vec3::new(1.0, 0.0, 0.0);
        let j = Vec3::new(0.0, 1.0, 0.0);
        let k = Vec3::new(0.0, 0.0, 1.0);

        // i × j = k
        assert_vectors(i.cross_right(j), k, TOLERANCE);
        assert_vectors(i.cross_left(j), -k, TOLERANCE);

        // j × k = i
        assert_vectors(j.cross_right(k), i, TOLERANCE);
        assert_vectors(j.cross_left(k), -i, TOLERANCE);

        // k × i = j
        assert_vectors(k.cross_right(i), j, TOLERANCE);
        assert_vectors(k.cross_left(i), -j, TOLERANCE);

        // Антикоммутативность: j × i = -k
        assert_vectors(j.cross_right(i), -k, TOLERANCE);
        assert_vectors(j.cross_left(i), k, TOLERANCE);

        // Тест с произвольными векторами
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let result = v1.cross_right(v2);
        let expected = Vec3::new(-3.0, 6.0, -3.0); // (2*6 - 3*5, 3*4 - 1*6, 1*5 - 2*4)
        assert_vectors(result, expected, TOLERANCE);
    }

    #[test]
    fn test_length() {
        // Нулевой вектор
        assert_floats(Vec3::zero().length(), 0.0, TOLERANCE);

        // Единичный вектор
        let unit = Vec3::new(1.0, 0.0, 0.0);
        assert_floats(unit.length(), 1.0, TOLERANCE);

        // Вектор с длиной 5 (3-4-5 треугольник)
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_floats(v.length(), 5.0, TOLERANCE);

        // Вектор с отрицательными координатами
        let v_neg = Vec3::new(-3.0, -4.0, 0.0);
        assert_floats(v_neg.length(), 5.0, TOLERANCE);

        // 3D вектор
        let v3d = Vec3::new(1.0, 2.0, 2.0);
        assert_floats(v3d.length(), 3.0, TOLERANCE); // sqrt(1 + 4 + 4) = 3
    }

    #[test]
    fn test_normalize() {
        // Нормализация единичного вектора
        let unit = Vec3::new(1.0, 0.0, 0.0);
        let normalized_unit = unit.normalize();
        assert_vectors(normalized_unit, unit, 1e-6);
        assert_floats(normalized_unit.length(), 1.0, 1e-6);

        // Нормализация ненулевого вектора
        let v = Vec3::new(3.0, 4.0, 0.0);
        let normalized_v = v.normalize();
        let expected = Vec3::new(0.6, 0.8, 0.0); // (3/5, 4/5, 0)
        assert_vectors(normalized_v, expected, 1e-6);
        assert_floats(normalized_v.length(), 1.0, 1e-6);

        // Нормализация вектора с отрицательными координатами
        let v_neg = Vec3::new(-3.0, -4.0, 0.0);
        let normalized_neg = v_neg.normalize();
        let expected_neg = Vec3::new(-0.6, -0.8, 0.0);
        assert_vectors(normalized_neg, expected_neg, 1e-6);
    }

    #[test]
    fn test_vector_addition() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        let result = v1 + v2;
        let expected = Vec3::new(5.0, 7.0, 9.0);
        assert_vectors(result, expected, 1e-6);

        // С отрицательными числами
        let v3 = Vec3::new(-1.0, -2.0, -3.0);
        let v4 = Vec3::new(2.0, 3.0, 4.0);

        let result2 = v3 + v4;
        let expected2 = Vec3::new(1.0, 1.0, 1.0);
        assert_vectors(result2, expected2, 1e-6);

        // AddAssign
        let mut v5 = Vec3::new(1.0, 1.0, 1.0);
        v5 += Vec3::new(2.0, 3.0, 4.0);
        assert_vectors(v5, Vec3::new(3.0, 4.0, 5.0), 1e-6);
    }

    #[test]
    fn test_vector_subtraction() {
        let v1 = Vec3::new(5.0, 7.0, 9.0);
        let v2 = Vec3::new(1.0, 2.0, 3.0);

        let result = v1 - v2;
        let expected = Vec3::new(4.0, 5.0, 6.0);
        assert_vectors(result, expected, 1e-6);

        // С отрицательными числами
        let v3 = Vec3::new(1.0, 1.0, 1.0);
        let v4 = Vec3::new(2.0, 3.0, 4.0);

        let result2 = v3 - v4;
        let expected2 = Vec3::new(-1.0, -2.0, -3.0);
        assert_vectors(result2, expected2, 1e-6);

        // SubAssign
        let mut v5 = Vec3::new(5.0, 5.0, 5.0);
        v5 -= Vec3::new(2.0, 3.0, 4.0);
        assert_vectors(v5, Vec3::new(3.0, 2.0, 1.0), 1e-6);
    }

    #[test]
    fn test_scalar_multiplication() {
        let v = Vec3::new(1.0, 2.0, 3.0);

        // Умножение на положительный скаляр
        let result = v * 2.0;
        let expected = Vec3::new(2.0, 4.0, 6.0);
        assert_vectors(result, expected, 1e-6);

        // Умножение на отрицательный скаляр
        let result2 = v * (-1.5);
        let expected2 = Vec3::new(-1.5, -3.0, -4.5);
        assert_vectors(result2, expected2, 1e-6);

        // Умножение на ноль
        let result3 = v * 0.0;
        assert_vectors(result3, Vec3::zero(), 1e-6);
    }

    #[test]
    fn test_edge_cases() {
        // Очень маленькие векторы
        let small = Vec3::new(1e-10, 2e-10, 3e-10);
        assert_floats(small.length(), 0.0, 1e-9);

        // Очень большие векторы
        let large = Vec3::new(1e10, 2e10, 3e10);
        let expected_length = (1e20f32 + 4e20 + 9e20).sqrt();
        assert_floats(large.length(), expected_length, 1e-5);
    }
}
