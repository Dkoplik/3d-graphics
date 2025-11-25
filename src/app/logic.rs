use crate::app::AthenianApp;
use egui::Response;
use image::ImageFormat;
use std::fs::File;
use std::io::BufReader;

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

        self.scene_renderer.render(
            &self.scene,
            &mut self.canvas,
            show_custom_axis,
            self.axis_point1,
            self.axis_point2,
        );
    }
}

// --------------------------------------------------
// Обработка управления
// --------------------------------------------------

impl AthenianApp {
    /// Обработать взаимодействие с холстом.
    pub fn handle_input(&mut self, response: &Response, ctx: &egui::Context) {
        self.handle_click(response);
        self.handle_drag(response);
        self.handle_right_drag(response);
        self.handle_camera_input(ctx);
        ctx.request_repaint();
    }

    /// Обработать клики по холсту.
    fn handle_click(&mut self, response: &Response) {
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(_pos) = response.hover_pos() {
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

    /// Обработать перетаскивание по холсту.
    fn handle_right_drag(&mut self, response: &Response) {
        if response.drag_stopped_by(egui::PointerButton::Secondary) {
            self.right_drag_prev_pos = None;
            return;
        }

        if !response.dragged_by(egui::PointerButton::Secondary) {
            return;
        }

        if let Some(drag_start) = self.right_drag_prev_pos
            && let Some(drag_cur) = response.hover_pos()
        {
            let transform = self
                .scene
                .camera
                .screen_to_global_transform(self.scene_renderer.projection_type, &self.canvas);
            let camera = &mut self.scene.camera;

            let z = (2.0 * camera.get_far_plane() + 10.0 * camera.get_near_plane()) / 12.0;
            // меняем местами y, дабы избежать инвертирования по вертикали
            let from = g3d::UVec3::new(drag_start.x, drag_cur.y, z)
                .apply_transform(transform)
                .unwrap();
            let to = g3d::UVec3::new(drag_cur.x, drag_start.y, z)
                .apply_transform(transform)
                .unwrap();
            camera.rotate(from, to);
        }

        self.right_drag_prev_pos = response.hover_pos();
    }

    /// Обработать перетаскивание для 3D.
    fn handle_3d_drag(&mut self, start: egui::Pos2, end: egui::Pos2) {
        let transform = self
            .scene
            .camera
            .screen_to_global_transform(self.scene_renderer.projection_type, &self.canvas);
        let mut from = g3d::Vec3::new(start.x, start.y, 0.0);
        let mut to = g3d::Vec3::new(end.x, end.y, 0.0);
        let cur_instrument = self.instrument;
        if let Some(model) = self.get_selected_model_mut() {
            from.z = model.get_position().z;
            from = from.apply_transform(transform).unwrap();
            to.z = model.get_position().z;
            to = to.apply_transform(transform).unwrap();

            match cur_instrument {
                Instrument::Move3D => {
                    let mut move_delta = to - from;
                    move_delta.x *= -1.0;
                    model.translate(move_delta);
                }
                Instrument::Rotate3D => {
                    let tmp = from.x;
                    from.x = to.x;
                    to.x = tmp;
                    model.rotate(from.normalize().unwrap(), to.normalize().unwrap());
                }
                Instrument::Scale3D => {
                    let scale_factor = to - from;
                    model.scale_vec(scale_factor);
                }
                Instrument::RotateAroundX => {
                    from.x = 0.0;
                    to.x = 0.0;
                    model.rotate(from.normalize().unwrap(), to.normalize().unwrap());
                }
                Instrument::RotateAroundY => {
                    from.y = 0.0;
                    to.y = 0.0;
                    let tmp = from.x;
                    from.x = to.x;
                    to.x = tmp;
                    model.rotate(from.normalize().unwrap(), to.normalize().unwrap());
                }
                Instrument::RotateAroundZ => {
                    from.z = 0.0;
                    to.z = 0.0;
                    let tmp = from.x;
                    from.x = to.x;
                    to.x = tmp;
                    model.rotate(from.normalize().unwrap(), to.normalize().unwrap());
                }
                Instrument::RotateAroundCustomLine => {
                    // Вращение вокруг произвольной оси обрабатывается отдельно
                }
            }
        }
    }

    fn handle_camera_input(&mut self, ctx: &egui::Context) {
        let distance = self.camera_controls.move_speed;

        if ctx.input(|i| i.key_pressed(egui::Key::W)) {
            self.scene.camera.move_forward(distance);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.scene.camera.move_backward(distance);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::A)) {
            self.scene.camera.move_left(distance);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::D)) {
            self.scene.camera.move_right(distance);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
            self.scene.camera.move_up(distance);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::E)) {
            self.scene.camera.move_down(distance);
        }
    }
}

// --------------------------------------------------
// Взаимодействие с 3D миром
// --------------------------------------------------

impl AthenianApp {
    /// Добавить фигуру (заменяет текущую)
    pub fn set_model(&mut self, model: g3d::Model) {
        self.scene.models.clear();
        self.scene.models.push(model);
        self.selected_3d_model_index = Some(0); // Автоматически выбираем добавленную фигуру
    }

    /// Получить текущую выбранную модель (мутабельно)
    pub fn get_selected_model_mut(&mut self) -> Option<&mut g3d::Model> {
        self.selected_3d_model_index
            .and_then(|index| self.scene.models.get_mut(index))
    }

    /// Получить текущую выбранную модель
    pub fn get_selected_model(&self) -> Option<&g3d::Model> {
        self.selected_3d_model_index
            .and_then(|index| self.scene.models.get(index))
    }

    pub fn add_tetrahedron(&mut self) {
        let mesh = g3d::Mesh::tetrahedron();
        let model = g3d::Model::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_hexahedron(&mut self) {
        let mesh = g3d::Mesh::hexahedron();
        let model = g3d::Model::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_octahedron(&mut self) {
        let mesh = g3d::Mesh::octahedron();
        let model = g3d::Model::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_icosahedron(&mut self) {
        let mesh = g3d::Mesh::icosahedron();
        let model = g3d::Model::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_dodecahedron(&mut self) {
        let mesh = g3d::Mesh::dodecahedron();
        let model = g3d::Model::from_mesh(mesh);
        self.add_model(model);
    }

    pub fn add_model(&mut self, model: g3d::Model) {
        self.scene.models.push(model);
        self.selected_3d_model_index = Some(self.scene.models.len() - 1);
    }

    pub fn translate_model(&mut self, delta: g3d::Vec3) {
        if let Some(model) = self.get_selected_model_mut() {
            model.translate(delta);
        }
    }

    // pub fn rotate_model(&mut self, axis: g3d::Vec3, angle_rad: f32) {
    //     if let Some(model) = self.get_selected_model_mut() {
    //         model.rotate_around_axis(axis, angle_rad);
    //     }
    // }

    pub fn scale_model(&mut self, factor: f32) {
        if let Some(model) = self.get_selected_model_mut() {
            model.uniform_scale(factor);
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
                model.mesh.local_frame.rotate(rotation);
            }
        }
    }

    // === ОПЕРАЦИИ С ОСВЕЩЕНИЕМ ===

    pub fn add_light_source(&mut self) {
        let new_light = g3d::LightSource {
            position: g3d::Point3::new(3.0, 3.0, 3.0),
            color: egui::Color32::WHITE,
            intensity: 1.0,
        };
        self.scene.lights.push(new_light);
        self.selected_light_index = Some(self.scene.lights.len() - 1);
    }

    // === ОПЕРАЦИИ С КАМЕРОЙ ===

    pub fn reset_camera(&mut self) {
        self.scene.camera = g3d::Camera::default();
    }

    pub fn load_obj_file(&mut self) {
        let file_path = rfd::FileDialog::new()
            .add_filter("OBJ files", &["obj"])
            .pick_file();

        if let Some(path) = file_path {
            match g3d::Model::load_from_obj(path.to_str().unwrap()) {
                Ok(model) => {
                    self.set_model(model);
                    println!("Модель успешно загружена");
                }
                Err(g3d::ObjLoadError::FileNotFound) => {
                    eprintln!("Файл не найден");
                }
                Err(g3d::ObjLoadError::InvalidFormat) => {
                    eprintln!("Неверный формат OBJ файла");
                }
                Err(g3d::ObjLoadError::UnsupportedFeature) => {
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
                    Err(g3d::ObjSaveError::WriteError) => {
                        eprintln!("Ошибка записи файла");
                    }
                    Err(g3d::ObjSaveError::InvalidData) => {
                        eprintln!("Неверные данные модели");
                    }
                }
            }
        } else {
            eprintln!("Нет выбранной модели для сохранения");
        }
    }

    pub fn create_rotation_model(&mut self) {
        // делаем так, чтобы в self.add_model(model); не было ошибки
        let params = std::mem::take(&mut self.rotation_params);

        if params.profile_points.len() < 2 {
            eprintln!("Профиль должен содержать хотя бы 2 точки");
            return;
        }

        // Преобразуем точки профиля в HVec3
        let profile_hvec: Vec<g3d::Point3> = params.profile_points.clone();

        // Получаем ось вращения
        let axis = params
            .axis_type
            .to_line(params.custom_axis_start, params.custom_axis_end);

        // Создаем mesh
        let mesh = g3d::Mesh::create_rotation_model(&profile_hvec, axis, params.segments);
        let model = g3d::Model::from_mesh(mesh);

        // Возвращаем параметры обратно
        self.rotation_params = params;

        self.add_model(model);
        println!(
            "Модель вращения создана с {} сегментами",
            self.rotation_params.segments
        );
    }

    /// Добавить точку к профилю вращения
    pub fn add_profile_point(&mut self, point: g3d::Point3) {
        self.rotation_params.profile_points.push(point);
    }

    /// Удалить последнюю точку профиля
    pub fn remove_last_profile_point(&mut self) {
        if self.rotation_params.profile_points.len() > 2 {
            self.rotation_params.profile_points.pop();
        } else {
            eprintln!("Профиль должен содержать хотя бы 2 точки");
        }
    }

    /// Очистить профиль вращения
    pub fn clear_profile(&mut self) {
        self.rotation_params.profile_points.clear();
        // Добавляем базовые точки
        self.rotation_params
            .profile_points
            .push(g3d::Point3::new(0.0, 1.0, 0.0));
        self.rotation_params
            .profile_points
            .push(g3d::Point3::new(1.0, 0.0, 0.0));
    }

    /// Сохранить модель вращения в OBJ
    pub fn save_rotation_model(&mut self) {
        if let Some(model) = self.get_selected_model() {
            let file_path = rfd::FileDialog::new()
                .add_filter("OBJ files", &["obj"])
                .set_title("Сохранить модель вращения")
                .set_file_name("rotation_model.obj")
                .save_file();

            if let Some(path) = file_path {
                match model.save_to_obj(path.to_str().unwrap()) {
                    Ok(()) => println!("Модель вращения сохранена в {}", path.display()),
                    Err(e) => eprintln!("Ошибка сохранения: {:?}", e),
                }
            }
        } else {
            eprintln!("Нет выбранной модели для сохранения");
        }
    }

    pub fn get_rotation_params_mut(&mut self) -> &mut RotationModelParams {
        &mut self.rotation_params
    }

    pub fn create_surface_from_function(
        &mut self,
        func: g3d::SurfaceFunction,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        divisions: usize,
    ) {
        let mesh =
            func.generate_surface_mesh((x_min, x_max), (y_min, y_max), (divisions, divisions));

        let model = g3d::Model::from_mesh(mesh);
        self.set_model(model);
    }

    /// Создать модель из функции двух переменных
    pub fn create_function_model(&mut self) {
        let mesh = self.selected_surface_function.generate_surface_mesh(
            (self.surface_x_min, self.surface_x_max),
            (self.surface_y_min, self.surface_y_max),
            (self.surface_divisions, self.surface_divisions),
        );

        let model = g3d::Model::from_mesh(mesh);
        self.set_model(model);
    }

    pub fn load_texture(&mut self) {
        let file_path = rfd::FileDialog::new()
            .add_filter("Image files", &["png", "jpg", "jpeg", "bmp", "tga"])
            .pick_file();

        if let Some(path) = file_path {
            match self.load_texture_from_file(path.to_str().unwrap()) {
                Ok(texture) => {
                    if let Some(model) = self.get_selected_model_mut() {
                        model.material.texture = Some(texture);
                        println!("Текстура успешно загружена и применена к модели");
                    } else {
                        panic!("Текстура загружена, но модель не выбрана");
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка загрузки текстуры: {}", e);
                }
            }
        }
    }

    // Вспомогательный метод для загрузки текстуры из файла
    fn load_texture_from_file(&self, file_path: &str) -> Result<g3d::Texture, String> {
        let file = File::open(file_path).map_err(|e| format!("Не удалось открыть файл: {}", e))?;
        let reader = BufReader::new(file);

        let img = image::load(
            reader,
            ImageFormat::from_path(file_path)
                .map_err(|e| format!("Неверный формат изображения: {}", e))?,
        )
        .map_err(|e| format!("Ошибка загрузки изображения: {}", e))?;

        Ok(g3d::Texture::new(img))
    }

    pub fn remove_texture(&mut self) {
        if let Some(model) = self.get_selected_model_mut() {
            model.material.texture = None;
            println!("Текстура удалена у выбранной модели");
        } else {
            panic!("Удаление текстуры, но модель не выбрана");
        }
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

#[derive(Debug, Clone)]
pub struct RotationModelParams {
    pub profile_points: Vec<g3d::Point3>,
    pub axis_type: AxisType,
    pub custom_axis_start: g3d::Point3,
    pub custom_axis_end: g3d::Point3,
    pub segments: usize,
}

impl Default for RotationModelParams {
    fn default() -> Self {
        Self {
            profile_points: vec![
                g3d::Point3::new(0.0, 1.0, 0.0),
                g3d::Point3::new(0.5, 0.8, 0.0),
                g3d::Point3::new(1.0, 0.0, 0.0),
            ],
            axis_type: AxisType::Center(CenterAxis::Y),
            custom_axis_start: g3d::Point3::new(0.0, 0.0, 0.0),
            custom_axis_end: g3d::Point3::new(0.0, 1.0, 0.0),
            segments: 16,
        }
    }
}

// Добавляем метод преобразования AxisType в Line3
impl AxisType {
    pub fn to_line(&self, custom_start: g3d::Point3, custom_end: g3d::Point3) -> g3d::Line3 {
        match self {
            AxisType::Center(CenterAxis::X) => g3d::Line3::from_points(
                g3d::Point3::new(0.0, 0.0, 0.0),
                g3d::Point3::new(1.0, 0.0, 0.0),
            ),
            AxisType::Center(CenterAxis::Y) => g3d::Line3::from_points(
                g3d::Point3::new(0.0, 0.0, 0.0),
                g3d::Point3::new(0.0, 1.0, 0.0),
            ),
            AxisType::Center(CenterAxis::Z) => g3d::Line3::from_points(
                g3d::Point3::new(0.0, 0.0, 0.0),
                g3d::Point3::new(0.0, 0.0, 1.0),
            ),
            AxisType::Custom => g3d::Line3::from_points(custom_start, custom_end),
        }
    }

    pub fn name(&self) -> String {
        match self {
            AxisType::Center(CenterAxis::X) => "Ось X".to_string(),
            AxisType::Center(CenterAxis::Y) => "Ось Y".to_string(),
            AxisType::Center(CenterAxis::Z) => "Ось Z".to_string(),
            AxisType::Custom => "Произвольная ось".to_string(),
        }
    }
}

// Также добавим метод для CenterAxis для удобства
impl CenterAxis {
    pub fn name(&self) -> &'static str {
        match self {
            CenterAxis::X => "Ось X",
            CenterAxis::Y => "Ось Y",
            CenterAxis::Z => "Ось Z",
        }
    }
}
