use crate::app::AthenianApp;
use egui::{Color32, Painter, Pos2, Response, Ui};
use g3d::{Line3, Model3, Point3, Transform3D, Vec3};

// --------------------------------------------------
// Обработка области рисования (холст)
// --------------------------------------------------

impl AthenianApp {
    /// Очистить холст от моделей.
    pub fn clear_canvas(&mut self) {
        self.scene.models.clear();
        self.selected_3d_model_index = None;
    }

    /// Выделяет место под текущий холст и выводит его на весь текущий размер экрана.
    pub fn allocate_canvas(&self, ui: &mut egui::Ui) -> (egui::Response, egui::Painter) {
        let available_size = ui.available_size();
        let canvas_size = self.canvas.size();
        let canvas_aspect_ratio = canvas_size[0] as f32 / canvas_size[1] as f32;

        let display_width = available_size.x.min(available_size.y * canvas_aspect_ratio);
        let display_height = display_width / canvas_aspect_ratio;

        let (canvas_response, painter) = ui.allocate_painter(
            egui::Vec2::new(display_width, display_height),
            egui::Sense::click_and_drag(),
        );
        return (canvas_response, painter);
    }

    /// Обновить текущую GPU текстуру для отображения.
    pub fn update_texture(&mut self, ctx: &egui::Context) {
        self.texture_handle = Some(ctx.load_texture(
            "canvas",
            self.canvas.to_color_image(),
            egui::TextureOptions::NEAREST, // Linear слишком размытый для отображения мелких пикселей
        ));
    }

    /// Срендерить сцену.
    pub fn render_scene(&mut self) {
        // Подготавливаем параметры для пользовательской оси
        let show_custom_axis =
            self.instrument == crate::app::logic::Instrument::RotateAroundCustomLine;

        self.scene.render(
            &mut self.canvas,
            g3d::ProjectionType::Perspective,
            g3d::RenderType::WireFrame,
            show_custom_axis,
            self.axis_point1,
            self.axis_point2,
        );
    }

    /// Преобразовать мировые координаты в экранные
    fn world_to_screen(&self, point: Point3) -> Pos2 {
        todo!()
        // let view_proj = self.camera.view_projection_matrix();
        // let transformed = view_proj.apply_to_point(point);

        // // Преобразуем из NDC (-1..1) в экранные координаты
        // let screen_x = (transformed.x + 1.0) * 0.5 * self.painter_width;
        // let screen_y = (1.0 - transformed.y) * 0.5 * self.painter_height;

        // Pos2::new(screen_x, screen_y)
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
    /// Добавить фигуру (заменяет текущую)
    pub fn set_model(&mut self, model: g3d::Model3) {
        self.scene.models.clear();
        self.scene.models.push(model);
        self.selected_3d_model_index = Some(0); // Автоматически выбираем добавленную фигуру
    }

    /// Получить текущую выбранную модель (мутабельно)
    pub fn get_selected_model_mut(&mut self) -> Option<&mut g3d::Model3> {
        self.selected_3d_model_index
            .and_then(|index| self.scene.models.get_mut(index))
    }

    /// Получить текущую выбранную модель
    pub fn get_selected_model(&self) -> Option<&g3d::Model3> {
        self.selected_3d_model_index
            .and_then(|index| self.scene.models.get(index))
    }

    /// Переместить 3D модель (применяется сразу к выбранной фигуре)
    fn move_3d_model(&mut self, start: egui::Pos2, end: egui::Pos2) {
        todo!()
        // if let Some(_) = self.get_selected_model() {
        //     let delta_x = (end.x - start.x) / self.painter_width * 4.0;
        //     let delta_y = (end.y - start.y) / self.painter_height * 4.0;

        //     let translation = Transform3D::translation(delta_x, -delta_y, 0.0);
        //     if let Some(model) = self.get_selected_model_mut() {
        //         model.apply_transform(translation);
        //     }
        // }
    }

    /// Повернуть 3D модель (применяется сразу к выбранной фигуре)
    fn rotate_3d_model(&mut self, start: egui::Pos2, end: egui::Pos2) {
        todo!()
        // if let Some(model) = self.get_selected_model() {
        //     let delta_x = (end.x - start.x) / self.painter_width * std::f32::consts::PI;
        //     let delta_y = (end.y - start.y) / self.painter_height * std::f32::consts::PI;

        //     let center = model.get_origin();

        //     // Вращение вокруг осей
        //     let rotation_x = Transform3D::rotation_around_line(
        //         Line3::new(center, Vec3::new(1.0, 0.0, 0.0)),
        //         delta_y,
        //     );
        //     let rotation_y = Transform3D::rotation_around_line(
        //         Line3::new(center, Vec3::new(0.0, 1.0, 0.0)),
        //         delta_x,
        //     );

        //     let combined_rotation = rotation_x.multiply(rotation_y);
        //     if let Some(model) = self.get_selected_model_mut() {
        //         model.apply_transform(combined_rotation);
        //     }
        // }
    }

    /// Масштабировать 3D модель (применяется сразу к выбранной фигуре)
    fn scale_3d_model(&mut self, start: egui::Pos2, end: egui::Pos2) {
        todo!()
        // if let Some(model) = self.get_selected_model() {
        //     let delta = ((end.x - start.x) + (end.y - start.y))
        //         / (self.painter_width + self.painter_height);
        //     let scale_factor = 1.0 + delta * 2.0;

        //     let center = model.get_origin();
        //     let scale = Transform3D::scale_relative_to_point(
        //         center,
        //         scale_factor,
        //         scale_factor,
        //         scale_factor,
        //     );

        //     if let Some(model) = self.get_selected_model_mut() {
        //         model.apply_transform(scale);
        //     }
        // }
    }

    /// Применить отражение относительно выбранной плоскости.
    pub fn apply_reflection(&mut self, plane_type: ReflectionPlane) {
        todo!()
        // if let Some(model) = self.get_selected_model() {
        //     let center = model.get_origin();

        //     let reflection = match plane_type {
        //         ReflectionPlane::XY => Transform3D::reflection_xy(),
        //         ReflectionPlane::XZ => Transform3D::reflection_xz(),
        //         ReflectionPlane::YZ => Transform3D::reflection_yz(),
        //     };

        //     // Применяем отражение относительно центра модели
        //     let transform = Transform3D::translation(-center.x, -center.y, -center.z)
        //         .multiply(reflection)
        //         .multiply(Transform3D::translation(center.x, center.y, center.z));

        //     if let Some(model) = self.get_selected_model_mut() {
        //         model.apply_transform(transform);
        //     }
        // }
    }

    // Простые методы для добавления многогранников (заменяют текущую фигуру)
    pub fn add_tetrahedron(&mut self) {
        todo!()
        // let tetrahedron = Model3::tetrahedron();
        // self.set_model(tetrahedron);
    }

    pub fn add_hexahedron(&mut self) {
        todo!()
        // let hexahedron = Model3::hexahedron();
        // self.set_model(hexahedron);
    }

    pub fn add_octahedron(&mut self) {
        todo!()
        // let octahedron = Model3::octahedron();
        // self.set_model(octahedron);
    }

    pub fn add_icosahedron(&mut self) {
        todo!()
        // let icosahedron = Model3::icosahedron();
        // self.set_model(icosahedron);
    }

    pub fn add_dodecahedron(&mut self) {
        todo!()
        // let dodecahedron = Model3::dodecahedron();
        // self.set_model(dodecahedron);
    }

    /// Повернуть модель вокруг оси (мышью)
    pub fn rotate_around_axis(&mut self, axis_type: AxisType, start: egui::Pos2, end: egui::Pos2) {
        todo!()
        //     if let Some(model) = self.get_selected_model() {
        //         let line = match axis_type {
        //             AxisType::Center(center_axis) => {
        //                 let center = model.get_origin();
        //                 let axis_vector = match center_axis {
        //                     CenterAxis::X => Vec3::new(1.0, 0.0, 0.0),
        //                     CenterAxis::Y => Vec3::new(0.0, 1.0, 0.0),
        //                     CenterAxis::Z => Vec3::new(0.0, 0.0, 1.0),
        //                 };
        //                 Line3::new(center, axis_vector)
        //             }
        //             AxisType::Custom => {
        //                 let point1 =
        //                     Point3::new(self.axis_point1_x, self.axis_point1_y, self.axis_point1_z);
        //                 let point2 =
        //                     Point3::new(self.axis_point2_x, self.axis_point2_y, self.axis_point2_z);
        //                 let direction = (point2 - point1).normalize();
        //                 Line3::new(point1, direction)
        //             }
        //         };

        //         // Вычисляем угол вращения на основе перемещения мыши
        //         let delta_x = (end.x - start.x) / self.painter_width * std::f32::consts::PI;
        //         let angle = delta_x * 2.0; // Множитель для более плавного вращения

        //         let rotation = Transform3D::rotation_around_line(line, angle);

        //         if let Some(model) = self.get_selected_model_mut() {
        //             model.apply_transform(rotation);
        //         }

        //         println!("Rotated around axis by {:.2} radians", angle);
        //     }
    }
}

// --------------------------------------------------
// Установка проекций
// --------------------------------------------------

impl AthenianApp {
    /// Установить перспективную проекцию
    pub fn set_perspective_projection(&mut self) {
        todo!()
        // self.camera.projection_type = g3d::ProjectionType::Perspective;

        // // Настройка камеры для перспективы
        // self.camera.position = Point3::new(0.0, 0.0, 10.0); // Камера смотрит вдоль Z
        // self.camera.set_fov_degrees(60.0);
        // self.camera
        //     .set_aspect_ratio(self.painter_width / self.painter_height);
    }

    /// Установить изометрическую проекцию
    pub fn set_isometric_projection(&mut self) {
        todo!()
        // self.camera.projection_type = g3d::ProjectionType::Isometric;

        // // Для изометрии используем специальные углы
        // self.camera.position = Point3::new(0.0, 0.0, 0.0); // Положение не важно для нашей простой проекции
    }
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
