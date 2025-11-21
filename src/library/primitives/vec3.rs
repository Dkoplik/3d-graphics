//! Объявление и реализация структуры `Vec3`.

use super::{HVec3, Point3, Transform3D, UVec3};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// Направление в 3D пространстве с координатами `x`, `y`, `z`.
///
/// Эту структуру надо использовать, если необходимо обозначить какое-то **направление** в
/// 3D пространстве. Для положения надо использовать `Point3`. О координатной системе
/// подробнее можно узнать в `CoordFrame`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// ========================================
// Различные конструкторы вектора
// ========================================

impl Vec3 {
    /// Создать вектор по 3-м координатам.
    ///
    /// # Examples
    /// ```rust
    /// let vec = Vec3::new(1.0, 2.0, 3.0);
    /// assert_eq!(vec.x, 1.0);
    /// assert_eq!(vec.y, 2.0);
    /// assert_eq!(vec.z, 3.0);
    /// ```
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Получить нулевой вектор (0.0, 0.0, 0.0).
    ///
    /// # Examples
    /// ```rust
    /// let vec = Vec3::zero();
    /// assert_eq!(vec.x, 0.0);
    /// assert_eq!(vec.y, 0.0);
    /// assert_eq!(vec.z, 0.0);
    /// ```
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
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
}

// ========================================
// Операции над 3D вектором
// ========================================

impl Vec3 {
    /// Скалярное произведение векторов.
    ///
    /// # Examples
    /// ```rust
    /// // (1*4) + (2*5) + (3*6) = 4 + 10 + 18 = 32
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = Vec3::new(4.0, 5.0, 6.0);
    /// assert_eq!(v1.dot(v2), 32.0);
    ///
    /// // Перпендикулярные вектора
    /// let v3 = Vec3::new(1.0, 0.0, 0.0);
    /// let v4 = Vec3::new(0.0, 1.0, 0.0);
    /// assert_eq!(v3.dot(v4), 0.0);
    /// ```
    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Возвращает косинус угла между 2-мя векторами.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 0.0, 0.0);
    /// let v2 = Vec3::new(0.2, 0.8, 0.0);
    /// let dot = v1.dot(v2);
    /// assert!(0.0 < dot);
    /// assert!(dot < 1.0);
    ///
    /// // Перпендикулярные вектора
    /// let v3 = Vec3::new(1.0, 0.0, 0.0);
    /// let v4 = Vec3::new(0.0, 1.0, 0.0);
    /// assert_eq!(v3.cos(v4), 0.0);
    /// ```
    #[inline]
    pub fn cos(self, other: Self) -> f32 {
        self.dot(other) / (self.length() * other.length())
    }

    /// Возвращает угол в радианах между 2-мя векторами.
    #[inline]
    pub fn angle_rad(self, other: Self) -> f32 {
        self.cos(other).acos()
    }

    /// Возвращает угол в градусах между 2-мя векторами.
    #[inline]
    pub fn angle_deg(self, other: Self) -> f32 {
        self.cos(other).acos().to_degrees()
    }

    /// Спроецировать текущий вектор на вектор `onto`.
    ///
    /// # Examples
    /// ```rust
    /// let vec = Vec3::new(1.0, 2.0, 3.0);
    /// let onto = UVec3::new(0.0, 1.0, 0.0);
    /// let projection = vec.projection(onto);
    /// assert_eq!(projection.x, 0.0);
    /// assert_eq!(projection.y, 2.0);
    /// assert_eq!(projection.z, 0.0);
    /// ```
    #[inline]
    pub fn projection(self, onto: UVec3) -> Self {
        self.dot(onto.into()) * onto
    }

    /// Найти перпендикулярную составляющую текущего вектора к вектору `onto`.
    ///
    /// # Examples
    /// ```rust
    /// let vec = Vec3::new(1.0, 2.0, 3.0);
    /// let onto = UVec3::new(0.0, 1.0, 0.0);
    /// let rejection = vec.rejection(onto);
    /// assert_eq!(rejection.x, 1.0);
    /// assert_eq!(rejection.y, 0.0);
    /// assert_eq!(rejection.z, 3.0);
    /// ```
    #[inline]
    pub fn rejection(self, onto: UVec3) -> Self {
        self - self.projection(onto)
    }

    /// Векторное произведение векторов.
    ///
    /// Поскольку все координатные системы являются **левыми**, то и векторное произведение левое.
    /// Иными словами, при умножении `self` на `other` направление результирующего вектора будет таковым,
    /// что если 4-мя пальцами левой руки прокрутить от `self` к `other` по наименьшему углу, то большой палец
    /// будет указывать на направление результирующего вектора.
    ///
    /// # Examples
    /// ```rust
    /// let vec_x = Vec3::plus_x();
    /// let vec_y = Vec3::plus_y();
    /// let vec_z = vec_x.cross(vec_y);
    /// assert_eq!(vec_z.x, 0.0);
    /// assert_eq!(vec_z.y, 0.0);
    /// assert_eq!(vec_z.z, 1.0);
    /// ```
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Получить квадрат длины вектора.
    ///
    /// # Examples
    /// ```rust
    /// let vec = Vec3::new(0.0, 4.0, 0.0);
    /// let sq_len = vec.length_squared();
    /// assert_eq!(sq_len, 16.0);
    /// ```
    #[inline]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Получить длину вектора.
    ///
    /// # Examples
    /// ```rust
    /// let vec = Vec3::new(0.0, 4.0, 0.0);
    /// let len = vec.length();
    /// assert_eq!(sq_len, 4.0);
    /// ```
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Привести вектор к единичной длине.
    ///
    /// Поскольку новый вектор будет единичной длины, он будет иметь тип `UVec3`.
    ///
    /// # Examples
    /// ```rust
    /// let vec = Vec3::new(0.0, 2.0, 3.0) // длина = 4;
    /// let uvec: UVec3 = vec.normalize();
    /// assert_eq!(uvec.x, 0.0);
    /// assert_eq!(uvec.y, 2.0 / 4.0);
    /// assert_eq!(uvec.z, 3.0 / 4.0);
    /// ```
    #[inline]
    pub fn normalize(self) -> UVec3 {
        debug_assert_ne!(
            self.length_squared(),
            0.0,
            "Попытка нормализовать вектор с нулевой длиной"
        );
        UVec3::from(self)
    }

    /// Является ли вектор нормализованным.
    ///
    /// Вектор `Vec3` может иметь единичную длину, но если это условие обязательно к выполнению, то,
    /// скорее всего, лучше использовать `UVec3`.
    #[inline]
    pub fn is_normalized(&self) -> bool {
        self.length_squared() - 1.0 <= 2.0 * f32::EPSILON
    }

    /// Приблизительное сравнение векторов на равенство.
    ///
    /// # Arguments
    /// - `other` - другой вектор, с которым происходит сравнение;
    /// - `tolerance` - допустимая погрешность. Если разница между координатами >=`tolerance`, то координаты считаются разными.
    #[inline]
    pub fn approx_equal(self, other: Self, tolerance: f32) -> bool {
        (self.x - other.x).abs() < tolerance
            && (self.y - other.y).abs() < tolerance
            && (self.z - other.z).abs() < tolerance
    }

    /// Применить преобразование к текущемы однородному `Vec3`. Эта операция **создаёт новый** вектор.
    #[inline]
    pub fn apply_transform(self, transform: Transform3D) -> Self {
        transform.apply_to_hvec(self.into()).into()
    }
}

impl Mul<Transform3D> for Vec3 {
    type Output = Vec3;

    /// Применить преобразование `Transform3D` к однородному `Vec3`.
    fn mul(self, rhs: Transform3D) -> Self::Output {
        self.apply_transform(rhs)
    }
}

impl MulAssign<Transform3D> for Vec3 {
    /// Применить преобразование `Transform3D` к вектору `Vec3`.
    fn mul_assign(&mut self, rhs: Transform3D) {
        *self = *self * rhs;
    }
}

impl Neg for Vec3 {
    type Output = Self;

    /// Создаёт из вектора `a` отрицательный вектор `-a`.
    ///
    /// # Examples
    /// ```rust
    /// let vec = Vec3::new(1.0, 2.0, 3.0);
    /// let neg_uvec = -vec;
    /// assert_eq!(neg_uvec.x, -1.0);
    /// assert_eq!(neg_uvec.y, -2.0);
    /// assert_eq!(neg_uvec.z, -3.0);
    /// ```
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    /// Находит сумму между двумя векторами по правилу параллелограмма.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = Vec3::new(0.0, -1.0, 1.0);
    /// let res: Vec3 = v1 + v2;
    /// assert_eq!(res.x, 1.0);
    /// assert_eq!(res.y, 1.0);
    /// assert_eq!(res.z, 4.0);
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3 {
    /// Находит сумму между двумя векторами по правилу параллелограмма.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = Vec3::new(0.0, -1.0, 1.0);
    /// v1 += v2;
    /// assert_eq!(v1.x, 1.0);
    /// assert_eq!(v1.y, 1.0);
    /// assert_eq!(v1.z, 4.0);
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Add<UVec3> for Vec3 {
    type Output = Self;

    /// Находит сумму между двумя векторами по правилу параллелограмма.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = UVec3::new(0.0, 1.0, 0.0);
    /// let res: Vec3 = v1 + v2;
    /// assert_eq!(res.x, 1.0);
    /// assert_eq!(res.y, 3.0);
    /// assert_eq!(res.z, 3.0);
    /// ```
    fn add(self, rhs: UVec3) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign<UVec3> for Vec3 {
    /// Находит сумму между двумя векторами по правилу параллелограмма.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = UVec3::new(0.0, 1.0, 0.0);
    /// v1 += v2;
    /// assert_eq!(v1.x, 1.0);
    /// assert_eq!(v1.y, 3.0);
    /// assert_eq!(v1.z, 3.0);
    /// ```
    fn add_assign(&mut self, rhs: UVec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    /// Находит разность между векторами по правилу параллелограмма.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = Vec3::new(0.0, -1.0, 1.0);
    /// let res: Vec3 = v1 - v2;
    /// assert_eq!(res.x, 1.0);
    /// assert_eq!(res.y, 3.0);
    /// assert_eq!(res.z, 2.0);
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl SubAssign for Vec3 {
    /// Находит разность между векторами по правилу параллелограмма.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = Vec3::new(0.0, -1.0, 1.0);
    /// v1 -= v2;
    /// assert_eq!(v1.x, 1.0);
    /// assert_eq!(v1.y, 3.0);
    /// assert_eq!(v1.z, 2.0);
    /// ```
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Sub<UVec3> for Vec3 {
    type Output = Self;

    /// Находит разность между векторами по правилу параллелограмма.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = UVec3::new(0.0, -1.0, 0.0);
    /// let res: Vec3 = v1 - v2;
    /// assert_eq!(res.x, 1.0);
    /// assert_eq!(res.y, 3.0);
    /// assert_eq!(res.z, 3.0);
    /// ```
    fn sub(self, rhs: UVec3) -> Self::Output {
        self + (-rhs)
    }
}

impl SubAssign<UVec3> for Vec3 {
    /// Находит разность между векторами по правилу параллелограмма.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let v2 = UVec3::new(0.0, -1.0, 0.0);
    /// v1 -= v2;
    /// assert_eq!(v1.x, 1.0);
    /// assert_eq!(v1.y, 3.0);
    /// assert_eq!(v1.z, 3.0);
    /// ```
    fn sub_assign(&mut self, rhs: UVec3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    /// Умножение вектора на скаляр.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let res = v1 * 2.0;
    /// assert_eq!(res.x, 2.0);
    /// assert_eq!(res.y, 4.0);
    /// assert_eq!(res.z, 6.0);
    /// ```
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    /// Умножение скаляра на вектор.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// let res = 2.0 * v1;
    /// assert_eq!(res.x, 2.0);
    /// assert_eq!(res.y, 4.0);
    /// assert_eq!(res.z, 6.0);
    /// ```
    fn mul(self, rhs: Vec3) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f32> for Vec3 {
    /// Умножение вектора на скаляр.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 3.0);
    /// v1 *= 2.0;
    /// assert_eq!(v1.x, 2.0);
    /// assert_eq!(v1.y, 4.0);
    /// assert_eq!(v1.z, 6.0);
    /// ```
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    /// Деление вектора на скаляр.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 4.0);
    /// let res = v1 / 2.0;
    /// assert_eq!(res.x, 0.5);
    /// assert_eq!(res.y, 1.0);
    /// assert_eq!(res.z, 2.0);
    /// ```
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl DivAssign<f32> for Vec3 {
    /// Деление вектора на скаляр.
    ///
    /// # Examples
    /// ```rust
    /// let v1 = Vec3::new(1.0, 2.0, 4.0);
    /// v1 /= 2.0;
    /// assert_eq!(v1.x, 0.5);
    /// assert_eq!(v1.y, 1.0);
    /// assert_eq!(v1.z, 2.0);
    /// ```
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl From<Point3> for Vec3 {
    /// Получить вектор из `Point3`.
    ///
    /// # Examples
    /// ```rust
    /// let point = Point3::new(1.0, 2.0, 3.0);
    /// let vec = Vec3::from(point);
    /// assert_eq!(vec.x, 1.0);
    /// assert_eq!(vec.y, 2.0);
    /// assert_eq!(vec.z, 3.0);
    /// ```
    fn from(value: Point3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl TryFrom<HVec3> for Vec3 {
    type Error = VecError;

    /// Получить направление из `HVec3`.
    ///
    /// `HVec3` описывает какое-то направление 3D пространства только если `w = 0`, в противном случае
    /// `HVec3` представляет собой точку, но не направление пространства.
    ///
    /// # Examples
    /// ```rust
    /// // если hvec - направление
    /// let hvec_direction = HVec3::new(1.0, 2.0, 3.0, 0.0);
    /// let vec = Vec3::try_from(hvec_direction).unwrap();
    /// assert_eq!(vec.x, 0.0);
    /// assert_eq!(vec.y, 0.0);
    /// assert_eq!(vec.z, 1.0);
    ///
    /// // если hvec - точка
    /// let hvec_position = HVec3::new(1.0, 2.0, 3.0, 1.0);
    /// let err = Point3::try_from(hvec_position).unwrap_err();
    /// assert_eq!(err, VecError(hvec_position));
    /// ```
    fn try_from(value: HVec3) -> Result<Self, Self::Error> {
        if value.w != 0.0 {
            Err(Self::Error)
        } else {
            Ok(Self::new(value.x, value.y, value.z))
        }
    }
}

/// Ошибка при преобразовании `HVec3` в `Vec3`.
///
/// Возникает когда компонента `w != 0`, то есть `HVec3` обозначает позицию,
/// поэтому не может быть направлением.
#[derive(Debug, Clone, Copy)]
struct VecError(HVec3);

impl Display for VecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} не может быть преобразован в Vec3 из-за w!=0", self.0)
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
