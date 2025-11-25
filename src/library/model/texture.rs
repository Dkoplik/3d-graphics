//! Объявление и реализация текстуры для 3D модели

use crate::library::utils;
use egui::Color32;
use image::{DynamicImage, RgbImage};

/// Текстура модели.
///
/// Благодаря текстуре модель может быть обёрнута в какую-то картинку вместо сплошного цвета.
#[derive(Debug, Clone)]
pub struct Texture {
    image: RgbImage,
}

impl Texture {
    /// Создать новую текстуру из `DynamicImage`.
    ///
    /// При загрузке картинок, crate `image` обычно возвращает `DynamicImage`,
    /// из которого можно сделать текстуру, конструктор сам сделает перевод в удобное представление.
    pub fn new(image: DynamicImage) -> Self {
        // в RgbImage
        let image = image.to_rgb8();
        Self { image }
    }

    /// Получить цвет текстуры в пикселе по UV-координатам.
    ///
    /// - `u` - горизонтальная ось в диапазоне [0.0, 1.0]
    /// - `v` - вертикальная ось в диапазоне [0.0, 1.0]
    #[inline]
    pub fn get_pixel_color(&self, u: f32, v: f32) -> Color32 {
        let (x, y) = self.transform_uv(u, v);
        utils::pixel_to_color(*self.image.get_pixel(x, y))
    }

    /// Преобразовать UV-координаты в целочисленные.
    #[inline]
    fn transform_uv(&self, u: f32, v: f32) -> (u32, u32) {
        debug_assert!(
            0.0 <= u && u <= 1.0,
            "текстурная координата u={} должна быть между 0.0 и 1.0",
            u
        );
        debug_assert!(
            0.0 <= v && v <= 1.0,
            "текстурная координата v={} должна быть между 0.0 и 1.0",
            v
        );

        let x = (u * (self.image.width() - 1) as f32).round() as u32;
        let y = (v * (self.image.height() - 1) as f32).round() as u32;
        (x, y)
    }
}
