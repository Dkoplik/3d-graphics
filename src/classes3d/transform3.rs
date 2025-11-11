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
        #[cfg(debug_assertions)]
        if dx == 0.0 && dy == 0.0 && dz == 0.0 {
            eprintln!(
                "Warning: получена матрица смещения на 0.0 по всем осям, так и было задумано?"
            );
        }

        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, // первая строка
                0.0, 1.0, 0.0, 0.0, // вторая строка
                0.0, 0.0, 1.0, 0.0, // третья строка
                dx, dy, dz, 1.0, // перемещение (4-я строка)
            ],
        }
    }

    /// Создает матрицу перемещения на основе вектора смещения.
    pub fn translation_vec(delta: Vec3) -> Self {
        Self::translation(delta.x, delta.y, delta.z)
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
        #[cfg(debug_assertions)]
        if sx == 0.0 && sy == 0.0 && sz == 0.0 {
            eprintln!(
                "Warning: получена матрица масштабирования на 0.0 по всем осям, так и было задумано?"
            );
        }

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

    /// Создает матрицу поворота вокруг произвольной оси, но эта ось проходит через начало коодринат.
    pub fn rotation_around_axis(axis: Vec3, angle: f32) -> Self {
        let axis = axis.normalize();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let one_minus_cos = 1.0 - cos_a;

        let x = axis.x;
        let y = axis.y;
        let z = axis.z;

        Self {
            m: [
                cos_a + x * x * one_minus_cos,
                x * y * one_minus_cos + z * sin_a,
                x * z * one_minus_cos - y * sin_a,
                0.0,
                x * y * one_minus_cos - z * sin_a,
                cos_a + y * y * one_minus_cos,
                y * z * one_minus_cos + x * sin_a,
                0.0,
                x * z * one_minus_cos + y * sin_a,
                y * z * one_minus_cos - x * sin_a,
                cos_a + z * z * one_minus_cos,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }
}

// --------------------------------------------------
// Конструкторы составных преобразований
// --------------------------------------------------

impl Transform3D {
    /// Масштабирование относительно точки anchor.
    pub fn scale_relative_to_point(anchor: Point3, sx: f32, sy: f32, sz: f32) -> Self {
        // Перенос якоря в начало координат -> масштабирование -> обратный перенос
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

    /// Создаёт матрицу поворота, которая совмещает вектор `from` с вектором `to`.
    ///
    /// Оба вектора должны быть нормализованы (иметь длину 1).
    pub fn rotation_aligning(from: Vec3, to: Vec3) -> Self {
        debug_assert!(
            (from.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Вектор from {:?} должен быть нормализован, но он имеет длину {}",
            from,
            from.length(),
        );
        debug_assert!(
            (to.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Вектор to {:?} должен быть нормализован, но он имеет длину {}",
            to,
            to.length(),
        );

        if from.approx_equal(to, 1e-9) {
            return Self::identity();
        }

        let cos_angle = from.dot(to);

        // Для численной стабильности
        if cos_angle >= 1.0 - 1e-7 {
            return Self::identity();
        }
        if cos_angle <= -1.0 + 1e-7 {
            // 180 градусов - находим ортогональную ось
            let mut axis = from.cross_right(Vec3::plus_x());
            if axis.length_squared() < 1e-9 {
                axis = from.cross_right(Vec3::plus_y());
            }
            axis = axis.normalize();

            return Self {
                m: [
                    2.0 * axis.x * axis.x - 1.0,
                    2.0 * axis.x * axis.y,
                    2.0 * axis.x * axis.z,
                    0.0,
                    2.0 * axis.x * axis.y,
                    2.0 * axis.y * axis.y - 1.0,
                    2.0 * axis.y * axis.z,
                    0.0,
                    2.0 * axis.x * axis.z,
                    2.0 * axis.y * axis.z,
                    2.0 * axis.z * axis.z - 1.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ],
            };
        }

        let axis = from.cross_right(to).normalize();
        let angle = cos_angle.acos();

        // Используем существующий метод вращения вокруг оси
        Self::rotation_around_axis(axis, angle)
    }

    /// Отражение относительно произвольной плоскости.
    pub fn reflection_plane(plane: Plane) -> Self {
        // 1. Переносим плоскость в начало координат
        let to_origin = Self::translation(-plane.origin.x, -plane.origin.y, -plane.origin.z);

        // 2. Совмещаем нормаль плоскости с осью Z
        let align_normal = Self::rotation_aligning(plane.normal, Vec3::new(0.0, 0.0, 1.0));

        // 3. Отражаем относительно плоскости XY
        let reflect = Self::scale(1.0, 1.0, -1.0);

        // 4. Обратные преобразования
        let inverse_align = align_normal
            .inverse()
            .expect("Для align всегда должна существовать обратная матрица");
        let from_origin = Self::translation(plane.origin.x, plane.origin.y, plane.origin.z);

        // Композиция операций
        to_origin
            .multiply(align_normal)
            .multiply(reflect)
            .multiply(inverse_align)
            .multiply(from_origin)
    }

    /// Поворот вокруг произвольной линии (оси).
    pub fn rotation_around_line(line: Line3, angle_rad: f32) -> Self {
        // 1. Переносим линию в начало координат
        let to_origin = Self::translation(-line.origin.x, -line.origin.y, -line.origin.z);

        // 2. Вращаем вокруг заданой оси.
        let rotate = Self::rotation_around_axis(line.direction, angle_rad);

        // 3. Обратное преобразование
        let from_origin = Self::translation(line.origin.x, line.origin.y, line.origin.z);

        // Композиция операций
        to_origin.multiply(rotate).multiply(from_origin)
    }
}

// --------------------------------------------------
// Конструкторы проекций
// --------------------------------------------------

impl Transform3D {
    /// Создаёт матрицу параллельной проекции (ортографической проекции).
    ///
    /// Параметры определяют объем усеченной пирамиды (viewing frustum):
    /// - `left`, `right`: координаты левой и правой плоскостей отсечения
    /// - `bottom`, `top`: координаты нижней и верхней плоскостей отсечения  
    /// - `near`, `far`: расстояние до ближней и дальней плоскостей отсечения
    ///
    /// После применения этой матрицы, объем преобразуется в нормализованный куб [-1, 1]^3.
    pub fn parallel(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        debug_assert!(
            left < right,
            "левая граница {} должна быть меньше правой {}",
            left,
            right
        );
        debug_assert!(
            bottom < top,
            "нижняя граница {} должна быть меньше верхней {}",
            bottom,
            top
        );
        debug_assert!(
            near > 0.0,
            "ближняя граница {} не может быть отрицательной",
            near
        );
        debug_assert!(
            near < far,
            "ближняя граница {} должна быть меньше дальней {}",
            near,
            far
        );

        let width = right - left;
        let height = top - bottom;
        let depth = far - near;

        Self {
            m: [
                2.0 / width,
                0.0,
                0.0,
                0.0, // первая строка
                0.0,
                2.0 / height,
                0.0,
                0.0, // вторая строка
                0.0,
                0.0,
                2.0 / depth,
                0.0, // третья строка
                -(right + left) / width,
                -(top + bottom) / height,
                (far + near) / depth,
                1.0,
            ],
        }
    }

    /// Создаёт симметричную матрицу параллельной проекции.
    ///
    /// Упрощенный вариант, где left = -right, bottom = -top.
    /// - `width`: полная ширина объема (right - left)
    /// - `height`: полная высота объема (top - bottom)  
    /// - `near`, `far`: координаты ближней и дальней плоскостей отсечения
    pub fn parallel_symmetric(width: f32, height: f32, near: f32, far: f32) -> Self {
        let half_width = width * 0.5;
        let half_height = height * 0.5;

        Self::parallel(
            -half_width,
            half_width,
            -half_height,
            half_height,
            near,
            far,
        )
    }

    /// Создает матрицу перспективной проекции (исправленная версия).
    ///
    /// Параметры:
    /// - `fov_rad`: вертикальный угол обзора в радианах
    /// - `aspect`: соотношение сторон (width / height)
    /// - `near`: расстояние до ближней плоскости отсечения (должно быть > 0)
    /// - `far`: расстояние до дальней плоскости отсечения (должно быть > near)
    ///
    /// После применения этой матрицы, координаты преобразуются в NDC.
    pub fn perspective(fov_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        debug_assert!(
            near > 0.0,
            "ближняя граница {} не может быть отрицательной",
            near
        );
        debug_assert!(
            near < far,
            "ближняя граница {} должна быть меньше дальней {}",
            near,
            far
        );
        debug_assert!(near < far);
        debug_assert!(
            aspect > 0.0,
            "соотношение сторон {} не может быть отрицательным",
            aspect
        );

        let tan = (fov_rad / 2.0).tan(); // ~ t / n
        let diff = far - near;

        Self {
            m: [
                1.0 / (aspect * tan),
                0.0,
                0.0,
                0.0, // первая строка
                0.0,
                1.0 / tan,
                0.0,
                0.0, // вторая строка
                0.0,
                0.0,
                (far + near) / diff,
                -1.0, // третья строка
                0.0,
                0.0,
                (2.0 * far * near) / diff,
                0.0, // четвертая строка
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
    ///
    /// При композиции двух матриц, правая (`other`) применяется **после** левой.
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
    pub fn apply_to_hvec(self, hvec: HVec3) -> HVec3 {
        //                      ^
        // лучше копировать, ибо вектор небольшой, а обращений много, из-за ссылка будет долгой

        /*
        x_new = x * m11 + y * m21 + z * m31 + w * m41
        y_new = x * m12 + y * m22 + z * m32 + w * m42
        z_new = x * m13 + y * m23 + z * m33 + w * m43
        w_new = x * m14 + y * m24 + z * m34 + w * m44
        */
        HVec3::new_full(
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
    pub fn determinant(self) -> f32 {
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

    /// Композиция 2-х матриц.
    ///
    /// При композиции, правая матрица `rhs` применяется **после** левой.
    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Line3, Plane, Point3, Vec3};

    const TOLERANCE: f32 = 1e-6;

    fn assert_floats(got: f32, expected: f32, tolerance: f32) {
        assert!(
            (got - expected).abs() <= tolerance,
            "ожидалось число {}, но получено {}, которое отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn assert_hvecs(got: HVec3, expected: HVec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    // --------------------------------------------------
    // Тесты базовых преобразований
    // --------------------------------------------------

    #[test]
    fn test_identity_matrix() {
        let identity = Transform3D::identity();

        // Проверка, что единичная матрица не меняет вектор
        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed = identity.apply_to_hvec(test_vec);
        assert_hvecs(transformed, test_vec, TOLERANCE);
    }

    #[test]
    fn test_translation() {
        // Тест перемещения с разными значениями по осям
        let translation = Transform3D::translation(2.0, 3.0, 4.0);
        let test_vec = HVec3::new(1.0, 1.0, 1.0);
        let transformed = translation.apply_to_hvec(test_vec);
        let expected = HVec3::new(3.0, 4.0, 5.0); // (1+2, 1+3, 1+4)

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_translation_vec() {
        // Тест перемещения с разными значениями по осям через вектор
        let delta = Vec3::new(2.0, 3.0, 4.0);
        let translation = Transform3D::translation_vec(delta);
        let test_vec = HVec3::new(1.0, 1.0, 1.0);
        let transformed = translation.apply_to_hvec(test_vec);
        let expected = HVec3::new(3.0, 4.0, 5.0); // (1+2, 1+3, 1+4)

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_translation_uniform() {
        // Тест равномерного перемещения
        let test_vec = HVec3::new(1.0, 1.0, 1.0);
        let uniform_translation = Transform3D::translation_uniform(5.0);
        let transformed_uniform = uniform_translation.apply_to_hvec(test_vec);
        let expected_uniform = HVec3::new(6.0, 6.0, 6.0); // (1+5, 1+5, 1+5)

        assert_hvecs(transformed_uniform, expected_uniform, TOLERANCE);
    }

    #[test]
    fn test_scale() {
        // Тест масштабирования с разными коэффициентами
        let scale = Transform3D::scale(2.0, 3.0, 4.0);
        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed = scale.apply_to_hvec(test_vec);
        let expected = HVec3::new(2.0, 6.0, 12.0); // (1*2, 2*3, 3*4)

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_scale_uniform() {
        // Тест равномерного масштабирования
        let uniform_scale = Transform3D::scale_uniform(2.0);
        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed_uniform = uniform_scale.apply_to_hvec(test_vec);
        let expected_uniform = HVec3::new(2.0, 4.0, 6.0); // (1*2, 2*2, 3*2)

        assert_hvecs(transformed_uniform, expected_uniform, TOLERANCE);
    }

    #[test]
    fn test_scale_zero_vector() {
        // Тест масштабирования нулевого вектора
        let scale = Transform3D::scale(2.0, 3.0, 4.0);
        let zero_vec = HVec3::zero();
        let transformed_zero = scale.apply_to_hvec(zero_vec);
        assert_hvecs(transformed_zero, zero_vec, TOLERANCE);
    }

    #[test]
    fn test_rotation_x_90() {
        // Поворот на 90 градусов вокруг оси X
        let rotation = Transform3D::rotation_x_deg(90.0);
        let test_vec = HVec3::forward();

        let transformed = rotation.apply_to_hvec(test_vec);
        let expected = HVec3::down();

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_rotation_x_180() {
        // Поворот на 180 градусов вокруг оси X
        let rotation = Transform3D::rotation_x_deg(180.0);
        let test_vec = HVec3::forward();

        let transformed = rotation.apply_to_hvec(test_vec);
        let expected = HVec3::backward();

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_rotation_x_360() {
        // Поворот на 360 градусов вокруг оси X
        let rotation = Transform3D::rotation_x_deg(360.0);
        let test_vec = HVec3::forward();

        let transformed = rotation.apply_to_hvec(test_vec);

        assert_hvecs(transformed, test_vec, TOLERANCE);
    }

    #[test]
    fn test_rotation_y_90() {
        // Поворот на 90 градусов вокруг оси Y
        let rotation = Transform3D::rotation_y_deg(90.0);
        let test_vec = HVec3::forward();

        let transformed = rotation.apply_to_hvec(test_vec);
        let expected = HVec3::right();

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_rotation_y_180() {
        // Поворот на 180 градусов вокруг оси Y
        let rotation = Transform3D::rotation_y_deg(180.0);
        let test_vec = HVec3::forward();

        let transformed = rotation.apply_to_hvec(test_vec);
        let expected = HVec3::backward();

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_rotation_y_360() {
        // Поворот на 360 градусов вокруг оси Y
        let rotation = Transform3D::rotation_y_deg(360.0);
        let test_vec = HVec3::forward();

        let transformed = rotation.apply_to_hvec(test_vec);

        assert_hvecs(transformed, test_vec, TOLERANCE);
    }

    #[test]
    fn test_rotation_z_90() {
        // Поворот на 90 градусов вокруг оси Z
        let rotation = Transform3D::rotation_z_deg(90.0);
        let test_vec = HVec3::right();

        let transformed = rotation.apply_to_hvec(test_vec);
        let expected = HVec3::up();

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_rotation_z_180() {
        // Поворот на 180 градусов вокруг оси Z
        let rotation = Transform3D::rotation_z_deg(180.0);
        let test_vec = HVec3::right();

        let transformed = rotation.apply_to_hvec(test_vec);
        let expected = HVec3::left();

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_rotation_z_360() {
        // Поворот на 360 градусов вокруг оси Z
        let rotation = Transform3D::rotation_z_deg(360.0);
        let test_vec = HVec3::right();

        let transformed = rotation.apply_to_hvec(test_vec);

        assert_hvecs(transformed, test_vec, TOLERANCE);
    }

    // --------------------------------------------------
    // Тесты составных преобразований
    // --------------------------------------------------

    #[test]
    fn test_scale_anchor() {
        let anchor = Point3::new(1.0, 1.0, 1.0);
        let scale = Transform3D::scale_relative_to_point(anchor, 2.0, 2.0, 2.0);

        // Точка в центре масштабирования не должна измениться
        let center_vec = HVec3::from(anchor);
        let transformed_center = scale.apply_to_hvec(center_vec);
        assert_hvecs(transformed_center, center_vec, TOLERANCE);
    }

    #[test]
    fn test_scale_relative_to_point() {
        let anchor = Point3::new(1.0, 1.0, 1.0);
        let scale = Transform3D::scale_relative_to_point(anchor, 2.0, 2.0, 2.0);

        // Точка на расстоянии должна масштабироваться относительно центра
        let test_vec = HVec3::new(2.0, 2.0, 2.0); // Расстояние от центра (1,1,1)
        let transformed = scale.apply_to_hvec(test_vec);
        let expected = HVec3::new(3.0, 3.0, 3.0); // Центр (1,1,1) + (1,1,1)*2 = (3,3,3)

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_reflection_xy() {
        // Отражение относительно плоскости XY
        let reflection_xy = Transform3D::reflection_xy();
        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed_xy = reflection_xy.apply_to_hvec(test_vec);
        let expected_xy = HVec3::new(1.0, 2.0, -3.0);
        assert_hvecs(transformed_xy, expected_xy, TOLERANCE);
    }

    #[test]
    fn test_reflection_xz() {
        // Отражение относительно плоскости XZ
        let reflection_xz = Transform3D::reflection_xz();
        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed_xz = reflection_xz.apply_to_hvec(test_vec);
        let expected_xz = HVec3::new(1.0, -2.0, 3.0);
        assert_hvecs(transformed_xz, expected_xz, TOLERANCE);
    }

    #[test]
    fn test_reflection_yz() {
        // Отражение относительно плоскости YZ
        let reflection_yz = Transform3D::reflection_yz();
        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed_yz = reflection_yz.apply_to_hvec(test_vec);
        let expected_yz = HVec3::new(-1.0, 2.0, 3.0);
        assert_hvecs(transformed_yz, expected_yz, TOLERANCE);
    }

    #[test]
    fn test_rotation_aligning_up_to_forward() {
        let from = Vec3::up();
        let to = Vec3::forward();

        let transform = Transform3D::rotation_aligning(from, to);
        let transformed = transform.apply_to_hvec(from.into());
        assert_hvecs(transformed, to.into(), TOLERANCE);
    }

    #[test]
    fn test_rotation_aligning_backward_to_forward() {
        let from = Vec3::backward();
        let to = Vec3::forward();

        let transform = Transform3D::rotation_aligning(from, to);
        let transformed = transform.apply_to_hvec(from.into());
        assert_hvecs(transformed, to.into(), TOLERANCE);
    }

    #[test]
    fn test_rotation_aligning_up_to_left() {
        let from = Vec3::up();
        let to = Vec3::left();

        let transform = Transform3D::rotation_aligning(from, to);
        let transformed = transform.apply_to_hvec(from.into());
        assert_hvecs(transformed, to.into(), TOLERANCE);
    }

    #[test]
    fn test_rotation_aligning_forward_to_right() {
        let from = Vec3::forward();
        let to = Vec3::right();

        let transform = Transform3D::rotation_aligning(from, to);
        let transformed = transform.apply_to_hvec(from.into());
        assert_hvecs(transformed, to.into(), TOLERANCE);

        // 3-ий вектор
        let vec = Vec3::right();
        // если "вперёд" было повёрнуто направо, то "право" должно стать "назад"
        let transformed = transform.apply_to_hvec(vec.into());
        assert_hvecs(transformed, HVec3::backward(), TOLERANCE);
    }

    #[test]
    fn test_rotation_aligning_complex_to_basis() {
        let from = Vec3::new(2.0, 3.0, 4.0).normalize();
        let to = Vec3::left();

        let transform = Transform3D::rotation_aligning(from, to);
        let transformed = transform.apply_to_hvec(from.into());
        assert_hvecs(transformed, to.into(), TOLERANCE);
    }

    #[test]
    fn test_rotation_aligning_basis_to_complex() {
        let from = Vec3::left();
        let to = Vec3::new(2.0, 3.0, 4.0).normalize();

        let transform = Transform3D::rotation_aligning(from, to);
        let transformed = transform.apply_to_hvec(from.into());
        assert_hvecs(transformed, to.into(), TOLERANCE);
    }

    #[test]
    fn test_rotation_aligning_complex_to_complex() {
        let from = Vec3::new(2.0, 3.0, 4.0).normalize();
        let to = Vec3::new(-6.0, 3.0, -1.0).normalize();

        let transform = Transform3D::rotation_aligning(from, to);
        let transformed = transform.apply_to_hvec(from.into());
        assert_hvecs(transformed, to.into(), TOLERANCE);
    }

    #[test]
    fn test_reflection_plane_xy() {
        // Отражение относительно плоскости XY
        let plane_yz = Plane::new(Point3::new(0.0, 0.0, 0.0), Vec3::plus_z());
        let reflection = Transform3D::reflection_plane(plane_yz);

        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed_xy = reflection.apply_to_hvec(test_vec);
        let expected_xy = HVec3::new(1.0, 2.0, -3.0);
        assert_hvecs(transformed_xy, expected_xy, TOLERANCE);
    }

    #[test]
    fn test_reflection_plane_xz() {
        // Отражение относительно плоскости XZ
        let plane_yz = Plane::new(Point3::new(0.0, 0.0, 0.0), Vec3::plus_y());
        let reflection = Transform3D::reflection_plane(plane_yz);

        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed_xz = reflection.apply_to_hvec(test_vec);
        let expected_xz = HVec3::new(1.0, -2.0, 3.0);
        assert_hvecs(transformed_xz, expected_xz, TOLERANCE);
    }

    #[test]
    fn test_reflection_plane_yz() {
        // Отражение относительно плоскости YZ
        let plane_yz = Plane::new(Point3::new(0.0, 0.0, 0.0), Vec3::plus_x());
        let reflection = Transform3D::reflection_plane(plane_yz);

        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed_yz = reflection.apply_to_hvec(test_vec);
        let expected_yz = HVec3::new(-1.0, 2.0, 3.0);
        assert_hvecs(transformed_yz, expected_yz, TOLERANCE);
    }

    #[test]
    fn test_rotation_around_line_x_90() {
        // Поворот вокруг оси X (должен совпадать с rotation_x)
        let x_axis = Line3::new(Point3::new(0.0, 0.0, 0.0), Vec3::forward());
        let rotation = Transform3D::rotation_around_line(x_axis, (90.0 as f32).to_radians());
        let test_vec = HVec3::right();

        let transformed = rotation.apply_to_hvec(test_vec);
        let expected = HVec3::up();

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_rotation_around_line_y_90() {
        // Поворот вокруг оси Y (должен совпадать с rotation_y)
        let y_axis = Line3::new(Point3::new(0.0, 0.0, 0.0), Vec3::up());
        let rotation = Transform3D::rotation_around_line(y_axis, (90.0 as f32).to_radians());
        let test_vec = HVec3::forward();

        let transformed = rotation.apply_to_hvec(test_vec);
        let expected = HVec3::right();

        assert_hvecs(transformed, expected, TOLERANCE);
    }

    #[test]
    fn test_rotation_around_line_z_90() {
        // Поворот вокруг оси X (должен совпадать с rotation_x)
        let z_axis = Line3::new(Point3::new(0.0, 0.0, 0.0), Vec3::forward());
        let rotation = Transform3D::rotation_around_line(z_axis, (90.0 as f32).to_radians());
        let test_vec = HVec3::up();

        let transformed = rotation.apply_to_hvec(test_vec);
        let expected = HVec3::left();

        assert_hvecs(transformed, expected, TOLERANCE);
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

        // Проверка последовательности преобразований
        let test_vec = HVec3::new(1.0, 1.0, 1.0);

        // Сначала перемещение, потом масштабирование
        let scaled_then_translated = scale.apply_to_hvec(translation.apply_to_hvec(test_vec));
        // Комбинированное преобразование
        let combined_result = combined.apply_to_hvec(test_vec);

        assert_hvecs(scaled_then_translated, combined_result, TOLERANCE);
    }

    #[test]
    fn test_inverse_identity() {
        // Тест обратной матрицы для тождественной.
        let inv_identity = Transform3D::identity()
            .inverse()
            .expect("У тождественной матрицы есть обратная - она же сама");

        // Проверка, что единичная матрица не меняет вектор
        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed = inv_identity.apply_to_hvec(test_vec);
        assert_hvecs(transformed, test_vec, TOLERANCE);
    }

    #[test]
    fn test_inverse_translation() {
        // Тест обратной матрицы для перемещения
        let translation = Transform3D::translation(2.0, 3.0, 4.0);
        let inverse_translation = translation.inverse().expect("Should have inverse");

        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let transformed = translation.apply_to_hvec(test_vec);
        let restored = inverse_translation.apply_to_hvec(transformed);

        assert_hvecs(restored, test_vec, TOLERANCE);
    }

    #[test]
    fn test_inverse_scale() {
        // Тест обратной матрицы для масштабирования
        let scale = Transform3D::scale(2.0, 3.0, 4.0);
        let inverse_scale = scale.inverse().expect("Should have inverse");

        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let scaled = scale.apply_to_hvec(test_vec);
        let restored_scale = inverse_scale.apply_to_hvec(scaled);

        assert_hvecs(restored_scale, test_vec, TOLERANCE);
    }

    #[test]
    fn test_inverse_rotation() {
        // Тест обратной матрицы для поворота
        let rotation = Transform3D::rotation_x_deg(45.0);
        let inverse_rotation = rotation.inverse().expect("Should have inverse");

        let test_vec = HVec3::new(1.0, 2.0, 3.0);
        let rotated = rotation.apply_to_hvec(test_vec);
        let restored_rotation = inverse_rotation.apply_to_hvec(rotated);

        assert_hvecs(restored_rotation, test_vec, TOLERANCE);
    }

    #[test]
    fn test_determinant() {
        // Определитель единичной матрицы должен быть 1
        let identity = Transform3D::identity();
        assert_floats(identity.determinant(), 1.0, TOLERANCE);

        // Определитель матрицы масштабирования
        let scale = Transform3D::scale(2.0, 3.0, 4.0);
        assert_floats(scale.determinant(), 24.0, TOLERANCE); // 2 * 3 * 4

        // Определитель матрицы перемещения должен быть 1 (афинное преобразование сохраняет объем)
        let translation = Transform3D::translation(1.0, 2.0, 3.0);
        assert_floats(translation.determinant(), 1.0, TOLERANCE);
    }
}
