//! Объявление и реализация Mesh'а 3D модели.
//!
//! По сути, это является каркасом модели, которого достаточно только
//! для рендера в формате wireframe.

use crate::{CoordFrame, Line3, Point3, Transform3D, UVec3, Vec3, library::utils};

mod polygon;
// re-export в модель
pub use polygon::Polygon;

/// Mesh модели.
///
/// Mesh представляет собой набор вершин (точек в пространстве), набор полигонов, которые объединяют
/// вершины в сетку, а также набор нормалей и текстурных координат для дальнейшей отрисовки модели.
/// Все векторы и точки хранятся в локальных координатах модели.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Все вершины Mesh'а модели.
    ///
    /// Вершины хранятся как 3D точки в **локальных** координатах Mesh'а.
    vertexes: Vec<Point3>,

    /// Все полигоны Mesh'а модели.
    ///
    /// Для оптимизации хранения, полигоны задаются индексами вершин из `vertexes`, а не копиями вершин.
    /// Иными словами, полигон - это просто массив (вектор) индексов вершин модели.
    polygons: Vec<Polygon>,

    /// Локальные координаты Mesh'а в 3D пространстве.
    pub local_frame: CoordFrame,

    /// Нормали вершин. Индексируются в том же порядке, что и вершины Mesh'а.
    normals: Option<Vec<UVec3>>,

    /// Соответствие между UV-координатами текстуры и вершинами.
    texture_coords: Option<Vec<(f32, f32)>>,
}

impl Mesh {
    // --------------------------------------------------
    // Вспомогательные статические методы
    // --------------------------------------------------

    /// Сгенерировать нормали по имеющимся полигонам.
    ///
    /// Если в модели уже содержатся какие-то нормали, то они будут удалены.
    pub fn generate_normals(&mut self) {
        let mut normals = vec![Vec3::zero(); self.vertexes.len()];
        let mut face_count = vec![0; self.vertexes.len()];

        // Вычисляем центр меша для согласованной ориентации нормалей
        let mesh_center = utils::calculate_center(&self.vertexes);

        // Для каждого полигона вычисляем нормаль и добавляем её к вершинам
        // получается, что нормали в вершинах вычисляются усреднением(будет ниже) нормалей смежных граней(как в презентации)
        for polygon in &self.polygons {
            let poly_normal = polygon.plane_normal(self, Some(mesh_center));

            for vertex_index in polygon.get_mesh_vertex_index_iter() {
                normals[vertex_index] = normals[vertex_index] + poly_normal;
                face_count[vertex_index] += 1;
            }
        }

        // Усредняем нормали
        for i in 0..normals.len() {
            if face_count[i] > 0 {
                normals[i] = normals[i] * (1.0 / face_count[i] as f32);
            }
        }

        self.normals = Some(
            normals
                .iter()
                .map(|&v| v.normalize().unwrap_or(UVec3::new(0.0, 0.0, 1.0)))
                .collect(),
        );

        // sanity check
        #[cfg(debug_assertions)]
        Self::assert_normals(&self.vertexes, self.normals.as_ref().unwrap());
    }

    /// Сгенерировать текстурные координаты по имеющимся полигонам.
    ///
    /// Если в модели уже содержатся какие-то текстурные координаты, то они будут удалены.
    pub fn generate_texture_coord(&mut self) {
        // Автоматически выбираем метод развертки на основе геометрии(возможно, реализуем в будущем. Сейчас - planar)
        if self.is_cylindrical_shape() {
            self.generate_texture_coord_cylindrical();
        } else {
            self.generate_texture_coord_planar();
        }

        // sanity check
        #[cfg(debug_assertions)]
        Self::assert_texture(&self.vertexes, self.texture_coords.as_ref().unwrap());
    }

    /// Сгенерировать текстурные координаты с цилиндрической разверткой
    fn generate_texture_coord_cylindrical(&mut self) {
        //todo
        todo!()
    }

    /// Планарная развертка
    fn generate_texture_coord_planar(&mut self) {
        let mut texture_coords = vec![(0.0, 0.0); self.vertexes.len()];
        let mut usage_count = vec![0; self.vertexes.len()];

        // Для каждого полигона вычисляем свою проекцию
        for polygon in &self.polygons {
            let vertex_indices: Vec<usize> = polygon.get_mesh_vertex_index_iter().collect();

            if vertex_indices.len() < 3 {
                continue;
            }

            // Вычисляем нормаль полигона для определения плоскости проекции
            let normal = polygon.plane_normal(self, None);
            let (u_axis, v_axis) = Self::get_projection_axes(normal);

            let (min_u, min_v, max_u, max_v) =
                Self::get_polygon_bounds(&self.vertexes, &vertex_indices, u_axis, v_axis);

            // Назначаем UV координаты для вершин этого полигона
            for vertex_index in vertex_indices {
                let vertex = Vec3::from(self.vertexes[vertex_index]);
                let u = (vertex.dot(u_axis) - min_u) / (max_u - min_u);
                let v = (vertex.dot(v_axis) - min_v) / (max_v - min_v);

                // Усредняем координаты для вершин, используемых в нескольких полигонах
                if usage_count[vertex_index] == 0 {
                    texture_coords[vertex_index] = (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
                } else {
                    let (old_u, old_v) = texture_coords[vertex_index];
                    let count = usage_count[vertex_index] as f32;
                    let new_u = (old_u * count + u) / (count + 1.0);
                    let new_v = (old_v * count + v) / (count + 1.0);
                    texture_coords[vertex_index] = (new_u.clamp(0.0, 1.0), new_v.clamp(0.0, 1.0));
                }

                usage_count[vertex_index] += 1;
            }
        }

        self.texture_coords = Some(texture_coords);
    }

    /// Определяет оси проекции на основе нормали
    fn get_projection_axes(normal: UVec3) -> (Vec3, Vec3) {
        // Выбираем плоскость проекции в зависимости от доминирующей оси нормали
        let abs_normal = Vec3::new(normal.x.abs(), normal.y.abs(), normal.z.abs());

        if abs_normal.x >= abs_normal.y && abs_normal.x >= abs_normal.z {
            // Проецируем на плоскость YZ
            (Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0))
        } else if abs_normal.y >= abs_normal.x && abs_normal.y >= abs_normal.z {
            // Проецируем на плоскость XZ
            (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0))
        } else {
            // Проецируем на плоскость XY
            (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0))
        }
    }

    /// Вычисляет границы полигона в выбранной плоскости проекции
    fn get_polygon_bounds(
        vertexes: &Vec<Point3>,
        vertex_indices: &[usize],
        u_axis: Vec3,
        v_axis: Vec3,
    ) -> (f32, f32, f32, f32) {
        let mut min_u = f32::MAX;
        let mut min_v = f32::MAX;
        let mut max_u = f32::MIN;
        let mut max_v = f32::MIN;

        for &index in vertex_indices {
            let vertex = Vec3::from(vertexes[index]);
            let u = vertex.dot(u_axis);
            let v = vertex.dot(v_axis);

            min_u = min_u.min(u);
            min_v = min_v.min(v);
            max_u = max_u.max(u);
            max_v = max_v.max(v);
        }

        // Защита от деления на ноль
        if max_u - min_u < 0.001 {
            max_u = min_u + 1.0;
        }
        if max_v - min_v < 0.001 {
            max_v = min_v + 1.0;
        }

        (min_u, min_v, max_u, max_v)
    }

    /// Проверить, является ли форма цилиндрической (подходит для вращения)
    fn is_cylindrical_shape(&self) -> bool {
        //TODO
        false
    }

    // --------------------------------------------------
    // Конструкторы
    // --------------------------------------------------

    /// Создать новый Mesh из уже известных данных.
    ///
    /// Локальная система координат этого Mesh'а будет совпадать с глобальной.
    fn new(
        vertexes: Vec<Point3>,
        polygons: Vec<Polygon>,
        normals: Option<Vec<UVec3>>,
        texture_coords: Option<Vec<(f32, f32)>>,
    ) -> Self {
        #[cfg(debug_assertions)]
        {
            Self::assert_polygons(&vertexes, &polygons);
            if let Some(normals) = &normals {
                Self::assert_normals(&vertexes, normals);
            }
            if let Some(texture_coords) = &texture_coords {
                Self::assert_texture(&vertexes, texture_coords);
            }
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
    pub fn from_polygons(vertexes: Vec<Point3>, polygons: Vec<Polygon>) -> Self {
        #[cfg(debug_assertions)]
        Self::assert_polygons(&vertexes, &polygons);

        let mut mesh = Self::new(vertexes, polygons, None, None);
        mesh.generate_normals();
        mesh.generate_texture_coord();

        mesh
    }

    /// Создать Mesh как модель вращения.
    ///
    /// `profile_points` - изначальные точки, на основе которых строится модель
    /// `axis` - ось, вокруг которой происходит вращение
    /// `parts` - количество разбиений
    pub fn create_rotation_model(profile_points: &[Point3], axis: Line3, parts: usize) -> Self {
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
            // Вращаем точку вокруг оси
            for i in 0..parts {
                let angle = angle_step * i as f32;
                let rotation = Transform3D::rotation_around_line(axis, angle);
                let rotated_point = profile_point.apply_transform(rotation).unwrap();
                vertexes.push(rotated_point);
            }
        }

        // Создаем полигоны
        let mut polygons = Vec::new();
        let profile_count = profile_points.len();
        let vertices_per_profile = parts;

        // Создаем полигоны между соседними профилями
        for profile_idx in 0..profile_count - 1 {
            for segment_idx in 0..parts {
                let current_ring_start = profile_idx * vertices_per_profile;
                let next_ring_start = (profile_idx + 1) * vertices_per_profile;

                let v0 = current_ring_start + segment_idx;
                let v1 = current_ring_start + (segment_idx + 1) % vertices_per_profile;
                let v2 = next_ring_start + (segment_idx + 1) % vertices_per_profile;
                let v3 = next_ring_start + segment_idx;
                polygons.push(Polygon::from_list(&[v0, v1, v2, v3]));
            }
        }

        // Создаем крышки (если нужно)
        Self::create_rotation_caps(&mut polygons, profile_count, vertices_per_profile);

        Self::from_polygons(vertexes, polygons)
    }

    /// Создает верхнюю и нижнюю крышки для модели вращения
    fn create_rotation_caps(
        polygons: &mut Vec<Polygon>,
        profile_count: usize,
        vertices_per_profile: usize,
    ) {
        // Нижняя крышка (первый профиль)
        if profile_count > 1 {
            let mut bottom_cap = Vec::new();
            for i in 0..vertices_per_profile {
                bottom_cap.push(i);
            }
            if bottom_cap.len() >= 3 {
                polygons.push(Polygon::from_list(&bottom_cap));
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
                polygons.push(Polygon::from_list(&top_cap));
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
        let (x0, x1) = x_range;
        let (y0, y1) = y_range;

        let dx = (x1 - x0) / x_steps as f32;
        let dy = (y1 - y0) / y_steps as f32;

        let mut vertexes = Vec::new();

        // Генерируем вершины
        for j in 0..=y_steps {
            for i in 0..=x_steps {
                let x = x0 + i as f32 * dx;
                let y = y0 + j as f32 * dy;
                let z = func(x, y);

                if z.is_finite() {
                    vertexes.push(Point3::new(x, y, z));
                } else {
                    vertexes.push(Point3::new(x, y, 0.0));
                }
            }
        }

        // Генерируем полигоны (треугольники)
        let mut polygons = Vec::new();
        for j in 0..y_steps {
            for i in 0..x_steps {
                let idx = |i: usize, j: usize| -> usize { j * (x_steps + 1) + i };

                // Первый треугольник
                polygons.push(Polygon::triangle(
                    idx(i, j),
                    idx(i + 1, j),
                    idx(i + 1, j + 1),
                ));

                // Второй треугольник
                polygons.push(Polygon::triangle(
                    idx(i, j),
                    idx(i + 1, j + 1),
                    idx(i, j + 1),
                ));
            }
        }

        Self::from_polygons(vertexes, polygons)
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
            Polygon::triangle(0, 1, 2),
            Polygon::triangle(0, 2, 3),
            Polygon::triangle(0, 3, 1),
            Polygon::triangle(1, 3, 2),
        ];

        Self::from_polygons(vertexes, polygons)
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
            Polygon::from_list(&[0, 1, 2, 3]),
            Polygon::from_list(&[4, 5, 6, 7]),
            Polygon::from_list(&[3, 2, 6, 7]),
            Polygon::from_list(&[0, 1, 5, 4]),
            Polygon::from_list(&[0, 3, 7, 4]),
            Polygon::from_list(&[1, 2, 6, 5]),
        ];

        Self::from_polygons(vertexes, polygons)
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
            Polygon::triangle(0, 2, 3), // верх-право-перед
            Polygon::triangle(0, 3, 4), // верх-перед-лево
            Polygon::triangle(0, 4, 5), // верх-лево-зад
            Polygon::triangle(0, 5, 2), // верх-зад-право
            // Нижние треугольники
            Polygon::triangle(1, 3, 2), // низ-перед-право
            Polygon::triangle(1, 4, 3), // низ-лево-перед
            Polygon::triangle(1, 5, 4), // низ-зад-лево
            Polygon::triangle(1, 2, 5), // низ-право-зад
        ];

        Self::from_polygons(vertexes, polygons)
    }

    /// Создание икосаэдра со сторонами единичной длины.
    pub fn icosahedron() -> Self {
        // Золотое сечение
        let phi = (1.0 + (5.0_f32).sqrt()) / 2.0;

        let vertexes = vec![
            // Верхние и нижние вершины
            Point3::new(0.0, 1.0, phi),   // 0: зад-верх
            Point3::new(0.0, 1.0, -phi),  // 1: перед-верх
            Point3::new(0.0, -1.0, phi),  // 2: зад-низ
            Point3::new(0.0, -1.0, -phi), // 3: перед-низ
            // Боковые вершины - передние
            Point3::new(1.0, phi, 0.0),  // 4: верх-право
            Point3::new(-1.0, phi, 0.0), // 5: верх-лево
            // Боковые вершины - задние
            Point3::new(1.0, -phi, 0.0),  // 6: низ-право
            Point3::new(-1.0, -phi, 0.0), // 7: низ-лево
            // Передние и задние вершины
            Point3::new(phi, 0.0, 1.0),   // 8: право-зад
            Point3::new(phi, 0.0, -1.0),  // 9: право-перед
            Point3::new(-phi, 0.0, 1.0),  // 10: лево-зад
            Point3::new(-phi, 0.0, -1.0), // 11: лево-перед
        ];

        // Нормализуем вершины
        let vertexes: Vec<Point3> = vertexes
            .into_iter()
            .map(|p| {
                let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
                Point3::new(p.x / len, p.y / len, p.z / len)
            })
            .collect();

        let polygons = vec![
            Polygon::triangle(0, 4, 8),
            Polygon::triangle(0, 8, 2),
            Polygon::triangle(0, 2, 10),
            Polygon::triangle(0, 10, 5),
            Polygon::triangle(0, 5, 4),
            Polygon::triangle(1, 9, 4),
            Polygon::triangle(1, 4, 5),
            Polygon::triangle(1, 5, 11),
            Polygon::triangle(1, 11, 3),
            Polygon::triangle(1, 3, 9),
            Polygon::triangle(4, 9, 8),
            Polygon::triangle(8, 9, 6),
            Polygon::triangle(6, 9, 3),
            Polygon::triangle(11, 5, 10),
            Polygon::triangle(10, 5, 0),
            Polygon::triangle(7, 6, 2),
            Polygon::triangle(7, 6, 3),
            Polygon::triangle(7, 2, 10),
            Polygon::triangle(7, 10, 11),
            Polygon::triangle(7, 11, 3),
        ];

        Self::from_polygons(vertexes, polygons)
    }

    /// Создание додекаэдра со сторонами единичной длины.
    pub fn dodecahedron() -> Self {
        let phi = (1.0 + (5.0_f32).sqrt()) / 2.0;
        let inv_phi = 1.0 / phi;

        let mut vertexes = vec![
            // (±1, ±1, ±1)
            Point3::new(1.0, 1.0, 1.0),    // 0
            Point3::new(1.0, 1.0, -1.0),   // 1
            Point3::new(1.0, -1.0, 1.0),   // 2
            Point3::new(1.0, -1.0, -1.0),  // 3
            Point3::new(-1.0, 1.0, 1.0),   // 4
            Point3::new(-1.0, 1.0, -1.0),  // 5
            Point3::new(-1.0, -1.0, 1.0),  // 6
            Point3::new(-1.0, -1.0, -1.0), // 7
            // (0, ±φ, ±1/φ)
            Point3::new(0.0, phi, inv_phi),   // 8
            Point3::new(0.0, phi, -inv_phi),  // 9
            Point3::new(0.0, -phi, inv_phi),  // 10
            Point3::new(0.0, -phi, -inv_phi), // 11
            // (±1/φ, 0, ±φ)
            Point3::new(inv_phi, 0.0, phi),   // 12
            Point3::new(-inv_phi, 0.0, phi),  // 13
            Point3::new(inv_phi, 0.0, -phi),  // 14
            Point3::new(-inv_phi, 0.0, -phi), // 15
            // (±φ, ±1/φ, 0)
            Point3::new(phi, inv_phi, 0.0),   // 16
            Point3::new(-phi, inv_phi, 0.0),  // 17
            Point3::new(phi, -inv_phi, 0.0),  // 18
            Point3::new(-phi, -inv_phi, 0.0), // 19
        ];

        vertexes = vertexes
            .into_iter()
            .map(|p| {
                let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
                Point3::new(p.x / len, p.y / len, p.z / len)
            })
            .collect();

        let polygons = vec![
            Polygon::from_list(&[0, 12, 2, 18, 16]),
            Polygon::from_list(&[0, 16, 1, 9, 8]),
            Polygon::from_list(&[0, 8, 4, 13, 12]),
            Polygon::from_list(&[10, 2, 12, 13, 6]),
            Polygon::from_list(&[10, 6, 19, 7, 11]),
            Polygon::from_list(&[10, 11, 3, 18, 2]),
            Polygon::from_list(&[5, 17, 4, 8, 9]),
            Polygon::from_list(&[5, 9, 1, 14, 15]),
            Polygon::from_list(&[5, 15, 7, 19, 17]),
            Polygon::from_list(&[3, 14, 1, 16, 18]),
            Polygon::from_list(&[3, 11, 7, 15, 14]),
            Polygon::from_list(&[4, 17, 19, 6, 13]),
        ];

        Self::from_polygons(vertexes, polygons)
    }

    // --------------------------------------------------
    // доступ к элементам модели
    // --------------------------------------------------

    /// Получить количество вершин в модели.
    pub fn vertex_count(&self) -> usize {
        self.vertexes.len()
    }

    /// Получить количество полигонов в модели.
    pub fn polygon_count(&self) -> usize {
        self.polygons.len()
    }

    /// Получить i-ую вершину модели в **локальных** координатах.
    pub fn get_local_vertex(&self, i: usize) -> Point3 {
        self.vertexes[i]
    }

    /// Получить i-ую вершину модели в **глобальных** координатах.
    pub fn get_global_vertex(&self, i: usize) -> Point3 {
        self.vertexes[i]
            .apply_transform(self.local_frame.local_to_global_matrix())
            .unwrap()
    }

    /// Получить i-ый полигон модели.
    pub fn get_polygon(&self, i: usize) -> &Polygon {
        &self.polygons[i]
    }

    /// Получить нормаль i-ой вершины модели в **локальных** координатах.
    pub fn get_local_normal(&self, i: usize) -> Option<UVec3> {
        let normals = self.normals.as_ref()?;
        normals.get(i).copied()
    }

    /// Получить нормаль i-ой вершины модели в **глобальных** координатах.
    pub fn get_global_normal(&self, i: usize) -> Option<UVec3> {
        // нормали ведут себя по-другому и умножаются на инвертированную матрицу.
        // так как нормаль вектор - то смещение применено не будет, тут всё ок
        let transform = self.local_frame.local_to_global_matrix();
        // .inverse()
        // .expect("Ожидалось наличие обратной матрицы");
        let local_normal = self.get_local_normal(i)?;
        Some(local_normal.apply_transform(transform).unwrap())
    }

    /// Получить текстурные координаты i-ой вершины модели.
    pub fn get_texture_coord(&self, i: usize) -> Option<(f32, f32)> {
        let texture_coords = self.texture_coords.as_ref()?;
        texture_coords.get(i).copied()
    }

    /// Получить итератор по всем вершинам модели в **локальных** координатах.
    pub fn get_local_vertex_iter(&self) -> impl Iterator<Item = Point3> {
        self.vertexes.iter().copied()
    }

    /// Получить итератор по всем вершинам модели в **глобальных** координатах.
    pub fn get_global_vertex_iter(&self) -> impl Iterator<Item = Point3> {
        let transform = self.local_frame.local_to_global_matrix();
        self.vertexes
            .iter()
            .map(move |&p| p.apply_transform(transform).unwrap())
    }

    /// Получить итератор по всем полигонам модели.
    pub fn get_polygon_iter(&self) -> impl Iterator<Item = &Polygon> {
        self.polygons.iter()
    }

    /// Получить итератор по всем нормалям модели в **локальных** координатах.
    ///
    /// Нормали идут в порядке соответствующих им вершин
    pub fn get_local_normals_iter(&self) -> Option<impl Iterator<Item = UVec3>> {
        let normals = self.normals.as_ref()?;
        Some(normals.iter().copied())
    }

    /// Получить итератор по всем нормалям модели в **глобальных** координатах.
    ///
    /// Нормали идут в порядке соответствующих им вершин
    pub fn get_global_normals_iter(&self) -> Option<impl Iterator<Item = UVec3>> {
        // нормали ведут себя по-другому и умножаются на инвертированную матрицу.
        // так как нормаль вектор - то смещение применено не будет, тут всё ок
        let transform = self.local_frame.local_to_global_matrix();
        // .inverse()
        // .expect("Ожидалось наличие обратной матрицы");
        Some(
            self.get_local_normals_iter()?
                .map(move |n| (n.apply_transform(transform).unwrap())),
        )
    }

    /// Получить итератор по всем текстурным координатам модели.
    ///
    /// Текстурные координаты идут в порядке соответсвующих им вершин.
    pub fn get_texture_coord_iter(&self) -> Option<impl Iterator<Item = (f32, f32)>> {
        let texture_coords = self.texture_coords.as_ref()?;
        Some(texture_coords.iter().copied())
    }

    // --------------------------------------------------
    // Вспомогательные методы
    // --------------------------------------------------

    /// Содержит ли модель нормали?
    pub fn has_normals(&self) -> bool {
        self.normals.is_some()
    }

    /// Содержит ли модель текстурные координаты?
    pub fn has_texture_coords(&self) -> bool {
        self.texture_coords.is_some()
    }

    /// Проверка полигонов на корректность.
    fn assert_polygons(vertexes: &Vec<Point3>, polygons: &Vec<Polygon>) {
        for polygon in polygons {
            for index in polygon.get_mesh_vertex_index_iter() {
                if index >= vertexes.len() {
                    panic!("Полигон содержит индекс несуществующей вершины");
                }
            }
        }
    }

    /// Проверка нормалей на корректность.
    fn assert_normals(vertexes: &Vec<Point3>, normals: &Vec<UVec3>) {
        assert_eq!(
            vertexes.len(),
            normals.len(),
            "Количество нормалей должно совпадать с количеством вершин Mesh'а"
        );
    }

    /// Проверка текстурных координат на корректность
    fn assert_texture(vertexes: &Vec<Point3>, texture_coords: &Vec<(f32, f32)>) {
        assert_eq!(
            vertexes.len(),
            texture_coords.len(),
            "Количество текстурных координат должно совпадать с количесвтом вершин Mesh'а"
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

#[cfg(test)]
mod mesh_tests {
    use crate::HVec3;

    use super::*;

    const TOLERANCE: f32 = 1e-6;

    fn assert_vecs(got: Vec3, expected: Vec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn assert_uvecs(got: UVec3, expected: UVec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался unit-вектор {:?}, но получен unit-вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn assert_hvecs(got: HVec3, expected: HVec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn assert_points(got: Point3, expected: Point3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидалась точка {:?}, но получена точка {:?}, одна из координат которой отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn generate_cube() -> Mesh {
        let vertexes = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(0.0, 1.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
        ];
        let polygons = vec![
            Polygon::from_list(&vec![0, 1, 2, 3]),
            Polygon::from_list(&vec![0, 1, 4, 5]),
            Polygon::from_list(&vec![4, 5, 6, 7]),
            Polygon::from_list(&vec![6, 7, 2, 3]),
            Polygon::from_list(&vec![1, 3, 5, 7]),
            Polygon::from_list(&vec![0, 2, 4, 6]),
        ];
        Mesh::from_polygons(vertexes, polygons)
    }

    #[test]
    fn test_vertex_global_to_global() {
        let cube = generate_cube();

        let global_vertexes: Vec<Point3> = cube.get_global_vertex_iter().collect();
        let expected_vertexes = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(0.0, 1.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
        ];

        for i in 0..global_vertexes.len() {
            assert_points(global_vertexes[i], expected_vertexes[i], TOLERANCE);
        }
    }

    #[test]
    fn test_vertex_local_translated_to_global() {
        let mut cube = generate_cube();
        cube.local_frame.origin.y += 5.0;

        let global_vertexes: Vec<Point3> = cube.get_global_vertex_iter().collect();
        let expected_vertexes = vec![
            Point3::new(0.0, 5.0, 0.0),
            Point3::new(1.0, 5.0, 0.0),
            Point3::new(0.0, 6.0, 0.0),
            Point3::new(1.0, 6.0, 0.0),
            Point3::new(0.0, 5.0, 1.0),
            Point3::new(1.0, 5.0, 1.0),
            Point3::new(0.0, 6.0, 1.0),
            Point3::new(1.0, 6.0, 1.0),
        ];

        for i in 0..global_vertexes.len() {
            assert_points(global_vertexes[i], expected_vertexes[i], TOLERANCE);
        }
    }

    #[test]
    fn test_vertex_local_rotated_to_global() {
        let mut cube = generate_cube();
        cube.local_frame.rotate(Transform3D::rotation_aligning(
            UVec3::forward(),
            UVec3::up(),
        ));

        let global_vertexes: Vec<Point3> = cube.get_global_vertex_iter().collect();
        let expected_vertexes = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, -1.0),
            Point3::new(1.0, 0.0, -1.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, -1.0),
            Point3::new(1.0, 1.0, -1.0),
        ];

        for i in 0..global_vertexes.len() {
            assert_points(global_vertexes[i], expected_vertexes[i], TOLERANCE);
        }
    }

    #[test]
    fn test_generated_normals() {
        let cube = generate_cube();

        let local_normals: Vec<UVec3> = cube.get_local_normals_iter().unwrap().collect();

        // у куба не могут быть все нормали быть одинаковыми
        let mut are_same = local_normals[0] == local_normals[1];
        for i in 1..(local_normals.len() - 1) {
            if !are_same {
                break;
            }
            are_same = local_normals[i] == local_normals[i + 1];
        }
        assert!(!are_same, "у куба нормали не могут быть одинаковыми");
    }

    #[test]
    fn test_normals_local_translated() {
        let mut cube = generate_cube();
        cube.local_frame.origin.y += 5.0;

        let local_normals: Vec<UVec3> = cube.get_local_normals_iter().unwrap().collect();
        let global_normals: Vec<UVec3> = cube.get_global_normals_iter().unwrap().collect();

        // нормали не должны были поменяться при смещении фигуры.
        for i in 0..global_normals.len() {
            assert_uvecs(global_normals[i], local_normals[i], TOLERANCE);
        }
    }

    #[test]
    fn test_normals_local_rotated() {
        let mut cube = generate_cube();
        cube.local_frame.rotate(Transform3D::rotation_aligning(
            UVec3::forward(),
            UVec3::up(),
        ));

        let global_normals: Vec<UVec3> = cube.get_global_normals_iter().unwrap().collect();

        // проверяем, что усреднённые нормали всё ещё перпендикулярны полигонам
        for polygon in cube.get_polygon_iter() {
            let mut normal = Vec3::zero();
            for index in polygon.get_mesh_vertex_index_iter() {
                normal += global_normals[index];
            }
            let normal = (normal / polygon.vertex_count() as f32)
                .normalize()
                .unwrap();
            let v0 = polygon.get_global_vertex(&cube, 0);
            let v1 = polygon.get_global_vertex(&cube, 1);
            let edge = (v1 - v0).normalize().unwrap();
            assert!(
                edge.dot(normal).abs() < TOLERANCE,
                "полученный усреднённый вектор должен быть перпендикулярен полигону, но их dot произведение ={}",
                edge.dot(normal)
            );
        }
    }
}
