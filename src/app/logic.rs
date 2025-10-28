use crate::app::AthenianApp;
use egui::{Color32, Painter, Pos2, Response, Ui};
use g3d::{Line3, Model3, Plane, Point3, Transform3D, Transformable3, Vec3};

// --------------------------------------------------
// Обработка области рисования (холст)
// --------------------------------------------------

impl AthenianApp {
    /// Выделить egui::painter на всю свободную область указанного UI элемента.
    pub fn allocate_painter(&mut self, ui: &mut Ui) -> (Response, Painter) {
        let available_size = ui.available_size();
        self.painter_width = available_size.x;
        self.painter_height = available_size.y;

        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(self.painter_width, self.painter_height),
            egui::Sense::click_and_drag(),
        );

        // цвет холста
        painter.rect_filled(response.rect, 0.0, Color32::WHITE);

        (response, painter)
    }

    /// Нарисовать холст.
    pub fn draw_canvas(&mut self, painter: &mut Painter) {
        let style = g3d::RenderStyle::default();
        self.scene.render(self.camera.clone(), painter, &style);

        if self.instrument == Instrument::RotateAroundCustomLine {
            self.draw_custom_axis_line(painter);
        }
    }

    /// Отрисовать линию, заданную двумя точками
    fn draw_custom_axis_line(&self, painter: &Painter) {
        let point1 = Point3::new(self.axis_point1_x, self.axis_point1_y, self.axis_point1_z);
        let point2 = Point3::new(self.axis_point2_x, self.axis_point2_y, self.axis_point2_z);

        let screen_point1 = self.world_to_screen(point1);
        let screen_point2 = self.world_to_screen(point2);

        let direction = egui::Vec2 {
            x: screen_point2.x - screen_point1.x,
            y: screen_point2.y - screen_point1.y,
        }
        .normalized();

        let line_length = (self.painter_width + self.painter_height) * 2.0; // Большая длина для выхода за границы

        let extended_start = screen_point1 - direction * line_length;
        let extended_end = screen_point2 + direction * line_length;

        painter.line_segment(
            [extended_start, extended_end],
            egui::Stroke::new(2.0, Color32::RED),
        );

        painter.circle_filled(screen_point1, 4.0, Color32::GREEN);
        painter.circle_filled(screen_point2, 4.0, Color32::BLUE);
    }

    /// Преобразовать мировые координаты в экранные
    fn world_to_screen(&self, point: Point3) -> Pos2 {
        // Применяем преобразования камеры к точке
        let view_proj = self.camera.view_projection_matrix();
        let transformed = view_proj.apply_to_point(point);

        // Преобразуем из NDC (-1..1) в экранные координаты
        let screen_x = (transformed.x + 1.0) * 0.5 * self.painter_width;
        let screen_y = (1.0 - transformed.y) * 0.5 * self.painter_height;

        Pos2::new(screen_x, screen_y)
    }
}

// --------------------------------------------------
// Обработка управления
// --------------------------------------------------

impl AthenianApp {
    /// Обработать взаимодействие с холстом.
    pub fn handle_input(&mut self, response: &Response) {
        self.handle_click(response);
        self.handle_drag(response);
    }

    /// Обработать клики по холсту.
    fn handle_click(&mut self, response: &Response) {
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pos) = response.hover_pos() {
                match &self.instrument {
                    _ => {
                        // Если есть фигура на сцене, автоматически выбираем её
                        if !self.scene.models.is_empty() {
                            self.selected_3d_model_index = Some(0);
                        }
                    }
                }
            }
        }
    }

    /// Обработать перетаскивание по холсту.
    fn handle_drag(&mut self, response: &Response) {
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            self.drag_prev_pos = None;
            return;
        }

        if !response.dragged_by(egui::PointerButton::Primary) {
            return;
        }

        if let Some(drag_start) = self.drag_prev_pos
            && let Some(drag_cur) = response.hover_pos()
        {
            self.handle_3d_drag(drag_start, drag_cur);
        }

        self.drag_prev_pos = response.hover_pos();
    }

    /// Обработать перетаскивание для 3D.
    fn handle_3d_drag(&mut self, start: egui::Pos2, end: egui::Pos2) {
        // Если есть фигура на сцене, применяем преобразования сразу
        if !self.scene.models.is_empty() {
            match &self.instrument {
                Instrument::Move3D => self.move_3d_model(start, end),
                Instrument::Rotate3D => self.rotate_3d_model(start, end),
                Instrument::Scale3D => self.scale_3d_model(start, end),
                Instrument::RotateAroundX => {
                    self.rotate_around_axis(AxisType::Center(CenterAxis::X), start, end)
                }
                Instrument::RotateAroundY => {
                    self.rotate_around_axis(AxisType::Center(CenterAxis::Y), start, end)
                }
                Instrument::RotateAroundZ => {
                    self.rotate_around_axis(AxisType::Center(CenterAxis::Z), start, end)
                }
                Instrument::RotateAroundCustomLine => {
                    self.rotate_around_axis(AxisType::Custom, start, end)
                }
                _ => {}
            }
        }
    }
}

// --------------------------------------------------
// Взаимодействие с 3D моделями
// --------------------------------------------------

impl AthenianApp {
    /// Переместить 3D модель (применяется сразу к выбранной фигуре)
    fn move_3d_model(&mut self, start: egui::Pos2, end: egui::Pos2) {
        if let Some(_) = self.get_selected_model() {
            let delta_x = (end.x - start.x) / self.painter_width * 4.0;
            let delta_y = (end.y - start.y) / self.painter_height * 4.0;

            let translation = Transform3D::translation(delta_x, -delta_y, 0.0);
            if let Some(model) = self.get_selected_model_mut() {
                model.apply_transform(translation);
            }
        }
    }

    /// Повернуть 3D модель (применяется сразу к выбранной фигуре)
    fn rotate_3d_model(&mut self, start: egui::Pos2, end: egui::Pos2) {
        if let Some(model) = self.get_selected_model() {
            let delta_x = (end.x - start.x) / self.painter_width * std::f32::consts::PI;
            let delta_y = (end.y - start.y) / self.painter_height * std::f32::consts::PI;

            let center = model.get_origin();

            // Вращение вокруг осей
            let rotation_x = Transform3D::rotation_around_line(
                Line3::new(center, Vec3::new(1.0, 0.0, 0.0)),
                delta_y,
            );
            let rotation_y = Transform3D::rotation_around_line(
                Line3::new(center, Vec3::new(0.0, 1.0, 0.0)),
                delta_x,
            );

            let combined_rotation = rotation_x.multiply(rotation_y);
            if let Some(model) = self.get_selected_model_mut() {
                model.apply_transform(combined_rotation);
            }
        }
    }

    /// Масштабировать 3D модель (применяется сразу к выбранной фигуре)
    fn scale_3d_model(&mut self, start: egui::Pos2, end: egui::Pos2) {
        if let Some(model) = self.get_selected_model() {
            let delta = ((end.x - start.x) + (end.y - start.y))
                / (self.painter_width + self.painter_height);
            let scale_factor = 1.0 + delta * 2.0;

            let center = model.get_origin();
            let scale = Transform3D::scale_relative_to_point(
                center,
                scale_factor,
                scale_factor,
                scale_factor,
            );

            if let Some(model) = self.get_selected_model_mut() {
                model.apply_transform(scale);
            }
        }
    }

    /// Применить отражение относительно выбранной плоскости.
    pub fn apply_reflection(&mut self, plane_type: ReflectionPlane) {
        if let Some(model) = self.get_selected_model() {
            let center = model.get_origin();

            let reflection = match plane_type {
                ReflectionPlane::XY => Transform3D::reflection_xy(),
                ReflectionPlane::XZ => Transform3D::reflection_xz(),
                ReflectionPlane::YZ => Transform3D::reflection_yz(),
            };

            // Применяем отражение относительно центра модели
            let transform = Transform3D::translation(-center.x, -center.y, -center.z)
                .multiply(reflection)
                .multiply(Transform3D::translation(center.x, center.y, center.z));

            if let Some(model) = self.get_selected_model_mut() {
                model.apply_transform(transform);
            }
        }
    }

    // Простые методы для добавления многогранников (заменяют текущую фигуру)
    pub fn add_tetrahedron(&mut self) {
        let mut tetrahedron = Model3::tetrahedron();
        self.center_model_position(&mut tetrahedron);
        self.set_model(tetrahedron);
    }

    pub fn add_hexahedron(&mut self) {
        let mut hexahedron = Model3::hexahedron();
        self.center_model_position(&mut hexahedron);
        self.set_model(hexahedron);
    }

    pub fn add_octahedron(&mut self) {
        let mut octahedron = Model3::octahedron();
        self.center_model_position(&mut octahedron);
        self.set_model(octahedron);
    }

    pub fn add_icosahedron(&mut self) {
        let mut icosahedron = Model3::icosahedron();
        self.center_model_position(&mut icosahedron);
        self.set_model(icosahedron);
    }

    pub fn add_dodecahedron(&mut self) {
        let mut dodecahedron = Model3::dodecahedron();
        self.center_model_position(&mut dodecahedron);
        self.set_model(dodecahedron);
    }

    /// Установить перспективную проекцию
    pub fn set_perspective_projection(&mut self) {
        self.current_projection = Projection::Perspective;
        self.camera.projection_type = g3d::ProjectionType::Perspective;
    }

    /// Установить изометрическую проекцию
    pub fn set_isometric_projection(&mut self) {
        self.current_projection = Projection::Isometric;
        self.camera.projection_type = g3d::ProjectionType::Isometric;
        self.camera.position = Point3::new(5.0, 5.0, 5.0);
        self.camera.look_at_target(Point3::new(0.0, 0.0, 0.0));
    }

    /// Установить диметрическую проекцию
    pub fn set_dimetric_projection(&mut self) {
        self.current_projection = Projection::Dimetric;
        self.camera.projection_type = g3d::ProjectionType::Dimetric {
            angle_x: 20.0,
            angle_z: 30.0,
            scale_y: 0.5,
        };
    }

    /// Установить триметрическую проекцию
    pub fn set_trimetric_projection(&mut self) {
        self.current_projection = Projection::Trimetric;
        self.camera.projection_type = g3d::ProjectionType::Trimetric {
            angle_x: 15.0,
            angle_z: 25.0,
            scale_x: 1.0,
            scale_y: 0.7,
            scale_z: 0.9,
        };
    }

    /// Установить кабинетную проекцию
    pub fn set_cabinet_projection(&mut self) {
        self.current_projection = Projection::Cabinet;
        self.camera.projection_type = g3d::ProjectionType::Cabinet {
            angle: 45.0,
            depth_scale: 0.5,
        };
        self.camera.position = Point3::new(0.0, 0.0, 10.0);
        self.camera.look_at_target(Point3::new(0.0, 0.0, 0.0));
    }

    /// Установить кавальерную проекцию
    pub fn set_cavalier_projection(&mut self) {
        self.current_projection = Projection::Cavalier;
        self.camera.projection_type = g3d::ProjectionType::Cavalier {
            angle: 45.0,
            depth_scale: 1.0,
        };
        self.camera.position = Point3::new(0.0, 0.0, 10.0);
        self.camera.look_at_target(Point3::new(0.0, 0.0, 0.0));
    }
    /// Применить матрицу проекции ко всем моделям на сцене
    fn apply_projection_transform(&mut self, projection: Transform3D) {
        // Для специальных проекций применяем преобразование к моделям
        // и используем простую камеру
        self.camera.position = Point3::new(0.0, 0.0, 10.0);
        self.camera.look_at_target(Point3::new(0.0, 0.0, 0.0));

        // Применяем проекцию ко всем моделям на сцене
        for i in 0..self.scene.models.len() {
            let model = self.scene.models[i].clone();
            let transformed_model = model.transform(projection);
            self.scene.models[i] = transformed_model;
        }
    }

    /// Повернуть модель вокруг оси (мышью)
    pub fn rotate_around_axis(&mut self, axis_type: AxisType, start: egui::Pos2, end: egui::Pos2) {
        if let Some(model) = self.get_selected_model() {
            let line = match axis_type {
                AxisType::Center(center_axis) => {
                    let center = model.get_origin();
                    let axis_vector = match center_axis {
                        CenterAxis::X => Vec3::new(1.0, 0.0, 0.0),
                        CenterAxis::Y => Vec3::new(0.0, 1.0, 0.0),
                        CenterAxis::Z => Vec3::new(0.0, 0.0, 1.0),
                    };
                    Line3::new(center, axis_vector)
                }
                AxisType::Custom => {
                    let point1 =
                        Point3::new(self.axis_point1_x, self.axis_point1_y, self.axis_point1_z);
                    let point2 =
                        Point3::new(self.axis_point2_x, self.axis_point2_y, self.axis_point2_z);
                    let direction = (point2 - point1).normalize();
                    Line3::new(point1, direction)
                }
            };

            // Вычисляем угол вращения на основе перемещения мыши
            let delta_x = (end.x - start.x) / self.painter_width * std::f32::consts::PI;
            let angle = delta_x * 2.0; // Множитель для более плавного вращения

            let rotation = Transform3D::rotation_around_line(line, angle);

            if let Some(model) = self.get_selected_model_mut() {
                model.apply_transform(rotation);
            }

            println!("Rotated around axis by {:.2} radians", angle);
        }
    }
}

// Вспомогательные функции
fn distance_point_to_line(point: Point3, line_origin: Point3, line_direction: Vec3) -> f32 {
    let point_vec: Vec3 = point.into();
    let origin_vec: Vec3 = line_origin.into();

    let v = point_vec - origin_vec;
    let d = line_direction;

    let cross = v.cross(d);
    cross.length() / d.length()
}

#[derive(Default, PartialEq)]
pub enum Instrument {
    #[default]
    Move3D,
    Rotate3D,
    Scale3D,
    RotateAroundX,
    RotateAroundY,
    RotateAroundZ,
    RotateAroundCustomLine,
}

impl ToString for Instrument {
    fn to_string(&self) -> String {
        match self {
            Self::Move3D => String::from("переместить 3D модель"),
            Self::Rotate3D => String::from("повернуть 3D модель"),
            Self::Scale3D => String::from("масштабировать 3D модель"),
            Self::RotateAroundX => String::from("вращать вокруг оси X"),
            Self::RotateAroundY => String::from("вращать вокруг оси Y"),
            Self::RotateAroundZ => String::from("вращать вокруг оси Z"),
            Self::RotateAroundCustomLine => String::from("вращать вокруг произвольной линии"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Projection {
    #[default]
    Perspective,
    Isometric,
    Dimetric,
    Trimetric,
    Cabinet,
    Cavalier,
}

impl ToString for Projection {
    fn to_string(&self) -> String {
        match self {
            Self::Perspective => "Perspective".to_string(),
            Self::Isometric => "Isometric".to_string(),
            Self::Dimetric => "Dimetric".to_string(),
            Self::Trimetric => "Trimetric".to_string(),
            Self::Cabinet => "Cabinet".to_string(),
            Self::Cavalier => "Cavalier".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReflectionPlane {
    XY,
    XZ,
    YZ,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CenterAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisType {
    Center(CenterAxis),
    Custom,
}
