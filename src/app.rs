pub mod logic;
pub mod ui;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControls {
    pub move_speed: f32,
    pub rotate_speed: f32,
}

/// Приложение-демонстрация 3D графики.
pub struct AthenianApp {
    scene: g3d::Scene,

    /// Текущий инструмент
    instrument: logic::Instrument,

    /// Начальная позиция перетаскивания
    drag_prev_pos: Option<egui::Pos2>,

    /// Холст
    canvas: g3d::Canvas,
    texture_handle: Option<egui::TextureHandle>,

    // отображение холста
    display_canvas_width: f32,
    display_canvas_height: f32,

    // 3D поля
    selected_3d_model_index: Option<usize>,
    angle_of_rotate: f32,

    // Поля для осей вращения
    axis_point1: g3d::Point3,
    axis_point2: g3d::Point3,

    /// Параметры для создания моделей вращения
    rotation_params: logic::RotationModelParams,

    // Настройки рендеринга
    scene_renderer: g3d::SceneRenderer,

    selected_light_index: Option<usize>,

    // Камера
    camera_controls: CameraControls,

    // график функции
    selected_surface_function: g3d::SurfaceFunction,
    surface_x_min: f64,
    surface_x_max: f64,
    surface_y_min: f64,
    surface_y_max: f64,
    surface_divisions: usize,
}

impl AthenianApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // белая тема
        cc.egui_ctx.set_theme(egui::Theme::Light);
        Self::default()
    }
}

impl Default for AthenianApp {
    fn default() -> Self {
        let mut scene = g3d::Scene::default();

        // Добавляем базовый источник света
        let light = g3d::LightSource {
            position: g3d::Point3::new(5.0, 5.0, 5.0),
            color: egui::Color32::WHITE,
            intensity: 1.0,
        };
        scene.lights.push(light);

        Self {
            scene,

            // Текущий инструмент
            instrument: Default::default(),

            // Начальная позиция перетаскивания
            drag_prev_pos: Default::default(),

            // Холст
            canvas: g3d::Canvas::new(800, 600),
            texture_handle: Default::default(),

            // отображение холста
            display_canvas_width: 0.0,
            display_canvas_height: 0.0,

            // 3D поля
            selected_3d_model_index: Default::default(),
            angle_of_rotate: 0.0,

            // Поля для осей вращения
            axis_point1: g3d::Point3::new(0.0, 0.0, 0.0),
            axis_point2: g3d::Point3::new(1.0, 0.0, 0.0),

            // Параметры для создания моделей вращения
            rotation_params: logic::RotationModelParams::default(),

            // Настройки рендеринга
            scene_renderer: Default::default(),

            selected_light_index: None,

            // камера
            camera_controls: CameraControls {
                move_speed: 0.5,
                rotate_speed: 0.01,
            },

            // график функции
            selected_surface_function: Default::default(),
            surface_x_min: -2.0,
            surface_x_max: 2.0,
            surface_y_min: -2.0,
            surface_y_max: 2.0,
            surface_divisions: 50,
        }
    }
}
