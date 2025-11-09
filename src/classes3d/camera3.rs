use crate::{Camera3, CoordFrame, Point3, Transform3D, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType {
    Perspective,
    Isometric,
}

impl Default for Camera3 {
    fn default() -> Self {
        Self::new(
            Point3::new(10.0, 10.0, 10.0),
            Vec3::new(-1.0, -1.0, 0.0).normalize(),
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
    pub fn new(
        position: Point3,
        direction: Vec3,
        up: Vec3,
        fov_rad: f32,
        aspect_ratio: f32,
        near_plane: f32,
        far_plane: f32,
    ) -> Self {
        debug_assert!(
            (direction.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "direction должен быть нормализован, но его длина равна {}",
            direction.length()
        );
        debug_assert!(
            (up.length() - 1.0).abs() < 2.0 * f32::EPSILON,
            "up должен быть нормализован, но его длина равна {}",
            up.length()
        );

        let local_frame = CoordFrame::new(direction, direction.cross(up).normalize(), up, position);
        Self::from_frame(local_frame, fov_rad, aspect_ratio, near_plane, far_plane)
    }

    /// Создаёт камеру с использованием координатной системы.
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

    /// Создаёт камеру из позиции и целевой точки. Направление вверх берётся как +z.
    pub fn from(position: Point3, target: Point3) -> Self {
        debug_assert!(
            !position.approx_equal(target, 1e-8),
            "камера в позиции {:?} не может смотреть в ту же точку {:?}",
            position,
            target
        );
        Self::look_at(position, target, Vec3::up())
    }

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
        self.local_frame.z
    }

    /// Возвращает направление вниз в **глобальных** координатах.
    pub fn down(&self) -> Vec3 {
        -self.local_frame.z
    }

    /// Возвращает направление обзора камеры (направлениев вперёд) в **глобальных** координатах.
    pub fn forward(&self) -> Vec3 {
        self.local_frame.x
    }

    /// Возвращает направление назад для камеры в **глобальных** координатах.
    pub fn backward(&self) -> Vec3 {
        -self.local_frame.x
    }

    /// Возвращает направление налево в **глобальных** координатах.
    pub fn left(&self) -> Vec3 {
        self.local_frame.y
    }

    /// Возвращает направление направо в **глобальных** координатах.
    pub fn right(&self) -> Vec3 {
        -self.local_frame.y
    }

    /// Возвращает точку, в которую смотрит камера.
    pub fn target(&self) -> Point3 {
        self.local_frame.origin + self.forward()
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

    // --------------------------------------------------
    // Вспомогательные методы
    // --------------------------------------------------

    // /// Возвращает луч из камеры через точку на экране (в нормализованных координатах [-1, 1]).
    // pub fn screen_point_to_ray(&self, screen_x: f32, screen_y: f32) -> Line3 {
    //     // Преобразуем нормализованные координаты экрана в направление луча
    //     let tan_half_fov = (self.fov / 2.0).tan();
    //     let x = screen_x * tan_half_fov * self.aspect_ratio;
    //     let y = screen_y * tan_half_fov;

    //     // Направление в пространстве камеры
    //     let right = self.right();
    //     let up = self.up;

    //     let ray_direction = (self.direction + right * x + up * y).normalize();

    //     Line3::new(self.position, ray_direction * self.far_plane)
    // }

    // /// Проверяет, находится ли точка внутри frustum камеры.
    // pub fn is_point_visible(&self, point: Point3) -> bool {
    //     let view_proj = self.view_projection_matrix();
    //     let clip_space = view_proj.apply_to_point(point);

    //     // Точка видима если находится в NDC кубе [-1, 1]
    //     clip_space.x >= -1.0
    //         && clip_space.x <= 1.0
    //         && clip_space.y >= -1.0
    //         && clip_space.y <= 1.0
    //         && clip_space.z >= -1.0
    //         && clip_space.z <= 1.0
    // }

    // /// Возвращает расстояние от камеры до точки.
    // pub fn distance_to(&self, point: Point3) -> f32 {
    //     (point - self.position).length()
    // }
}
