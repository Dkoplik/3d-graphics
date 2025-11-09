pub mod logic;
pub mod ui;

// --------------------------------------------------
// Базовое определение приложения
// --------------------------------------------------

/// Приложение-демонстрация аффинных преобразований.
pub struct AthenianApp {
    scene: g3d::Scene,

    /// Текущий инструмент
    instrument: logic::Instrument,

    /// Начальная позиция перетаскивания
    drag_prev_pos: Option<egui::Pos2>,

    /// Холст
    canvas: g3d::Canvas,
    texture_handle: Option<egui::TextureHandle>,

    // отоображение холста
    display_canvas_width: f32,
    display_canvas_height: f32,

    // 3D поля
    selected_3d_model_index: Option<usize>,
    angle_of_rotate: f32,

    // Поля для осей вращения
    axis_point1: g3d::Point3,
    axis_point2: g3d::Point3,
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
        Self {
            scene: Default::default(),
            instrument: Default::default(),
            drag_prev_pos: Default::default(),
            canvas: Default::default(),
            texture_handle: Default::default(),
            display_canvas_width: Default::default(),
            display_canvas_height: Default::default(),
            selected_3d_model_index: Default::default(),
            angle_of_rotate: Default::default(),
            axis_point1: g3d::Point3::new(0.0, 0.0, 0.0),
            axis_point2: g3d::Point3::new(1.0, 0.0, 0.0),
        }
    }
}
