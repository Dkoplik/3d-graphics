use crate::{CoordFrame, Point3, Transform3D, Vec3};

impl CoordFrame {
    /// Создать новую координатную систему по 3-м базисам и точке, из которой эти базисы выходят.
    /// Базисы должны быть **ортонормированными**.
    pub fn new(x: Vec3, y: Vec3, z: Vec3, origin: Point3) -> Self {
        debug_assert!(
            (x.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Базис x длиной {} должен быть нормирован",
            x.length()
        );
        debug_assert!(
            (y.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Базис y длиной {} должен быть нормирован",
            y.length()
        );
        debug_assert!(
            (z.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Базис z длиной {} должен быть нормирован",
            z.length()
        );

        debug_assert!(
            x.dot(y).abs() < 2.0 * f32::EPSILON,
            "Базисы x {:?} и y{:?} должны быть ортогональными, но их произведение равно {}",
            x,
            y,
            x.dot(y)
        );
        debug_assert!(
            x.dot(z).abs() < 2.0 * f32::EPSILON,
            "Базисы x {:?} и z{:?} должны быть ортогональными, но их произведение равно {}",
            x,
            z,
            x.dot(z)
        );
        debug_assert!(
            y.dot(z).abs() < 2.0 * f32::EPSILON,
            "Базисы y {:?} и z{:?} должны быть ортогональными, но их произведение равно {}",
            y,
            z,
            y.dot(z)
        );

        Self { x, y, z, origin }
    }

    /// Создать новую координатную систему по 2-м векторам и точке.
    ///
    /// 3-ий вектор строится автоматически перпендикулярно 2-м указанным. 2 заданных вектора
    /// должны быть **ортонормированными**.
    pub fn from_2(x: Vec3, y: Vec3, origin: Point3) -> Self {
        debug_assert!(
            (x.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Базис x длиной {} должен быть нормирован",
            x.length()
        );
        debug_assert!(
            (y.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Базис y длиной {} должен быть нормирован",
            y.length()
        );
        debug_assert!(
            x.dot(y).abs() < 2.0 * f32::EPSILON,
            "Базисы x {:?} и y{:?} должны быть ортогональными, но их произведение равно {}",
            x,
            y,
            x.dot(y)
        );

        let z = x.cross(y).normalize();
        Self::new(x, y, z, origin)
    }

    /// Создать координатную систему, идентичную глобальной.
    pub fn global() -> Self {
        Self::new(
            Vec3::plus_x(),
            Vec3::plus_y(),
            Vec3::plus_z(),
            Point3::zero(),
        )
    }

    /// Получить матрицу преобразования из текущих локальных координат в глобальные.
    ///
    /// Эта матрица преобразует вектор из локальной системы координат в глобальную.
    pub fn local_to_global_matrix(&self) -> Transform3D {
        // Сдвигаем координаты к глобальным
        let translate = Transform3D::translation_vec(-Vec3::from(self.origin));

        // Выравниваем локальную ось X с глобальной осью X
        let align_x = Transform3D::rotation_aligning(self.x, Vec3::plus_x());

        // После выравнивания X надо выровнять Y
        let transformed_y = Vec3::from(align_x.apply_to_hvec(self.y.into()));
        let align_y = Transform3D::rotation_aligning(transformed_y, Vec3::plus_y());

        // Композиция операций
        translate.multiply(align_x).multiply(align_y)
    }

    /// Получить матрицу преобразования из глобальных координат в текущие локальные.
    ///
    /// Эта матрица преобразует вектор из глобальной системы координат в локальную.
    pub fn global_to_local_matrix(&self) -> Transform3D {
        // Выравниваем ось X с локальной
        let align_x = Transform3D::rotation_aligning(self.x, Vec3::plus_x());

        // После выравнивания X надо выровнять Y
        let transformed_y = Vec3::from(align_x.apply_to_hvec(self.y.into()));
        let align_y = Transform3D::rotation_aligning(transformed_y, Vec3::plus_y());

        // Сдвигаем координаты к локальным
        let translate = Transform3D::translation_vec(-Vec3::from(self.origin));

        // Композиция операций
        translate.multiply(align_x).multiply(align_y)
    }

    /// Получить матрицу преобразования из текущих локальных координат в другие локальные координаты.
    pub fn local_to_other_local_matrix(&self, other: &CoordFrame) -> Transform3D {
        self.local_to_global_matrix()
            .multiply(other.global_to_local_matrix())
    }

    /// Получить матрицу преобразования из других локальных координат в текущие локальные.
    pub fn other_local_to_local_matrix(&self, other: &CoordFrame) -> Transform3D {
        other
            .local_to_global_matrix()
            .multiply(self.global_to_local_matrix())
    }

    /// Преобразовать локальную систему координат.
    pub fn apply_transform(&mut self, transform: &Transform3D) {
        self.x = Vec3::from(transform.apply_to_hvec(self.x.into()));
        self.y = Vec3::from(transform.apply_to_hvec(self.y.into()));
        self.z = Vec3::from(transform.apply_to_hvec(self.z.into()));
        self.origin = Point3::from(Vec3::from(
            transform.apply_to_hvec(Vec3::from(self.origin).into()),
        ));
    }
}

impl Default for CoordFrame {
    /// Возвращает глобальную систему координат как систему по-умолчанию.
    ///
    /// По сути алиас для конструктора `global()`.
    fn default() -> Self {
        Self::global()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HVec3, Point3, Vec3};

    const TOLERANCE: f32 = 1e-8;

    fn assert_hvecs(got: HVec3, expected: HVec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    #[test]
    fn test_global_to_global() {
        // Локальные координаты идентичны глобальным
        let frame = CoordFrame::global();
        let global_vec = HVec3::new(1.0, 2.0, 3.0);

        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        assert_hvecs(local_vec, global_vec, TOLERANCE);

        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_translated() {
        // Локальная система смещена относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::plus_x(), Vec3::plus_y(), Point3::new(1.0, 1.0, 1.0));
        let global_vec = HVec3::new(1.0, 2.0, 3.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(0.0, 1.0, 2.0); // (1.0 - 1.0, 2.0 - 1.0, 3.0 - 1.0)
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::plus_y(), Vec3::minus_x(), Point3::zero());
        let global_vec = HVec3::new(1.0, 2.0, 3.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(2.0, -1.0, 3.0); // z тот же, x направлен вдоль y, а y теперь -x
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }
}
