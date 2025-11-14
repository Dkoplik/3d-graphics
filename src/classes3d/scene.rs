use crate::Scene;

impl Default for Scene {
    fn default() -> Self {
        Self {
            models: Vec::new(),
            camera: Default::default(),
            lights: Vec::new(),
        }
    }
}
