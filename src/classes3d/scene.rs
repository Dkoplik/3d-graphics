use egui::Painter;

use crate::{Camera3, Model3, RenderStyle, Scene, Transformable3};

impl Scene {
    /// Создать пустую сцену.
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    /// Добавить новую модель на сцену.
    pub fn add_model(&mut self, model: Model3) {
        self.models.push(model);
    }

    /// Нарисовать сцену на экран со всеми нужными преобразованиями.
    pub fn render(&self, camera: Camera3, painter: &mut Painter, style: &RenderStyle) {
        self.models
            .iter()
            .cloned()
            .map(|model| {
                model
                    .to_world_coordinates()
                    .transform(camera.view_projection_matrix())
            })
            .for_each(|model| model.draw(painter, style));
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

// Создать сцену из модели
impl From<Model3> for Scene {
    fn from(value: Model3) -> Self {
        Self {
            models: vec![value],
        }
    }
}
