//! Реализация структуры `Vec3`.

//! Реализация структуры `Vec3`.

use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::{HVec3, Point3, Vec3};

impl Vec3 {
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
    /// Координатная система правкорукая (right-handed), направлением вверх считается `+z`, как в `Blender`,
    /// поэтому вектор имеет вид `(0.0, 0.0, 1.0)`.
    pub fn up() -> Self {
        Self::plus_z()
    }

    /// Получить единичный вектор с направлением "вниз".
    ///
    /// Координатная система правкорукая (right-handed), направлением вниз считается `-z`, как в `Blender`,
    /// поэтому вектор имеет вид `(0.0, 0.0, -1.0)`.
    pub fn down() -> Self {
        Self::minus_z()
    }

    /// Получить единичный вектор с направлением "влево".
    ///
    /// Координатная система правкорукая (right-handed), направлением влево считается `+y`, как в `Blender`,
    /// поэтому вектор имеет вид `(0.0, 1.0, 0.0)`.
    pub fn left() -> Self {
        Self::plus_y()
    }

    /// Получить единичный вектор с направлением "вправо".
    ///
    /// Координатная система правкорукая (right-handed), направлением вправо считается `-y`, как в `Blender`,
    /// поэтому вектор имеет вид `(0.0, -1.0, 0.0)`.
    pub fn right() -> Self {
        Self::minus_y()
    }

    /// Получить единичный вектор с направлением "прямо".
    ///
    /// Координатная система правкорукая (right-handed), направлением прямо считается `+x`, как в `Blender`,
    /// поэтому вектор имеет вид `(1.0, 0.0, 0.0)`.
    pub fn forward() -> Self {
        Self::plus_x()
    }

    /// Получить единичный вектор с направлением "назад".
    ///
    /// Координатная система правкорукая (right-handed), направлением назад считается `-x`, как в `Blender`,
    /// поэтому вектор имеет вид `(-1.0, 0.0, 0.0)`.
    pub fn backward() -> Self {
        Self::minus_x()
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

    /// Векторное произведение векторов.
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Получить длину вектора.
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Привести вектор к единичной длине.
    pub fn normalize(self) -> Self {
        let len = self.length();
        Self {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
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
    /// Создаёт из вектора `a` отрицательный вектор `-a`.
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    /// Находит сумму между двумя векторами по правилу параллелограмма.
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

impl From<Point3> for Vec3 {
    /// Получить вектор из `Point3`.
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
        Self {
            x: value.x / value.w,
            y: value.y / value.w,
            z: value.z / value.w,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        // (1*4) + (2*5) + (3*6) = 4 + 10 + 18 = 32
        assert_eq!(v1.dot(v2), 32.0);

        // Тест с перпендикулярными векторами (должен быть 0)
        let v3 = Vec3::new(1.0, 0.0, 0.0);
        let v4 = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(v3.dot(v4), 0.0);

        // Тест с отрицательными координатами
        let v5 = Vec3::new(-1.0, 2.0, -3.0);
        let v6 = Vec3::new(2.0, -1.0, 1.0);
        assert_eq!(v5.dot(v6), -2.0 - 2.0 - 3.0); // -1*2 + 2*(-1) + (-3)*1
    }

    #[test]
    fn test_cross_product() {
        // Базовые векторы i, j, k
        let i = Vec3::new(1.0, 0.0, 0.0);
        let j = Vec3::new(0.0, 1.0, 0.0);
        let k = Vec3::new(0.0, 0.0, 1.0);

        // i × j = k
        assert!(i.cross(j).approx_equal(&k, 1e-6));

        // j × k = i
        assert!(j.cross(k).approx_equal(&i, 1e-6));

        // k × i = j
        assert!(k.cross(i).approx_equal(&j, 1e-6));

        // Антикоммутативность: j × i = -k
        assert!(j.cross(i).approx_equal(&(-k), 1e-6));

        // Тест с произвольными векторами
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let result = v1.cross(v2);
        let expected = Vec3::new(-3.0, 6.0, -3.0); // (2*6 - 3*5, 3*4 - 1*6, 1*5 - 2*4)
        assert!(result.approx_equal(&expected, 1e-6));
    }

    #[test]
    fn test_length() {
        // Нулевой вектор
        assert_eq!(Vec3::zero().length(), 0.0);

        // Единичный вектор
        let unit = Vec3::new(1.0, 0.0, 0.0);
        assert_eq!(unit.length(), 1.0);

        // Вектор с длиной 5 (3-4-5 треугольник)
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length(), 5.0);

        // Вектор с отрицательными координатами
        let v_neg = Vec3::new(-3.0, -4.0, 0.0);
        assert_eq!(v_neg.length(), 5.0);

        // 3D вектор
        let v3d = Vec3::new(1.0, 2.0, 2.0);
        assert_eq!(v3d.length(), 3.0); // sqrt(1 + 4 + 4) = 3
    }

    #[test]
    fn test_normalize() {
        // Нормализация единичного вектора
        let unit = Vec3::new(1.0, 0.0, 0.0);
        let normalized_unit = unit.normalize();
        assert!(normalized_unit.approx_equal(&unit, 1e-6));
        assert_eq!(normalized_unit.length(), 1.0);

        // Нормализация ненулевого вектора
        let v = Vec3::new(3.0, 4.0, 0.0);
        let normalized_v = v.normalize();
        let expected = Vec3::new(0.6, 0.8, 0.0); // (3/5, 4/5, 0)
        assert!(normalized_v.approx_equal(&expected, 1e-6));
        assert!((normalized_v.length() - 1.0).abs() < 1e-6);

        // Нормализация вектора с отрицательными координатами
        let v_neg = Vec3::new(-3.0, -4.0, 0.0);
        let normalized_neg = v_neg.normalize();
        let expected_neg = Vec3::new(-0.6, -0.8, 0.0);
        assert!(normalized_neg.approx_equal(&expected_neg, 1e-6));

        // Нормализация нулевого вектора (должен остаться нулевым)
        let zero = Vec3::zero();
        let normalized_zero = zero.normalize();
        assert!(normalized_zero.approx_equal(&zero, 1e-6));
    }

    #[test]
    fn test_vector_addition() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        let result = v1 + v2;
        let expected = Vec3::new(5.0, 7.0, 9.0);
        assert!(result.approx_equal(&expected, 1e-6));

        // С отрицательными числами
        let v3 = Vec3::new(-1.0, -2.0, -3.0);
        let v4 = Vec3::new(2.0, 3.0, 4.0);

        let result2 = v3 + v4;
        let expected2 = Vec3::new(1.0, 1.0, 1.0);
        assert!(result2.approx_equal(&expected2, 1e-6));

        // AddAssign
        let mut v5 = Vec3::new(1.0, 1.0, 1.0);
        v5 += Vec3::new(2.0, 3.0, 4.0);
        assert!(v5.approx_equal(&Vec3::new(3.0, 4.0, 5.0), 1e-6));
    }

    #[test]
    fn test_vector_subtraction() {
        let v1 = Vec3::new(5.0, 7.0, 9.0);
        let v2 = Vec3::new(1.0, 2.0, 3.0);

        let result = v1 - v2;
        let expected = Vec3::new(4.0, 5.0, 6.0);
        assert!(result.approx_equal(&expected, 1e-6));

        // С отрицательными числами
        let v3 = Vec3::new(1.0, 1.0, 1.0);
        let v4 = Vec3::new(2.0, 3.0, 4.0);

        let result2 = v3 - v4;
        let expected2 = Vec3::new(-1.0, -2.0, -3.0);
        assert!(result2.approx_equal(&expected2, 1e-6));

        // SubAssign
        let mut v5 = Vec3::new(5.0, 5.0, 5.0);
        v5 -= Vec3::new(2.0, 3.0, 4.0);
        assert!(v5.approx_equal(&Vec3::new(3.0, 2.0, 1.0), 1e-6));
    }

    #[test]
    fn test_scalar_multiplication() {
        let v = Vec3::new(1.0, 2.0, 3.0);

        // Умножение на положительный скаляр
        let result = v * 2.0;
        let expected = Vec3::new(2.0, 4.0, 6.0);
        assert!(result.approx_equal(&expected, 1e-6));

        // Умножение на отрицательный скаляр
        let result2 = v * (-1.5);
        let expected2 = Vec3::new(-1.5, -3.0, -4.5);
        assert!(result2.approx_equal(&expected2, 1e-6));

        // Умножение на ноль
        let result3 = v * 0.0;
        assert!(result3.approx_equal(&Vec3::zero(), 1e-6));
    }

    #[test]
    fn test_edge_cases() {
        // Очень маленькие векторы
        let small = Vec3::new(1e-10, 2e-10, 3e-10);
        assert!(small.length() > 0.0);

        // Очень большие векторы
        let large = Vec3::new(1e10, 2e10, 3e10);
        assert!(large.length() > 0.0);
    }
}
