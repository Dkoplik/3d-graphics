use crate::{Line3, Point3, Vec3};
use crate::{Plane, Transform3D};

impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}

// --------------------------------------------------
// Конструкторы базовых преобразований
// --------------------------------------------------

impl Transform3D {
    /// Создает единичную матрицу преобразования.
    pub fn identity() -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, // первая строка
                0.0, 1.0, 0.0, 0.0, // вторая строка
                0.0, 0.0, 1.0, 0.0, // третья строка
                0.0, 0.0, 0.0, 1.0, // 4-ая строка (перемещение)
            ],
        }
    }

    /// Создает матрицу перемещения с одинаковым значением по всем осям.
    pub fn translation_uniform(d: f32) -> Self {
        Self::translation(d, d, d)
    }

    /// Создает матрицу перемещения с разными значениями по осям.
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, // первая строка
                0.0, 1.0, 0.0, 0.0, // вторая строка
                0.0, 0.0, 1.0, 0.0, // третья строка
                x, y, z, 1.0,
            ], // перемещение
        }
    }

    /// Создает матрицу масштабирования с одинаковым коэффициентом по всем осям.
    pub fn scale_uniform(s: f32) -> Self {
        Self::scale(s, s, s)
    }

    /// Создает матрицу масштабирования с разными коэффициентами по осям.
    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        Self {
            m: [
                sx, 0.0, 0.0, 0.0, // первая строка
                0.0, sy, 0.0, 0.0, // вторая строка
                0.0, 0.0, sz, 0.0, // третья строка
                0.0, 0.0, 0.0, 1.0, // перемещение
            ],
        }
    }

    /// Создает матрицу поворота вокруг оси X (в радианах).
    pub fn rotation_x_rad(angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, // первая строка
                0.0, cos_a, sin_a, 0.0, // вторая строка
                0.0, -sin_a, cos_a, 0.0, // третья строка
                0.0, 0.0, 0.0, 1.0, // перемещение
            ],
        }
    }

    /// Создает матрицу поворота вокруг оси X (в градусах).
    pub fn rotation_x_deg(angle: f32) -> Self {
        Self::rotation_x_rad(angle.to_radians())
    }

    /// Создает матрицу поворота вокруг оси Y (в радианах).
    pub fn rotation_y_rad(angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self {
            m: [
                cos_a, 0.0, -sin_a, 0.0, // первая строка
                0.0, 1.0, 0.0, 0.0, // вторая строка
                sin_a, 0.0, cos_a, 0.0, // третья строка
                0.0, 0.0, 0.0, 1.0, // перемещение
            ],
        }
    }

    /// Создает матрицу поворота вокруг оси Y (в градусах).
    pub fn rotation_y_deg(angle: f32) -> Self {
        Self::rotation_y_rad(angle.to_radians())
    }

    /// Создает матрицу поворота вокруг оси Z (в радианах).
    pub fn rotation_z_rad(angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self {
            m: [
                cos_a, sin_a, 0.0, 0.0, // первая строка
                -sin_a, cos_a, 0.0, 0.0, // вторая строка
                0.0, 0.0, 1.0, 0.0, // третья строка
                0.0, 0.0, 0.0, 1.0, // перемещение
            ],
        }
    }

    /// Создает матрицу поворота вокруг оси Z (в градусах).
    pub fn rotation_z_deg(angle: f32) -> Self {
        Self::rotation_z_rad(angle.to_radians())
    }
}

// --------------------------------------------------
// Конструкторы составных преобразований
// --------------------------------------------------

impl Transform3D {
    /// Масштабирование относительно точки anchor.
    pub fn scale_relative_to_point(anchor: Point3, sx: f32, sy: f32, sz: f32) -> Self {
        // Перенос точки в начало координат -> масштабирование -> обратный перенос
        Self::translation(-anchor.x, -anchor.y, -anchor.z)
            .multiply(Self::scale(sx, sy, sz))
            .multiply(Self::translation(anchor.x, anchor.y, anchor.z))
    }

    /// Отражение относительно плоскости XY.
    pub fn reflection_xy() -> Self {
        Self::scale(1.0, 1.0, -1.0)
    }

    /// Отражение относительно плоскости XZ.
    pub fn reflection_xz() -> Self {
        Self::scale(1.0, -1.0, 1.0)
    }

    /// Отражение относительно плоскости YZ.
    pub fn reflection_yz() -> Self {
        Self::scale(-1.0, 1.0, 1.0)
    }

    /// Отражение относительно произвольной плоскости.
    pub fn reflection_plane(plane: Plane) -> Self {
        todo!("Проверьте пж корректность reflection_plane в матрице 3д");
        let n = plane.normal;
        let d = -n.dot(plane.origin.into());

        let a = n.x;
        let b = n.y;
        let c = n.z;

        Self {
            m: [
                1.0 - 2.0 * a * a,
                -2.0 * a * b,
                -2.0 * a * c,
                0.0, // 1-ая строка
                -2.0 * a * b,
                1.0 - 2.0 * b * b,
                -2.0 * b * c,
                0.0, // 2-ая строка
                -2.0 * a * c,
                -2.0 * b * c,
                1.0 - 2.0 * c * c,
                0.0, // 3-я строка
                -2.0 * a * d,
                -2.0 * b * d,
                -2.0 * c * d,
                1.0, // 4-ая строка
            ],
        }
    }

    /// Поворот вокруг произвольной линии (оси).
    pub fn rotation_around_line(line: Line3, angle_rad: f32) -> Self {
        let direction = line.direction;
        let point = line.origin;

        let u = direction.x;
        let v = direction.y;
        let w = direction.z;

        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        let one_minus_cos = 1.0 - cos_a;

        let rotation_matrix = Self {
            m: [
                u * u * one_minus_cos + cos_a,
                u * v * one_minus_cos - w * sin_a,
                u * w * one_minus_cos + v * sin_a,
                0.0, // первая строка
                u * v * one_minus_cos + w * sin_a,
                v * v * one_minus_cos + cos_a,
                v * w * one_minus_cos - u * sin_a,
                0.0, // вторая строка
                u * w * one_minus_cos - v * sin_a,
                v * w * one_minus_cos + u * sin_a,
                w * w * one_minus_cos + cos_a,
                0.0, // третья строка
                0.0,
                0.0,
                0.0,
                1.0, // четвёртая строка (перемещение)
            ],
        };

        // Комбинируем с переносами для установки оси в нужное положение
        Self::translation(-point.x, -point.y, -point.z)
            .multiply(rotation_matrix)
            .multiply(Self::translation(point.x, point.y, point.z))
    }
}

// --------------------------------------------------
// Конструкторы проекций
// --------------------------------------------------

impl Transform3D {
    /// Создает матрицу перспективной проекции (row-major).
    pub fn perspective(fov_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_rad / 2.0).tan();
        let range_inv = 1.0 / (near - far);

        Self {
            m: [
                f / aspect,
                0.0,
                0.0,
                0.0, // первая строка
                0.0,
                f,
                0.0,
                0.0, // вторая строка
                0.0,
                0.0,
                (near + far) * range_inv,
                -1.0, // третья строка
                0.0,
                0.0,
                2.0 * near * far * range_inv,
                0.0, // четвёртая строка (смещение)
            ],
        }
    }

    /// Создает ортографическую матрицу проекции
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let r_l = 1.0 / (right - left);
        let t_b = 1.0 / (top - bottom);
        let f_n = 1.0 / (far - near);

        Self {
            m: [
                2.0 * r_l,
                0.0,
                0.0,
                -(right + left) * r_l, // первая строка
                0.0,
                2.0 * t_b,
                0.0,
                -(top + bottom) * t_b, // вторая строка
                0.0,
                0.0,
                -2.0 * f_n,
                -(far + near) * f_n, // третья строка
                0.0,
                0.0,
                0.0,
                1.0, // четвёртая строка
            ],
        }
    }

    /// Создает видовую матрицу.
    pub fn look_at(eye: Point3, target: Point3, up: Vec3) -> Self {
        let f = (target - eye).normalize();
        let s = f.cross(up.normalize()).normalize();
        let u = s.cross(f);

        Self {
            m: [
                s.x,
                s.y,
                s.z,
                -s.dot(eye.into()), // первая строка
                u.x,
                u.y,
                u.z,
                -u.dot(eye.into()), // вторая строка
                -f.x,
                -f.y,
                -f.z,
                f.dot(eye.into()), // третья строка
                0.0,
                0.0,
                0.0,
                1.0, // четвёртая строка
            ],
        }
    }

    /// Создает матрицу изометрической проекции.
    /// Изометрия: углы 120° между осями, одинаковое масштабирование по всем осям.
    pub fn isometric() -> Self {
        // Стандартные углы изометрии: 35.264° по X, 45° по Z
        let angle_x = 35.264_f32.to_radians();
        let angle_z = 45.0_f32.to_radians();

        Self::rotation_z_rad(angle_z).multiply(Self::rotation_x_rad(angle_x))
    }

    /// Создает матрицу диметрической проекции.
    /// Диметрия: два масштабных коэффициента одинаковы, третий отличается.
    pub fn dimetric(angle_x_deg: f32, angle_z_deg: f32, scale_y: f32) -> Self {
        let angle_x = angle_x_deg.to_radians();
        let angle_z = angle_z_deg.to_radians();

        Self::rotation_z_rad(angle_z)
            .multiply(Self::rotation_x_rad(angle_x))
            .multiply(Self::scale(1.0, scale_y, 1.0))
    }

    /// Создает матрицу триметрической проекции.
    /// Триметрия: все три масштабных коэффициента разные.
    pub fn trimetric(
        angle_x_deg: f32,
        angle_z_deg: f32,
        scale_x: f32,
        scale_y: f32,
        scale_z: f32,
    ) -> Self {
        let angle_x = angle_x_deg.to_radians();
        let angle_z = angle_z_deg.to_radians();

        Self::rotation_z_rad(angle_z)
            .multiply(Self::rotation_x_rad(angle_x))
            .multiply(Self::scale(scale_x, scale_y, scale_z))
    }

    /// Создает матрицу кабинетной проекции.
    /// Кабинетная: все оси имеют разный масштаб, часто используется в технических чертежах.
    pub fn cabinet(angle_deg: f32, depth_scale: f32) -> Self {
        let angle = angle_deg.to_radians();

        Self {
            m: [
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                depth_scale * angle.cos(),
                depth_scale * angle.sin(),
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    /// Создает матрицу кавальерной проекции.
    /// Кавальерная: все линии проецируются в натуральную величину.
    pub fn cavalier(angle_deg: f32, depth_scale: f32) -> Self {
        let angle = angle_deg.to_radians();

        Self {
            m: [
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                depth_scale * angle.cos(),
                depth_scale * angle.sin(),
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    /// Создает матрицу аксонометрической проекции с произвольными углами.
    pub fn axonometric(angle_x_deg: f32, angle_y_deg: f32, angle_z_deg: f32) -> Self {
        let angle_x = angle_x_deg.to_radians();
        let angle_y = angle_y_deg.to_radians();
        let angle_z = angle_z_deg.to_radians();

        Self::rotation_x_rad(angle_x)
            .multiply(Self::rotation_y_rad(angle_y))
            .multiply(Self::rotation_z_rad(angle_z))
    }
}

// --------------------------------------------------
// Вспомогательные функции
// --------------------------------------------------

impl Transform3D {
    /// Умножение (композиция) матриц преобразования.
    pub fn multiply(self, other: Self) -> Self {
        let mut result = [0.0; 16];

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i * 4 + j] += self.m[i * 4 + k] * other.m[k * 4 + j];
                }
            }
        }

        Self { m: result }
    }

    /// Применяет преобразование к точке с учётом перспективы.
    pub fn apply_to_point(&self, point: Point3) -> Point3 {
        // Умножаем точку как вектор [x, y, z, 1] на матрицу row-major
        let x = point.x * self.m[0] + point.y * self.m[4] + point.z * self.m[8] + self.m[12];
        let y = point.x * self.m[1] + point.y * self.m[5] + point.z * self.m[9] + self.m[13];
        let z = point.x * self.m[2] + point.y * self.m[6] + point.z * self.m[10] + self.m[14];
        let w = point.x * self.m[3] + point.y * self.m[7] + point.z * self.m[11] + self.m[15];

        if w != 0.0 && w != 1.0 {
            Point3::new(x / w, y / w, z / w)
        } else {
            Point3::new(x, y, z)
        }
    }

    /// Применяет преобразование к вектору (без перспективы).
    pub fn apply_to_vector(&self, vector: Vec3) -> Vec3 {
        Vec3 {
            x: vector.x * self.m[0] + vector.y * self.m[4] + vector.z * self.m[8] + self.m[12],
            y: vector.x * self.m[1] + vector.y * self.m[5] + vector.z * self.m[9] + self.m[13],
            z: vector.x * self.m[2] + vector.y * self.m[6] + vector.z * self.m[10] + self.m[14],
        }
    }

    /// Возвращает транспонированную матрицу.
    pub fn transpose(self) -> Self {
        Self {
            m: [
                self.m[0], self.m[4], self.m[8], self.m[12], // первая строка
                self.m[1], self.m[5], self.m[9], self.m[13], // вторая строка
                self.m[2], self.m[6], self.m[10], self.m[14], // третья строка
                self.m[3], self.m[7], self.m[11], self.m[15], // 4-ая строка
            ],
        }
    }

    /// Возвращает обратную матрицу (если возможно).
    pub fn inverse(self) -> Option<Self> {
        // Упрощенная реализация для афинных преобразований
        // Для полной реализации нужно использовать алгебраические дополнения
        let det = self.determinant();
        if det == 0.0 {
            return None;
        }

        // Для афинных преобразований можно использовать упрощенный расчет
        let inv_det = 1.0 / det;
        let mut result = [0.0; 16];

        // Вычисляем обратную матрицу 3x3 для поворота/масштабирования
        result[0] = (self.m[5] * self.m[10] - self.m[6] * self.m[9]) * inv_det;
        result[1] = (self.m[2] * self.m[9] - self.m[1] * self.m[10]) * inv_det;
        result[2] = (self.m[1] * self.m[6] - self.m[2] * self.m[5]) * inv_det;

        result[4] = (self.m[6] * self.m[8] - self.m[4] * self.m[10]) * inv_det;
        result[5] = (self.m[0] * self.m[10] - self.m[2] * self.m[8]) * inv_det;
        result[6] = (self.m[2] * self.m[4] - self.m[0] * self.m[6]) * inv_det;

        result[8] = (self.m[4] * self.m[9] - self.m[5] * self.m[8]) * inv_det;
        result[9] = (self.m[1] * self.m[8] - self.m[0] * self.m[9]) * inv_det;
        result[10] = (self.m[0] * self.m[5] - self.m[1] * self.m[4]) * inv_det;

        // Вычисляем обратное перемещение
        result[12] = -(self.m[12] * result[0] + self.m[13] * result[4] + self.m[14] * result[8]);
        result[13] = -(self.m[12] * result[1] + self.m[13] * result[5] + self.m[14] * result[9]);
        result[14] = -(self.m[12] * result[2] + self.m[13] * result[6] + self.m[14] * result[10]);

        result[3] = 0.0;
        result[7] = 0.0;
        result[11] = 0.0;
        result[15] = 1.0;

        Some(Self { m: result })
    }

    /// Вычисляет определитель матрицы.
    pub fn determinant(&self) -> f32 {
        // Для 4x4 матрицы
        let a = self.m[0];
        let b = self.m[4];
        let c = self.m[8];
        let d = self.m[12];
        let e = self.m[1];
        let f = self.m[5];
        let g = self.m[9];
        let h = self.m[13];
        let i = self.m[2];
        let j = self.m[6];
        let k = self.m[10];
        let l = self.m[14];
        let m = self.m[3];
        let n = self.m[7];
        let o = self.m[11];
        let p = self.m[15];

        a * f * k * p
            + a * g * l * n
            + a * h * j * o
            + b * e * l * o
            + b * g * i * p
            + b * h * k * m
            + c * e * j * p
            + c * f * l * m
            + c * h * i * n
            + d * e * k * n
            + d * f * i * o
            + d * g * j * m
            - a * f * l * o
            - a * g * j * p
            - a * h * k * n
            - b * e * k * p
            - b * g * l * m
            - b * h * i * o
            - c * e * l * n
            - c * f * i * p
            - c * h * j * m
            - d * e * j * o
            - d * f * k * m
            - d * g * i * n
    }
}

// Реализация оператора умножения для удобства
impl std::ops::Mul for Transform3D {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPSILON: f32 = 1e-6;

    fn assert_point_approx_eq(p1: Point3, p2: Point3) {
        assert!((p1.x - p2.x).abs() < EPSILON, "x: {} != {}", p1.x, p2.x);
        assert!((p1.y - p2.y).abs() < EPSILON, "y: {} != {}", p1.y, p2.y);
        assert!((p1.z - p2.z).abs() < EPSILON, "z: {} != {}", p1.z, p2.z);
    }

    fn assert_vec_approx_eq(v1: Vec3, v2: Vec3) {
        assert!((v1.x - v2.x).abs() < EPSILON, "x: {} != {}", v1.x, v2.x);
        assert!((v1.y - v2.y).abs() < EPSILON, "y: {} != {}", v1.y, v2.y);
        assert!((v1.z - v2.z).abs() < EPSILON, "z: {} != {}", v1.z, v2.z);
    }

    fn assert_matrix_approx_eq(m1: Transform3D, m2: Transform3D) {
        for i in 0..16 {
            assert!(
                (m1.m[i] - m2.m[i]).abs() < EPSILON,
                "Element {}: {} != {}",
                i,
                m1.m[i],
                m2.m[i]
            );
        }
    }

    #[test]
    fn test_identity() {
        let identity = Transform3D::identity();
        let point = Point3::new(1.0, 2.0, 3.0);
        let result = identity.apply_to_point(point);

        assert_point_approx_eq(point, result);
    }

    #[test]
    fn test_translation() {
        let translation = Transform3D::translation(2.0, 3.0, 4.0);
        let point = Point3::new(1.0, 1.0, 1.0);
        let result = translation.apply_to_point(point);

        assert_point_approx_eq(Point3::new(3.0, 4.0, 5.0), result);
    }

    #[test]
    fn test_translation_uniform() {
        let translation = Transform3D::translation_uniform(5.0);
        let point = Point3::new(1.0, 2.0, 3.0);
        let result = translation.apply_to_point(point);

        assert_point_approx_eq(Point3::new(6.0, 7.0, 8.0), result);
    }

    #[test]
    fn test_scale() {
        let scale = Transform3D::scale(2.0, 3.0, 4.0);
        let point = Point3::new(1.0, 2.0, 3.0);
        let result = scale.apply_to_point(point);

        assert_point_approx_eq(Point3::new(2.0, 6.0, 12.0), result);
    }

    #[test]
    fn test_scale_uniform() {
        let scale = Transform3D::scale_uniform(2.0);
        let point = Point3::new(1.0, 2.0, 3.0);
        let result = scale.apply_to_point(point);

        assert_point_approx_eq(Point3::new(2.0, 4.0, 6.0), result);
    }

    #[test]
    fn test_rotation_x() {
        let rotation = Transform3D::rotation_x_rad(PI / 2.0);
        let point = Point3::new(0.0, 1.0, 0.0);
        let result = rotation.apply_to_point(point);

        assert_point_approx_eq(Point3::new(0.0, 0.0, 1.0), result);
    }

    #[test]
    fn test_rotation_y() {
        let rotation = Transform3D::rotation_y_rad(PI / 2.0);
        let point = Point3::new(0.0, 0.0, 1.0);
        let result = rotation.apply_to_point(point);

        assert_point_approx_eq(Point3::new(1.0, 0.0, 0.0), result);
    }

    #[test]
    fn test_rotation_z() {
        let rotation = Transform3D::rotation_z_rad(PI / 2.0);
        let point = Point3::new(1.0, 0.0, 0.0);
        let result = rotation.apply_to_point(point);

        assert_point_approx_eq(Point3::new(0.0, 1.0, 0.0), result);
    }

    #[test]
    fn test_rotation_degrees() {
        let rotation_rad = Transform3D::rotation_x_rad(PI / 4.0);
        let rotation_deg = Transform3D::rotation_x_deg(45.0);

        let point = Point3::new(0.0, 1.0, 0.0);
        let result_rad = rotation_rad.apply_to_point(point);
        let result_deg = rotation_deg.apply_to_point(point);

        assert_point_approx_eq(result_rad, result_deg);
    }

    #[test]
    fn test_multiply_identity() {
        let identity = Transform3D::identity();
        let translation = Transform3D::translation(1.0, 2.0, 3.0);

        let result1 = identity.multiply(translation);
        let result2 = translation.multiply(identity);

        assert_matrix_approx_eq(translation, result1);
        assert_matrix_approx_eq(translation, result2);
    }

    #[test]
    fn test_multiply_translation_scale() {
        let translation = Transform3D::translation(1.0, 2.0, 3.0);
        let scale = Transform3D::scale(2.0, 3.0, 4.0);

        let combined = translation.multiply(scale);
        let point = Point3::new(1.0, 1.0, 1.0);
        let result = combined.apply_to_point(point);

        // Сначала масштабирование, потом перемещение
        assert_point_approx_eq(Point3::new(3.0, 5.0, 7.0), result);
    }

    #[test]
    fn test_scale_relative_to_point() {
        let center = Point3::new(1.0, 1.0, 1.0);
        let scale = Transform3D::scale_relative_to_point(center, 2.0, 2.0, 2.0);

        // Точка в центре должна остаться на месте
        let center_result = scale.apply_to_point(center);
        assert_point_approx_eq(center, center_result);

        // Точка на расстоянии 1 от центра должна отдалиться на 2
        let point = Point3::new(2.0, 1.0, 1.0);
        let result = scale.apply_to_point(point);
        assert_point_approx_eq(Point3::new(3.0, 1.0, 1.0), result);
    }

    #[test]
    fn test_reflection_xy() {
        let reflection = Transform3D::reflection_xy();
        let point = Point3::new(1.0, 2.0, 3.0);
        let result = reflection.apply_to_point(point);

        assert_point_approx_eq(Point3::new(1.0, 2.0, -3.0), result);
    }

    #[test]
    fn test_reflection_xz() {
        let reflection = Transform3D::reflection_xz();
        let point = Point3::new(1.0, 2.0, 3.0);
        let result = reflection.apply_to_point(point);

        assert_point_approx_eq(Point3::new(1.0, -2.0, 3.0), result);
    }

    #[test]
    fn test_reflection_yz() {
        let reflection = Transform3D::reflection_yz();
        let point = Point3::new(1.0, 2.0, 3.0);
        let result = reflection.apply_to_point(point);

        assert_point_approx_eq(Point3::new(-1.0, 2.0, 3.0), result);
    }

    #[test]
    fn test_reflection_plane() {
        let normal = Vec3::new(0.0, 1.0, 0.0); // Плоскость XZ
        let origin = Point3::new(0.0, 0.0, 0.0);
        let reflection = Transform3D::reflection_plane(Plane::new(origin, normal));

        let point = Point3::new(1.0, 2.0, 3.0);
        let result = reflection.apply_to_point(point);

        assert_point_approx_eq(Point3::new(1.0, -2.0, 3.0), result);
    }

    #[test]
    fn test_rotation_around_line() {
        let line = Line3::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let rotation = Transform3D::rotation_around_line(line, PI / 2.0);

        let point = Point3::new(1.0, 0.0, 0.0);
        let result = rotation.apply_to_point(point);

        // Вращение вокруг оси Y на 90° должно превратить (1,0,0) в (0,0,-1)
        assert_point_approx_eq(Point3::new(0.0, 0.0, -1.0), result);
    }

    #[test]
    fn test_apply_to_vector() {
        let translation = Transform3D::translation(1.0, 2.0, 3.0);
        let vector = Vec3::new(1.0, 0.0, 0.0);
        let result = translation.apply_to_vector(vector);

        // Векторы не должны подвергаться перемещению
        assert_vec_approx_eq(vector, result);

        // Проверка масштабирования векторов
        let scale = Transform3D::scale(2.0, 3.0, 4.0);
        let scaled_vector = scale.apply_to_vector(vector);
        assert_vec_approx_eq(Vec3::new(2.0, 0.0, 0.0), scaled_vector);
    }

    #[test]
    fn test_perspective_projection() {
        let perspective = Transform3D::perspective(PI / 2.0, 1.0, 0.1, 100.0);
        let point = Point3::new(0.0, 0.0, 5.0);
        let result = perspective.apply_to_point(point);

        // Точка на оси Z должна проецироваться в (0, 0, ~0.98) в NDC
        assert!(result.z > 0.9 && result.z < 1.0);
        assert!(result.x.abs() < EPSILON);
        assert!(result.y.abs() < EPSILON);
    }

    #[test]
    fn test_orthographic_projection() {
        let ortho = Transform3D::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0);
        let point = Point3::new(0.5, 0.5, 5.0);
        let result = ortho.apply_to_point(point);

        // Ортографическая проекция не искажает координаты X, Y
        assert!(result.x > 0.0 && result.x < 1.0);
        assert!(result.y > 0.0 && result.y < 1.0);
    }

    #[test]
    fn test_look_at() {
        let eye = Point3::new(0.0, 0.0, 5.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);

        let view_matrix = Transform3D::look_at(eye, target, up);
        let point = Point3::new(0.0, 0.0, 0.0);
        let result = view_matrix.apply_to_point(point);

        // Точка в начале координат должна оказаться перед камерой
        assert!(result.z < 0.0);
    }

    #[test]
    fn test_isometric() {
        let isometric = Transform3D::isometric();
        let point = Point3::new(1.0, 0.0, 0.0);
        let result = isometric.apply_to_point(point);

        // Изометрия сохраняет расстояния (приблизительно)
        let original_length = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
        let result_length =
            (result.x * result.x + result.y * result.y + result.z * result.z).sqrt();

        assert!((original_length - result_length).abs() < 0.1);
    }

    #[test]
    fn test_dimetric() {
        let dimetric = Transform3D::dimetric(20.0, 30.0, 0.5);
        let point = Point3::new(1.0, 1.0, 1.0);
        let result = dimetric.apply_to_point(point);

        // Проверяем что преобразование применяется без паники
        assert!(!result.x.is_nan());
        assert!(!result.y.is_nan());
        assert!(!result.z.is_nan());
    }

    #[test]
    fn test_trimetric() {
        let trimetric = Transform3D::trimetric(15.0, 25.0, 1.0, 0.7, 0.9);
        let point = Point3::new(1.0, 2.0, 3.0);
        let result = trimetric.apply_to_point(point);

        // Проверяем что разные масштабы применяются
        assert!(result.x != result.y);
        assert!(result.y != result.z);
    }

    #[test]
    fn test_cabinet_projection() {
        let cabinet = Transform3D::cabinet(45.0, 0.5);
        let point = Point3::new(1.0, 1.0, 1.0);
        let result = cabinet.apply_to_point(point);

        // Кабинетная проекция имеет сокращение по глубине
        assert!(result.z < 1.0);
    }

    #[test]
    fn test_transpose() {
        let original =
            Transform3D::translation(1.0, 2.0, 3.0).multiply(Transform3D::rotation_x_rad(PI / 4.0));

        let transposed = original.transpose();
        let transposed_twice = transposed.transpose();

        // Двойное транспонирование должно вернуть исходную матрицу
        assert_matrix_approx_eq(original, transposed_twice);
    }

    #[test]
    fn test_operator_overload() {
        let t1 = Transform3D::translation(1.0, 2.0, 3.0);
        let t2 = Transform3D::scale(2.0, 2.0, 2.0);

        let manual = t1.multiply(t2);
        let overloaded = t1 * t2;

        assert_matrix_approx_eq(manual, overloaded);
    }

    #[test]
    fn test_complex_transformation_chain() {
        // Сложная цепочка преобразований
        let transform = Transform3D::translation(1.0, 2.0, 3.0)
            * Transform3D::rotation_x_deg(45.0)
            * Transform3D::rotation_y_deg(30.0)
            * Transform3D::scale(2.0, 1.0, 1.5)
            * Transform3D::translation(-1.0, -1.0, -1.0);

        let point = Point3::new(1.0, 2.0, 3.0);
        let result = transform.apply_to_point(point);

        // Проверяем что преобразование работает без ошибок
        assert!(!result.x.is_nan());
        assert!(!result.y.is_nan());
        assert!(!result.z.is_nan());
    }

    #[test]
    fn test_zero_scale() {
        let scale = Transform3D::scale(0.0, 0.0, 0.0);
        let point = Point3::new(1.0, 2.0, 3.0);
        let result = scale.apply_to_point(point);

        assert_point_approx_eq(Point3::new(0.0, 0.0, 0.0), result);
    }

    #[test]
    fn test_vector_normalization_in_reflection() {
        let normal = Vec3::new(1.0, 1.0, 1.0); // Ненормализованный вектор
        let origin = Point3::new(0.0, 0.0, 0.0);
        let reflection = Transform3D::reflection_plane(Plane::new(origin, normal));

        let test_point = Point3::new(1.0, 0.0, 0.0);
        let result = reflection.apply_to_point(test_point);

        // Проверяем что отражение работает даже с ненормализованным вектором
        assert!(!result.x.is_nan());
    }
}
