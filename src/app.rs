pub mod logic;
pub mod ui;

// --------------------------------------------------
// Базовое определение приложения
// --------------------------------------------------

/// Приложение-демонстрация аффинных преобразований.
#[derive(Default)]
pub struct AthenianApp {
    scene: g3d::Scene,

    /// Текущий инструмент
    instrument: logic::Instrument,

    /// Начальная позиция перетаскивания
    drag_prev_pos: Option<egui::Pos2>,

    // Размеры холста
    painter_width: f32,
    painter_height: f32,
}

impl AthenianApp {
    /// Инициализация приложения.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // белая тема
        cc.egui_ctx.set_theme(egui::Theme::Light);
        Self::default()
    }
}
