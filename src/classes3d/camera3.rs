use crate::{Camera3, CoordFrame, Line3, Point3, Transform3D, Vec3};

impl Default for Camera3 {
    fn default() -> Self {
        Self::new(
            Point3::new(0.0, 0.0, -10.0),
            Vec3::forward(),
            Vec3::up(),
            (60.0 as f32).to_radians(),
            16.0 / 9.0,
            0.1,
            100.0,
        )
    }
}

impl Camera3 {
    /// Создает новую камеру с указанными параметрами.
    ///
    /// `position` - позиция камеры в **глобальных** координатах.
    /// `look_direction` - направление обзора камеры (вперёд) в **глобальных** координатах.
    /// `up` - направление камеры вверх в **глобальных** координатах.
    /// `fov_rad` - угол обзора в радианах
    /// `aspect_ratio` - соотношение сторон (ширина к высоте)
    /// `near_plane` - расстояние до ближней границы отсечения
    /// `far_plane` - расстояние для дальней границы отсечения
    pub fn new(
        position: Point3,
        look_direction: Vec3,
        up: Vec3,
        fov_rad: f32,
        aspect_ratio: f32,
        near_plane: f32,
        far_plane: f32,
    ) -> Self {
        debug_assert!(
            (look_direction.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "look_direction должен быть нормализован, но его длина равна {}",
            look_direction.length()
        );
        debug_assert!(
            (up.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "up должен быть нормализован, но его длина равна {}",
            up.length()
        );

        // в координатах камеры +z должно быть направлено в саму камеру, поэтому вектор направления ОТ камеры будет -z.
        let forward = -look_direction;
        let local_frame = CoordFrame::from_2(forward, up, position);

        Self::from_frame(local_frame, fov_rad, aspect_ratio, near_plane, far_plane)
    }

    /// Создаёт камеру с использованием координатной системы.
    ///
    /// Камера смотрит в направлении backward для локальной системы.
    pub fn from_frame(
        local_frame: CoordFrame,
        fov_rad: f32,
        aspect_ratio: f32,
        near_plane: f32,
        far_plane: f32,
    ) -> Self {
        debug_assert!(
            near_plane < far_plane,
            "near_plane {} должен быть ближе к камере, чем far_plane {}",
            near_plane,
            far_plane
        );
        debug_assert!(fov_rad > 0.0, "fov {} должен быть больше 0", fov_rad);

        Self {
            local_frame,
            fov: fov_rad,
            aspect_ratio,
            near_plane,
            far_plane,
        }
    }

    /// Создаёт камеру из позиции и целевой точки. Направление вверх такое же, как и в глобальной системе.
    // pub fn from(position: Point3, target: Point3) -> Self {
    //     debug_assert!(
    //         !position.approx_equal(target, 1e-8),
    //         "камера в позиции {:?} не может смотреть в ту же точку {:?}",
    //         position,
    //         target
    //     );
    //     Self::look_at(position, target, Vec3::up())
    // }

    /// Создает камеру смотрящую в указанную точку.
    pub fn look_at(position: Point3, target: Point3, up: Vec3) -> Self {
        debug_assert!(
            !position.approx_equal(target, 1e-8),
            "камера в позиции {:?} не может смотреть в ту же точку {:?}",
            position,
            target
        );

        let direction = (target - position).normalize();

        debug_assert!(
            direction.dot(up).abs() < 1e-8,
            "направление камеры {:?} должно быть перпендикулярно направлению вверх {:?}",
            direction,
            up
        );

        Self::new(
            position,
            direction,
            up.normalize(),
            (60.0 as f32).to_radians(),
            16.0 / 9.0,
            0.1,
            100.0,
        )
    }

    /// Возвращает направление вверх в **глобальных** координатах.
    pub fn up(&self) -> Vec3 {
        self.local_frame.up()
    }

    /// Возвращает направление вниз в **глобальных** координатах.
    pub fn down(&self) -> Vec3 {
        self.local_frame.down()
    }

    /// Возвращает направление направлениев вперёд в **глобальных** координатах.
    pub fn forward(&self) -> Vec3 {
        // поскольку координатная система смотрит в обратную сторону
        self.local_frame.backward()
    }

    /// Возвращает направление назад в **глобальных** координатах.
    pub fn backward(&self) -> Vec3 {
        // поскольку координатная система смотрит в обратную сторону
        self.local_frame.forward()
    }

    /// Возвращает направление налево в **глобальных** координатах.
    pub fn left(&self) -> Vec3 {
        // поскольку координатная система смотрит в обратную сторону
        self.local_frame.right()
    }

    /// Возвращает направление направо в **глобальных** координатах.
    pub fn right(&self) -> Vec3 {
        // поскольку координатная система смотрит в обратную сторону
        self.local_frame.left()
    }

    /// Возвращает точку, в которую смотрит камера.
    pub fn target(&self) -> Point3 {
        self.local_frame.origin + self.get_direction()
    }

    /// Возвращает локальные координаты камеры.
    pub fn get_local_frame(&self) -> &CoordFrame {
        &self.local_frame
    }

    /// Возвращает изменяемые локальные координаты камеры.
    ///
    /// Путём применения трансформаций к локальным координатам, можно двигать камеру.
    pub fn get_local_frame_mut(&mut self) -> &mut CoordFrame {
        &mut self.local_frame
    }

    /// Применить преобразование к камере.
    pub fn apply_transform(&mut self, transform: &Transform3D) {
        self.local_frame.apply_transform(transform);
    }

    // --------------------------------------------------
    // Доступ и изменение параметров камеры
    // --------------------------------------------------

    /// Возвращает поле зрения (в радианах)
    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    /// Устанавливает поле зрения (в радианах).
    pub fn set_fov(&mut self, fov_rad: f32) {
        debug_assert!(fov_rad > 0.0, "fov {} должен быть положительным", fov_rad);
        debug_assert!(
            fov_rad.to_degrees() < 180.0,
            "fov {} должен быть до 180 градусов",
            fov_rad
        );

        self.fov = fov_rad;
    }

    /// Возвращает поле зрения (в градусах)
    pub fn get_fov_degrees(&self) -> f32 {
        self.fov.to_degrees()
    }

    /// Устанавливает поле зрения (в градусах).
    pub fn set_fov_degrees(&mut self, fov_deg: f32) {
        debug_assert!(fov_deg > 0.0, "fov {} должен быть положительным", fov_deg);
        debug_assert!(
            fov_deg < 180.0,
            "fov {} должен быть до 180 градусов",
            fov_deg
        );

        self.set_fov(fov_deg.to_radians());
    }

    /// Возвращает соотношение сторон.
    pub fn get_aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    /// Устанавливает соотношение сторон.
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        debug_assert!(
            aspect_ratio > 0.0,
            "соотношение сторон {} должно быть положительным",
            aspect_ratio
        );

        self.aspect_ratio = aspect_ratio;
    }

    /// Возвращает ближнюю плоскость отсечения.
    pub fn get_near_plane(&self) -> f32 {
        self.near_plane
    }

    /// Устанавливает ближнюю плоскость отсечения.
    pub fn set_near_plane(&mut self, near_plane: f32) {
        debug_assert!(
            near_plane > 0.0,
            "ближняя плоскость отсечения {} должна быть положительной",
            near_plane
        );
        debug_assert!(
            near_plane < self.far_plane,
            "ближняя плоскость {} должна быть ближе дальней {}",
            near_plane,
            self.far_plane
        );

        self.near_plane = near_plane;
    }

    /// Возвращает дальнюю плоскость отсечения.
    pub fn get_far_plane(&self) -> f32 {
        self.far_plane
    }

    /// Устанавливает дальнюю плоскость отсечения.
    pub fn set_far_plane(&mut self, far_plane: f32) {
        debug_assert!(
            self.near_plane < far_plane,
            "ближняя плоскость {} должна быть ближе дальней {}",
            self.near_plane,
            far_plane
        );

        self.far_plane = far_plane.max(self.near_plane + 0.1);
    }

    pub fn get_position(&self) -> Point3 {
        self.local_frame.position()
    }

    pub fn set_position(&mut self, position: Point3) {
        self.local_frame.origin = position;
    }

    pub fn get_target(&self) -> Point3 {
        self.get_position() + self.forward()
    }

    pub fn set_target(&mut self, target: Point3) {
        let from = self.get_target() - self.get_position();
        let to = target - self.get_position();
        self.rotate(from, to);
    }

    pub fn get_direction(&self) -> Vec3 {
        self.forward()
    }

    pub fn set_direction(&mut self, direction: Vec3, up: Vec3) {
        let forward = -direction;
        let new_frame = CoordFrame::from_2(
            forward.normalize(),
            up.cross_left(forward).normalize(),
            self.local_frame.origin,
        );
        self.local_frame = new_frame;
    }

    /// Двигает в направлении камеры.
    pub fn move_forward(&mut self, distance: f32) {
        self.local_frame.origin = self.local_frame.origin + self.forward() * distance;
    }

    /// Двигает против направления камеры.
    pub fn move_backward(&mut self, distance: f32) {
        self.local_frame.origin = self.local_frame.origin + self.backward() * distance;
    }

    pub fn move_right(&mut self, distance: f32) {
        self.local_frame.origin = self.local_frame.origin + self.right() * distance;
    }

    pub fn move_left(&mut self, distance: f32) {
        self.local_frame.origin = self.local_frame.origin + self.left() * distance;
    }

    pub fn move_up(&mut self, distance: f32) {
        self.local_frame.origin = self.local_frame.origin + self.up() * distance;
    }

    pub fn move_down(&mut self, distance: f32) {
        self.local_frame.origin = self.local_frame.origin + self.down() * distance;
    }

    /// Повернуть камеру из направления `from` в направление `to` в **локальных** координатах.
    ///
    /// Сами `from` и `to` указываются в **глобальных** координатах.
    pub fn rotate(&mut self, from: Vec3, to: Vec3) {
        // привести к локальным координатам модели
        let to_local = self.local_frame.global_to_local_matrix();
        let from = Vec3::from(to_local.apply_to_hvec(from.into())).normalize();
        let to = Vec3::from(to_local.apply_to_hvec(to.into())).normalize();
        self.apply_transform(&Transform3D::rotation_aligning(from, to));
    }

    // --------------------------------------------------
    // Вспомогательные методы
    // --------------------------------------------------

    /// Возвращает луч из камеры через точку на экране (в нормализованных координатах [-1, 1]).
    pub fn screen_point_to_ray(&self, screen_x: f32, screen_y: f32) -> Line3 {
        // Преобразуем нормализованные координаты экрана в направление луча
        let tan_half_fov = (self.fov / 2.0).tan();
        let x = screen_x * tan_half_fov * self.aspect_ratio;
        let y = screen_y * tan_half_fov;

        // Направление в пространстве камеры
        let right = self.right();
        let up = self.up();

        let ray_direction = (self.get_direction() + right * x + up * y).normalize();

        Line3::new(self.get_position(), ray_direction * self.far_plane)
    }

    /// Возвращает расстояние от камеры до точки.
    pub fn distance_to(&self, point: Point3) -> f32 {
        (point - self.get_position()).length()
    }
}

#[cfg(test)]
mod camera_tests {
    use crate::HVec3;

    use super::*;
    use std::f32::consts::PI;

    const TOLERANCE: f32 = 1e-6;

    fn assert_vecs(got: Vec3, expected: Vec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn assert_points(got: Point3, expected: Point3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидалась точка {:?}, но получена {:?}, одна из координат которой отличается более чем на {}",
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

    #[test]
    fn test_camera_look_at_constructor() {
        let position = Point3::new(0.0, 0.0, 10.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let camera = Camera3::look_at(position, target, Vec3::up());

        assert_points(camera.get_position(), position, TOLERANCE);

        // Camera should look toward target
        let direction = camera.get_direction();
        let expected_direction = (target - position).normalize();
        assert_vecs(direction, expected_direction, TOLERANCE);

        // Up vector should be maintained
        assert_vecs(camera.up(), Vec3::up(), TOLERANCE);
    }

    #[test]
    fn test_camera_with_custom_up_vector() {
        let position = Point3::new(0.0, 0.0, 10.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let custom_up = Vec3::new(1.0, 1.0, 0.0).normalize();

        let camera = Camera3::look_at(position, target, custom_up);

        // Camera up should match custom up (normalized)
        assert_vecs(camera.up(), custom_up.normalize(), TOLERANCE);
    }

    #[test]
    fn test_camera_global_to_local_origin() {
        let camera = Camera3::new(
            Point3::new(0.0, 0.0, -10.0),
            Vec3::forward(),
            Vec3::up(),
            PI / 3.0,
            16.0 / 9.0,
            0.1,
            100.0,
        );

        let global_to_local = camera.get_local_frame().global_to_local_matrix();

        let world_origin = Point3::zero();
        let camera_space_origin: Point3 = global_to_local.apply_to_hvec(world_origin.into()).into();

        assert_points(camera_space_origin, Point3::new(0.0, 0.0, -10.0), TOLERANCE);
    }

    #[test]
    fn test_camera_global_to_local_rotated() {
        // Camera looking along X axis (rotated 90 degrees around Y)
        let camera = Camera3::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::right(),
            Vec3::up(),
            PI / 3.0,
            16.0 / 9.0,
            0.1,
            100.0,
        );

        let global_to_local = camera.get_local_frame().global_to_local_matrix();

        let world_point = Point3::new(5.0, 0.0, 0.0);
        let camera_space_point: Point3 = global_to_local.apply_to_hvec(world_point.into()).into();

        assert_points(camera_space_point, Point3::new(0.0, 0.0, -5.0), TOLERANCE);
    }

    #[test]
    fn test_camera_global_to_local_with_offset() {
        let camera = Camera3::new(
            Point3::new(2.0, 3.0, 5.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::up(),
            PI / 3.0,
            16.0 / 9.0,
            0.1,
            100.0,
        );

        let global_to_local = camera.get_local_frame().global_to_local_matrix();

        // Test point at camera position should be at origin in camera space
        let camera_pos = camera.get_position();
        let camera_space_pos: Point3 = global_to_local.apply_to_hvec(camera_pos.into()).into();
        assert_points(camera_space_pos, Point3::new(0.0, 0.0, 0.0), TOLERANCE);

        // Test point in front of camera
        let point_in_front = Point3::new(2.0, 3.0, 0.0); // Same XY, different Z
        let camera_space_front: Point3 =
            global_to_local.apply_to_hvec(point_in_front.into()).into();
        assert!(camera_space_front.z < 0.0); // Should be in front (negative Z in camera space)
    }

    #[test]
    fn test_camera_direction_vectors_orthonormal() {
        let camera = Camera3::default();

        let forward = camera.forward();
        let right = camera.right();
        let up = camera.up();

        // All vectors should be unit length
        assert!((forward.length() - 1.0).abs() < TOLERANCE);
        assert!((right.length() - 1.0).abs() < TOLERANCE);
        assert!((up.length() - 1.0).abs() < TOLERANCE);

        // All vectors should be orthogonal
        assert!(forward.dot(right).abs() < TOLERANCE);
        assert!(forward.dot(up).abs() < TOLERANCE);
        assert!(right.dot(up).abs() < TOLERANCE);
    }

    // ========================================
    // Projection Transformation Tests
    // ========================================

    #[test]
    fn test_perspective_projection_near_plane() {
        let fov = PI / 3.0; // 60 degrees
        let aspect = 16.0 / 9.0;
        let near = 0.1;
        let far = 100.0;

        let proj_matrix = Transform3D::perspective(fov, aspect, near, far);

        // A point on the near plane should project to Z = 1 in NDC
        let point_on_near_plane = HVec3::new(0.0, 0.0, -near);
        let projected_point = proj_matrix.apply_to_hvec(point_on_near_plane);

        // After perspective divide, Z should be 1
        let ndc_point: Point3 = projected_point.into();
        assert!((ndc_point.z - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_perspective_projection_far_plane() {
        let fov = PI / 3.0;
        let aspect = 16.0 / 9.0;
        let near = 0.1;
        let far = 100.0;

        let proj_matrix = Transform3D::perspective(fov, aspect, near, far);

        // A point on the far plane should project to Z = -1 in NDC
        let point_on_far_plane = HVec3::new(0.0, 0.0, -far);
        let projected_point = proj_matrix.apply_to_hvec(point_on_far_plane);

        let ndc_point: Point3 = projected_point.into();
        assert!((ndc_point.z - (-1.0)).abs() < TOLERANCE);
    }

    #[test]
    fn test_perspective_projection_frustum() {
        let fov = PI / 2.0; // 90 degrees
        let aspect = 1.0; // Square aspect
        let near = 1.0;
        let far = 10.0;

        let proj_matrix = Transform3D::perspective(fov, aspect, near, far);

        // For 90 degree FOV and near=1, points at near plane corners should be at ±1 in X and Y
        let top_right_near = HVec3::new(1.0, 1.0, -1.0); // At near plane, tan(45)=1
        let projected = proj_matrix.apply_to_hvec(top_right_near);
        let ndc: Point3 = projected.into();

        // Should be near the top-right corner of NDC
        assert!(ndc.x > 0.9 && ndc.x <= 1.0);
        assert!(ndc.y > 0.9 && ndc.y <= 1.0);
        assert!((ndc.z - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_parallel_projection() {
        let left = -10.0;
        let right = 10.0;
        let bottom = -10.0;
        let top = 10.0;
        let near = 0.1;
        let far = 100.0;

        let proj_matrix = Transform3D::parallel(left, right, bottom, top, near, far);

        // Test left-bottom-near corner
        let lbn = HVec3::new(left, bottom, -near);
        let projected = proj_matrix.apply_to_hvec(lbn);
        let ndc: Point3 = projected.into();

        assert!((ndc.x - (-1.0)).abs() < TOLERANCE);
        assert!((ndc.y - (-1.0)).abs() < TOLERANCE);
        assert!((ndc.z - 1.0).abs() < TOLERANCE);

        // Test right-top-far corner
        let rtf = HVec3::new(right, top, -far);
        let projected = proj_matrix.apply_to_hvec(rtf);
        let ndc: Point3 = projected.into();

        assert!((ndc.x - 1.0).abs() < TOLERANCE);
        assert!((ndc.y - 1.0).abs() < TOLERANCE);
        assert!((ndc.z - (-1.0)).abs() < TOLERANCE);
    }

    // ========================================
    // Combined View-Projection Tests
    // ========================================

    #[test]
    fn test_view_projection_combined() {
        // Simple camera at origin looking down -Z
        let camera = Camera3::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::up(),
            PI / 3.0,
            1.0, // Square aspect for simplicity
            0.1,
            100.0,
        );

        let view_matrix = camera.get_local_frame().global_to_local_matrix();
        let proj_matrix = Transform3D::perspective(PI / 3.0, 1.0, 0.1, 100.0);

        let view_proj_matrix = view_matrix.multiply(proj_matrix);

        // Point directly in front of camera
        let world_point = Point3::new(0.0, 0.0, -5.0);
        let view_space: Point3 = view_matrix.apply_to_hvec(world_point.into()).into();
        let clip_space: Point3 = view_proj_matrix.apply_to_hvec(world_point.into()).into();

        // In view space, point should be at (0, 0, -5)
        assert_points(view_space, Point3::new(0.0, 0.0, -5.0), TOLERANCE);

        // In clip space (before perspective divide), Z should be between -1 and 1
        assert!(clip_space.z > -1.0 && clip_space.z < 1.0);
    }

    #[test]
    fn test_view_projection_frustum_culling() {
        let camera = Camera3::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::up(),
            PI / 2.0, // 90 degree FOV
            1.0,
            1.0,
            10.0,
        );

        let view_matrix = camera.get_local_frame().global_to_local_matrix();
        let proj_matrix = Transform3D::perspective(PI / 2.0, 1.0, 1.0, 10.0);
        let view_proj_matrix = view_matrix.multiply(proj_matrix);

        // Test points that should be inside frustum
        let inside_points = [
            Point3::new(0.0, 0.0, -5.0),   // Center, middle distance
            Point3::new(0.5, 0.5, -2.0),   // Corner, near distance
            Point3::new(-0.5, -0.5, -8.0), // Opposite corner, far distance
        ];

        for point in inside_points {
            let clip_space: Point3 = view_proj_matrix.apply_to_hvec(point.into()).into();
            // After perspective divide, points inside frustum should be in [-1, 1] range
            let abs_x = clip_space.x.abs();
            let abs_y = clip_space.y.abs();
            let abs_z = clip_space.z.abs();

            assert!(
                abs_x <= 1.0,
                "Point {:?} has x={} outside NDC",
                point,
                clip_space.x
            );
            assert!(
                abs_y <= 1.0,
                "Point {:?} has y={} outside NDC",
                point,
                clip_space.y
            );
            assert!(
                abs_z <= 1.0,
                "Point {:?} has z={} outside NDC",
                point,
                clip_space.z
            );
        }

        // Test points that should be outside frustum
        let outside_points = [
            Point3::new(0.0, 0.0, -15.0), // Behind far plane
            Point3::new(0.0, 0.0, -0.5),  // In front of near plane
            Point3::new(2.0, 0.0, -1.1),  // Outside right side
        ];

        for point in outside_points {
            let clip_space: Point3 = view_proj_matrix.apply_to_hvec(point.into()).into();
            // At least one coordinate should be outside [-1, 1] range
            let abs_x = clip_space.x.abs();
            let abs_y = clip_space.y.abs();
            let abs_z = clip_space.z.abs();

            assert!(
                abs_x > 1.0 || abs_y > 1.0 || abs_z > 1.0,
                "Point {:?} should be outside frustum but is at ({}, {}, {})",
                point,
                clip_space.x,
                clip_space.y,
                clip_space.z
            );
        }
    }

    #[test]
    fn test_camera_movement_consistency() {
        let mut camera = Camera3::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::forward(),
            Vec3::up(),
            PI / 2.0, // 90 degree FOV
            1.0,
            1.0,
            10.0,
        );
        let initial_position = camera.get_position();

        // Move camera and verify position changes
        camera.move_forward(2.0);
        camera.move_right(1.0);
        camera.move_up(0.5);

        let new_position = camera.get_position();

        // Position should have changed
        assert_points(
            new_position,
            initial_position + Vec3::new(1.0, 0.5, 2.0),
            TOLERANCE,
        );
    }

    #[test]
    fn test_camera_setters() {
        let mut camera = Camera3::default();

        // Test position setter
        let new_position = Point3::new(1.0, 2.0, 3.0);
        camera.set_position(new_position);
        assert_points(camera.get_position(), new_position, TOLERANCE);

        // Test direction setter
        let new_direction = Vec3::right();
        camera.set_direction(new_direction, Vec3::up());
        assert_vecs(camera.get_direction(), new_direction, TOLERANCE);

        // Test FOV setter
        let new_fov_deg = 45.0;
        camera.set_fov_degrees(new_fov_deg);
        assert!((camera.get_fov_degrees() - new_fov_deg).abs() < TOLERANCE);

        // Test aspect ratio setter
        let new_aspect = 4.0 / 3.0;
        camera.set_aspect_ratio(new_aspect);
        assert!((camera.get_aspect_ratio() - new_aspect).abs() < TOLERANCE);
    }

    // ========================================
    // Edge Case Tests
    // ========================================

    #[test]
    #[should_panic(expected = "камера в позиции")]
    fn test_camera_look_at_same_position() {
        let position = Point3::new(1.0, 1.0, 1.0);
        Camera3::look_at(position, position, Vec3::up());
    }

    #[test]
    #[should_panic(expected = "fov")]
    fn test_camera_invalid_fov() {
        let mut camera = Camera3::default();
        camera.set_fov_degrees(0.0); // Should panic for zero FOV
    }

    #[test]
    #[should_panic(expected = "ближняя плоскость")]
    fn test_camera_invalid_near_plane() {
        let mut camera = Camera3::default();
        camera.set_near_plane(-1.0); // Should panic for negative near plane
    }
}
