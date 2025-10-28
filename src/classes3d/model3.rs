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
    /// Получить центр модели.
    pub fn get_origin(&self) -> Point3 {
        self.origin
    }

    /// Получить вершины модели.
    pub fn get_vertexes(&self) -> &Vec<Point3> {
        &self.vertexes
    }

    /// Получить полигоны модели.
    pub fn get_polygons(&self) -> &Vec<Polygon3> {
        &self.polygons
    }

    /// Установить центр модели.
    pub fn set_origin(&mut self, origin: Point3) {
        self.origin = origin;
    }
    
    /// Создание тетраэдра со сторонами единичной длины.
    pub fn tetrahedron() -> Self {
        // Координаты правильного тетраэдра с длиной ребра = 1
        let height = (2.0 / 3.0_f32).sqrt(); // высота тетраэдра
        let base_height = (3.0_f32).sqrt() / 3.0; // высота основания
        
        let vertexes = vec![
            // Вершина тетраэдра
            Point3::new(0.0, 0.0, height),
            // Основание (равносторонний треугольник)
            Point3::new(0.0, base_height, 0.0),
            Point3::new(0.5, -base_height / 2.0, 0.0),
            Point3::new(-0.5, -base_height / 2.0, 0.0),
        ];

        let polygons = vec![
            Polygon3::triangle(0, 1, 2),
            Polygon3::triangle(0, 2, 3),
            Polygon3::triangle(0, 3, 1),
            Polygon3::triangle(1, 3, 2),
        ];

        Model3::new(Point3::new(0.0, 0.0, 0.0), vertexes, polygons)
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

/// Создание икосаэдра со сторонами единичной длины.
pub fn icosahedron() -> Self {
    // Золотое сечение
    let phi = (1.0 + (5.0_f32).sqrt()) / 2.0;
    
    let vertexes = vec![
        // Верхние и нижние вершины
        Point3::new(0.0, 1.0, phi),    // 0: зад-верх
        Point3::new(0.0, 1.0, -phi),   // 1: перед-верх
        Point3::new(0.0, -1.0, phi),   // 2: зад-низ
        Point3::new(0.0, -1.0, -phi),  // 3: перед-низ
        
        // Боковые вершины - передние
        Point3::new(1.0, phi, 0.0),    // 4: верх-право
        Point3::new(-1.0, phi, 0.0),   // 5: верх-лево
        
        // Боковые вершины - задние  
        Point3::new(1.0, -phi, 0.0),   // 6: низ-право
        Point3::new(-1.0, -phi, 0.0),  // 7: низ-лево
        
        // Передние и задние вершины
        Point3::new(phi, 0.0, 1.0),    // 8: право-зад
        Point3::new(phi, 0.0, -1.0),   // 9: право-перед
        Point3::new(-phi, 0.0, 1.0),   // 10: лево-зад
        Point3::new(-phi, 0.0, -1.0),  // 11: лево-перед
    ];

    // Нормализуем вершины
    let vertexes: Vec<Point3> = vertexes.into_iter()
        .map(|p| {
            let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
            Point3::new(p.x / len, p.y / len, p.z / len)
        })
        .collect();

    let polygons = vec![
        Polygon3::triangle(0, 4, 8),
        Polygon3::triangle(0, 8, 2),
        Polygon3::triangle(0, 2, 10),  
        Polygon3::triangle(0, 10, 5),
        Polygon3::triangle(0, 5, 4),
        
        Polygon3::triangle(1, 9, 4),
        Polygon3::triangle(1, 4, 5),
        Polygon3::triangle(1, 5, 11),
        Polygon3::triangle(1, 11, 3),
        Polygon3::triangle(1, 3, 9),
        
        Polygon3::triangle(4, 9, 8),
        Polygon3::triangle(8, 9, 6), 
        Polygon3::triangle(6, 9, 3),
        Polygon3::triangle(11, 5, 10),
        Polygon3::triangle(10, 5, 0),
        
        Polygon3::triangle(7, 6, 2),
        Polygon3::triangle(7, 6, 3),
        Polygon3::triangle(7, 2, 10),
        Polygon3::triangle(7, 10, 11),
        Polygon3::triangle(7, 11, 3),
    ];

    let origin = Point3::new(0.0, 0.0, 0.0);

    Model3::new(origin, vertexes, polygons)
}

/// Создание додекаэдра со сторонами единичной длины.
pub fn dodecahedron() -> Self {
    // Золотое сечение
    let phi = (1.0 + (5.0_f32).sqrt()) / 2.0;
    let inv_phi = 1.0 / phi;
    
    let vertexes = vec![
        // (±1, ±1, ±1)
        Point3::new(1.0, 1.0, 1.0),        // 0
        Point3::new(1.0, 1.0, -1.0),       // 1
        Point3::new(1.0, -1.0, 1.0),       // 2
        Point3::new(1.0, -1.0, -1.0),      // 3
        Point3::new(-1.0, 1.0, 1.0),       // 4
        Point3::new(-1.0, 1.0, -1.0),      // 5
        Point3::new(-1.0, -1.0, 1.0),      // 6
        Point3::new(-1.0, -1.0, -1.0),     // 7
        
        // (0, ±1/φ, ±φ)
        Point3::new(0.0, inv_phi, phi),    // 8
        Point3::new(0.0, inv_phi, -phi),   // 9
        Point3::new(0.0, -inv_phi, phi),   // 10
        Point3::new(0.0, -inv_phi, -phi),  // 11
        
        // (±1/φ, ±φ, 0)
        Point3::new(inv_phi, phi, 0.0),    // 12
        Point3::new(inv_phi, -phi, 0.0),   // 13
        Point3::new(-inv_phi, phi, 0.0),   // 14
        Point3::new(-inv_phi, -phi, 0.0),  // 15
        
        // (±φ, 0, ±1/φ)
        Point3::new(phi, 0.0, inv_phi),    // 16
        Point3::new(phi, 0.0, -inv_phi),   // 17
        Point3::new(-phi, 0.0, inv_phi),   // 18
        Point3::new(-phi, 0.0, -inv_phi),  // 19
    ];

    // Нормализуем вершины
    let vertexes: Vec<Point3> = vertexes.into_iter()
        .map(|p| {
            let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
            Point3::new(p.x / len, p.y / len, p.z / len)
        })
        .collect();

    let polygons = vec![
        Polygon3::from_list(&[0, 8, 10, 2, 16]),
        Polygon3::from_list(&[0, 16, 17, 1, 12]),
        Polygon3::from_list(&[0, 12, 14, 4, 8]),
        Polygon3::from_list(&[1, 9, 11, 3, 17]),
        Polygon3::from_list(&[2, 10, 6, 15, 13]),
        Polygon3::from_list(&[3, 11, 7, 15, 13]),
        Polygon3::from_list(&[3, 13, 2, 16, 17]),
        Polygon3::from_list(&[4, 18, 19, 5, 14]),
        Polygon3::from_list(&[5, 19, 7, 11, 9]),
        Polygon3::from_list(&[6, 15, 7, 19, 18]),
    ];

    let origin = Point3::new(0.0, 0.0, 0.0);

    Model3::new(origin, vertexes, polygons)
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
    /// Нарисовать модель.
    pub fn draw(&self, painter: &mut Painter, style: &RenderStyle) {
        // Преобразуем 3D точки в 2D с помощью простой ортографической проекции
        let projected_points: Vec<egui::Pos2> = self
            .vertexes
            .iter()
            .map(|vertex| {
                // Простая ортографическая проекция (игнорируем Z для демонстрации)
                // Масштабируем координаты чтобы они поместились в видимую область
                let scale = 100.0; // Масштаб для видимости
                let center_x = 450.0; // Центр холста (предполагаемый размер)
                let center_y = 300.0;

                egui::Pos2::new(
                    center_x + vertex.x * scale,
                    center_y - vertex.y * scale, // Инвертируем Y для правильной ориентации
                )
            })
            .collect();

        // Рисуем рёбра
        for polygon in &self.polygons {
            if polygon.vertexes.len() >= 2 {
                let points: Vec<egui::Pos2> = polygon
                    .vertexes
                    .iter()
                    .map(|&index| projected_points[index])
                    .collect();

                // Рисуем линии между вершинами полигона
                for i in 0..points.len() {
                    let start = points[i];
                    let end = points[(i + 1) % points.len()];

                    painter.line_segment([start, end], (style.edge_width, style.edge_color));
                }
            }
        }

        for &point in &projected_points {
            painter.circle_filled(point, style.vertex_radius, style.vertex_color);
        }
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
