//! Реализация текстуры для 3D модели

use crate::Texture;
use egui::Color32;
use image::DynamicImage;

impl Texture {
    pub fn new(image: DynamicImage) -> Self {
        Self(image.to_rgb8())
    }

    /// Получить цвет текстуры в пикселе по UV-координатам.
    pub fn get_pixel_color(&self, u: f32, v: f32) -> Color32 {
        let (x, y) = self.transform_uv(u, v);

        // sanity check
        debug_assert!(
            self.check_bounds(x, y),
            "Выход за границы текстуры: x={} y={}",
            x,
            y
        );

        pixel_to_color(*self.0.get_pixel(x, y))
    }

    /// Проверить границы текстуры.
    #[inline]
    fn check_bounds(&self, x: u32, y: u32) -> bool {
        x < self.0.width() && y < self.0.height() && x > 0 && y > 0
    }

    /// Преобразовать UV-координаты в целочисленные
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

        let x = (u * self.0.width() as f32).round() as u32;
        let y = (v * self.0.height() as f32).round() as u32;
        (x, y)
    }
}

fn pixel_to_color(pixel: image::Rgb<u8>) -> Color32 {
    Color32::from_rgb(pixel[0], pixel[1], pixel[2])
}
