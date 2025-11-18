use crate::{CoordFrame, Point3, Transform3D, Vec3};

impl CoordFrame {
    /// Создать новую координатную систему по 3-м базисам и точке, из которой эти базисы выходят.
    /// Базисы должны быть **ортонормированными**.
    pub fn new(forward: Vec3, right: Vec3, up: Vec3, origin: Point3) -> Self {
        debug_assert!(
            !forward.approx_equal(Vec3::zero(), 1e-7),
            "вектор forward не может быть нулевым"
        );
        debug_assert!(
            !right.approx_equal(Vec3::zero(), 1e-7),
            "вектор right не может быть нулевым"
        );
        debug_assert!(
            !up.approx_equal(Vec3::zero(), 1e-7),
            "вектор up не может быть нулевым"
        );

        let forward = forward.normalize();
        let right = right.normalize();

        // убедиться в ортогональности базисов (возможно накопление ошибок)
        let up = right.cross_left(forward).normalize();
        let right = forward.cross_left(up).normalize();

        Self {
            forward,
            right,
            up,
            origin,
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Создать новую **левую** координатную систему по 2-м векторам и точке.
    ///
    /// 3-ий вектор строится автоматически перпендикулярно 2-м указанным. 2 заданных вектора
    /// должны быть **ортогональными**.
    pub fn from_2(forward: Vec3, up: Vec3, origin: Point3) -> Self {
        let right = forward.cross_left(up);
        Self::new(forward, right, up, origin)
    }

    /// Создать координатную систему, идентичную глобальной.
    pub fn global() -> Self {
        Self::new(Vec3::forward(), Vec3::right(), Vec3::up(), Point3::zero())
    }

    /// Вернуть направление вверх локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn up(&self) -> Vec3 {
        self.up
    }

    /// Вернуть направление вниз локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn down(&self) -> Vec3 {
        -self.up
    }

    /// Вернуть направление вперёд локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn forward(&self) -> Vec3 {
        self.forward
    }

    /// Вернуть направление назад локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn backward(&self) -> Vec3 {
        -self.forward
    }

    /// Вернуть направление влево локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn left(&self) -> Vec3 {
        -self.right
    }

    /// Вернуть направление вправо локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn right(&self) -> Vec3 {
        self.right
    }

    pub fn position(&self) -> Point3 {
        self.origin
    }

    /// Получить матрицу преобразования из текущих локальных координат в глобальные.
    ///
    /// Эта матрица преобразует вектор из локальной системы координат в глобальную.
    pub fn local_to_global_matrix(&self) -> Transform3D {
        let rotation = Transform3D {
            m: [
                self.right().x,
                self.right().y,
                self.right().z,
                0.0,
                self.up().x,
                self.up().y,
                self.up().z,
                0.0,
                self.forward().x,
                self.forward().y,
                self.forward().z,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        };
        let scale = Transform3D::scale(self.scale.x, self.scale.y, self.scale.z);
        let translate = Transform3D::translation_vec(self.origin.into());
        // масштабирование -> поворот -> перемещение
        scale.multiply(rotation).multiply(translate)
    }

    /// Получить матрицу преобразования из глобальных координат в текущие локальные.
    ///
    /// Эта матрица преобразует вектор из глобальной системы координат в локальную.
    pub fn global_to_local_matrix(&self) -> Transform3D {
        let inv_rotation = Transform3D {
            m: [
                self.right().x,
                self.up().x,
                self.forward().x,
                0.0,
                self.right().y,
                self.up().y,
                self.forward().y,
                0.0,
                self.right().z,
                self.up().z,
                self.forward().z,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        };
        let inv_scale =
            Transform3D::scale(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        let inv_translate = Transform3D::translation_vec(-Vec3::from(self.origin));
        // переместить в точку отчёта -> повернуть к локальной -> масштабировать к локальной
        inv_translate.multiply(inv_rotation).multiply(inv_scale)
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

    /// Сместить координатную систему на `vec`.
    pub fn translate_vec(&mut self, vec: Vec3) {
        self.origin = self.origin + vec;
    }

    /// Масштабирует координатную систему в соответсвии с `vec`.
    ///
    /// Каждая координата `vec` является коэфициентом масштабирования каждой из осей.
    pub fn scale_vec(&mut self, vec: Vec3) {
        self.scale = Vec3::new(
            self.scale.x * vec.x,
            self.scale.y * vec.y,
            self.scale.z * vec.z,
        );
    }

    /// Повернуть локальную систему координат через `transform`.
    ///
    /// `transform` должен содержать только вращение.
    pub fn rotate(&mut self, transform: Transform3D) {
        self.forward = self.forward * transform;
        self.up = self.up * transform;
        self.right = self.right * transform;

        // избавляемся от ошибок для сохранения ортонормированности базиса
        self.up = self.right.cross_left(self.forward).normalize();
        self.right = self.forward.cross_left(self.up).normalize();
        self.forward = self.forward.normalize();

        #[cfg(debug_assertions)]
        self.assert_orthonormal();
    }

    /// Вспомогательный метод для проверки ортонормированности координатной системы.
    fn assert_orthonormal(&self) {
        debug_assert!(
            (self.forward.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Базис self.forward длиной {} должен быть нормирован",
            self.forward.length()
        );
        debug_assert!(
            (self.right.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Базис self.right длиной {} должен быть нормирован",
            self.right.length()
        );
        debug_assert!(
            (self.up.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "Базис self.up длиной {} должен быть нормирован",
            self.up.length()
        );

        debug_assert!(
            self.forward.dot(self.right).abs() < 2.0 * f32::EPSILON,
            "Базисы self.forward {:?} и self.right{:?} должны быть ортогональными, но их произведение равно {}",
            self.forward,
            self.right,
            self.forward.dot(self.right)
        );
        debug_assert!(
            self.forward.dot(self.up).abs() < 2.0 * f32::EPSILON,
            "Базисы self.forward {:?} и self.up{:?} должны быть ортогональными, но их произведение равно {}",
            self.forward,
            self.up,
            self.forward.dot(self.up)
        );
        debug_assert!(
            self.right.dot(self.up).abs() < 2.0 * f32::EPSILON,
            "Базисы self.right {:?} и self.up{:?} должны быть ортогональными, но их произведение равно {}",
            self.right,
            self.up,
            self.right.dot(self.up)
        );
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

    const TOLERANCE: f32 = 1e-6;

    fn assert_hvecs(got: HVec3, expected: HVec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn assert_vecs(got: Vec3, expected: Vec3, tolerance: f32) {
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
    fn test_from_2_constructor_global() {
        // Создаётся локальная система идентичная глобальной, но по 2-м векторам
        let frame = CoordFrame::from_2(Vec3::forward(), Vec3::up(), Point3::zero());

        // проверяем на совпадение с глобальной
        assert_vecs(frame.forward(), Vec3::forward(), TOLERANCE);
        assert_vecs(frame.up(), Vec3::up(), TOLERANCE);
        assert_vecs(frame.right(), Vec3::right(), TOLERANCE);
    }

    #[test]
    fn test_local_is_translated_1() {
        // Локальная система смещена относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::forward(), Vec3::up(), Point3::new(1.0, 1.0, 1.0));
        let global_vec = HVec3::zero();

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(-1.0, -1.0, -1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_translated_2() {
        // Локальная система смещена относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::forward(), Vec3::up(), Point3::new(1.0, 1.0, 1.0));
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
    fn test_local_is_rotated_90_1() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::up(), Vec3::backward(), Point3::zero());
        let global_vec = HVec3::new(1.0, 2.0, 3.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(1.0, -3.0, 2.0); // x тот же, +z теперь +y, +y теперь -z
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_90_2() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::right(), Vec3::up(), Point3::zero());
        let global_vec = HVec3::new(0.0, 0.0, 10.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(-10.0, 0.0, 0.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_180() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::backward(), Vec3::up(), Point3::zero());
        let global_vec = HVec3::new(1.0, 2.0, 3.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(-1.0, 2.0, -3.0); // y тот же, z теперь -z, а x теперь -x
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_and_translated_1() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::up(), Vec3::backward(), Point3::new(1.0, 1.0, 1.0));
        let global_vec = HVec3::new(1.0, 2.0, 3.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(1.0 - 1.0, -3.0 + 1.0, 2.0 - 1.0); // x тот же, +z теперь +y, +y теперь -z + возврат на (1.0, 1.0, 1.0)
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_and_translated_2() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::backward(), Vec3::up(), Point3::new(0.0, 0.0, 0.0));
        let global_vec = HVec3::new(0.0, 0.0, 10.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(0.0, 0.0, -10.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_and_translated_3() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(Vec3::backward(), Vec3::up(), Point3::new(0.0, 0.0, -10.0));
        let global_vec = HVec3::new(0.0, 0.0, 0.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(0.0, 0.0, -10.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }
}
