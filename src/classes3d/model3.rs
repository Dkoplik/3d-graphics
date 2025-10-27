use egui::Painter;

use crate::{Model3, Point3, Polygon3, RenderStyle, Transform3D, Transformable3};

impl Model3 {
    pub fn new(origin: Point3, vertexes: Vec<Point3>, polygons: Vec<Polygon3>) -> Self {
        Model3 {
            origin,
            vertexes,
            polygons,
        }
    }

    /// Создание тетраэдра со сторонами единичной длины.
    pub fn tetrahedron() -> Self {
        // Координаты правильного тетраэдра с длиной ребра = 1
        let a = 1.0 / (2.0 * (2.0 as f32).sqrt());
        let b = 1.0 / (6.0 as f32).sqrt();
        
        let vertexes = vec![
            Point3::new(0.0, 0.0, b),                           // Верхняя вершина
            Point3::new(2.0 * a, 0.0, -a),                      // Нижняя вершина 1
            Point3::new(-a, (3.0 as f32).sqrt() * a, -a),       // Нижняя вершина 2
            Point3::new(-a, (-3.0 as f32).sqrt() * a, -a),      // Нижняя вершина 3
        ];

        let polygons = vec![
            Polygon3::triangle(0, 1, 2),
            Polygon3::triangle(0, 2, 3),
            Polygon3::triangle(0, 3, 1),
            Polygon3::triangle(1, 3, 2),  // Основание
        ];

        let origin = Point3::new(0.0, 0.0, 0.0);

        Model3::new(origin, vertexes, polygons)
    }

    /// Создание гексаэдра со сторонами единичной длины.
    pub fn hexahedron() -> Self {
        // Куб с длиной ребра = 1, центрированный в начале координат
        let half = 0.5;
        
        let vertexes = vec![
            // Нижняя грань
            Point3::new(-half, -half, -half),
            Point3::new(half, -half, -half), 
            Point3::new(half, half, -half),  
            Point3::new(-half, half, -half), 
            // Верхняя грань
            Point3::new(-half, -half, half), 
            Point3::new(half, -half, half),  
            Point3::new(half, half, half),   
            Point3::new(-half, half, half),  
        ];

        let polygons = vec![
            // Нижняя грань
            Polygon3::triangle(0, 1, 2),
            Polygon3::triangle(0, 2, 3),
            // Верхняя грань
            Polygon3::triangle(4, 6, 5),
            Polygon3::triangle(4, 7, 6),
            // Передняя грань
            Polygon3::triangle(3, 2, 6),
            Polygon3::triangle(3, 6, 7),
            // Задняя грань
            Polygon3::triangle(0, 5, 1),
            Polygon3::triangle(0, 4, 5),
            // Левая грань
            Polygon3::triangle(0, 3, 7),
            Polygon3::triangle(0, 7, 4),
            // Правая грань
            Polygon3::triangle(1, 5, 6),
            Polygon3::triangle(1, 6, 2),
        ];

        let origin = Point3::new(0.0, 0.0, 0.0);

        Model3::new(origin, vertexes, polygons)
    }

    /// Создание октаэдра со сторонами единичной длины.
    pub fn octahedron() -> Self {
        // Октаэдр с длиной ребра = 1, центрированный в начале координат
        let a = 1.0 / (2.0 as f32).sqrt(); // Для получения длины ребра = 1
        
        let vertexes = vec![
            // Верхняя и нижняя вершины
            Point3::new(0.0, 0.0, a), 
            Point3::new(0.0, 0.0, -a),
            // Вершины в плоскости XY
            Point3::new(a, 0.0, 0.0), 
            Point3::new(0.0, a, 0.0), 
            Point3::new(-a, 0.0, 0.0),
            Point3::new(0.0, -a, 0.0),
        ];

        let polygons = vec![
            // Верхние треугольники
            Polygon3::triangle(0, 2, 3), // верх-право-перед
            Polygon3::triangle(0, 3, 4), // верх-перед-лево
            Polygon3::triangle(0, 4, 5), // верх-лево-зад
            Polygon3::triangle(0, 5, 2), // верх-зад-право
            // Нижние треугольники
            Polygon3::triangle(1, 3, 2), // низ-перед-право
            Polygon3::triangle(1, 4, 3), // низ-лево-перед
            Polygon3::triangle(1, 5, 4), // низ-зад-лево
            Polygon3::triangle(1, 2, 5), // низ-право-зад
        ];

        let origin = Point3::new(0.0, 0.0, 0.0);

        Model3::new(origin, vertexes, polygons)
    }

    /// Создание икосаэдр со сторонами единичной длины.
    pub fn icosahedron() -> Self {
        // TODO
        todo!("Сделать икосаэдр");
    }

    /// Создание тетраэдра со сторонами единичной длины.
    pub fn dodecahedron() -> Self {
        // TODO
        todo!("Сделать додекаэдр");
    }

    /// Получить матрицу преобразования к мировым координатам.
    pub fn get_world_transform(&self) -> Transform3D {
        Transform3D::translation(self.origin.x, self.origin.y, self.origin.z)
    }

    /// Получить копию модели, но в глобальных координатах.
    pub fn to_world_coordinates(self) -> Self {
        let transform = self.get_world_transform();
        self.transform(transform)
    }

    /// Нарисовать модель.
    pub fn draw(&self, painter: &mut Painter, style: &RenderStyle) {
        self.polygons.iter().for_each(|polygon| {
            polygon.draw(&self.vertexes, painter, style.edge_color, style.edge_width)
        });
        self.vertexes
            .iter()
            .for_each(|vertex| vertex.draw(painter, style.vertex_color, style.vertex_radius));
    }
}

impl Transformable3 for Model3 {
    fn transform(self, transform: crate::Transform3D) -> Self {
        let origin = self.origin.transform(transform);
        let vertexes = self
            .vertexes
            .iter()
            .cloned()
            .map(|vertex| vertex.transform(transform))
            .collect();
        Self::new(origin, vertexes, self.polygons)
    }

    fn apply_transform(&mut self, transform: crate::Transform3D) {
        self.origin.apply_transform(transform);
        self.vertexes
            .iter_mut()
            .for_each(|vertex| vertex.apply_transform(transform));
    }
}
