use crate::{
    Camera3, Canvas, HVec3, Model3, Point3, ProjectionType, RenderType, Scene, Transform3D,
};
use egui::{Color32, Pos2};

impl Scene {
    /// Создать пустую сцену.
    pub fn new(camera: Camera3) -> Self {
        Self {
            models: Vec::new(),
            camera,
        }
    }

    /// Добавить новую модель на сцену.
    pub fn add_model(&mut self, model: Model3) {
        self.models.push(model);
    }

    /// Нарисовать сцену на холст со всеми нужными преобразованиями.
    pub fn render(
        &self,
        canvas: &mut Canvas,
        projection_type: ProjectionType,
        render_type: RenderType,
        show_custom_axis: bool,
        axis_point1: Point3,
        axis_point2: Point3,
    ) {
        // Стереть прошлый кадр.
        canvas.clear(Color32::GRAY);
        let global_to_screen_transform =
            self.get_view_projection_transform(projection_type, canvas);

        self.draw_coordinate_axes(canvas, &global_to_screen_transform);

        if show_custom_axis {
            self.draw_custom_axis_line(
                canvas,
                &global_to_screen_transform,
                axis_point1,
                axis_point2,
            );
        }

        for model in &self.models {
            match render_type {
                RenderType::WireFrame => {
                    self.render_model_wireframe(canvas, global_to_screen_transform, model)
                }
            }
        }
    }

    /// Получить матрицу преобразования из глоабльных координат в экранные (viewport, он же canvas)
    fn get_view_projection_transform(
        &self,
        projection_type: ProjectionType,
        canvas: &Canvas,
    ) -> Transform3D {
        let proj_matrix = match projection_type {
            ProjectionType::Parallel => Transform3D::parallel_symmetric(
                canvas.width as f32,
                canvas.height as f32,
                self.camera.get_near_plane(),
                self.camera.get_far_plane(),
            ),
            ProjectionType::Perspective => Transform3D::perspective(
                self.camera.get_fov(),
                self.camera.get_aspect_ratio(),
                self.camera.get_near_plane(),
                self.camera.get_far_plane(),
            ),
        };

        let scale_x = canvas.width as f32 / 2.0; // растянуть NDC по ширине
        let scale_y = canvas.height as f32 / 2.0; // растянуть NDC по высоте

        self.camera
            .get_local_frame()
            .global_to_local_matrix()
            .multiply(proj_matrix) // вот тут получается NDC с координатами [-1, +1]
            .multiply(Transform3D::translation_uniform(1.0)) // теперь координаты [0, +2]
            .multiply(Transform3D::scale(scale_x, scale_y, 1.0)) // теперь экранные
    }

    /// Отрисовка пользовательской оси для вращения
    fn draw_custom_axis_line(
        &self,
        canvas: &mut Canvas,
        view_proj_matrix: &Transform3D,
        point1: Point3,
        point2: Point3,
    ) {
        // Проецируем точки в 2D используя нашу систему проекций
        let screen_point1 = self.project_point(point1, view_proj_matrix);
        let screen_point2 = self.project_point(point2, view_proj_matrix);

        // Вычисляем направление линии
        let direction = (screen_point2 - screen_point1).normalized();

        // Удлиняем линию для лучшей видимости
        let extension_length = 500.0;
        let extended_start = screen_point1 - direction * extension_length;
        let extended_end = screen_point2 + direction * extension_length;

        let orange = Color32::from_rgb(255, 165, 0);
        canvas.draw_sharp_line(extended_start, extended_end, orange);

        canvas.circle_filled(screen_point1, 4.0, Color32::GREEN);
        canvas.circle_filled(screen_point2, 4.0, Color32::BLUE);
    }

    /// Отрисовка координатных осей
    fn draw_coordinate_axes(&self, canvas: &mut Canvas, view_proj_matrix: &Transform3D) {
        let axis_length = 2.0; // Длина осей
        let origin = Point3::new(0.0, 0.0, 0.0);

        let x_axis_end = Point3::new(axis_length, 0.0, 0.0);
        let y_axis_end = Point3::new(0.0, axis_length, 0.0);
        let z_axis_end = Point3::new(0.0, 0.0, axis_length);

        let origin_2d = self.project_point(origin, view_proj_matrix);
        let x_end_2d = self.project_point(x_axis_end, view_proj_matrix);
        let y_end_2d = self.project_point(y_axis_end, view_proj_matrix);
        let z_end_2d = self.project_point(z_axis_end, view_proj_matrix);

        // Рисуем оси с разными цветами
        // Ось X - красная
        canvas.draw_sharp_line(origin_2d, x_end_2d, Color32::RED);

        // Ось Y - зелёная
        canvas.draw_sharp_line(origin_2d, y_end_2d, Color32::GREEN);

        // Ось Z - синяя
        canvas.draw_sharp_line(origin_2d, z_end_2d, Color32::BLUE);
    }

    fn project_point(&self, point: Point3, view_proj_matrix: &Transform3D) -> Pos2 {
        let proj_point: Point3 = view_proj_matrix.apply_to_hvec(point.into()).into();
        Pos2::new(proj_point.x, proj_point.y)
    }

    fn render_model_wireframe(
        &self,
        canvas: &mut Canvas,
        view_proj_matrix: Transform3D,
        model: &Model3,
    ) {
        // Преобразование из координат модели в проекцию NDC.
        let full_transform = model
            .get_mesh()
            .get_local_frame()
            .local_to_global_matrix()
            .multiply(view_proj_matrix);

        let projected_vertexes: Vec<HVec3> = model
            .get_mesh()
            .get_vertexes()
            .iter()
            .map(|vertex| vertex.apply_transform(&full_transform))
            .collect();

        // Рисуем рёбра
        for polygon in model.get_mesh().get_polygons() {
            // Проекция вершин полигона
            let points: Vec<HVec3> = polygon
                .get_vertexes()
                .iter()
                .map(|&index| projected_vertexes[index])
                .collect();

            for i in 0..points.len() {
                let start = points[i];
                let end = points[(i + 1) % points.len()];

                let start_pos = Pos2::new(start.x, start.y);
                let end_pos = Pos2::new(end.x, end.y);
                canvas.draw_sharp_line(start_pos, end_pos, Color32::WHITE);
            }
        }

        // Рисуем вершины
        for &vertex in &projected_vertexes {
            let pos = Pos2::new(vertex.x, vertex.y);
            canvas.circle_filled(pos, 3.0, Color32::WHITE);
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(Camera3::default())
    }
}
