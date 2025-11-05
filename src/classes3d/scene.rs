use crate::{Camera3, Model3, Point3, RenderStyle, Scene, Transformable3};
use egui::{Color32, Painter};

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

    //Нарисовать сцену на экран со всеми нужными преобразованиями.
    pub fn render(
        &self,
        camera: Camera3,
        painter: &mut Painter,
        style: &RenderStyle,
        show_custom_axis: bool,
        axis_point1: Point3,
        axis_point2: Point3,
    ) {
        self.draw_coordinate_axes(painter, &camera);

        if show_custom_axis {
            self.draw_custom_axis_line(painter, &camera, axis_point1, axis_point2);
        }

        for model in &self.models {
            self.render_simple(model, painter, style, &camera);
        }
    }

    /// Отрисовка пользовательской оси для вращения
    fn draw_custom_axis_line(
        &self,
        painter: &Painter,
        camera: &Camera3,
        point1: Point3,
        point2: Point3,
    ) {
        // Проецируем точки в 2D используя нашу систему проекций
        let screen_point1 = self.project_point(point1, camera);
        let screen_point2 = self.project_point(point2, camera);

        // Вычисляем направление линии
        let direction = (screen_point2 - screen_point1).normalized();

        // Удлиняем линию для лучшей видимости
        let extension_length = 500.0;
        let extended_start = screen_point1 - direction * extension_length;
        let extended_end = screen_point2 + direction * extension_length;

        painter.line_segment(
            [extended_start, extended_end],
            egui::Stroke::new(2.0, Color32::from_rgb(255, 165, 0)), // Оранжевый цвет
        );

        painter.circle_filled(screen_point1, 4.0, Color32::GREEN);
        painter.circle_filled(screen_point2, 4.0, Color32::BLUE);
    }

    /// Отрисовка координатных осей
    fn draw_coordinate_axes(&self, painter: &mut Painter, camera: &Camera3) {
        let axis_length = 2.0; // Длина осей
        let origin = Point3::new(0.0, 0.0, 0.0);

        let x_axis_end = Point3::new(axis_length, 0.0, 0.0);
        let y_axis_end = Point3::new(0.0, axis_length, 0.0);
        let z_axis_end = Point3::new(0.0, 0.0, axis_length);

        let origin_2d = self.project_point(origin, camera);
        let x_end_2d = self.project_point(x_axis_end, camera);
        let y_end_2d = self.project_point(y_axis_end, camera);
        let z_end_2d = self.project_point(z_axis_end, camera);

        // Рисуем оси с разными цветами
        // Ось X - красная
        painter.line_segment([origin_2d, x_end_2d], egui::Stroke::new(3.0, Color32::RED));
        painter.text(
            x_end_2d + egui::Vec2::new(5.0, -5.0),
            egui::Align2::LEFT_TOP,
            "X",
            egui::FontId::default(),
            Color32::RED,
        );

        // Ось Y - зелёная
        painter.line_segment(
            [origin_2d, y_end_2d],
            egui::Stroke::new(3.0, Color32::GREEN),
        );
        painter.text(
            y_end_2d + egui::Vec2::new(5.0, -5.0),
            egui::Align2::LEFT_TOP,
            "Y",
            egui::FontId::default(),
            Color32::GREEN,
        );

        // Ось Z - синяя
        painter.line_segment([origin_2d, z_end_2d], egui::Stroke::new(3.0, Color32::BLUE));
        painter.text(
            z_end_2d + egui::Vec2::new(5.0, -5.0),
            egui::Align2::LEFT_TOP,
            "Z",
            egui::FontId::default(),
            Color32::BLUE,
        );

        // Рисуем начало координат
        painter.circle_filled(origin_2d, 4.0, Color32::BLACK);
        painter.text(
            origin_2d + egui::Vec2::new(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            "O",
            egui::FontId::default(),
            Color32::BLACK,
        );
    }

    fn render_simple(
        &self,
        model: &Model3,
        painter: &mut Painter,
        style: &RenderStyle,
        camera: &Camera3,
    ) {
        let projected_points: Vec<egui::Pos2> = model
            .get_vertexes()
            .iter()
            .map(|vertex| {
                // Применяем мировое преобразование модели
                let world_vertex = *vertex + model.get_origin().into();

                self.project_point(world_vertex, camera)
            })
            .collect();

        // Рисуем рёбра
        for polygon in model.get_polygons() {
            if polygon.vertexes.len() >= 2 {
                let points: Vec<egui::Pos2> = polygon
                    .vertexes
                    .iter()
                    .map(|&index| projected_points[index])
                    .collect();

                for i in 0..points.len() {
                    let start = points[i];
                    let end = points[(i + 1) % points.len()];
                    painter.line_segment([start, end], (style.edge_width, style.edge_color));
                }
            }
        }

        // Рисуем вершины
        for &point in &projected_points {
            painter.circle_filled(point, style.vertex_radius, style.vertex_color);
        }
    }

    /// Проецирование 3D точки в 2D в зависимости от типа проекции
    fn project_point(&self, point: Point3, camera: &Camera3) -> egui::Pos2 {
        match camera.projection_type {
            crate::ProjectionType::Perspective => {
                // Перспективная проекция
                let distance = 5.0; // Расстояние от камеры
                let scale = 100.0;
                let center_x = 450.0;
                let center_y = 300.0;

                let factor = distance / (distance + point.z);
                let x_proj = point.x * factor;
                let y_proj = point.y * factor;

                egui::Pos2::new(center_x + x_proj * scale, center_y - y_proj * scale)
            }
            crate::ProjectionType::Isometric => {
                // Изометрическая проекция
                let scale = 80.0;
                let center_x = 450.0;
                let center_y = 300.0;

                // Стандартная изометрия: углы 30° для X и Z
                let x_proj = point.x * 0.866 - point.z * 0.866; // cos(30°) = 0.866
                let y_proj = point.y + (point.x + point.z) * 0.5; // sin(30°) = 0.5

                egui::Pos2::new(center_x + x_proj * scale, center_y - y_proj * scale)
            }
            _ => {
                // По умолчанию - ортографическая проекция
                let scale = 100.0;
                let center_x = 450.0;
                let center_y = 300.0;

                egui::Pos2::new(center_x + point.x * scale, center_y - point.y * scale)
            }
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(Camera3::default())
    }
}
