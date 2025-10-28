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

    pub angle_of_rotate: f32,

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

    /// Центрировать позицию модели
    pub fn center_model_position(&self, model: &mut g3d::Model3) {
        model.set_origin(g3d::Point3::new(0.0, 0.0, 0.0));
    }

    /// Центрировать текущую фигуру на холсте
    pub fn center_model(&mut self) {
        if let Some(model) = self.get_selected_model_mut() {
            model.set_origin(g3d::Point3::new(0.0, 0.0, 0.0));
        }
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
            angle_of_rotate: Default::default(),
            axis_point1: g3d::Point3::new(0.0, 0.0, 0.0),
            axis_point2: g3d::Point3::new(1.0, 0.0, 0.0),
        }
    }
}
