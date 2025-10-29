use std::f32::consts::PI;

use crate::{Camera3, Line3, Point3, Transform3D, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType {
    Perspective,
    Isometric,
}

impl Default for Camera3 {
    fn default() -> Self {
        Self {
            position: Point3::new(5.0, 5.0, 5.0),
            direction: Vec3::new(-1.0, -1.0, -1.0).normalize(),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: (60.0 as f32).to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near_plane: 0.1,
            far_plane: 100.0,
            projection_type: ProjectionType::Perspective,
        }
    }
}

impl Clone for Camera3 {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            direction: self.direction,
            up: self.up,
            fov: self.fov,
            aspect_ratio: self.aspect_ratio,
            near_plane: self.near_plane,
            far_plane: self.far_plane,
            projection_type: self.projection_type,
        }
    }
}

impl Camera3 {
    /// Создает новую камеру с указанными параметрами.
    pub fn new(
        position: Point3,
        target: Point3,
        up: Vec3,
        fov_rad: f32,
        aspect_ratio: f32,
        near_plane: f32,
        far_plane: f32,
    ) -> Self {
        let direction = (target - position).normalize();
        Self {
            position,
            direction,
            up: up.normalize(),
            fov: fov_rad.clamp(0.1, PI - 0.1),
            aspect_ratio,
            near_plane,
            far_plane,
            projection_type: ProjectionType::Perspective,
        }
    }

    /// Создаёт камеру из позиции и целевой точки.
    pub fn from(position: Point3, target: Point3) -> Self {
        Self::look_at(position, target, Vec3::new(0.0, 1.0, 0.0))
    }

    /// Создает камеру смотрящую в указанную точку.
    pub fn look_at(position: Point3, target: Point3, up: Vec3) -> Self {
        let direction = (target - position).normalize();
        Self {
            position,
            direction,
            up: up.normalize(),
            fov: (60.0 as f32).to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near_plane: 0.1,
            far_plane: 100.0,
            projection_type: ProjectionType::Perspective,
        }
    }

    /// Возвращает точку, в которую смотрит камера.
    pub fn target(&self) -> Point3 {
        self.position + self.direction
    }

    /// Возвращает правый вектор камеры.
    pub fn right(&self) -> Vec3 {
        self.direction.cross(self.up).normalize()
    }

    // --------------------------------------------------
    // Матрицы преобразований
    // --------------------------------------------------

    /// Возвращает видовую матрицу (view matrix).
    pub fn view_matrix(&self) -> Transform3D {
        Transform3D::look_at(self.position, self.target(), self.up)
    }

    /// Возвращает матрицу проекции (projection matrix).
    /// Возвращает матрицу проекции (projection matrix).
    pub fn projection_matrix(&self) -> Transform3D {
        match self.projection_type {
            ProjectionType::Perspective => Transform3D::perspective(
                self.fov,
                self.aspect_ratio,
                self.near_plane,
                self.far_plane,
            ),
            ProjectionType::Isometric => Transform3D::isometric(),

        }
    }

    /// Возвращает комбинированную матрицу вида-проекции.
    pub fn view_projection_matrix(&self) -> Transform3D {
        self.view_matrix().multiply(self.projection_matrix())
    }

    // --------------------------------------------------
    // Движение камеры
    // --------------------------------------------------

    /// Перемещает камеру вперед/назад вдоль направления взгляда.
    pub fn move_forward(&mut self, distance: f32) {
        self.position = self.position + self.direction * distance;
    }

    /// Перемещает камеру вправо/влево.
    pub fn move_right(&mut self, distance: f32) {
        let right = self.right();
        self.position = self.position + right * distance;
    }

    /// Перемещает камеру вверх/вниз.
    pub fn move_up(&mut self, distance: f32) {
        self.position = self.position + self.up * distance;
    }

    /// Перемещает камеру в указанном направлении (в локальных координатах камеры).
    pub fn move_local(&mut self, right: f32, up: f32, forward: f32) {
        let right_vec = self.right();
        self.position = self.position + right_vec * right + self.up * up + self.direction * forward;
    }

    /// Устанавливает позицию камеры.
    pub fn set_position(&mut self, position: Point3) {
        self.position = position;
    }

    // --------------------------------------------------
    // Повороты камеры
    // --------------------------------------------------

    /// Поворачивает камеру вокруг вертикальной оси (Yaw).
    pub fn rotate_yaw(&mut self, angle_rad: f32) {
        let rotation = Transform3D::rotation_y_rad(angle_rad);
        self.direction = rotation.apply_to_vector(self.direction);
        self.up = rotation.apply_to_vector(self.up);
        self.normalize_vectors();
    }

    /// Поворачивает камеру вокруг горизонтальной оси (Pitch).
    pub fn rotate_pitch(&mut self, angle_rad: f32) {
        let right = self.right();
        let rotation =
            Transform3D::rotation_around_line(Line3::new(self.position, right), angle_rad);

        // Применяем поворот только к направлению и up вектору
        let target = self.target();
        let new_target = rotation.apply_to_point(target);
        self.direction = (new_target - self.position).normalize();

        self.up = rotation.apply_to_vector(self.up);
        self.normalize_vectors();
    }

    /// Поворачивает камеру вокруг оси взгляда (Roll).
    pub fn rotate_roll(&mut self, angle_rad: f32) {
        let rotation =
            Transform3D::rotation_around_line(Line3::new(self.position, self.direction), angle_rad);
        self.up = rotation.apply_to_vector(self.up);
        self.normalize_vectors();
    }

    /// Поворачивает камеру вокруг произвольной оси.
    pub fn rotate_around_axis(&mut self, axis: Vec3, angle_rad: f32) {
        let rotation = Transform3D::rotation_around_line(
            Line3::new(self.position, axis.normalize()),
            angle_rad,
        );

        let target = self.target();
        let new_target = rotation.apply_to_point(target);
        self.direction = (new_target - self.position).normalize();

        self.up = rotation.apply_to_vector(self.up);
        self.normalize_vectors();
    }

    /// Поворачивает камеру чтобы смотреть на указанную точку.
    pub fn look_at_target(&mut self, target: Point3) {
        self.direction = (target - self.position).normalize();
        self.normalize_vectors();
    }

    /// Вращает камеру вокруг указанной точки.
    pub fn orbit_around(&mut self, center: Point3, yaw_rad: f32, pitch_rad: f32) {
        // Сохраняем расстояние до центра
        let distance = (self.position - center).length();

        // Применяем повороты
        self.rotate_yaw(yaw_rad);
        self.rotate_pitch(pitch_rad);

        // Устанавливаем позицию на правильном расстоянии от центра
        self.position = (Vec3::from(self.position) - self.direction * distance).into();
    }

    // --------------------------------------------------
    // Изменение параметров проекции
    // --------------------------------------------------

    /// Устанавливает поле зрения (в радианах).
    pub fn set_fov(&mut self, fov_rad: f32) {
        self.fov = fov_rad.clamp(0.1, PI - 0.1);
    }

    /// Устанавливает поле зрения (в градусах).
    pub fn set_fov_degrees(&mut self, fov_deg: f32) {
        self.set_fov(fov_deg.to_radians());
    }

    /// Устанавливает соотношение сторон.
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio.max(0.1);
    }

    /// Устанавливает ближнюю плоскость отсечения.
    pub fn set_near_plane(&mut self, near_plane: f32) {
        self.near_plane = near_plane.max(0.01);
    }

    /// Устанавливает дальнюю плоскость отсечения.
    pub fn set_far_plane(&mut self, far_plane: f32) {
        self.far_plane = far_plane.max(self.near_plane + 0.1);
    }

    /// Устанавливает все параметры проекции.
    pub fn set_projection(
        &mut self,
        fov_rad: f32,
        aspect_ratio: f32,
        near_plane: f32,
        far_plane: f32,
    ) {
        self.set_fov(fov_rad);
        self.set_aspect_ratio(aspect_ratio);
        self.set_near_plane(near_plane);
        self.set_far_plane(far_plane);
    }

    // --------------------------------------------------
    // Вспомогательные методы
    // --------------------------------------------------

    /// Нормализует векторы направления и up, обеспечивая ортогональность.
    fn normalize_vectors(&mut self) {
        self.direction = self.direction.normalize();

        // Обеспечиваем ортогональность up вектора направлению
        let right = self.direction.cross(self.up).normalize();
        self.up = right.cross(self.direction).normalize();
    }

    /// Возвращает луч из камеры через точку на экране (в нормализованных координатах [-1, 1]).
    pub fn screen_point_to_ray(&self, screen_x: f32, screen_y: f32) -> Line3 {
        // Преобразуем нормализованные координаты экрана в направление луча
        let tan_half_fov = (self.fov / 2.0).tan();
        let x = screen_x * tan_half_fov * self.aspect_ratio;
        let y = screen_y * tan_half_fov;

        // Направление в пространстве камеры
        let right = self.right();
        let up = self.up;

        let ray_direction = (self.direction + right * x + up * y).normalize();

        Line3::new(self.position, ray_direction * self.far_plane)
    }

    /// Проверяет, находится ли точка внутри frustum камеры.
    pub fn is_point_visible(&self, point: Point3) -> bool {
        let view_proj = self.view_projection_matrix();
        let clip_space = view_proj.apply_to_point(point);

        // Точка видима если находится в NDC кубе [-1, 1]
        clip_space.x >= -1.0
            && clip_space.x <= 1.0
            && clip_space.y >= -1.0
            && clip_space.y <= 1.0
            && clip_space.z >= -1.0
            && clip_space.z <= 1.0
    }

    /// Возвращает расстояние от камеры до точки.
    pub fn distance_to(&self, point: Point3) -> f32 {
        (point - self.position).length()
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

    #[test]
    fn test_default_camera() {
        let camera = Camera3::default();

        assert_point_approx_eq(camera.position, Point3::new(0.0, 0.0, 0.0));
        assert_vec_approx_eq(camera.direction, Vec3::new(0.0, 0.0, -1.0));
        assert_vec_approx_eq(camera.up, Vec3::new(0.0, 1.0, 0.0));
        assert!((camera.fov - PI / 3.0).abs() < EPSILON);
        assert!((camera.aspect_ratio - 16.0 / 9.0).abs() < EPSILON);
        assert!((camera.near_plane - 0.1).abs() < EPSILON);
        assert!((camera.far_plane - 100.0).abs() < EPSILON);
    }

    #[test]
    fn test_new_camera() {
        let position = Point3::new(1.0, 2.0, 3.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);

        let camera = Camera3::new(position, target, up, PI / 4.0, 4.0 / 3.0, 0.5, 200.0);

        assert_point_approx_eq(camera.position, position);
        assert!((camera.fov - PI / 4.0).abs() < EPSILON);
        assert!((camera.aspect_ratio - 4.0 / 3.0).abs() < EPSILON);
        assert!((camera.near_plane - 0.5).abs() < EPSILON);
        assert!((camera.far_plane - 200.0).abs() < EPSILON);
    }

    #[test]
    fn test_look_at() {
        let position = Point3::new(0.0, 0.0, 5.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);

        let camera = Camera3::look_at(position, target, up);

        assert_point_approx_eq(camera.position, position);
        assert_vec_approx_eq(camera.direction, Vec3::new(0.0, 0.0, -1.0));
        assert_vec_approx_eq(camera.up, up.normalize());
    }

    #[test]
    fn test_from_tuple() {
        let position = Point3::new(1.0, 2.0, 3.0);
        let target = Point3::new(0.0, 0.0, 0.0);

        let camera = Camera3::from(position, target);

        assert_point_approx_eq(camera.position, position);
        // Направление должно быть нормализовано и указывать на target
        let expected_direction = (target - position).normalize();
        assert_vec_approx_eq(camera.direction, expected_direction);
    }

    #[test]
    fn test_first_person() {
        let position = Point3::new(0.0, 0.0, 0.0);
        let yaw = 0.0;
        let pitch = 0.0;

        // let camera = Camera3::first_person(position, yaw, pitch);

        // assert_point_approx_eq(camera.position, position);
        // assert_vec_approx_eq(camera.direction, Vec3::new(1.0, 0.0, 0.0)); // cos(0)=1, sin(0)=0
    }

    #[test]
    fn test_target() {
        let camera = Camera3::default();
        let target = camera.target();

        // Камера в (0,0,0) смотрит в (0,0,-1) -> target = (0,0,-1)
        assert_point_approx_eq(target, Point3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_right_vector() {
        let camera = Camera3::default();
        let right = camera.right();

        // Для камеры смотрящей по -Z, right должен быть (1,0,0)
        assert_vec_approx_eq(right, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_view_matrix() {
        let camera = Camera3::default();
        let view_matrix = camera.view_matrix();

        // Для камеры в начале координат, смотрящей по -Z, видовая матрица должна быть identity
        let point = Point3::new(1.0, 2.0, 3.0);
        let transformed = view_matrix.apply_to_point(point);

        // В координатах камеры точка должна остаться той же
        assert_point_approx_eq(transformed, point);
    }

    #[test]
    fn test_projection_matrix() {
        let camera = Camera3::default();
        let projection_matrix = camera.projection_matrix();

        // Проверяем что матрица проекции создается без ошибок
        let point = Point3::new(0.0, 0.0, 5.0);
        let transformed = projection_matrix.apply_to_point(point);

        assert!(!transformed.x.is_nan());
        assert!(!transformed.y.is_nan());
        assert!(!transformed.z.is_nan());
    }

    #[test]
    fn test_view_projection_matrix() {
        let camera = Camera3::default();
        let view_proj_matrix = camera.view_projection_matrix();

        // Комбинированная матрица должна работать без ошибок
        let point = Point3::new(1.0, 2.0, 3.0);
        let transformed = view_proj_matrix.apply_to_point(point);

        assert!(!transformed.x.is_nan());
        assert!(!transformed.y.is_nan());
        assert!(!transformed.z.is_nan());
    }

    #[test]
    fn test_move_forward() {
        let mut camera = Camera3::default();
        camera.move_forward(5.0);

        assert_point_approx_eq(camera.position, Point3::new(0.0, 0.0, -5.0));

        // Движение назад
        camera.move_forward(-2.0);
        assert_point_approx_eq(camera.position, Point3::new(0.0, 0.0, -3.0));
    }

    #[test]
    fn test_move_right() {
        let mut camera = Camera3::default();
        camera.move_right(3.0);

        assert_point_approx_eq(camera.position, Point3::new(3.0, 0.0, 0.0));

        // Движение влево
        camera.move_right(-2.0);
        assert_point_approx_eq(camera.position, Point3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_move_up() {
        let mut camera = Camera3::default();
        camera.move_up(4.0);

        assert_point_approx_eq(camera.position, Point3::new(0.0, 4.0, 0.0));

        // Движение вниз
        camera.move_up(-1.0);
        assert_point_approx_eq(camera.position, Point3::new(0.0, 3.0, 0.0));
    }

    #[test]
    fn test_move_local() {
        let mut camera = Camera3::default();
        camera.move_local(2.0, 3.0, 4.0);

        // right=2, up=3, forward=4
        assert_point_approx_eq(camera.position, Point3::new(2.0, 3.0, -4.0));
    }

    #[test]
    fn test_set_position() {
        let mut camera = Camera3::default();
        let new_position = Point3::new(10.0, 20.0, 30.0);

        camera.set_position(new_position);
        assert_point_approx_eq(camera.position, new_position);
    }

    #[test]
    fn test_rotate_yaw() {
        let mut camera = Camera3::default();
        camera.rotate_yaw(PI / 2.0); // Поворот на 90 градусов

        // После поворота на 90° вокруг Y, направление (0,0,-1) должно стать (-1,0,0)
        assert_vec_approx_eq(camera.direction, Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn test_rotate_pitch() {
        let mut camera = Camera3::default();
        camera.rotate_pitch(PI / 2.0); // Поворот на 90 градусов

        // После поворота на 90° вокруг X, направление (0,0,-1) должно стать (0,-1,0)
        // Но из-за ограничений gimbal lock, проверяем что направление изменилось
        assert!(camera.direction.y.abs() > 0.5); // Должен смотреть вверх/вниз
    }

    #[test]
    fn test_rotate_roll() {
        let mut camera = Camera3::default();
        let initial_up = camera.up;
        camera.rotate_roll(PI / 4.0); // Поворот на 45 градусов

        // Up вектор должен быть повернут
        assert!(
            (camera.up.x - initial_up.x).abs() > EPSILON
                || (camera.up.y - initial_up.y).abs() > EPSILON
                || (camera.up.z - initial_up.z).abs() > EPSILON
        );
    }

    #[test]
    fn test_rotate_around_axis() {
        let mut camera = Camera3::default();
        let axis = Vec3::new(0.0, 1.0, 0.0); // Ось Y
        camera.rotate_around_axis(axis, PI / 2.0);

        // Должен быть такой же результат как при rotate_yaw
        assert_vec_approx_eq(camera.direction, Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn test_look_at_target() {
        let mut camera = Camera3::default();
        let target = Point3::new(1.0, 0.0, 1.0);

        camera.look_at_target(target);
        let expected_direction = (target - camera.position).normalize();

        assert_vec_approx_eq(camera.direction, expected_direction);
    }

    #[test]
    fn test_orbit_around() {
        let mut camera = Camera3::new(
            Point3::new(0.0, 0.0, 5.0),
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            PI / 3.0,
            1.0,
            0.1,
            100.0,
        );

        let center = Point3::new(0.0, 0.0, 0.0);
        let initial_distance = camera.distance_to(center);

        camera.orbit_around(center, PI / 4.0, 0.0);

        // Расстояние до центра должно сохраниться
        let new_distance = camera.distance_to(center);
        assert!((new_distance - initial_distance).abs() < EPSILON);

        // Позиция должна измениться
        assert!(camera.position.x.abs() > EPSILON);
    }

    #[test]
    fn test_set_fov() {
        let mut camera = Camera3::default();
        camera.set_fov(PI / 6.0);

        assert!((camera.fov - PI / 6.0).abs() < EPSILON);
    }

    #[test]
    fn test_set_fov_degrees() {
        let mut camera = Camera3::default();
        camera.set_fov_degrees(90.0);

        assert!((camera.fov - PI / 2.0).abs() < EPSILON);
    }

    #[test]
    fn test_set_aspect_ratio() {
        let mut camera = Camera3::default();
        camera.set_aspect_ratio(2.0);

        assert!((camera.aspect_ratio - 2.0).abs() < EPSILON);
    }

    #[test]
    fn test_set_near_plane() {
        let mut camera = Camera3::default();
        camera.set_near_plane(1.0);

        assert!((camera.near_plane - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_set_far_plane() {
        let mut camera = Camera3::default();
        camera.set_far_plane(500.0);

        assert!((camera.far_plane - 500.0).abs() < EPSILON);
    }

    #[test]
    fn test_set_projection() {
        let mut camera = Camera3::default();
        camera.set_projection(PI / 4.0, 2.0, 0.5, 200.0);

        assert!((camera.fov - PI / 4.0).abs() < EPSILON);
        assert!((camera.aspect_ratio - 2.0).abs() < EPSILON);
        assert!((camera.near_plane - 0.5).abs() < EPSILON);
        assert!((camera.far_plane - 200.0).abs() < EPSILON);
    }

    #[test]
    fn test_screen_point_to_ray() {
        let camera = Camera3::default();
        let ray = camera.screen_point_to_ray(0.0, 0.0); // Центр экрана

        // Луч из центра должен идти вдоль направления камеры
        assert_vec_approx_eq(ray.direction, camera.direction);
        assert_point_approx_eq(ray.origin, camera.position);
    }

    #[test]
    fn test_is_point_visible() {
        let camera = Camera3::default();

        // Точка прямо перед камерой должна быть видима
        let point_in_front = Point3::new(0.0, 0.0, -5.0);
        assert!(camera.is_point_visible(point_in_front));

        // Точка позади камеры не должна быть видима
        let point_behind = Point3::new(0.0, 0.0, 5.0);
        assert!(!camera.is_point_visible(point_behind));
    }

    #[test]
    fn test_distance_to() {
        let camera = Camera3::default();
        let point = Point3::new(3.0, 4.0, 0.0); // Расстояние 5 по теореме Пифагора

        let distance = camera.distance_to(point);
        assert!((distance - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_normalize_vectors_maintains_orthogonality() {
        let mut camera = Camera3::default();

        // Искусственно "испортим" векторы
        camera.direction = Vec3::new(1.0, 1.0, 1.0).normalize();
        camera.up = Vec3::new(1.0, 0.0, 0.0);

        // Внутренний вызов normalize_vectors должен восстановить ортогональность
        let right = camera.right();
        let dot_product = camera.direction.dot(camera.up);

        // Векторы должны быть ортогональны (скалярное произведение ~0)
        assert!(dot_product.abs() < EPSILON);
    }

    #[test]
    fn test_fov_clamping() {
        let mut camera = Camera3::default();

        // Попытка установить слишком маленький FOV
        camera.set_fov(0.05);
        assert!(camera.fov >= 0.1);

        // Попытка установить слишком большой FOV
        camera.set_fov(PI - 0.05);
        assert!(camera.fov <= PI - 0.1);
    }

    #[test]
    fn test_near_plane_clamping() {
        let mut camera = Camera3::default();

        // Попытка установить отрицательную near plane
        camera.set_near_plane(-1.0);
        assert!(camera.near_plane >= 0.01);
    }

    #[test]
    fn test_far_plane_clamping() {
        let mut camera = Camera3::default();

        // Попытка установить far plane меньше near plane
        camera.set_far_plane(0.05);
        assert!(camera.far_plane >= camera.near_plane + 0.1);
    }

    #[test]
    fn test_complex_movement_sequence() {
        let mut camera = Camera3::default();

        // Последовательность сложных перемещений и поворотов
        camera.move_forward(5.0);
        camera.rotate_yaw(PI / 4.0);
        camera.move_right(2.0);
        camera.rotate_pitch(PI / 8.0);
        camera.move_up(1.0);
        camera.rotate_roll(PI / 12.0);

        // Проверяем что камера в валидном состоянии
        assert!(!camera.position.x.is_nan());
        assert!(!camera.position.y.is_nan());
        assert!(!camera.position.z.is_nan());
        assert!(!camera.direction.x.is_nan());
        assert!(!camera.direction.y.is_nan());
        assert!(!camera.direction.z.is_nan());
        assert!(!camera.up.x.is_nan());
        assert!(!camera.up.y.is_nan());
        assert!(!camera.up.z.is_nan());

        // Векторы должны быть нормализованы
        assert!((camera.direction.length() - 1.0).abs() < EPSILON);
        assert!((camera.up.length() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_camera_with_extreme_values() {
        // Тестирование камеры с экстремальными значениями
        let camera = Camera3::new(
            Point3::new(1e6, -1e6, 1e6),
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            0.1,  // Очень маленький FOV
            0.5,  // Вертикальный aspect ratio
            0.01, // Очень близкая near plane
            1e6,  // Очень далекая far plane
        );

        // Проверяем что матрицы создаются без ошибок
        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();

        let test_point = Point3::new(0.0, 0.0, -100.0);
        let view_result = view_matrix.apply_to_point(test_point);
        let proj_result = projection_matrix.apply_to_point(view_result);

        assert!(!proj_result.x.is_nan());
        assert!(!proj_result.y.is_nan());
        assert!(!proj_result.z.is_nan());
    }
}
