//! Реализация холста для 2D рисования.

use std::ops::{Index, IndexMut};

use egui::{Color32, ColorImage, Pos2, Vec2};

use std::collections::VecDeque;

use crate::Canvas;

impl Default for Canvas {
    fn default() -> Self {
        Self::new(900, 600)
    }
}

// --------------------------------------------------
// Создание и базовые методы
// --------------------------------------------------

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        debug_assert!(width > 0, "ширина холста не может быть нулевой");
        debug_assert!(height > 0, "высота холста не может быть нулевой");

        Self {
            pixels: vec![Color32::GRAY; width * height],
            buffer: vec![f32::MIN; width * height],
            width,
            height,
        }
    }

    #[inline]
    /// Проверить границы полотна.
    fn check_bounds(&self, x: usize, y: usize) -> bool {
        //x < self.width && y < self.height && x > 0 && y > 0
        x < self.width && y < self.height
    }

    /// Преобразовать холст в ColorImage для дальнейшего использования в egui.
    pub fn to_color_image(&self) -> ColorImage {
        ColorImage {
            size: self.size(),
            source_size: Vec2 {
                x: self.width as f32,
                y: self.height as f32,
            },
            pixels: self.pixels.clone(),
        }
    }

    /// Размеры холста вида [ширина, высота].
    pub fn size(&self) -> [usize; 2] {
        [self.width, self.height]
    }

    /// Заполнить весь холст указанным цветом
    pub fn clear(&mut self, color: Color32) {
        self.pixels.fill(color);
        self.buffer.fill(f32::MIN);
    }

    /// Проверить и обновить значение z-буфера
    ///
    /// Если новое значение z больше текущего, то возвращает true и обновляет буфер,
    /// иначе возвращает false.
    pub fn test_and_set_z(&mut self, x: usize, y: usize, z: f32) -> bool {
        debug_assert!(
            x < self.width,
            "x {} должен быть меньше ширины {}",
            x,
            self.width
        );
        debug_assert!(
            y < self.height,
            "y {} должен быть меньше ширины {}",
            y,
            self.height
        );

        let index = y * self.width + x;
        if z > self.buffer[index] {
            self.buffer[index] = z;
            true
        } else {
            false
        }
    }

    /// Проверить значение z-буфера
    ///
    /// Если новое значение z больше либо равно текущему, то возвращает true.
    pub fn test_z(&self, x: usize, y: usize, z: f32) -> bool {
        debug_assert!(
            x < self.width,
            "x {} должен быть меньше ширины {}",
            x,
            self.width
        );
        debug_assert!(
            y < self.height,
            "y {} должен быть меньше ширины {}",
            y,
            self.height
        );

        let index = y * self.width + x;
        z >= self.buffer[index]
    }
}

// --------------------------------------------------
// Доступ к отдельным пикселям холста
// --------------------------------------------------

impl Index<(usize, usize)> for Canvas {
    type Output = Color32;

    // index = (x, y)
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        debug_assert!(self.check_bounds(x, y));
        &self.pixels[y * self.width + x]
    }
}

impl IndexMut<(usize, usize)> for Canvas {
    // index = (x, y)
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        debug_assert!(self.check_bounds(x, y));
        &mut self.pixels[y * self.width + x]
    }
}

// --------------------------------------------------
// Заливка
// --------------------------------------------------

impl Canvas {
    /// Вспомогательная функция для нахождения границ линии
    fn find_line_bounds(&self, x: usize, y: usize, old_color: Color32) -> (usize, usize) {
        let mut left = x;
        let mut right = x;

        while self.check_bounds(left, y) && self[(left, y)] == old_color {
            left -= 1;
        }
        left += 1;

        while self.check_bounds(right, y) && self[(right, y)] == old_color {
            right += 1;
        }
        right -= 1;

        (left, right)
    }

    /// Вспомогательная функция для проверки и добавления точки в стек
    fn check_and_push(
        &self,
        x: usize,
        y: usize,
        old_color: Color32,
        stack: &mut VecDeque<(usize, usize)>,
    ) {
        if self.check_bounds(x, y) && self[(x, y)] == old_color {
            stack.push_back((x, y));
        }
    }

    /// Рекурсивная заливка изображения.
    /// pos - позиция, в которой применяется заливка;
    /// color - цвет заливки;
    /// connectivity - тип заливки (4-х или 8-ми связная);
    pub fn fill_with_color(&mut self, pos: Pos2, color: Color32, connectivity: Connectivity) {
        let start_x = pos.x as usize;
        let start_y = pos.y as usize;

        if !self.check_bounds(start_x, start_y) {
            return;
        }

        let old_color = self[(start_x, start_y)];
        if old_color == color {
            return;
        }

        let mut stack = VecDeque::new();
        stack.push_back((start_x, start_y));

        while let Some((x, y)) = stack.pop_front() {
            if !self.check_bounds(x, y) || self[(x, y)] != old_color {
                continue;
            }

            let (left, right) = self.find_line_bounds(x, y, old_color);

            for i in left..=right {
                if self.check_bounds(i, y) {
                    self[(i, y)] = color;
                }
            }

            for i in left..=right {
                self.check_and_push(i, y - 1, old_color, &mut stack);
                self.check_and_push(i, y + 1, old_color, &mut stack);
            }
            match connectivity {
                Connectivity::FOUR => {}
                Connectivity::EIGHT => {
                    if left > 0 {
                        self.check_and_push(left - 1, y - 1, old_color, &mut stack);
                        self.check_and_push(left - 1, y + 1, old_color, &mut stack);
                    }
                    self.check_and_push(right + 1, y - 1, old_color, &mut stack);
                    self.check_and_push(right + 1, y + 1, old_color, &mut stack);
                }
            }
        }
    }

    /// Рекурсивная заливка изображения.
    /// pos - позиция, в которой применяется заливка;
    /// img - изображение для заливки;
    /// connectivity - тип заливки (4-х или 8-ми связная);
    pub fn fill_with_img(&mut self, pos: Pos2, img: &ColorImage, connectivity: Connectivity) {
        let start_x = pos.x as usize;
        let start_y = pos.y as usize;

        if !self.check_bounds(start_x, start_y) {
            return;
        }

        let old_color = self[(start_x, start_y)];
        let img_width = img.width();
        let img_height = img.height();

        let mut stack = VecDeque::new();
        stack.push_back((start_x, start_y));

        while let Some((x, y)) = stack.pop_front() {
            if !self.check_bounds(x, y) || self[(x, y)] != old_color {
                continue;
            }

            let (left, right) = self.find_line_bounds(x, y, old_color);

            let mut im_y = (y as i32 - start_y as i32) as i32;
            while im_y <= 0 {
                im_y += img_height as i32;
            }

            for i in left..=right {
                let mut im_x = i as i32 - start_x as i32;
                while im_x <= 0 {
                    im_x += img_width as i32;
                }

                let img_x = (im_x as usize).rem_euclid(img_width);
                let img_y = (im_y as usize).rem_euclid(img_height);

                if img_x < img_width && img_y < img_height {
                    self[(i, y)] = img[(img_x, img_y)];
                }
            }

            for i in left..=right {
                self.check_and_push(i, y - 1, old_color, &mut stack);
                self.check_and_push(i, y + 1, old_color, &mut stack);
            }
            match connectivity {
                Connectivity::FOUR => {}
                Connectivity::EIGHT => {
                    if left > 0 {
                        self.check_and_push(left - 1, y - 1, old_color, &mut stack);
                        self.check_and_push(left - 1, y + 1, old_color, &mut stack);
                    }
                    self.check_and_push(right + 1, y - 1, old_color, &mut stack);
                    self.check_and_push(right + 1, y + 1, old_color, &mut stack);
                }
            }
        }
    }

    /// Выделение границы связной области
    /// start_pos - начальная точка на границе;
    /// boundary_color - цвет границы;
    /// Возвращает список точек границы в порядке обхода
    pub fn trace_boundary(&self, start_pos: Pos2) -> Vec<Pos2> {
        let mut boundary_points = Vec::new();
        let start_x = start_pos.x as usize;
        let start_y = start_pos.y as usize;

        if !self.check_bounds(start_x, start_y) {
            return boundary_points;
        }

        let boundary_color = self[(start_x, start_y)];

        let mut current_x = start_x;
        let mut current_y = start_y;

        let mut prev_direction = 0;
        let directions: [(i32, i32); 8] = [
            (1, 0),   // вправо
            (1, -1),  // вправо-вверх
            (0, -1),  // вверх
            (-1, -1), // влево-вверх
            (-1, 0),  // влево
            (-1, 1),  // влево-вниз
            (0, 1),   // вниз
            (1, 1),   // вправо-вниз
        ];

        loop {
            boundary_points.push(Pos2::new(current_x as f32, current_y as f32));

            let mut found_next = false;
            prev_direction = (prev_direction + 6) % 8;

            for offset in 0..8 {
                let direction_index = (prev_direction + offset) % 8;
                let (dx, dy) = directions[direction_index];

                let next_x = if dx >= 0 {
                    current_x + dx as usize
                } else {
                    current_x - (-dx) as usize
                };

                let next_y = if dy >= 0 {
                    current_y + dy as usize
                } else {
                    current_y - (-dy) as usize
                };

                if self[(next_x, next_y)] == boundary_color && self.check_bounds(next_x, next_y) {
                    current_x = next_x;
                    current_y = next_y;
                    prev_direction = direction_index;
                    found_next = true;
                    break;
                }
            }

            if current_x == start_x && current_y == start_y {
                break;
            }

            if !found_next {
                break;
            }

            if boundary_points.len() > self.width * self.height {
                break;
            }
        }

        boundary_points
    }

    /// Нарисовать границу поверх изображения
    /// boundary_points - список точек границы;
    /// color - цвет для рисования границы;
    pub fn draw_boundary(&mut self, boundary_points: &[Pos2], color: Color32) {
        for &point in boundary_points {
            let x = point.x as usize;
            let y = point.y as usize;
            self[(x, y)] = color;
        }
    }
}

// --------------------------------------------------
// Рисование линий
// --------------------------------------------------

impl Canvas {
    /// Рисование линии алгоритмом Брезенхема.
    /// pos1 - первая точка линии;
    /// pos2 - вторая точка линии;
    /// color - цвет линии;
    pub fn draw_sharp_line(&mut self, pos1: Pos2, pos2: Pos2, color: Color32) {
        let mut x0 = pos1.x.round() as i32;
        let mut y0 = pos1.y.round() as i32;
        let x1 = pos2.x.round() as i32;
        let y1 = pos2.y.round() as i32;
        let dx = x1.abs_diff(x0) as i32;
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1.abs_diff(y0) as i32);
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;

        loop {
            // ✅ ДОБАВЬТЕ ЭТУ ПРОВЕРКУ:
            if (x0 as usize) < self.width && (y0 as usize) < self.height {
                self[(x0 as usize, y0 as usize)] = color;
            }

            let e2 = 2 * error;
            if e2 >= dy {
                if x0 == x1 {
                    break;
                }
                error += dy;
                x0 += sx;
            }
            if e2 <= dx {
                if y0 == y1 {
                    break;
                }
                error += dx;
                y0 += sy;
            }
        }
    }

    /// Рисование линии алгоритмом Ву.
    /// pos1 - первая точка линии;
    /// pos2 - вторая точка линии;
    /// color - цвет линии;
    pub fn draw_smooth_line_simple(&mut self, pos1: Pos2, pos2: Pos2, color: Color32) {
        let mut x1 = pos1.x;
        let mut y1 = pos1.y;
        let mut x2 = pos2.x;
        let mut y2 = pos2.y;

        let steep = (y2 - y1).abs() > (x2 - x1).abs();
        if steep {
            std::mem::swap(&mut x1, &mut y1);
            std::mem::swap(&mut x2, &mut y2);
        }
        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
            std::mem::swap(&mut y1, &mut y2);
        }

        let dx = x2 - x1;
        let dy = y2 - y1;
        let gradient = dy / dx;

        let mut intery = y1 + gradient;

        for x in (x1 as i32)..=(x2 as i32) {
            let y_floor = intery as i32;
            let intensity1 = 1.0 - (intery - y_floor as f32);
            let intensity2 = intery - y_floor as f32;

            if steep {
                self.set_pixel(y_floor, x, color, intensity1);
                self.set_pixel(y_floor + 1, x, color, intensity2);
            } else {
                self.set_pixel(x, y_floor, color, intensity1);
                self.set_pixel(x, y_floor + 1, color, intensity2);
            }

            intery += gradient;
        }

        for x in (x1 as i32)..=(x2 as i32) {
            let y_floor = intery as i32;
            let intensity1 = 1.0 - (intery - y_floor as f32);
            let intensity2 = intery - y_floor as f32;

            if steep {
                if (y_floor as usize) < self.height && (x as usize) < self.width {
                    // ✅ ДОБАВЬТЕ
                    self.set_pixel(y_floor, x, color, intensity1);
                }
                if ((y_floor + 1) as usize) < self.height && (x as usize) < self.width {
                    // ✅ ДОБАВЬТЕ
                    self.set_pixel(y_floor + 1, x, color, intensity2);
                }
            } else {
                if (x as usize) < self.width && (y_floor as usize) < self.height {
                    // ✅ ДОБАВЬТЕ
                    self.set_pixel(x, y_floor, color, intensity1);
                }
                if (x as usize) < self.width && ((y_floor + 1) as usize) < self.height {
                    // ✅ ДОБАВЬТЕ
                    self.set_pixel(x, y_floor + 1, color, intensity2);
                }
            }
            intery += gradient;
        }
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: Color32, intensity: f32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let background = self[(x as usize, y as usize)];

            let bg_r = background.r() as f32;
            let bg_g = background.g() as f32;
            let bg_b = background.b() as f32;
            let bg_a = background.a() as f32;

            let fg_r = color.r() as f32;
            let fg_g = color.g() as f32;
            let fg_b = color.b() as f32;
            let fg_a = color.a() as f32;

            let result_r = (bg_r * (1.0 - intensity) + fg_r * intensity) as u8;
            let result_g = (bg_g * (1.0 - intensity) + fg_g * intensity) as u8;
            let result_b = (bg_b * (1.0 - intensity) + fg_b * intensity) as u8;
            let result_a = (bg_a * (1.0 - intensity) + fg_a * intensity) as u8;

            self[(x as usize, y as usize)] =
                Color32::from_rgba_premultiplied(result_r, result_g, result_b, result_a);
        }
    }

    pub fn circle_filled(&mut self, center: Pos2, radius: f32, color: Color32) {
        let from_x = ((center.x - radius).round() as usize)
            .max(0)
            .min(self.width - 1);
        let to_x = ((center.x + radius).round() as usize)
            .max(0)
            .min(self.width - 1);

        let from_y = ((center.y - radius).round() as usize)
            .max(0)
            .min(self.height - 1);
        let to_y = ((center.y + radius).round() as usize)
            .max(0)
            .min(self.height - 1);

        for x in from_x..to_x {
            for y in from_y..to_y {
                if ((x * x + y * y) as f32).sqrt() <= radius {
                    self[(x, y)] = color;
                }
            }
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
/// Вариант связности, нужен для заливки.
pub enum Connectivity {
    /// 4-х связная заливка
    FOUR,
    #[default]
    /// 8-ми связная заливка
    EIGHT,
}

impl Connectivity {
    pub fn get_name(&self) -> String {
        match self {
            Connectivity::FOUR => String::from("4-х связная"),
            Connectivity::EIGHT => String::from("8-ми связная"),
        }
    }
}
