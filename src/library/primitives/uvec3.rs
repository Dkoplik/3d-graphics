//! Объявление и реализация структуры `UVec3`.

// используем все примитивы
use crate::library::primitives::*;
use std::{
    fmt::Display,
    ops::{Add, Div, Mul, MulAssign, Neg, Sub},
};

/// Направление единичной длины в 3D пространстве с координатами `x`, `y`, `z`.
///
/// Эту структуру надо использовать, если необходимо обозначить какое-то **направление** в
/// 3D пространстве, при этом строго **единичной длины**. Для произвольного направления лучше
/// подойдёт `Vec3`, для положения `Point3`. О координатной системе подробнее можно узнать в `CoordFrame`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// ========================================
// Различные конструкторы вектора
// ========================================

impl UVec3 {
    /// Создать unit-вектор по 3-м координатам.
    ///
    /// Указываемые координаты могут содержать вектор любой длины, но в итоге
    /// будет создан вектор того же направления, но единичной длины.
    ///
    /// # Examples
    /// ```rust
    /// let uvec = UVec3::new(0.0, 2.0, 3.0) // длина = 4;
    /// assert_eq!(uvec.x, 0.0);
    /// assert_eq!(uvec.y, 2.0 / 4.0);
    /// assert_eq!(uvec.z, 3.0 / 4.0);
    /// ```
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let len = (x * x + y * y + z * z).sqrt();
        debug_assert_ne!(
            len, 0.0,
            "Попытка создать единичный вектор UVec3 с нулевой длиной"
        );
        Self {
            x: x / len,
            y: y / len,
            z: z / len,
        }
    }

    /// Получить единичный вектор (1.0, 0.0, 0.0) в глобальных координатах.
    pub fn plus_x() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Получить единичный вектор (-1.0, 0.0, 0.0) в глобальных координатах.
    pub fn minus_x() -> Self {
        Self {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Получить единичный вектор (0.0, 1.0, 0.0) в глобальных координатах.
    pub fn plus_y() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    /// Получить единичный вектор (0.0, -1.0, 0.0) в глобальных координатах.
    pub fn minus_y() -> Self {
        Self {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        }
    }

    /// Получить единичный вектор (0.0, 0.0, 1.0) в глобальных координатах.
    pub fn plus_z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    /// Получить единичный вектор (0.0, 0.0, -1.0) в глобальных координатах.
    pub fn minus_z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        }
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
}

// ========================================
// Операции над единичным 3D вектором
// ========================================

impl UVec3 {
    /// Скалярное произведение векторов.
    ///
    /// # Examples
    /// ```rust
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

    /// Возвращает угл в радианах между 2-мя векторами.
    #[inline]
    pub fn angle_rad(self, other: Self) -> f32 {
        self.cos(other).acos()
    }

    /// Возвращает угл в градусах между 2-мя векторами.
    #[inline]
    pub fn angle_deg(self, other: Self) -> f32 {
        self.cos(other).acos().to_degrees()
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
    /// let vec_x = UVec3::plus_x();
    /// let vec_y = UVec3::plus_y();
    /// let vec_z: Vec3 = vec_x.cross(vec_y);
    /// assert_eq!(vec_z.x, 0.0);
    /// assert_eq!(vec_z.y, 0.0);
    /// assert_eq!(vec_z.z, 1.0);
    /// ```
    #[inline]
    pub fn cross(self, other: Self) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
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

    /// Применить преобразование к текущему вектору `UVec3`. Эта операция **создаёт новый** вектор.
    /// Вектор остаётся нормализованным после преобразования.
    pub fn apply_transform(self, transform: Transform3D) -> Self {
        todo!("А точно ли это стоит реализовывать вот так вот?");
        UVec3::from(HVec3::from(self) * transform)
    }
}

impl Mul<Transform3D> for UVec3 {
    type Output = Vec3;

    /// Применить преобразование `Transform3D` к вектору `UVec3`.
    fn mul(self, rhs: Transform3D) -> Self::Output {
        self.apply_transform(rhs)
    }
}

impl MulAssign<Transform3D> for UVec3 {
    /// Применить преобразование `Transform3D` к вектору `Vec3`.
    fn mul_assign(&mut self, rhs: Transform3D) {
        *self = *self * rhs;
    }
}

impl Neg for UVec3 {
    type Output = Self;

    /// Создаёт из вектора `a` отрицательный вектор `-a`.
    ///
    /// # Examples
    /// ```rust
    /// let uvec = UVec3::new(1.0, 0.0, 0.0);
    /// let neg_uvec = -uvec;
    /// assert_eq!(neg_uvec.x, -1.0);
    /// assert_eq!(neg_uvec.y, 0.0);
    /// assert_eq!(neg_uvec.z, 0.0);
    /// ```
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Add for UVec3 {
    type Output = Vec3;

    /// Находит сумму между двумя векторами по правилу параллелограмма.
    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<Vec3> for UVec3 {
    type Output = Vec3;

    /// Находит сумму между двумя векторами по правилу параллелограмма.
    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for UVec3 {
    type Output = Vec3;

    /// Находит разность между векторами по правилу параллелограмма.
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Sub<Vec3> for UVec3 {
    type Output = Vec3;

    /// Находит разность между векторами по правилу параллелограмма.
    fn sub(self, rhs: Vec3) -> Self::Output {
        self + (-rhs)
    }
}

impl Mul<f32> for UVec3 {
    type Output = Vec3;

    /// Умножение вектора на скаляр.
    fn mul(self, rhs: f32) -> Self::Output {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f32> for UVec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl TryFrom<Vec3> for UVec3 {
    type Error = UVecError;

    /// Получить вектор из `Vec3`.
    fn try_from(value: Vec3) -> Result<Self, Self::Error> {
        if value.x == 0.0 && value.y == 0.0 && value.z == 0.0 {
            Err(UVecError::ZeroVec)
        } else {
            Ok(UVec3::new(value.x, value.y, value.z))
        }
    }
}

impl TryFrom<Point3> for UVec3 {
    type Error = UVecError;

    /// Получить вектор из `Point3`.
    fn try_from(value: Point3) -> Result<Self, Self::Error> {
        if value.x == 0.0 && value.y == 0.0 && value.z == 0.0 {
            Err(UVecError::ZeroPoint)
        } else {
            Ok(UVec3::new(value.x, value.y, value.z))
        }
    }
}

impl TryFrom<HVec3> for UVec3 {
    type Error = UVecError;

    /// Получить вектор из направления `HVec3`.
    ///
    /// HVec3 должно быть направлением, то есть w = 0, в противном случае это позиция,
    /// которую сначала надо перевести в Point3, а потом только в Vec3.
    fn try_from(value: HVec3) -> Result<Self, Self::Error> {
        if value.x == 0.0 && value.y == 0.0 && value.z == 0.0 {
            Err(UVecError::ZeroVec)
        } else if value.w != 0.0 {
            Err(UVecError::PositionHVec(value))
        } else {
            Ok(UVec3::new(value.x, value.y, value.z))
        }
    }
}

/// Ошибки при преобразовании в UVec
#[derive(Debug, Clone, Copy)]
enum UVecError {
    /// Vec3 является нулевым
    ZeroVec,
    /// Point3 является нулевой
    ZeroPoint,
    /// HVec3 является позицией, а не направлением.
    PositionHVec(HVec3),
}

impl Display for UVecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ZeroVec => write!(f, "Попытка создать единичный вектор из нулевого вектора"),
            Self::ZeroPoint => write!(f, "Попытка создать единичный вектор из нулевой точки"),
            UVecError::PositionHVec(hvec) => {
                write!(f, "Попытка создать единичный вектор из позиции {}", hvec)
            }
        }
    }
}
