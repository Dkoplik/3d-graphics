// Пока что не уверен, нужен ли этот класс.
use crate::Transform2D;

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

// --------------------------------------------------
// Конструкторы базовых преобразований
// --------------------------------------------------

impl Transform2D {
    /// Тождественное преобразование.
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: 1.0,
            f: 0.0,
        }
    }

    /// Вращение в радианах (против часовой стрелки).
    pub fn rotation(angle_rad: f32) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self {
            a: cos_a,
            b: sin_a,
            c: 0.0,
            d: -sin_a,
            e: cos_a,
            f: 0.0,
        }
    }

    /// Вращение в градусах (против часовой стрелки).
    pub fn rotation_degrees(angle_deg: f32) -> Self {
        Self::rotation(angle_deg.to_radians())
    }

    /// Параллельный сдвиг.
    pub fn translation(dx: f32, dy: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: dx,
            d: 0.0,
            e: 1.0,
            f: dy,
        }
    }

    /// Растяжение-сжатие.
    pub fn scaling(kx: f32, ky: f32) -> Self {
        Self {
            a: kx,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: ky,
            f: 0.0,
        }
    }

    /// Равномерное растяжение-сжатие (по x и по y одинаковое растяжение).
    pub fn uniform_scaling(scale: f32) -> Self {
        Self::scaling(scale, scale)
    }

    /// Перенос.
    pub fn shear(shx: f32, shy: f32) -> Self {
        Self {
            a: 1.0,
            b: shx,
            c: 0.0,
            d: shy,
            e: 1.0,
            f: 0.0,
        }
    }

    /// Зеркальное отражение по оси X.
    pub fn reflection_x() -> Self {
        Self::scaling(1.0, -1.0)
    }

    /// Зеркальное отражение по оси Y.
    pub fn reflection_y() -> Self {
        Self::scaling(-1.0, 1.0)
    }
}

// --------------------------------------------------
// Базовые операции
// --------------------------------------------------

impl Transform2D {
    /// Перемножение с другим преобразованием (композиция преобразований).
    pub fn multiply(&self, other: &Self) -> Self {
        // TODO в прошлый раз работало некорреткно!!!
        todo!("Текущая реализации умножения 2D преобразований некорретна!");
        // [a, d, 0]
        // [b, e, 0]
        // [c, f, 1]
        Self {
            a: self.a * other.a + self.b * other.d,
            b: self.a * other.b + self.b * other.e,
            c: self.a * other.c + self.b * other.f + self.c,
            d: self.d * other.a + self.e * other.d,
            e: self.d * other.b + self.e * other.e,
            f: self.d * other.c + self.e * other.f + self.f,
        }
    }

    /// Применить преобразование к координатам.
    pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        let new_x = self.a * x + self.b * y + self.c;
        let new_y = self.d * x + self.e * y + self.f;
        (new_x, new_y)
    }

    /// Применить преобразование к позиции (egui::Pos2).
    pub fn apply_to_pos(&self, pos: egui::Pos2) -> egui::Pos2 {
        let (x, y) = self.apply(pos.x, pos.y);
        egui::Pos2 { x, y }
    }

    /// Обратная матрица
    pub fn inverse(&self) -> Self {
        let det = self.determinant();

        // Матрица необратима => это не афинное преобразование.
        if det.abs() < 1e-12 {
            panic!("Матрица не является обратимой, следовательно, это не афинное преобразование");
        }

        let inv_det = 1.0 / det;

        Self {
            a: self.e * inv_det,
            b: -self.d * inv_det,
            c: (self.d * self.f - self.c * self.e) * inv_det,
            d: -self.b * inv_det,
            e: self.a * inv_det,
            f: (self.c * self.b - self.a * self.f) * inv_det,
        }
    }

    /// Определитель матрицы преобразования.
    pub fn determinant(&self) -> f32 {
        self.a * self.e - self.b * self.d
    }

    /// Является ли проеобразование тождественным?
    /// tolerance - допустимая погрешность;
    pub fn is_identity(&self, tolerance: f32) -> bool {
        (self.a - 1.0).abs() < tolerance
            && (self.e - 1.0).abs() < tolerance
            && self.b.abs() < tolerance
            && self.c.abs() < tolerance
            && self.d.abs() < tolerance
            && self.f.abs() < tolerance
    }
}

// --------------------------------------------------
// Конструкторы составных (сложных) преобразований
// --------------------------------------------------

impl Transform2D {
    /// Поворот вокруг заданной точки.
    pub fn rotation_around_point(angle_rad: f32, center_x: f32, center_y: f32) -> Self {
        Self::translation(center_x, center_y)
            .multiply(&Self::rotation(angle_rad))
            .multiply(&Self::translation(-center_x, -center_y))
    }

    /// Поворот в градусах вокруг заданной точки.
    pub fn rotation_degree_around_point(angle_degree: f32, center_x: f32, center_y: f32) -> Self {
        Self::rotation_around_point(angle_degree.to_radians(), center_x, center_y)
    }

    /// Поворот вокруг заданной позиции (egui::Pos2).
    pub fn rotation_around_pos(angle_rad: f32, pos: egui::Pos2) -> Self {
        Self::rotation_around_point(angle_rad, pos.x, pos.y)
    }

    /// Поворот в градусах вокруг заданной позиции (egui::Pos2).
    pub fn rotation_degree_around_pos(angle_degree: f32, pos: egui::Pos2) -> Self {
        Self::rotation_around_pos(angle_degree.to_radians(), pos)
    }

    /// Масштабирование относительно заданной точки.
    pub fn scaling_around_point(kx: f32, ky: f32, center_x: f32, center_y: f32) -> Self {
        Self::translation(center_x, center_y)
            .multiply(&Self::scaling(kx, ky))
            .multiply(&Self::translation(-center_x, -center_y))
    }

    /// Масштабирование относительно заданной позиции (egui::Pos2).
    pub fn scaling_around_pos(kx: f32, ky: f32, pos: egui::Pos2) -> Self {
        Self::scaling_around_point(kx, ky, pos.x, pos.y)
    }

    /// Равномерное масштабирование относительно заданной точки.
    pub fn uniform_scaling_around_point(scale: f32, center_x: f32, center_y: f32) -> Self {
        Self::scaling_around_point(scale, scale, center_x, center_y)
    }

    /// Равномерное масштабирование относительно заданной позиции (egui::Pos2).
    pub fn uniform_scaling_around_pos(scale: f32, pos: egui::Pos2) -> Self {
        Self::scaling_around_point(scale, scale, pos.x, pos.y)
    }
}

// --------------------------------------------------
// Операции над преобразованиями
// --------------------------------------------------

// Эти трейты переобределяют операцию '*' для этой структуры
impl std::ops::Mul for Transform2D {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        self.multiply(&other)
    }
}

impl std::ops::Mul<&Transform2D> for Transform2D {
    type Output = Self;

    fn mul(self, other: &Transform2D) -> Self {
        self.multiply(other)
    }
}
