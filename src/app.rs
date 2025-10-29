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

    pub angle_of_rotate: f32,

    // Поля для осей вращения
    pub axis_point1_x: f32,
    pub axis_point1_y: f32,
    pub axis_point1_z: f32,
    pub axis_point2_x: f32,
    pub axis_point2_y: f32,
    pub axis_point2_z: f32,
}

impl AthenianApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // белая тема
        cc.egui_ctx.set_theme(egui::Theme::Light);
        Self::default()
    }
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

    /// Очистить холст от моделей.
    pub fn clear_canvas(&mut self) {
        self.scene.models.clear();
        self.selected_3d_model_index = None;
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
            angle_of_rotate: Default::default(),
            axis_point1_x: 0.0,
            axis_point1_y: 0.0,
            axis_point1_z: 0.0,
            axis_point2_x: 1.0,
            axis_point2_y: 0.0,
            axis_point2_z: 0.0,
        }
    }
}
