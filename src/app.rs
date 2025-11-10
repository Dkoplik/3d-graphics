pub mod logic;
pub mod ui;

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

    // Настройки рендеринга
    render_options: g3d::classes3d::scene::RenderOptions,

    // Материалы и текстуры
    current_material: g3d::Material,

    selected_light_index: Option<usize>,

    // Камера
    camera_controls: CameraControls,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControls {
    pub position: g3d::Point3,
    pub target: g3d::Point3,
    pub fov: f32,
    pub move_speed: f32,
    pub rotate_speed: f32,
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
        scene.add_light(light);

        Self {
            scene,
            instrument: Default::default(),
            drag_prev_pos: Default::default(),
            canvas: g3d::Canvas::new(800, 600),
            texture_handle: Default::default(),
            display_canvas_width: 800.0,
            display_canvas_height: 600.0,
            selected_3d_model_index: Default::default(),
            angle_of_rotate: 0.0,
            axis_point1: g3d::Point3::new(0.0, 0.0, 0.0),
            axis_point2: g3d::Point3::new(1.0, 0.0, 0.0),
            render_options: Default::default(),
            current_material: Default::default(),
            selected_light_index: None,
            camera_controls: CameraControls {
                position: g3d::Point3::new(10.0, 10.0, 10.0),
                target: g3d::Point3::new(0.0, 0.0, 0.0),
                fov: 60.0,
                move_speed: 0.5,
                rotate_speed: 0.01,
            },
        }
    }
}
