use std::ops::{Add, Sub};
use egui::{Color32, Painter, Pos2};

use crate::{Point3, Transformable3, Vec3};

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Приблизительное сравнение точек на равенство.
    pub fn approx_equal(&self, other: &Point3, tolerance: f32) -> bool {
        if (self.x - other.x).abs() >= tolerance {
            false
        } else if (self.y - other.y).abs() >= tolerance {
            false
        } else if (self.z - other.z).abs() >= tolerance {
            false
        } else {
            true
        }
    }

    /// Нарисовать точку (вершину).
    pub fn draw(&self, painter: &mut Painter, color: Color32, radius: f32) {
        painter.circle_filled(Pos2::new(self.x, self.y), radius, color);
    }
}

impl Sub for Point3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Add<Vec3> for Point3 {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl From<Vec3> for Point3 {
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl Transformable3 for Point3 {
    fn transform(self, transform: crate::Transform3D) -> Self {
        transform.apply_to_point(self)
    }

    fn apply_transform(&mut self, transform: crate::Transform3D) {
        let transformed = self.transform(transform);
        self.x = transformed.x;
        self.y = transformed.y;
        self.z = transformed.z;
    }
}
