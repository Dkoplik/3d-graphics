//! Реализация текстуры для 3D модели

use std::ops::Index;

use egui::Color32;

use crate::Texture;

impl Texture {
    /// Создать новую текстуру по набору пикселей и размерам текстуры.
    pub fn new(image: Vec<Color32>, width: usize, height: usize) -> Self {
        debug_assert!(width > 0, "ширина холста не может быть нулевой");
        debug_assert!(height > 0, "высота холста не может быть нулевой");
        debug_assert_eq!(
            image.len(),
            width * height,
            "размер буфера текстуры {} не совпадает с размером {}x{}",
            image.len(),
            width,
            height
        );

        Self {
            image,
            width,
            height,
        }
    }

    /// Проверить границы текстуры.
    #[inline]
    fn check_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && x > 0 && y > 0
    }

    /// Преобразовать UV-координаты в целочисленные
    #[inline]
    fn transform_uv(&self, u: f32, v: f32) -> (usize, usize) {
        let x = (u * self.width as f32).round() as usize;
        let y = (v * self.height as f32).round() as usize;
        (x, y)
    }
}

// --------------------------------------------------
// Доступ к отдельным пикселям UV-текстуры
// --------------------------------------------------

impl Index<(f32, f32)> for Texture {
    type Output = Color32;

    /// Получить пиксель текстуры по UV-координатам
    fn index(&self, index: (f32, f32)) -> &Self::Output {
        let (x, y) = self.transform_uv(index.0, index.1);
        debug_assert!(self.check_bounds(x, y));
        &self.image[y * self.width + x]
    }
}
