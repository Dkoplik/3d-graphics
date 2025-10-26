use egui::{Color32, Painter, Pos2, epaint::PathStroke};

use crate::{Point3, Polygon3, RenderStyle};

// --------------------------------------------------
// Конструкторы
// --------------------------------------------------
impl Polygon3 {
    /// Создание пустого полигона.
    pub fn new() -> Self {
        Self { vertexes: vec![] }
    }

    /// Создать треугольник.
    pub fn triangle(p1: usize, p2: usize, p3: usize) -> Self {
        Self {
            vertexes: vec![p1, p2, p3],
        }
    }

    /// Создать полигон из списка индексов вершин.
    pub fn from_list(vertexes: &[usize]) -> Self {
        Self {
            vertexes: vertexes.into(),
        }
    }
}

// --------------------------------------------------
// Операции над полигоном (его изменение)
// --------------------------------------------------

impl Polygon3 {
    /// Добавить вершину (точку) в полигон.
    pub fn add_vertex(&mut self, index: usize) {
        self.vertexes.push(index);
    }
}

// --------------------------------------------------
// Проверки полигона
// --------------------------------------------------

impl Polygon3 {
    /// Состоит ли полигон только из одной вершины?
    pub fn is_vertex(&self) -> bool {
        self.vertexes.len() == 1
    }

    /// Состоит ли полигон только из одного ребра?
    pub fn is_edge(&self) -> bool {
        self.vertexes.len() == 2
    }

    /// Полигон является треугольником?
    pub fn is_triangle(&self) -> bool {
        self.vertexes.len() == 3
    }
}

impl Polygon3 {
    /// Нарисовать рёбра полигона.
    pub fn draw(&self, points: &[Point3], painter: &mut Painter, color: Color32, width: f32) {
        let mut lines: Vec<Pos2> = self
            .vertexes
            .iter()
            .cloned()
            .map(|index| Pos2::new(points[index].x, points[index].y))
            .collect();
        if lines.len() > 0 {
            lines.push(*lines.last().unwrap());
        }
        painter.line(
            lines,
            PathStroke {
                width,
                color: egui::epaint::ColorMode::Solid(color),
                kind: egui::StrokeKind::Middle,
            },
        );
    }
}
