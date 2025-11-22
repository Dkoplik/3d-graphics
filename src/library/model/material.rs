use super::Texture;
use egui::Color32;
use std::fmt::Display;

/// Материал модели.
///
/// Материал задаёт сплошной цвет модели и его поведение при освещении.
#[derive(Debug, Clone)]
pub struct Material {
    /// Цвет всего объекта
    pub color: egui::Color32,
    /// Текстура объекта, если имеется
    pub texture: Option<Texture>,
    /// Как совмещать текстуру с цветом материала
    pub blend_mode: TextureBlendMode,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Color32::WHITE,
            texture: None,
            blend_mode: TextureBlendMode::default(),
        }
    }
}

impl Material {
    /// Получить цвет пикселя модели по UV-координатам с учётом материала.
    ///
    /// Обращаю внимание, что тут происходит только смешивание текстуры и материала.
    /// Освещение и шейдинг тут никак не учитываются.
    pub fn get_uv_color(&self, u: f32, v: f32) -> Color32 {
        let (u, v) = self.cycle_texture(u, v);
        if let Some(texture) = &self.texture {
            self.blend_mode
                .blend(texture.get_pixel_color(u, v), self.color)
        } else {
            self.color
        }
    }

    /// Если UV-координаты выходят за границы текстуры, то зацикливаем её.
    fn cycle_texture(&self, u: f32, v: f32) -> (f32, f32) {
        // зацикливаем текстуру при выходе за границы
        let new_u = if u == 1.0 { 1.0 } else { u.fract() };
        let new_v = if v == 1.0 { 1.0 } else { v.fract() };
        (new_u, new_v)
    }
}

/// Тип взаимодействия между текстурой и цветом материала.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum TextureBlendMode {
    /// Текстура полностью заменяет цвет материала.
    Replace,
    /// Текстура умножется на цвет материала.
    #[default]
    Modulate,
    /// К текстуре добавляется цвет материала.
    Additive,
}

impl TextureBlendMode {
    /// Объединить пиксель текстуры и материала.
    fn blend(&self, texture_color: Color32, material_color: Color32) -> Color32 {
        match self {
            Self::Replace => texture_color,
            Self::Modulate => texture_color * material_color,
            Self::Additive => texture_color + material_color,
        }
    }
}

impl Display for TextureBlendMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Replace => f.write_str("Замена"),
            Self::Modulate => f.write_str("Умножение"),
            Self::Additive => f.write_str("Сложение"),
        }
    }
}
