//! Реализация структуры `Vec3`.

use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::{HVec3, Point3, Vec3};

impl Vec3 {
    /// Создать вектор по 3-м координатам.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Получить единичный вектор с направлением "вверх" в **глобальных** координатах.
    ///
    /// Координатная система правкорукая (right-handed), направлением вверх считается `+z`, как в `Blender`,
    /// поэтому вектор имеет вид `(0.0, 0.0, 1.0)`.
    pub fn up() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    /// Получить единичный вектор с направлением "вниз" в **глобальных** координатах.
    ///
    /// Координатная система правкорукая (right-handed), направлением вниз считается `-z`, как в `Blender`,
    /// поэтому вектор имеет вид `(0.0, 0.0, -1.0)`.
    pub fn down() -> Self {
        Self::new(0.0, 0.0, -1.0)
    }

    /// Получить единичный вектор с направлением "влево" в **глобальных** координатах.
    ///
    /// Координатная система правкорукая (right-handed), направлением влево считается `+y`, как в `Blender`,
    /// поэтому вектор имеет вид `(0.0, 1.0, 0.0)`.
    pub fn left() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    /// Получить единичный вектор с направлением "вправо" в **глобальных** координатах.
    ///
    /// Координатная система правкорукая (right-handed), направлением вправо считается `-y`, как в `Blender`,
    /// поэтому вектор имеет вид `(0.0, -1.0, 0.0)`.
    pub fn right() -> Self {
        Self::new(0.0, -1.0, 0.0)
    }

    /// Получить единичный вектор с направлением "прямо" в **глобальных** координатах.
    ///
    /// Координатная система правкорукая (right-handed), направлением прямо считается `+x`, как в `Blender`,
    /// поэтому вектор имеет вид `(1.0, 0.0, 0.0)`.
    pub fn forward() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    /// Получить единичный вектор с направлением "назад" в **глобальных** координатах.
    ///
    /// Координатная система правкорукая (right-handed), направлением назад считается `-x`, как в `Blender`,
    /// поэтому вектор имеет вид `(-1.0, 0.0, 0.0)`.
    pub fn backward() -> Self {
        Self::new(-1.0, 0.0, 0.0)
    }

    /// Скалярное произведение векторов.
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
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
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            self
        }
    }

    /// Приблизительное сравнение векторов на равенство.
    ///
    /// ## Arguments
    /// - `other` - другой вектор, с которым происходит сравнение;
    /// - `tolerance` - допустимая погрешность. Если разница между координатами >=`tolerance`, то координаты считаются разными.
    pub fn approx_equal(&self, other: &Self, tolerance: f32) -> bool {
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

impl MulAssign<f32> for Point3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
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
        Self {
            x: value.x / value.w,
            y: value.y / value.w,
            z: value.z / value.w,
        }
    }
}
