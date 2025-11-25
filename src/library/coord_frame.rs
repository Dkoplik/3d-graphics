//! Объявление и реализация структуры `CoordFrame`.

use crate::{Point3, Transform3D, UVec3, Vec3, library::utils};

/// Локальная **левая** координатная система с ортонормированным базисом в 3D пространтсве.
///
/// Поддерживаются только ортонормированный базис (векторы базиса перпендикулярны друг другу и нормализованны).
/// Эта структура представляет собой локальную коодринатную систему какого-либо объекта, в пределах которой объект записан.
/// Через указанный базис системы можно получить части объекта в глобальных координатах.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoordFrame {
    /// Направление вперёд локальной системы координат. Сам базис указывается в **глобальных** координатах.
    forward: UVec3,
    /// Направление вправо локальной системы координат. Сам базис указывается в **глобальных** координатах.
    right: UVec3,
    /// Направление вверх локальной системы координат. Сам базис указывается в **глобальных** координатах.
    up: UVec3,
    /// Точка (0.0, 0.0, 0.0) локальной координатной системы. Сама точка `origin` указывается в
    /// **глобальных** координатах,задаёт нулевую точку локальных координат.
    pub origin: Point3,
    /// Вектор масштабирования каждой координатной оси локальной системы.
    pub scale: Vec3,
}

impl CoordFrame {
    /// Создать новую координатную систему по 3-м базисам и точке, из которой эти базисы выходят.
    /// Базисы должны быть **ортонормированными**.
    pub fn new(forward: UVec3, right: UVec3, up: UVec3, origin: Point3) -> Self {
        let (forward, right, up) = utils::ensure_orthonormal(forward, right, up);
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
    pub fn from_2(forward: UVec3, up: UVec3, origin: Point3) -> Self {
        let right = up
            .cross(forward)
            .normalize()
            .expect("forward и up не должны быть параллельны друг другу");
        let (forward, right, up) = utils::ensure_orthonormal(forward, right, up);
        Self {
            forward,
            right,
            up,
            origin,
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Создать координатную систему, идентичную глобальной.
    pub fn global() -> Self {
        Self {
            forward: UVec3::forward(),
            right: UVec3::right(),
            up: UVec3::up(),
            origin: Point3::zero(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Вернуть направление вверх локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn up(&self) -> UVec3 {
        self.up
    }

    /// Вернуть направление вниз локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn down(&self) -> UVec3 {
        -self.up
    }

    /// Вернуть направление вперёд локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn forward(&self) -> UVec3 {
        self.forward
    }

    /// Вернуть направление назад локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn backward(&self) -> UVec3 {
        -self.forward
    }

    /// Вернуть направление влево локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn left(&self) -> UVec3 {
        -self.right
    }

    /// Вернуть направление вправо локальной координатной системы.
    ///
    /// Возвращаемый вектор будет в **глобальных** координатах.
    pub fn right(&self) -> UVec3 {
        self.right
    }

    /// Получить матрицу преобразования из текущих локальных координат в глобальные.
    ///
    /// Эта матрица преобразует вектор из локальной системы координат в глобальную.
    pub fn local_to_global_matrix(&self) -> Transform3D {
        let rotation = Transform3D::rotation_from_basis(self.forward(), self.right(), self.up());
        let scale = Transform3D::scale(self.scale.x, self.scale.y, self.scale.z);
        let translate = Transform3D::translation_vec(Vec3::from(self.origin));

        // масштабирование -> поворот -> перемещение
        scale.multiply(rotation).multiply(translate)
    }

    /// Получить матрицу преобразования из глобальных координат в текущие локальные.
    ///
    /// Эта матрица преобразует вектор из глобальной системы координат в локальную.
    pub fn global_to_local_matrix(&self) -> Transform3D {
        let inv_rotation = Transform3D::rotation_to_basis(self.forward(), self.right(), self.up());
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
    pub fn scale_by_vec(&mut self, vec: Vec3) {
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
        self.forward = self.forward.apply_transform(transform).unwrap();
        self.up = self.up.apply_transform(transform).unwrap();
        self.right = self.right.apply_transform(transform).unwrap();

        (self.forward, self.right, self.up) =
            utils::ensure_orthonormal(self.forward, self.right, self.up);

        #[cfg(debug_assertions)]
        self.assert_orthonormal();
    }

    /// Отразить координатную систему в плоскости XY.
    pub fn reflect_xy(&mut self) {
        // отразить по xy это то же, что и поменять направление z
        let backward = self.backward();
        self.forward = backward;
    }

    /// Отразить координатную систему в плоскости XZ.
    pub fn reflect_xz(&mut self) {
        // отразить по xz это то же, что и поменять направление y
        let down = self.down();
        self.up = down;
    }

    /// Отразить координатную систему в плоскости YZ.
    pub fn reflect_yz(&mut self) {
        // отразить по yz это то же, что и поменять направление x
        let left = self.left();
        self.right = left;
    }

    /// Вспомогательный метод для проверки ортонормированности координатной системы.
    fn assert_orthonormal(&self) {
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

    fn assert_uvecs(got: UVec3, expected: UVec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался unit-вектор {:?}, но получен unit-вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    #[test]
    fn test_global_to_global() {
        // Локальные координаты идентичны глобальным
        let frame = CoordFrame::global();
        let global_vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        assert_hvecs(local_vec, global_vec, TOLERANCE);

        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_from_2_constructor_global() {
        // Создаётся локальная система идентичная глобальной, но по 2-м векторам
        let frame = CoordFrame::from_2(UVec3::forward(), UVec3::up(), Point3::zero());

        // проверяем на совпадение с глобальной
        assert_uvecs(frame.forward(), UVec3::forward(), TOLERANCE);
        assert_uvecs(frame.up(), UVec3::up(), TOLERANCE);
        assert_uvecs(frame.right(), UVec3::right(), TOLERANCE);
    }

    #[test]
    fn test_local_is_translated_1() {
        // Локальная система смещена относительно глоабальной
        let frame = CoordFrame::from_2(UVec3::forward(), UVec3::up(), Point3::new(1.0, 1.0, 1.0));
        let global_vec = HVec3::new(0.0, 0.0, 0.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(-1.0, -1.0, -1.0, 1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_translated_2() {
        // Локальная система смещена относительно глоабальной
        let frame = CoordFrame::from_2(UVec3::forward(), UVec3::up(), Point3::new(1.0, 1.0, 1.0));
        let global_vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(0.0, 1.0, 2.0, 1.0); // (1.0 - 1.0, 2.0 - 1.0, 3.0 - 1.0)
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_translated_3() {
        // Локальная система смещена относительно глоабальной
        let frame = CoordFrame::from_2(UVec3::forward(), UVec3::up(), Point3::new(5.0, 0.0, 0.0));
        let local_vec = HVec3::new(2.0, 0.0, 0.0, 1.0);

        // Глобальный вектор в локальный
        let global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        let expected = HVec3::new(7.0, 0.0, 0.0, 1.0);
        assert_hvecs(global_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        assert_hvecs(back_to_local_vec, local_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_90_1() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(UVec3::up(), UVec3::backward(), Point3::zero());
        let global_vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(1.0, -3.0, 2.0, 1.0); // x тот же, +z теперь +y, +y теперь -z
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_90_2() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(UVec3::right(), UVec3::up(), Point3::zero());
        let global_vec = HVec3::new(0.0, 0.0, 10.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(-10.0, 0.0, 0.0, 1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_90_3() {
        // Локальная система повёрнута относительно глоабальной
        let mut frame = CoordFrame::global();
        frame.rotate(Transform3D::rotation_aligning(
            UVec3::forward(),
            UVec3::up(),
        ));
        assert_uvecs(frame.forward(), UVec3::up(), TOLERANCE);
        assert_uvecs(frame.right(), UVec3::right(), TOLERANCE);
        assert_uvecs(frame.up(), UVec3::backward(), TOLERANCE);

        let global_vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(1.0, -3.0, 2.0, 1.0); // x тот же, +z теперь +y, +y теперь -z
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_90_4() {
        // Локальная система повёрнута относительно глоабальной
        let mut frame = CoordFrame::global();
        frame.rotate(Transform3D::rotation_aligning(
            UVec3::forward(),
            UVec3::right(),
        ));
        assert_uvecs(frame.forward(), UVec3::right(), TOLERANCE);
        assert_uvecs(frame.right(), UVec3::backward(), TOLERANCE);
        assert_uvecs(frame.up(), UVec3::up(), TOLERANCE);

        let global_vec = HVec3::new(0.0, 0.0, 10.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(-10.0, 0.0, 0.0, 1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_180() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(UVec3::backward(), UVec3::up(), Point3::zero());
        let global_vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(-1.0, 2.0, -3.0, 1.0); // y тот же, z теперь -z, а x теперь -x
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_and_translated_1() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(UVec3::up(), UVec3::backward(), Point3::new(1.0, 1.0, 1.0));
        let global_vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(1.0 - 1.0, -3.0 + 1.0, 2.0 - 1.0, 1.0); // x тот же, +z теперь +y, +y теперь -z + возврат на (1.0, 1.0, 1.0)
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_and_translated_2() {
        // Локальная система повёрнута относительно глоабальной
        let frame = CoordFrame::from_2(UVec3::backward(), UVec3::up(), Point3::new(0.0, 0.0, 0.0));
        let global_vec = HVec3::new(0.0, 0.0, 10.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(0.0, 0.0, -10.0, 1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_and_translated_3() {
        // Локальная система повёрнута относительно глоабальной
        let frame =
            CoordFrame::from_2(UVec3::backward(), UVec3::up(), Point3::new(0.0, 0.0, -10.0));
        let global_vec = HVec3::new(0.0, 0.0, 0.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(0.0, 0.0, -10.0, 1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_and_translated_4() {
        // Локальная система повёрнута относительно глоабальной
        let mut frame = CoordFrame::global();
        frame.rotate(Transform3D::rotation_aligning(
            UVec3::forward(),
            UVec3::up(),
        ));
        frame.translate_vec(Vec3::new(1.0, 1.0, 1.0));
        let global_vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(1.0 - 1.0, -3.0 + 1.0, 2.0 - 1.0, 1.0); // x тот же, +z теперь +y, +y теперь -z + возврат на (1.0, 1.0, 1.0)
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_rotated_and_translated_5() {
        // Локальная система повёрнута относительно глоабальной
        let mut frame = CoordFrame::global();
        frame.rotate(Transform3D::rotation_aligning(
            UVec3::forward(),
            UVec3::backward(),
        ));
        frame.translate_vec(Vec3::new(0.0, 0.0, -10.0));
        let global_vec = HVec3::new(0.0, 0.0, 0.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(0.0, 0.0, -10.0, 1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_scaled_y() {
        // Локальная система масштабируется в 2 раза по оси Y
        let mut frame =
            CoordFrame::from_2(UVec3::forward(), UVec3::up(), Point3::new(0.0, 0.0, 0.0));
        frame.scale_by_vec(Vec3::new(1.0, 2.0, 1.0));
        let global_vec = HVec3::new(1.0, 3.0, 0.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(1.0, 1.5, 0.0, 1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_uniform_scaled() {
        // Локальная система масштабируется в 2 раза по всем осям
        let mut frame = CoordFrame::from_2(UVec3::forward(), UVec3::up(), Point3::zero());
        frame.scale_by_vec(Vec3::new(2.0, 2.0, 2.0));
        let global_vec = HVec3::new(1.0, 3.0, 4.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(0.5, 1.5, 2.0, 1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_scaled_and_translated() {
        // Локальная система масштабируется в 2 раза по всем осям
        let mut frame =
            CoordFrame::from_2(UVec3::forward(), UVec3::up(), Point3::new(1.0, 2.0, 3.0));
        frame.scale_by_vec(Vec3::new(2.0, 2.0, 2.0));
        let global_vec = HVec3::new(1.0, 3.0, 4.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new((1.0 - 1.0) / 2.0, (3.0 - 2.0) / 2.0, (4.0 - 3.0) / 2.0, 1.0);
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_scaled_and_rotated() {
        // Локальная система повёрнута относительно глоабальной
        let mut frame = CoordFrame::from_2(UVec3::up(), UVec3::backward(), Point3::zero());
        frame.scale_by_vec(Vec3::new(2.0, 2.0, 2.0));
        let global_vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(1.0 / 2.0, -3.0 / 2.0, 2.0 / 2.0, 1.0); // x тот же, +z теперь +y, +y теперь -z
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }

    #[test]
    fn test_local_is_complex() {
        // Локальная система повёрнута относительно глоабальной
        let mut frame =
            CoordFrame::from_2(UVec3::up(), UVec3::backward(), Point3::new(1.0, 1.0, 1.0));
        frame.scale_by_vec(Vec3::new(2.0, 2.0, 2.0));
        let global_vec = HVec3::new(1.0, 2.0, 3.0, 1.0);

        // Глобальный вектор в локальный
        let local_vec = frame.global_to_local_matrix().apply_to_hvec(global_vec);
        let expected = HVec3::new(0.0, -1.0, 0.5, 1.0); // x тот же, +z теперь +y, +y теперь -z
        assert_hvecs(local_vec, expected, TOLERANCE);

        // Обратно в глобальный
        let back_to_global_vec = frame.local_to_global_matrix().apply_to_hvec(local_vec);
        assert_hvecs(back_to_global_vec, global_vec, TOLERANCE);
    }
}
