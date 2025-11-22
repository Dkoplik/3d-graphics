//! Объявление и реализация структуры `Point3`.

use super::{HVec3, Transform3D, UVec3, Vec3};
use std::{
    fmt::Display,
    ops::{Add, Sub},
};

/// Точка в 3D пространстве с координатами `x`, `y`, `z`.
///
/// Эту структуру надо использовать, если необходимо обозначить какое-то **положение** в
/// 3D пространстве. Для направления надо использовать `Vec3`. О координатной системе
/// подробнее можно узнать в `CoordFrame`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    /// Создать новую точку `(x, y, z)` по 3-м координатам.
    ///
    /// # Examples
    /// ```rust
    /// let point = Point3::new(1.0, 2.0, 3.0);
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 2.0);
    /// assert_eq!(point.z, 3.0);
    /// ```
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Создать точку `(0.0, 0.0, 0.0)`.
    ///
    /// # Examples
    /// ```rust
    /// let zero_point = Point3::zero();
    /// assert_eq!(point.x, 0.0);
    /// assert_eq!(point.y, 0.0);
    /// assert_eq!(point.z, 0.0);
    /// ```
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

    /// Применить преобразование `transform` к точке `Point3`. Эта операция **создаёт новую** точку.
    ///
    /// # Examples
    /// ```rust
    /// let original_point = Point3::zero();
    /// let transform = Transform3D::translation(1.0, 2.0, 3.0);
    /// let translated_point = original_point.apply_transform(transform);
    /// assert_eq!(translated_point.x, 1.0);
    /// assert_eq!(translated_point.y, 2.0);
    /// assert_eq!(translated_point.z, 3.0);
    /// ```
    pub fn apply_transform(self, transform: Transform3D) -> Result<Self, PointError> {
        Point3::try_from(HVec3::from(self) * transform)
    }
}

impl Display for Point3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point3(x: {}, y: {}, z: {})", self.x, self.y, self.z)
    }
}

impl Sub for Point3 {
    type Output = Vec3;

    /// Производит операцию `-` между двумя точками, получая вектор.
    ///
    /// # Returns
    /// Возвращает вектор, направленный из вычитаемой (правой) точки в уменьшаемую (левую) точку.
    ///
    /// # Examples
    /// ```rust
    /// let point1 = Point3::new(1.0, 1.0, 1.0);
    /// let point2 = Point3::new(3.0, 4.0, 5.0);
    /// let vec: Vec3 = point - zero_point;
    /// assert_eq!(vec.x, 2.0);
    /// assert_eq!(vec.y, 3.0);
    /// assert_eq!(vec.z, 4.0);
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Add<Vec3> for Point3 {
    type Output = Self;

    /// Выполняет операцию `+` между точкой и вектором.
    ///
    /// По смыслу операция представляет собой смещение текущей точки на заданный вектор.
    ///
    /// # Examples
    /// ```rust
    /// let original_point = Point3::new(1.0, 2.0, 3.0);
    /// let vec = Vec3::new(-2.0, 3.0, 1.0);
    /// let moved_point = original_point + vec;
    /// assert_eq!(moved_point.x, -1.0);
    /// assert_eq!(moved_point.y, 5.0);
    /// assert_eq!(moved_point.z, 4.0);
    /// ```
    fn add(self, rhs: Vec3) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl From<Vec3> for Point3 {
    /// Получить точку из `Vec3`.
    ///
    /// # Examples
    /// ```rust
    /// let vec = Vec3::new(1.0, 2.0, 3.0);
    /// let point: Point3 = Point3::from(vec);
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 2.0);
    /// assert_eq!(point.z, 3.0);
    /// ```
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<UVec3> for Point3 {
    /// Получить точку из `UVec3`.
    ///
    /// # Examples
    /// ```rust
    /// let uvec = UVec3::new(0.0, 1.0, 0.0);
    /// let point: Point3 = Point3::from(uvec);
    /// assert_eq!(point.x, 0.0);
    /// assert_eq!(point.y, 1.0);
    /// assert_eq!(point.z, 0.0);
    /// ```
    fn from(value: UVec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl TryFrom<HVec3> for Point3 {
    type Error = PointError;

    /// Получить точку из `HVec3`.
    ///
    /// `HVec3` описывает какую-то точку 3D пространства только если `w != 0`, в противном случае
    /// `HVec3` представляет собой направление, но не точку пространства.
    ///
    /// # Examples
    /// ```rust
    /// // если hvec - точка
    /// let hvec_position = HVec3::new(1.0, 2.0, 3.0, 1.0);
    /// let point = Point3::try_from(hvec_position).unwrap();
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 2.0);
    /// assert_eq!(point.z, 3.0);
    ///
    /// // если hvec - направление
    /// let hvec_direction = HVec3::new(1.0, 2.0, 3.0, 0.0);
    /// let err = Point3::try_from(hvec_direction).unwrap_err();
    /// assert_eq!(err, PointError(hvec_direction));
    /// ```
    fn try_from(value: HVec3) -> Result<Self, Self::Error> {
        if value.w == 0.0 {
            Err(PointError(value))
        } else {
            Ok(Self::new(
                value.x / value.w,
                value.y / value.w,
                value.z / value.w,
            ))
        }
    }
}

/// Ошибка при преобразовании `HVec3` в `Point3`.
///
/// Возникает когда компонента `w=0`, то есть `HVec3` обозначает направление,
/// поэтому не может быть точкой.
#[derive(Debug, Clone, Copy)]
pub struct PointError(HVec3);

impl Display for PointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} не может быть преобразован в Point3 из-за w=0",
            self.0
        )
    }
}
