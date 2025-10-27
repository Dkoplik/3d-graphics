pub mod logic;
pub mod ui;

// --------------------------------------------------
// Базовое определение приложения
// --------------------------------------------------

/// Приложение-демонстрация аффинных преобразований.
pub struct AthenianApp {
    pub scene: g3d::Scene,

    /// Текущий инструмент
    instrument: logic::Instrument,

    /// Начальная позиция перетаскивания
    drag_prev_pos: Option<egui::Pos2>,

    // Размеры холста
    painter_width: f32,
    painter_height: f32,

    // 3D поля
    pub camera: g3d::Camera3,
    pub selected_3d_model_index: Option<usize>,
    pub current_projection: crate::app::logic::Projection,

    // Поля для осей вращения
    axis_point1: g3d::Point3,
    axis_point2: g3d::Point3,
}

impl AthenianApp {
    /// Инициализация приложения.
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
            painter_width: Default::default(),
            painter_height: Default::default(),
            camera: Default::default(),
            selected_3d_model_index: Default::default(),
            current_projection: Default::default(),
            axis_point1: g3d::Point3::new(0.0, 0.0, 0.0),
            axis_point2: g3d::Point3::new(1.0, 0.0, 0.0),
        }
    }
}
