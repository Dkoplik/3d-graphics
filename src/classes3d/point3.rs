//! Реализация структуры `Point3`.

//! Реализация структуры `Point3`.

use std::ops::{Add, Sub};

use crate::{Point3, Vec3};
use crate::{Point3, Vec3};

impl Point3 {
    /// Создать новую точку по 3-м координатам.
    /// Создать новую точку по 3-м координатам.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Приблизительное сравнение точек на равенство.
    ///
    /// # Arguments
    /// - `other` - другая точка, с которой происходит сравнение;
    /// - `tolerance` - допустимая погрешность. Если разница между координатами >=`tolerance`, то координаты считаются разными.
    pub fn approx_equal(&self, other: &Self, tolerance: f32) -> bool {
        (self.x - other.x).abs() < tolerance
            && (self.y - other.y).abs() < tolerance
            && (self.z - other.z).abs() < tolerance
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
    /// Выполняет операцию `+` между точкой и вектором.
    ///
    /// По смыслу операция представляет собой смещение текущей точки на заданный вектор.
    fn add(self, rhs: Vec3) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl From<Vec3> for Point3 {
    /// Получить точку из `Vec3`.
    /// Получить точку из `Vec3`.
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}
