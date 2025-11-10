use crate::app::AthenianApp;
use egui::{Color32, Painter, Pos2, Response, Ui};
use g3d::{
    Line3, Model3, Point3, Transform3D, Vec3, classes3d::model3::ObjLoadError,
    classes3d::model3::ObjSaveError,
};

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

    /// Рендеринг сцены с учетом всех настроек
    pub fn render_scene(&mut self) {
        // Рендерим в зависимости от выбранного режима
        let show_custom_axis = self.instrument == Instrument::RotateAroundCustomLine;

        self.scene.render(
            &mut self.canvas,
            self.render_options,
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
        let delta_x = (end.x - start.x) / self.display_canvas_width;
        let delta_y = (end.y - start.y) / self.display_canvas_height;

        let cur_instrument = self.instrument;
        if let Some(model) = self.get_selected_model_mut() {
            match cur_instrument {
                Instrument::Move3D => {
                    let move_delta = g3d::Vec3::new(delta_x * 5.0, -delta_y * 5.0, 0.0);
                    model.translate(move_delta);
                }
                Instrument::Rotate3D => {
                    let rotation_x = g3d::Transform3D::rotation_y_rad(delta_x * 2.0);
                    let rotation_y = g3d::Transform3D::rotation_x_rad(delta_y * 2.0);
                    model.apply_transform(&rotation_x);
                    model.apply_transform(&rotation_y);
                }
                Instrument::Scale3D => {
                    let scale_factor = 1.0 + (delta_x + delta_y) * 2.0;
                    model.scale(scale_factor, scale_factor, scale_factor);
                }
                Instrument::RotateAroundX => {
                    let rotation = g3d::Transform3D::rotation_x_rad(delta_x * 2.0);
                    model.apply_transform(&rotation);
                }
                Instrument::RotateAroundY => {
                    let rotation = g3d::Transform3D::rotation_y_rad(delta_x * 2.0);
                    model.apply_transform(&rotation);
                }
                Instrument::RotateAroundZ => {
                    let rotation = g3d::Transform3D::rotation_z_rad(delta_x * 2.0);
                    model.apply_transform(&rotation);
                }
                Instrument::RotateAroundCustomLine => {
                    // Вращение вокруг произвольной оси обрабатывается отдельно
                }
            }
        }
    }
}

// --------------------------------------------------
// Взаимодействие с 3D миром
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

    pub fn add_tetrahedron(&mut self) {
        let mesh = g3d::Mesh::tetrahedron();
        let model = g3d::Model3::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_hexahedron(&mut self) {
        let mesh = g3d::Mesh::hexahedron();
        let model = g3d::Model3::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_octahedron(&mut self) {
        let mesh = g3d::Mesh::octahedron();
        let model = g3d::Model3::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_icosahedron(&mut self) {
        let mesh = g3d::Mesh::icosahedron();
        let model = g3d::Model3::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_dodecahedron(&mut self) {
        let mesh = g3d::Mesh::dodecahedron();
        let model = g3d::Model3::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_model(&mut self, model: g3d::Model3) {
        self.scene.models.push(model);
        self.selected_3d_model_index = Some(self.scene.models.len() - 1);
    }

    pub fn translate_model(&mut self, delta: g3d::Vec3) {
        if let Some(model) = self.get_selected_model_mut() {
            model.translate(delta);
        }
    }

    pub fn rotate_model(&mut self, axis: g3d::Vec3, angle_rad: f32) {
        if let Some(model) = self.get_selected_model_mut() {
            model.rotate_around_axis(axis, angle_rad);
        }
    }

    pub fn scale_model(&mut self, factor: f32) {
        if let Some(model) = self.get_selected_model_mut() {
            model.scale(factor, factor, factor);
        }
    }

    pub fn apply_reflection(&mut self, plane_type: ReflectionPlane) {
        if let Some(model) = self.get_selected_model_mut() {
            let reflection = match plane_type {
                ReflectionPlane::XY => g3d::Transform3D::reflection_xy(),
                ReflectionPlane::XZ => g3d::Transform3D::reflection_xz(),
                ReflectionPlane::YZ => g3d::Transform3D::reflection_yz(),
            };
            model.apply_transform(&reflection);
        }
    }

    pub fn apply_custom_rotation(&mut self) {
        if self.get_selected_model().is_some() {
            let axis_line = g3d::Line3::from_points(self.axis_point1, self.axis_point2);
            let rotation = g3d::Transform3D::rotation_around_line(
                axis_line,
                self.angle_of_rotate.to_radians(),
            );
            if let Some(model) = self.get_selected_model_mut() {
                model.apply_transform(&rotation);
            }
        }
    }

    pub fn apply_material_to_selected(&mut self) {
        // TODO: Применить материал к выбранной модели
        todo!("Применение материала к моделе")
    }

    // === ОПЕРАЦИИ С ОСВЕЩЕНИЕМ ===

    pub fn add_light_source(&mut self) {
        let new_light = g3d::LightSource {
            position: g3d::Point3::new(3.0, 3.0, 3.0),
            color: egui::Color32::WHITE,
            intensity: 1.0,
        };
        self.scene.add_light(new_light);
        self.selected_light_index = Some(self.scene.lights.len() - 1);
    }

    // === ОПЕРАЦИИ С КАМЕРОЙ ===

    pub fn reset_camera(&mut self) {
        let camera = &mut self.scene.camera;

        // Обновляем позицию и направление
        camera.set_position(g3d::Point3::new(10.0, 10.0, 10.0));
        camera.set_direction(
            (g3d::Point3::new(0.0, 0.0, 0.0) - camera.get_position()).normalize(),
            g3d::Vec3::up(),
        );

        // Обновляем поле зрения
        camera.set_fov_degrees(60.0);
    }

    pub fn set_front_view(&mut self) {
        let camera = &mut self.scene.camera;

        // Обновляем позицию и направление
        camera.set_position(g3d::Point3::new(0.0, 0.0, 10.0));
        camera.set_direction(
            (g3d::Point3::new(0.0, 0.0, 0.0) - camera.get_position()).normalize(),
            g3d::Vec3::up(),
        );
    }

    pub fn set_top_view(&mut self) {
        let camera = &mut self.scene.camera;

        // Обновляем позицию и направление
        camera.set_position(g3d::Point3::new(0.0, 10.0, 0.0));
        camera.set_direction(
            (g3d::Point3::new(0.0, 0.0, 0.0) - camera.get_position()).normalize(),
            g3d::Vec3::up(),
        );
    }

    pub fn load_obj_file(&mut self) {
        let file_path = rfd::FileDialog::new()
            .add_filter("OBJ files", &["obj"])
            .pick_file();

        if let Some(path) = file_path {
            match Model3::load_from_obj(path.to_str().unwrap()) {
                Ok(model) => {
                    self.set_model(model);
                    println!("Модель успешно загружена");
                }
                Err(ObjLoadError::FileNotFound) => {
                    eprintln!("Файл не найден");
                }
                Err(ObjLoadError::InvalidFormat) => {
                    eprintln!("Неверный формат OBJ файла");
                }
                Err(ObjLoadError::UnsupportedFeature) => {
                    eprintln!("Файл содержит неподдерживаемые функции");
                }
            }
        }
    }

    pub fn save_obj_file(&mut self) {
        if let Some(model) = self.get_selected_model() {
            // Показываем диалог сохранения файла
            let file_path = rfd::FileDialog::new()
                .add_filter("OBJ files", &["obj"])
                .set_file_name("model.obj")
                .save_file();

            if let Some(path) = file_path {
                match model.save_to_obj(path.to_str().unwrap()) {
                    Ok(()) => {
                        println!("Модель успешно сохранена");
                    }
                    Err(ObjSaveError::WriteError) => {
                        eprintln!("Ошибка записи файла");
                    }
                    Err(ObjSaveError::InvalidData) => {
                        eprintln!("Неверные данные модели");
                    }
                }
            }
        } else {
            eprintln!("Нет выбранной модели для сохранения");
        }
    }

    pub fn create_rotation_model(&mut self) {
        // TODO: Реализовать создание модели вращения
        todo!("создание модели вращения")
    }

    pub fn create_function_model(&mut self) {
        // TODO: Реализовать создание модели из функции
        todo!("создание модели из графика функции")
    }

    pub fn load_texture(&mut self) {
        // TODO: Реализовать загрузку текстур
        todo!("загрузить текстуру для модели")
    }

    pub fn remove_texture(&mut self) {
        // TODO: Удалить текстуру у выбранной модели
        todo!("удалить текстуру у модели")
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

#[derive(Default, PartialEq, Clone, Copy)]
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
