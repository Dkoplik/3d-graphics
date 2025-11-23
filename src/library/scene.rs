use crate::{Camera, LightSource, Model};

/// Сцена в 3-х мерном пространстве с 3-х мерными объектами (моделями).
#[derive(Debug, Clone)]
pub struct Scene {
    /// Модели на сцене.
    pub models: Vec<Model>,
    /// Камера в 3-х мерной сцене.
    pub camera: Camera,
    /// Источики света.
    pub lights: Vec<LightSource>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            models: Vec::new(),
            camera: Default::default(),
            lights: Vec::new(),
        }
    }
}
