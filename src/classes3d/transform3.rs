//! Реализация матрицы преобразования 4x4 для 4D векторов (для `HVec3`).

use crate::{HVec3, Line3, Point3, Vec3};
use crate::{Plane, Transform3D};

impl Default for Transform3D {
    /// Создаёт тождественную (единичную) матрицу как матрицу по-умолчанию.
    fn default() -> Self {
        Self::identity()
    }
}

// --------------------------------------------------
// Конструкторы базовых преобразований
// --------------------------------------------------

impl Transform3D {
    /// Создает единичную (тождественную) матрицу преобразования.
    ///
    /// Эта матрица преобразования ничего не меняет в исходном векторе.
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

    /// Создает матрицу перемещения на одинаковое значение по всем осям.
    ///
    /// После применения этой матрицы к вектору, он будет смещён по всем осям на значение `d`.
    pub fn translation_uniform(d: f32) -> Self {
        Self::translation(d, d, d)
    }

    /// Создает матрицу перемещения с разными значениями по осям.
    ///
    /// После применения этой матрицы к вектору, он будет смещён на
    /// значения `dx`, `dy`  и `dz` по соответствующим осям.
    pub fn translation(dx: f32, dy: f32, dz: f32) -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, // первая строка
                0.0, 1.0, 0.0, 0.0, // вторая строка
                0.0, 0.0, 1.0, 0.0, // третья строка
                dx, dy, dz, 1.0, // перемещение (4-я строка)
            ],
        }
    }

    /// Создает матрицу масштабирования с одинаковым коэффициентом по всем осям.
    ///
    /// После применения этой матрицы к вектору, тот будет масштабирован равномерно по всем осям на значение `s`.
    pub fn scale_uniform(s: f32) -> Self {
        Self::scale(s, s, s)
    }

    /// Создает матрицу масштабирования с разными коэффициентами по осям.
    ///
    /// После применения этой матрицы к вектору, тот будет масштабирован на значения `sx`, `sy`, `sz` по соответствующим осям.
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
    /// Создает матрицу перспективной проекции.
    pub fn perspective(fov_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_rad / 2.0).tan();
        let range_inv = 1.0 / (near - far);

        Self {
            m: [
                f / aspect,
                0.0,
                0.0,
                0.0, // 1 строка
                0.0,
                f,
                0.0,
                0.0, // 2 строка
                0.0,
                0.0,
                (near + far) * range_inv,
                -1.0, // 3 строка
                0.0,
                0.0,
                2.0 * near * far * range_inv,
                0.0, // 4 строка
            ],
        }
    }

    /// Создает ортографическую матрицу проекции.
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let r_l = 1.0 / (right - left);
        let t_b = 1.0 / (top - bottom);
        let f_n = 1.0 / (far - near);

        Self {
            m: [
                2.0 * r_l,
                0.0,
                0.0,
                -(right + left) * r_l, // 1 строка
                0.0,
                2.0 * t_b,
                0.0,
                -(top + bottom) * t_b, // 2 строка
                0.0,
                0.0,
                -2.0 * f_n,
                -(far + near) * f_n, // 3 строка
                0.0,
                0.0,
                0.0,
                1.0, // 4 строка
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
                -s.dot(eye.into()), // 1 строка
                u.x,
                u.y,
                u.z,
                -u.dot(eye.into()), // 2 строка
                -f.x,
                -f.y,
                -f.z,
                f.dot(eye.into()), // 3 строка
                0.0,
                0.0,
                0.0,
                1.0, // 4 строка
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

    /// Применить преобразование к однородному вектору `HVec3`. Возвращает **новый** вектор.
    ///
    /// Эта операция выполняет произведение **строчного** вектора `hvec` на матрицу преобразования:
    /// ```txt
    ///                 | m11 m12 m13 m14 |
    /// (x, y, z, w) x  | m21 m22 m23 m24 | = (x_new, y_new, z_new, w_new)
    ///                 | m31 m32 m33 m34 |
    ///                 | m41 m42 m43 m44 |
    /// ```
    pub fn apply_to_hvec(&self, hvec: &HVec3) -> HVec3 {
        /*
        x_new = x * m11 + y * m21 + z * m31 + w * m41
        y_new = x * m12 + y * m22 + z * m32 + w * m42
        z_new = x * m13 + y * m23 + z * m33 + w * m43
        w_new = x * m14 + y * m24 + z * m34 + w * m44
        */
        HVec3::new(
            hvec.x * self.m[0] + hvec.y * self.m[4] + hvec.z * self.m[8] + hvec.w * self.m[12],
            hvec.x * self.m[1] + hvec.y * self.m[5] + hvec.z * self.m[9] + hvec.w * self.m[13],
            hvec.x * self.m[2] + hvec.y * self.m[6] + hvec.z * self.m[10] + hvec.w * self.m[14],
            hvec.x * self.m[3] + hvec.y * self.m[7] + hvec.z * self.m[11] + hvec.w * self.m[15],
        )
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
    use crate::{Line3, Plane, Point3, Vec3};

    const TOLERANCE: f32 = 1e-6;

    // Вспомогательная функция для сравнения матриц
    fn matrices_approx_equal(a: &Transform3D, b: &Transform3D, tolerance: f32) -> bool {
        for i in 0..16 {
            if (a.m[i] - b.m[i]).abs() >= tolerance {
                return false;
            }
        }
        true
    }

    // Вспомогательная функция для создания тестовых векторов
    fn test_hvec3(x: f32, y: f32, z: f32) -> HVec3 {
        HVec3::new(x, y, z, 1.0)
    }

    // --------------------------------------------------
    // Тесты базовых преобразований
    // --------------------------------------------------

    #[test]
    fn test_identity_matrix() {
        let identity = Transform3D::identity();
        let expected = Transform3D {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        };

        assert!(matrices_approx_equal(&identity, &expected, TOLERANCE));

        // Проверка, что единичная матрица не меняет вектор
        let test_vec = test_hvec3(1.0, 2.0, 3.0);
        let transformed = identity.apply_to_hvec(&test_vec);
        assert!(transformed.approx_equal(&test_vec, TOLERANCE));
    }

    #[test]
    fn test_translation() {
        // Тест перемещения с разными значениями по осям
        let translation = Transform3D::translation(2.0, 3.0, 4.0);
        let test_vec = test_hvec3(1.0, 1.0, 1.0);
        let transformed = translation.apply_to_hvec(&test_vec);
        let expected = test_hvec3(3.0, 4.0, 5.0); // (1+2, 1+3, 1+4)

        assert!(transformed.approx_equal(&expected, TOLERANCE));

        // Тест равномерного перемещения
        let uniform_translation = Transform3D::translation_uniform(5.0);
        let transformed_uniform = uniform_translation.apply_to_hvec(&test_vec);
        let expected_uniform = test_hvec3(6.0, 6.0, 6.0); // (1+5, 1+5, 1+5)

        assert!(transformed_uniform.approx_equal(&expected_uniform, TOLERANCE));
    }

    #[test]
    fn test_scale() {
        // Тест масштабирования с разными коэффициентами
        let scale = Transform3D::scale(2.0, 3.0, 4.0);
        let test_vec = test_hvec3(1.0, 2.0, 3.0);
        let transformed = scale.apply_to_hvec(&test_vec);
        let expected = test_hvec3(2.0, 6.0, 12.0); // (1*2, 2*3, 3*4)

        assert!(transformed.approx_equal(&expected, TOLERANCE));

        // Тест равномерного масштабирования
        let uniform_scale = Transform3D::scale_uniform(2.0);
        let transformed_uniform = uniform_scale.apply_to_hvec(&test_vec);
        let expected_uniform = test_hvec3(2.0, 4.0, 6.0); // (1*2, 2*2, 3*2)

        assert!(transformed_uniform.approx_equal(&expected_uniform, TOLERANCE));

        // Тест масштабирования нулевого вектора
        let zero_vec = test_hvec3(0.0, 0.0, 0.0);
        let transformed_zero = scale.apply_to_hvec(&zero_vec);
        assert!(transformed_zero.approx_equal(&zero_vec, TOLERANCE));
    }

    #[test]
    fn test_rotation_x() {
        // Поворот на 90 градусов вокруг оси X
        let rotation = Transform3D::rotation_x_deg(90.0);
        let test_vec = test_hvec3(0.0, 1.0, 0.0); // Вектор вдоль оси Y

        let transformed = rotation.apply_to_hvec(&test_vec);
        let expected = test_hvec3(0.0, 0.0, 1.0); // Должен стать вектором вдоль оси Z

        assert!(transformed.approx_equal(&expected, TOLERANCE));

        // Поворот на 180 градусов
        let rotation_180 = Transform3D::rotation_x_deg(180.0);
        let transformed_180 = rotation_180.apply_to_hvec(&test_vec);
        let expected_180 = test_hvec3(0.0, -1.0, 0.0); // Должен инвертироваться по Y

        assert!(transformed_180.approx_equal(&expected_180, TOLERANCE));
    }

    #[test]
    fn test_rotation_y() {
        // Поворот на 90 градусов вокруг оси Y
        let rotation = Transform3D::rotation_y_deg(90.0);
        let test_vec = test_hvec3(0.0, 0.0, 1.0); // Вектор вдоль оси Z

        let transformed = rotation.apply_to_hvec(&test_vec);
        let expected = test_hvec3(1.0, 0.0, 0.0); // Должен стать вектором вдоль оси X

        assert!(transformed.approx_equal(&expected, TOLERANCE));

        // Проверка радиан
        let rotation_rad = Transform3D::rotation_y_rad(std::f32::consts::PI / 2.0);
        let transformed_rad = rotation_rad.apply_to_hvec(&test_vec);
        assert!(transformed_rad.approx_equal(&expected, TOLERANCE));
    }

    #[test]
    fn test_rotation_z() {
        // Поворот на 90 градусов вокруг оси Z
        let rotation = Transform3D::rotation_z_deg(90.0);
        let test_vec = test_hvec3(1.0, 0.0, 0.0); // Вектор вдоль оси X

        let transformed = rotation.apply_to_hvec(&test_vec);
        let expected = test_hvec3(0.0, 1.0, 0.0); // Должен стать вектором вдоль оси Y

        assert!(transformed.approx_equal(&expected, TOLERANCE));
    }

    // --------------------------------------------------
    // Тесты составных преобразований
    // --------------------------------------------------

    #[test]
    fn test_scale_relative_to_point() {
        let anchor = Point3::new(1.0, 1.0, 1.0);
        let scale = Transform3D::scale_relative_to_point(anchor, 2.0, 2.0, 2.0);

        // Точка в центре масштабирования не должна измениться
        let center_vec = HVec3::from(anchor);
        let transformed_center = scale.apply_to_hvec(&center_vec);
        assert!(transformed_center.approx_equal(&center_vec, TOLERANCE));

        // Точка на расстоянии должна масштабироваться относительно центра
        let test_vec = test_hvec3(2.0, 2.0, 2.0); // Расстояние от центра (1,1,1)
        let transformed = scale.apply_to_hvec(&test_vec);
        let expected = test_hvec3(3.0, 3.0, 3.0); // Центр (1,1,1) + (1,1,1)*2 = (3,3,3)

        assert!(transformed.approx_equal(&expected, TOLERANCE));
    }

    #[test]
    fn test_reflections() {
        // Отражение относительно плоскости XY
        let reflection_xy = Transform3D::reflection_xy();
        let test_vec = test_hvec3(1.0, 2.0, 3.0);
        let transformed_xy = reflection_xy.apply_to_hvec(&test_vec);
        let expected_xy = test_hvec3(1.0, 2.0, -3.0);
        assert!(transformed_xy.approx_equal(&expected_xy, TOLERANCE));

        // Отражение относительно плоскости XZ
        let reflection_xz = Transform3D::reflection_xz();
        let transformed_xz = reflection_xz.apply_to_hvec(&test_vec);
        let expected_xz = test_hvec3(1.0, -2.0, 3.0);
        assert!(transformed_xz.approx_equal(&expected_xz, TOLERANCE));

        // Отражение относительно плоскости YZ
        let reflection_yz = Transform3D::reflection_yz();
        let transformed_yz = reflection_yz.apply_to_hvec(&test_vec);
        let expected_yz = test_hvec3(-1.0, 2.0, 3.0);
        assert!(transformed_yz.approx_equal(&expected_yz, TOLERANCE));
    }

    #[test]
    fn test_reflection_plane() {
        // Отражение относительно плоскости YZ (x=0)
        let plane_yz = Plane::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let reflection = Transform3D::reflection_plane(plane_yz);

        let test_vec = test_hvec3(2.0, 3.0, 4.0);
        let transformed = reflection.apply_to_hvec(&test_vec);
        let expected = test_hvec3(-2.0, 3.0, 4.0); // x меняет знак

        assert!(transformed.approx_equal(&expected, TOLERANCE));

        // Отражение относительно плоскости XZ (y=0)
        let plane_xz = Plane::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let reflection_xz = Transform3D::reflection_plane(plane_xz);

        let transformed_xz = reflection_xz.apply_to_hvec(&test_vec);
        let expected_xz = test_hvec3(2.0, -3.0, 4.0); // y меняет знак

        assert!(transformed_xz.approx_equal(&expected_xz, TOLERANCE));
    }

    #[test]
    fn test_rotation_around_line() {
        // Поворот вокруг оси X (должен совпадать с rotation_x)
        let x_axis = Line3::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let rotation = Transform3D::rotation_around_line(x_axis, 90.0_f32.to_radians());

        let test_vec = test_hvec3(0.0, 1.0, 0.0);
        let transformed = rotation.apply_to_hvec(&test_vec);
        let expected = test_hvec3(0.0, 0.0, 1.0);

        assert!(transformed.approx_equal(&expected, TOLERANCE));

        // Поворот вокруг произвольной линии
        let custom_axis = Line3::new(Point3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 0.0));
        let custom_rotation =
            Transform3D::rotation_around_line(custom_axis, 180.0_f32.to_radians());

        // Точка на оси не должна измениться
        let point_on_axis = HVec3::from(Point3::new(1.0, 2.0, 1.0));
        let transformed_axis = custom_rotation.apply_to_hvec(&point_on_axis);
        assert!(transformed_axis.approx_equal(&point_on_axis, TOLERANCE));
    }

    // --------------------------------------------------
    // Тесты произведения матриц и композиции преобразований
    // --------------------------------------------------

    #[test]
    fn test_matrix_multiplication() {
        let translation = Transform3D::translation(1.0, 2.0, 3.0);
        let scale = Transform3D::scale(2.0, 2.0, 2.0);

        // Умножение матриц
        let combined = translation.multiply(scale);

        // Проверка с помощью оператора *
        let combined_op = translation * scale;

        assert!(matrices_approx_equal(&combined, &combined_op, TOLERANCE));

        // Проверка последовательности преобразований
        let test_vec = test_hvec3(1.0, 1.0, 1.0);

        // Сначала масштабирование, потом перемещение
        let scaled_then_translated = translation.apply_to_hvec(&scale.apply_to_hvec(&test_vec));
        // Комбинированное преобразование
        let combined_result = combined.apply_to_hvec(&test_vec);

        assert!(scaled_then_translated.approx_equal(&combined_result, TOLERANCE));
    }

    #[test]
    fn test_transpose() {
        let original = Transform3D {
            m: [
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0,
            ],
        };

        let transposed = original.transpose();
        let expected = Transform3D {
            m: [
                1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0, 4.0, 8.0, 12.0,
                16.0,
            ],
        };

        assert!(matrices_approx_equal(&transposed, &expected, TOLERANCE));

        // Двойная транспонизация должна вернуть исходную матрицу
        let double_transposed = transposed.transpose();
        assert!(matrices_approx_equal(
            &double_transposed,
            &original,
            TOLERANCE
        ));
    }

    #[test]
    fn test_inverse() {
        // Тест обратной матрицы для перемещения
        let translation = Transform3D::translation(2.0, 3.0, 4.0);
        let inverse_translation = translation.inverse().expect("Should have inverse");

        let test_vec = test_hvec3(1.0, 2.0, 3.0);
        let transformed = translation.apply_to_hvec(&test_vec);
        let restored = inverse_translation.apply_to_hvec(&transformed);

        assert!(restored.approx_equal(&test_vec, TOLERANCE));

        // Тест обратной матрицы для масштабирования
        let scale = Transform3D::scale(2.0, 3.0, 4.0);
        let inverse_scale = scale.inverse().expect("Should have inverse");

        let scaled = scale.apply_to_hvec(&test_vec);
        let restored_scale = inverse_scale.apply_to_hvec(&scaled);

        assert!(restored_scale.approx_equal(&test_vec, TOLERANCE));

        // Тест обратной матрицы для поворота
        let rotation = Transform3D::rotation_x_deg(45.0);
        let inverse_rotation = rotation.inverse().expect("Should have inverse");

        let rotated = rotation.apply_to_hvec(&test_vec);
        let restored_rotation = inverse_rotation.apply_to_hvec(&rotated);

        assert!(restored_rotation.approx_equal(&test_vec, TOLERANCE));

        // Тест с вырожденной матрицей (масштабирование с нулевым коэффициентом)
        let degenerate_scale = Transform3D::scale(0.0, 1.0, 1.0);
        assert!(degenerate_scale.inverse().is_none());
    }

    #[test]
    fn test_determinant() {
        // Определитель единичной матрицы должен быть 1
        let identity = Transform3D::identity();
        assert!((identity.determinant() - 1.0).abs() < TOLERANCE);

        // Определитель матрицы масштабирования
        let scale = Transform3D::scale(2.0, 3.0, 4.0);
        assert!((scale.determinant() - 24.0).abs() < TOLERANCE); // 2 * 3 * 4

        // Определитель матрицы перемещения должен быть 1 (афинное преобразование сохраняет объем)
        let translation = Transform3D::translation(1.0, 2.0, 3.0);
        assert!((translation.determinant() - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_complex_transformation_sequence() {
        // Комплексная последовательность преобразований: поворот -> масштабирование -> перемещение
        let rotation = Transform3D::rotation_y_deg(90.0);
        let scale = Transform3D::scale(2.0, 1.0, 1.0);
        let translation = Transform3D::translation(5.0, 0.0, 0.0);

        let combined = translation * scale * rotation;

        let test_vec = test_hvec3(0.0, 0.0, 1.0); // Вектор вдоль оси Z

        // Ожидаемый результат:
        // 1. Поворот на 90° вокруг Y: (0,0,1) -> (1,0,0)
        // 2. Масштабирование по X в 2 раза: (1,0,0) -> (2,0,0)
        // 3. Перемещение на (5,0,0): (2,0,0) -> (7,0,0)
        let expected = test_hvec3(7.0, 0.0, 0.0);
        let result = combined.apply_to_hvec(&test_vec);

        assert!(result.approx_equal(&expected, TOLERANCE));
    }

    #[test]
    fn test_hvec3_apply_transform() {
        let mut test_vec = test_hvec3(1.0, 2.0, 3.0);
        let translation = Transform3D::translation(1.0, 1.0, 1.0);

        test_vec.apply_transform(&translation);

        let expected = test_hvec3(2.0, 3.0, 4.0);
        assert!(test_vec.approx_equal(&expected, TOLERANCE));
    }
}
