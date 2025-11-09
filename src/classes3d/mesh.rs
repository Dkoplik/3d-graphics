//! Реализация Mesh'а 3D модели.
//!
//! По сути, это является каркасом модели, которого достаточно только
//! для рендера в формате wireframe.

// --------------------------------------------------
// Mesh
// --------------------------------------------------

use crate::{CoordFrame, HVec3, Mesh};

impl Mesh {
    /// Создать новый Mesh из уже известных вершин и полигонов.
    ///
    /// Локальная система координат этого Mesh'а будет совпадать с глобальной.
    pub fn new(vertexes: Vec<HVec3>, polygons: Vec<Polygon3>) -> Self {
        #[cfg(debug_assertions)]
        for polygon in polygons.clone() {
            assert!(
                polygon.get_vertexes().len() >= 3,
                "Полигон должен быть хотя бы из 3 вершин"
            );
            for index in polygon.get_vertexes() {
                if *index >= vertexes.len() {
                    panic!("Полигон содержит индекс несуществующей вершины");
                }
            }
        }

        Mesh {
            vertexes,
            polygons,
            local_frame: CoordFrame::global(),
        }
    }

    /// Создание тетраэдра со сторонами единичной длины.
    pub fn tetrahedron() -> Self {
        // Координаты правильного тетраэдра с длиной ребра = 1
        let height = (2.0 / 3.0_f32).sqrt(); // высота тетраэдра
        let base_height = (3.0_f32).sqrt() / 3.0; // высота основания

        let vertexes = vec![
            // Вершина тетраэдра
            HVec3::new(0.0, 0.0, height),
            // Основание (равносторонний треугольник)
            HVec3::new(0.0, base_height, 0.0),
            HVec3::new(0.5, -base_height / 2.0, 0.0),
            HVec3::new(-0.5, -base_height / 2.0, 0.0),
        ];

        let polygons = vec![
            Polygon3::triangle(0, 1, 2),
            Polygon3::triangle(0, 2, 3),
            Polygon3::triangle(0, 3, 1),
            Polygon3::triangle(1, 3, 2),
        ];

        Self::new(vertexes, polygons)
    }

    /// Создание гексаэдра со сторонами единичной длины.
    pub fn hexahedron() -> Self {
        // Куб с длиной ребра = 1, центрированный в начале координат
        let half = 0.5;

        let vertexes = vec![
            // Нижняя грань
            HVec3::new(-half, -half, -half),
            HVec3::new(half, -half, -half),
            HVec3::new(half, half, -half),
            HVec3::new(-half, half, -half),
            // Верхняя грань
            HVec3::new(-half, -half, half),
            HVec3::new(half, -half, half),
            HVec3::new(half, half, half),
            HVec3::new(-half, half, half),
        ];

        let polygons = vec![
            Polygon3::from_list(&[0, 1, 2, 3]),
            Polygon3::from_list(&[4, 5, 6, 7]),
            Polygon3::from_list(&[3, 2, 6, 7]),
            Polygon3::from_list(&[0, 1, 5, 4]),
            Polygon3::from_list(&[0, 3, 7, 4]),
            Polygon3::from_list(&[1, 2, 6, 5]),
        ];

        Self::new(vertexes, polygons)
    }

    /// Создание октаэдра со сторонами единичной длины.
    pub fn octahedron() -> Self {
        // Октаэдр с длиной ребра = 1, центрированный в начале координат
        let a = 1.0 / (2.0 as f32).sqrt(); // Для получения длины ребра = 1

        let vertexes = vec![
            // Верхняя и нижняя вершины
            HVec3::new(0.0, 0.0, a),
            HVec3::new(0.0, 0.0, -a),
            // Вершины в плоскости XY
            HVec3::new(a, 0.0, 0.0),
            HVec3::new(0.0, a, 0.0),
            HVec3::new(-a, 0.0, 0.0),
            HVec3::new(0.0, -a, 0.0),
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

        Self::new(vertexes, polygons)
    }

    /// Создание икосаэдра со сторонами единичной длины.
    pub fn icosahedron() -> Self {
        // Золотое сечение
        let phi = (1.0 + (5.0_f32).sqrt()) / 2.0;

        let vertexes = vec![
            // Верхние и нижние вершины
            HVec3::new(0.0, 1.0, phi),   // 0: зад-верх
            HVec3::new(0.0, 1.0, -phi),  // 1: перед-верх
            HVec3::new(0.0, -1.0, phi),  // 2: зад-низ
            HVec3::new(0.0, -1.0, -phi), // 3: перед-низ
            // Боковые вершины - передние
            HVec3::new(1.0, phi, 0.0),  // 4: верх-право
            HVec3::new(-1.0, phi, 0.0), // 5: верх-лево
            // Боковые вершины - задние
            HVec3::new(1.0, -phi, 0.0),  // 6: низ-право
            HVec3::new(-1.0, -phi, 0.0), // 7: низ-лево
            // Передние и задние вершины
            HVec3::new(phi, 0.0, 1.0),   // 8: право-зад
            HVec3::new(phi, 0.0, -1.0),  // 9: право-перед
            HVec3::new(-phi, 0.0, 1.0),  // 10: лево-зад
            HVec3::new(-phi, 0.0, -1.0), // 11: лево-перед
        ];

        // Нормализуем вершины
        let vertexes: Vec<HVec3> = vertexes
            .into_iter()
            .map(|p| {
                let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
                HVec3::new(p.x / len, p.y / len, p.z / len)
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

        Self::new(vertexes, polygons)
    }

    /// Создание додекаэдра со сторонами единичной длины.
    pub fn dodecahedron() -> Self {
        // Золотое сечение
        let phi = (1.0 + (5.0_f32).sqrt()) / 2.0;
        let inv_phi = 1.0 / phi;

        let vertexes = vec![
            // (±1, ±1, ±1)
            HVec3::new(1.0, 1.0, 1.0),    // 0
            HVec3::new(1.0, 1.0, -1.0),   // 1
            HVec3::new(1.0, -1.0, 1.0),   // 2
            HVec3::new(1.0, -1.0, -1.0),  // 3
            HVec3::new(-1.0, 1.0, 1.0),   // 4
            HVec3::new(-1.0, 1.0, -1.0),  // 5
            HVec3::new(-1.0, -1.0, 1.0),  // 6
            HVec3::new(-1.0, -1.0, -1.0), // 7
            // (0, ±1/φ, ±φ)
            HVec3::new(0.0, inv_phi, phi),   // 8
            HVec3::new(0.0, inv_phi, -phi),  // 9
            HVec3::new(0.0, -inv_phi, phi),  // 10
            HVec3::new(0.0, -inv_phi, -phi), // 11
            // (±1/φ, ±φ, 0)
            HVec3::new(inv_phi, phi, 0.0),   // 12
            HVec3::new(inv_phi, -phi, 0.0),  // 13
            HVec3::new(-inv_phi, phi, 0.0),  // 14
            HVec3::new(-inv_phi, -phi, 0.0), // 15
            // (±φ, 0, ±1/φ)
            HVec3::new(phi, 0.0, inv_phi),   // 16
            HVec3::new(phi, 0.0, -inv_phi),  // 17
            HVec3::new(-phi, 0.0, inv_phi),  // 18
            HVec3::new(-phi, 0.0, -inv_phi), // 19
        ];

        // Нормализуем вершины
        let vertexes: Vec<HVec3> = vertexes
            .into_iter()
            .map(|p| {
                let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
                HVec3::new(p.x / len, p.y / len, p.z / len)
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

        Self::new(vertexes, polygons)
    }

    /// Получить систему координат Mesh'а.
    pub fn get_local_frame(&self) -> &CoordFrame {
        &self.local_frame
    }

    /// Получить изменяемую систему координат Mesh'а.
    pub fn get_local_frame_mut(&mut self) -> &mut CoordFrame {
        &mut self.local_frame
    }

    pub fn get_vertexes(&self) -> &Vec<HVec3> {
        &self.vertexes
    }

    pub fn get_polygons(&self) -> &Vec<Polygon3> {
        &self.polygons
    }
}

// --------------------------------------------------
// Полигон
// --------------------------------------------------

/// Представление одного полигона модели. Только для внутреннего использования.
/// Дабы избежать копирования вершин, полигоны только хранят индексы вершин из Mesh'а.
///
/// Полигоны должны быть построены строго **по часовой стрелке**, в противном случае не получится
/// построить нормали для модели.
#[derive(Debug, Clone)]
pub struct Polygon3 {
    vertexes: Vec<usize>,
}

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

    /// Добавить вершину (точку) в полигон.
    pub fn add_vertex(&mut self, index: usize) {
        self.vertexes.push(index);
    }

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

    /// Получить список индексов вершин.
    pub fn get_vertexes(&self) -> &Vec<usize> {
        &self.vertexes
    }
}
