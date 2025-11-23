use crate::Point3;

/// Точечный источник света.
///
/// Свет от этого источника направлен по все стороны.
#[derive(Debug, Clone, Copy)]
pub struct LightSource {
    pub position: Point3,
    pub color: egui::Color32,
    pub intensity: f32,
}

impl LightSource {
    pub fn new(position: Point3, color: egui::Color32, intensity: f32) -> Self {
        LightSource {
            position,
            color,
            intensity,
        }
    }
}
