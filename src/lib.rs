use egui::Color32;
pub mod classes3d;

// Экспортируем ProjectionType для использования в других модулях
pub use classes3d::camera3::ProjectionType;

/// Файл содержит только определения основных классов для 3D.
/// Сами реализации в отдельных файлах, дабы задать более удобную структуру проекта.

/// Точка в 3D пространстве.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Вектор (направление) в 3D пространстве.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Плоскость в 3D пространстве.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    /// Точка, через которую проходит плоскость.
    pub origin: Point3,
    /// Нормаль плоскости в виде единичного вектора.
    pub normal: Vec3,
}

/// Линия в 3D пространстве, заданная 2-мя точками.
#[derive(Debug, Clone, Copy)]
pub struct Line3 {
    /// Точка, через которую проходит прямая.
    pub origin: Point3,
    /// Направление прямой в виде единичного вектора.
    pub direction: Vec3,
}

/// Полигон модели в 3D пространстве. Предполагается использование только в пределах модели.
#[derive(Debug, Clone)]
pub struct Polygon3 {
    /// Вершины полигона как индексы из вектора точек модели.
    vertexes: Vec<usize>,
}

/// Модель (объект) в 3D пространстве.
#[derive(Debug, Clone)]
pub struct Model3 {
    /// Все 3D точки (вершины), используемые для данной модели.
    /// Их координаты задаются относительно центра модели.
    vertexes: Vec<Point3>,
    /// Все полигоны данной 3D модели.
    polygons: Vec<Polygon3>,
    /// Центр модели, вокруг которого она строится.
    origin: Point3,
}

/// Сцена в 3-х мерном пространстве с 3-х мерными объектами (моделями).
#[derive(Debug, Clone)]
pub struct Scene {
    /// Модели на сцене.
    pub models: Vec<Model3>,
}

/// Камера в 3-х мерном пространстве.
pub struct Camera3 {
    /// Позиция камеры в простанстве.
    pub position: Point3,
    /// В какую сторону камера смотрит.
    pub direction: Vec3,
    /// Направление вверх для камеры.
    pub up: Vec3,
    /// Field of View.
    pub fov: f32,
    /// Соотношение сторон.
    pub aspect_ratio: f32,
    /// С какого расстояния от точки камеры отображать объекты.
    pub near_plane: f32,
    /// До какого расстояния отображать объекты.
    pub far_plane: f32,
    /// Тип проекции камеры.
    pub projection_type: ProjectionType,
}

/// Преобразование в 3-х мерном пространстве.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3D {
    // Матрица преобразования 4x4 в виде одномерного массива в row-major порядке.
    pub m: [f32; 16],
}

// TODO Пока что не уверен, нужен ли этот класс.
/// Аффинное 2D преобразование
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    // Матрица аффинного преобразования:
    // [a, d, 0]
    // [b, e, 0]
    // [c, f, 1]
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

/// К объекту можно применить преобразование в 3D пространстве
pub trait Transformable3 {
    /// Применить преобразование, получив новый объект.
    fn transform(self, transform: Transform3D) -> Self;
    /// Применить преобразование к текущему объекту.
    fn apply_transform(&mut self, transform: Transform3D);
}

pub struct RenderStyle {
    pub vertex_color: Color32,
    pub vertex_radius: f32,
    pub edge_color: Color32,
    pub edge_width: f32,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            vertex_color: Color32::BLACK,
            vertex_radius: 2.5,
            edge_color: Color32::BLACK,
            edge_width: 1.0,
        }
    }
}
