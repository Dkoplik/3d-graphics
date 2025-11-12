use crate::Material;

use egui::Color32;

/// Взаимодействие между текстурой и цветом материала.
#[derive(Default, Debug, Clone, Copy)]
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
    /// Смешать пиксель текстуры и материала.
    pub fn blend(&self, texture_color: Color32, material_color: Color32) -> Color32 {
        match self {
            Self::Replace => texture_color,
            Self::Modulate => texture_color * material_color,
            Self::Additive => texture_color + material_color,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Color32::WHITE,
            texture: None,
            blend_mode: TextureBlendMode::default(),
            shininess: 32.0,
            specular_strength: 0.5,
        }
    }
}

impl Material {
    /// Получить цвет пикселя модели по UV-координатам с учётом материала.
    ///
    /// Обращаю внимание, что тут происходит только смешивание текстуры и материала.
    /// Освещение и шейдинг тут никак не учитываются.
    pub fn get_uv_color(&self, u: f32, v: f32) -> Color32 {
        if let Some(texture) = &self.texture {
            self.blend_mode.blend(texture[(u, v)], self.color)
        } else {
            self.color
        }
    }
}
