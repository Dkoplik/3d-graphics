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

    /// Установить первую точку оси.
    pub fn set_axis_point1(&mut self, pos: egui::Pos2) {
        // Преобразуем экранные координаты в 3D координаты на плоскости z=0
        let screen_x = (pos.x / self.painter_width) * 4.0 - 2.0;
        let screen_y = 2.0 - (pos.y / self.painter_height) * 4.0;

        self.axis_point1 = Point3::new(screen_x, screen_y, 0.0);

        println!(
            "Axis point 1 set to: ({}, {}, {})",
            self.axis_point1.x, self.axis_point1.y, self.axis_point1.z
        );
    }

    /// Установить вторую точку оси.
    pub fn set_axis_point2(&mut self, pos: egui::Pos2) {
        // Преобразуем экранные координаты в 3D координаты на плоскости z=0
        let screen_x = (pos.x / self.painter_width) * 4.0 - 2.0;
        let screen_y = 2.0 - (pos.y / self.painter_height) * 4.0;

        self.axis_point2 = Point3::new(screen_x, screen_y, 0.0);

        println!(
            "Axis point 2 set to: ({}, {}, {})",
            self.axis_point2.x, self.axis_point2.y, self.axis_point2.z
        );
    }

    /// Обработать клики по холсту.
    fn handle_click(&mut self, response: &Response) {
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pos) = response.hover_pos() {
                match &self.instrument {
                    Instrument::SetAxisPoint1 => self.set_axis_point1(pos),
                    Instrument::SetAxisPoint2 => self.set_axis_point2(pos),
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

    /// Масштабировать относительно центра
    pub fn scale_around_center(&mut self, scale_x: f32, scale_y: f32, scale_z: f32) {
        if let Some(model) = self.get_selected_model() {
            let center = model.get_origin();

            let scale = Transform3D::scale_relative_to_point(center, scale_x, scale_y, scale_z);
            if let Some(model) = self.get_selected_model_mut() {
                model.apply_transform(scale);
            }
        }
    }

    /// Повернуть вокруг оси через центр
    pub fn rotate_around_center_axis(&mut self, axis: CenterAxis, angle_degrees: f32) {
        if let Some(model) = self.get_selected_model() {
            let center = model.get_origin();
            let angle_rad = angle_degrees.to_radians();

            let axis_vector = match axis {
                CenterAxis::X => Vec3::new(1.0, 0.0, 0.0),
                CenterAxis::Y => Vec3::new(0.0, 1.0, 0.0),
                CenterAxis::Z => Vec3::new(0.0, 0.0, 1.0),
            };

            let rotation =
                Transform3D::rotation_around_line(Line3::new(center, axis_vector), angle_rad);
            if let Some(model) = self.get_selected_model_mut() {
                model.apply_transform(rotation);
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
        self.camera.set_projection(
            std::f32::consts::PI / 3.0, // 60 degrees FOV
            self.painter_width / self.painter_height,
            0.1,
            100.0,
        );
    }

    /// Установить изометрическую проекцию
    pub fn set_isometric_projection(&mut self) {
        self.current_projection = Projection::Isometric;
        // Для аксонометрических проекций используем специальную камеру
        self.camera.position = Point3::new(5.0, 5.0, 5.0);
        self.camera.look_at_target(Point3::new(0.0, 0.0, 0.0));
        self.camera.set_projection(
            std::f32::consts::PI / 4.0,
            self.painter_width / self.painter_height,
            0.1,
            100.0,
        );
    }

    /// Установить диметрическую проекцию
    pub fn set_dimetric_projection(&mut self) {
        self.current_projection = Projection::Dimetric;
        self.camera.position = Point3::new(7.0, 5.0, 7.0);
        self.camera.look_at_target(Point3::new(0.0, 0.0, 0.0));
        self.camera.set_projection(
            std::f32::consts::PI / 4.0,
            self.painter_width / self.painter_height,
            0.1,
            100.0,
        );
    }

    /// Установить триметрическую проекцию
    pub fn set_trimetric_projection(&mut self) {
        self.current_projection = Projection::Trimetric;
        self.camera.position = Point3::new(8.0, 6.0, 4.0);
        self.camera.look_at_target(Point3::new(0.0, 0.0, 0.0));
        self.camera.set_projection(
            std::f32::consts::PI / 4.0,
            self.painter_width / self.painter_height,
            0.1,
            100.0,
        );
    }

    /// Установить кабинетную проекцию
    pub fn set_cabinet_projection(&mut self) {
        self.current_projection = Projection::Cabinet;
        let cabinet_transform = Transform3D::cabinet(45.0, 0.5);
        self.apply_projection_transform(cabinet_transform);
    }

    /// Установить кавальерную проекцию
    pub fn set_cavalier_projection(&mut self) {
        self.current_projection = Projection::Cavalier;
        let cavalier_transform = Transform3D::cavalier(45.0, 1.0);
        self.apply_projection_transform(cavalier_transform);
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

#[derive(Default)]
pub enum Instrument {
    SetPoint,
    #[default]
    Move3D,
    Rotate3D,
    Scale3D,
    RotateAroundAxis,
    SetAxisPoint1,
    SetAxisPoint2,
}

impl ToString for Instrument {
    fn to_string(&self) -> String {
        match self {
            Self::SetPoint => String::from("изменить точку"),
            Self::Move3D => String::from("переместить 3D модель"),
            Self::Rotate3D => String::from("повернуть 3D модель"),
            Self::Scale3D => String::from("масштабировать 3D модель"),
            Self::RotateAroundAxis => String::from("поворот вокруг оси"),
            Self::SetAxisPoint1 => String::from("установить точку оси 1"),
            Self::SetAxisPoint2 => String::from("установить точку оси 2"),
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
