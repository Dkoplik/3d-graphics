//! Реализация Mesh'а 3D модели.
//!
//! По сути, это является каркасом модели, которого достаточно только
//! для рендера в формате wireframe.

use crate::{CoordFrame, HVec3, Line3, Mesh, Vec3, Transform3D};

impl Mesh {
    // --------------------------------------------------
    // Вспомогательные статические методы
    // --------------------------------------------------

    /// Сгенерировать карту нормалей по имеющимся полигонам.
    pub fn generate_normals(vertexes: &Vec<HVec3>, polygons: &Vec<Polygon3>) -> Vec<Vec3> {
        #[cfg(debug_assertions)]
        Self::assert_polygons(vertexes, polygons);

        // TODO надо просто обойти все полигоны и для их вершин посчитать нормали и усреднить, если у одной вершины их несколько
        todo!("Генерация нормалей");
    }

    /// Сгенерировать текстурные координаты по имеющимся полигонам.
    pub fn generate_texture_coord(
        vertexes: &Vec<HVec3>,
        polygons: &Vec<Polygon3>,
    ) -> Vec<(f32, f32)> {
        #[cfg(debug_assertions)]
        Self::assert_polygons(vertexes, polygons);

        // TODO какую-нибудь развертку сделать, чтобы полигоны всей модели можно было уместить на UV текстуре
        todo!("Генерация координат текстуры");
    }

    // --------------------------------------------------
    // Конструкторы
    // --------------------------------------------------

    /// Создать новый Mesh из уже известных данных.
    ///
    /// Локальная система координат этого Mesh'а будет совпадать с глобальной.
    pub fn new(
        vertexes: Vec<HVec3>,
        polygons: Vec<Polygon3>,
        normals: Vec<Vec3>,
        texture_coords: Vec<(f32, f32)>,
    ) -> Self {
        #[cfg(debug_assertions)]
        {
            Self::assert_polygons(&vertexes, &polygons);
            Self::assert_normals(&vertexes, &normals);
            Self::assert_texture(&vertexes, &texture_coords);
        }

        Mesh {
            vertexes,
            polygons,
            local_frame: CoordFrame::global(),
            normals,
            texture_coords,
        }
    }

    /// Создать новый Mesh из вершин и полигонов.
    ///
    /// Нормали и координаты текстур будут сгенерированы автоматически.
    pub fn from_polygons(vertexes: Vec<HVec3>, polygons: Vec<Polygon3>) -> Self {
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

        let normals = Self::generate_normals(&vertexes, &polygons);
        let texture_coords = Self::generate_texture_coord(&vertexes, &polygons);
        Self::new(vertexes, polygons, normals, texture_coords)
    }

    /// Создать Mesh как модель вращения.
    ///
    /// `profile_points` - изначальные точки, на основе которых строится модель
    /// `axis` - ось, вокруг которой происходит вращение
    /// `parts` - количество разбиений
    pub fn create_rotation_model(profile_points: &[HVec3], axis: Line3, parts: usize) -> Self {
        let angle_step = 2.0 * std::f32::consts::PI / parts as f32;

          if parts < 3 {
            panic!("Количество разбиений должно быть не менее 3");
        }
        if profile_points.len() < 2 {
            panic!("Профиль должен содержать хотя бы 2 точки");
        }

        let angle_step = 2.0 * std::f32::consts::PI / parts as f32;
        
        // Создаем все вершины вращения
        let mut vertexes = Vec::new();
        
        // Для каждой точки профиля создаем кольцо вершин
        for profile_point in profile_points {
            let point_vec = Vec3::new(profile_point.x, profile_point.y, profile_point.z);
            
            // Вращаем точку вокруг оси
            for i in 0..=parts {
                let angle = angle_step * i as f32;
                let rotation = Transform3D::rotation_around_line(axis, angle);
                let rotated_point = rotation.apply_to_hvec(*profile_point);
                vertexes.push(rotated_point);
            }
        }

        // Создаем полигоны
        let mut polygons = Vec::new();
        let profile_count = profile_points.len();
        let vertices_per_profile = parts + 1;

        // Создаем полигоны между соседними профилями
        for profile_idx in 0..profile_count - 1 {
            for segment_idx in 0..parts {
                let current_ring_start = profile_idx * vertices_per_profile;
                let next_ring_start = (profile_idx + 1) * vertices_per_profile;
                
                let v0 = current_ring_start + segment_idx;
                let v1 = current_ring_start + (segment_idx + 1) % vertices_per_profile;
                let v2 = next_ring_start + (segment_idx + 1) % vertices_per_profile;
                let v3 = next_ring_start + segment_idx;

                // Создаем два треугольника для каждого квада
                polygons.push(Polygon3::triangle(v0, v1, v2));
                polygons.push(Polygon3::triangle(v0, v2, v3));
            }
        }

        // Создаем крышки (если нужно)
        Self::create_rotation_caps(&mut polygons, profile_count, vertices_per_profile);

        Self::from_polygons(vertexes, polygons)
    }

    /// Создает верхнюю и нижнюю крышки для модели вращения
    fn create_rotation_caps(polygons: &mut Vec<Polygon3>, profile_count: usize, vertices_per_profile: usize) {
        // Нижняя крышка (первый профиль)
        if profile_count > 1 {
            let mut bottom_cap = Vec::new();
            for i in 0..vertices_per_profile {
                bottom_cap.push(i);
            }
            if bottom_cap.len() >= 3 {
                polygons.push(Polygon3::from_list(&bottom_cap));
            }
        }

        // Верхняя крышка (последний профиль)
        if profile_count > 1 {
            let top_profile_start = (profile_count - 1) * vertices_per_profile;
            let mut top_cap = Vec::new();
            for i in 0..vertices_per_profile {
                top_cap.push(top_profile_start + i);
            }
            // Реверсируем для правильной ориентации нормали
            top_cap.reverse();
            if top_cap.len() >= 3 {
                polygons.push(Polygon3::from_list(&top_cap));
            }
        }
    }

    /// Создать Mesh как график функции от 2-х переменных
    ///
    /// `func` - функция от двух переменных `f(x, y) = z`
    /// `x_range` - границы отсечения по оси x
    /// `y_range` - границы отсечения по оси y
    /// `x_steps` - шаг по оси x
    /// `y_steps` - шаг по оси y
    pub fn from_function<F>(
        func: F,
        x_range: (f32, f32),
        y_range: (f32, f32),
        x_steps: usize,
        y_steps: usize,
    ) -> Self
    where
        F: Fn(f32, f32) -> f32,
    {
        // TODO: Надо тупо вычислять график, будут вершины, а потом как-то в полигоны объединить.
        todo!("Сделать модель по графику")
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

        Self::from_polygons(vertexes, polygons)
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

        Self::from_polygons(vertexes, polygons)
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

        Self::from_polygons(vertexes, polygons)
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

        Self::from_polygons(vertexes, polygons)
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

        Self::from_polygons(vertexes, polygons)
    }

    // --------------------------------------------------
    // setter'ы и getter'ы
    // --------------------------------------------------

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

    pub fn get_normals(&self) -> &Vec<Vec3> {
        &self.normals
    }

    pub fn get_texture_coords(&self) -> &Vec<(f32, f32)> {
        &self.texture_coords
    }

    // --------------------------------------------------
    // Вспомогательные методы
    // --------------------------------------------------

    /// Проверка полигонов на корректность.
    fn assert_polygons(vertexes: &Vec<HVec3>, polygons: &Vec<Polygon3>) {
        for polygon in polygons {
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
    }

    /// Проверка нормалей на корректность
    fn assert_normals(vertexes: &Vec<HVec3>, normals: &Vec<Vec3>) {
        assert!(
            normals.len() == vertexes.len(),
            "Количество нормалей {} должно совпадать с количеством вершин {}",
            normals.len(),
            vertexes.len()
        );

        for normal in normals {
            assert!(
                (normal.length() - 1.0).abs() < 2.0 * f32::EPSILON,
                "Нормаль длиной {} должена быть нормированной",
                normal.length()
            );
        }
    }

    /// Проверка текстурных координат на корректность
    fn assert_texture(vertexes: &Vec<HVec3>, texture_coords: &Vec<(f32, f32)>) {
        assert!(
            texture_coords.len() == vertexes.len(),
            "Количество текстурных координат {} должно совпадать с количеством вершин {}",
            texture_coords.len(),
            vertexes.len()
        );

        for (u, v) in texture_coords.clone() {
            assert!(
                (u >= 0.0) && (u <= 1.0),
                "коодрината u {} должна быть в диапазоне [0, 1]",
                u
            );
            assert!(
                (v >= 0.0) && (v <= 1.0),
                "коодрината v {} должна быть в диапазоне [0, 1]",
                v
            );
        }
    }
}

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
    // --------------------------------------------------
    // Конструкторы
    // --------------------------------------------------

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

    // --------------------------------------------------
    // Вспомогательные методы
    // --------------------------------------------------

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

    /// Получить нормаль к полигону.
    pub fn get_normal(&self) -> Vec3 {
        // TODO Тупо через векторное произведение сделать и не забыть нормализовать
        todo!("Нормаль полигона")
    }
}
