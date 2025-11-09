//! Реализация холста для 2D рисования.

use std::ops::{Index, IndexMut};

use egui::{Color32, ColorImage, Pos2, Vec2};

use std::collections::VecDeque;

use crate::Canvas;

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

// =============== Реализация холста ===============

impl Default for Canvas {
    fn default() -> Self {
        Self::new(900, 600)
    }
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![Color32::WHITE; width * height],
            width,
            height,
        }
    }

    #[inline]
    /// Проверить границы полотна.
    fn check_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && x > 0 && y > 0
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
    }
}

// =============== Доступ к отдельным пикселям холста ===============

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

// =============== Растровые алгоритмы над холстом ===============

// Задание 1 (всякие заливки)
impl Canvas {
    // Сюда можно приватные вспомогательные методы, если нужно

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

// Задание 2 (линии)
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
            self[(x0 as usize, y0 as usize)] = color;
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
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: Color32, intensity: f32) {
        if x >= 0 && y >= 0 {
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

// Задание 3 (растеризация треугольника с градиентом)
impl Canvas {
    /// Вычисление барицентрических координат через систему уравнений
    fn compute_barycentric_coords(
        &self,
        p: Pos2,
        a: Pos2,
        b: Pos2,
        c: Pos2,
    ) -> Option<(f32, f32, f32)> {
        let det = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);

        // Если определитель близок к нулю, треугольник вырожденный
        if det.abs() < 1e-10 {
            return None;
        }

        let alpha = ((b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y)) / det;
        let beta = ((c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y)) / det;
        let gamma = 1.0 - alpha - beta;

        Some((alpha, beta, gamma))
    }

    /// Интерполяция цвета по барицентрическим координатам
    fn interpolate_color(
        &self,
        alpha: f32,
        beta: f32,
        gamma: f32,
        color1: Color32,
        color2: Color32,
        color3: Color32,
    ) -> Color32 {
        let r = (alpha * color1.r() as f32 + beta * color2.r() as f32 + gamma * color3.r() as f32)
            .round() as u8;
        let g = (alpha * color1.g() as f32 + beta * color2.g() as f32 + gamma * color3.g() as f32)
            .round() as u8;
        let b = (alpha * color1.b() as f32 + beta * color2.b() as f32 + gamma * color3.b() as f32)
            .round() as u8;
        let a = (alpha * color1.a() as f32 + beta * color2.a() as f32 + gamma * color3.a() as f32)
            .round() as u8;

        Color32::from_rgba_premultiplied(r, g, b, a)
    }

    /// Градиентная растеризация треугольника через барицентрические координаты.
    /// pos[1..3] - 3 точки треугольника;
    /// color[1..3] - цвета соответствующих точек;
    pub fn draw_gradient_triangle(
        &mut self,
        pos1: Pos2,
        pos2: Pos2,
        pos3: Pos2,
        color1: Color32,
        color2: Color32,
        color3: Color32,
    ) {
        // ограничивающий прямоугольник
        let min_x = pos1.x.min(pos2.x.min(pos3.x)).floor() as usize;
        let min_y = pos1.y.min(pos2.y.min(pos3.y)).floor() as usize;
        let max_x = pos1.x.max(pos2.x.max(pos3.x)).ceil() as usize;
        let max_y = pos1.y.max(pos2.y.max(pos3.y)).ceil() as usize;

        // цикл по пикселям ограничевающего прямогольника
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x >= self.width || y >= self.height {
                    continue;
                }

                let pixel_pos = Pos2::new(x as f32, y as f32);

                // барицентрические координаты
                if let Some((alpha, beta, gamma)) =
                    self.compute_barycentric_coords(pixel_pos, pos1, pos2, pos3)
                {
                    // пиксель внутри треугольника
                    if alpha >= 0.0 && beta >= 0.0 && gamma >= 0.0 {
                        // интерполяция цвета
                        let color =
                            self.interpolate_color(alpha, beta, gamma, color1, color2, color3);
                        self[(x, y)] = color;
                    }
                }
            }
        }
    }
}
