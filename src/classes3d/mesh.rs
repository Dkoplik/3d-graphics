//! Реализация Mesh'а 3D модели.
//!
//! По сути, это является каркасом модели, которого достаточно только
//! для рендера в формате wireframe.

use crate::{CoordFrame, HVec3, Line3, Mesh, Transform3D, Vec3};
use crate::{CoordFrame, HVec3, Line3, Mesh, Transform3D, Vec3};

impl Mesh {
    // --------------------------------------------------
    // Вспомогательные статические методы
    // --------------------------------------------------

    /// Сгенерировать карту нормалей по имеющимся полигонам.
    pub fn generate_normals(vertexes: &Vec<HVec3>, polygons: &Vec<Polygon3>) -> Vec<Vec3> {
        let mut normals = vec![Vec3::new(0.0, 0.0, 0.0); vertexes.len()];

        let mut normals = vec![Vec3::zero(); vertexes.len()];
        let mut face_count = vec![0; vertexes.len()];

        // Вычисляем центр меша для согласованной ориентации нормалей
        let mesh_center = Self::calculate_center(vertexes);

        // Для каждого полигона вычисляем нормаль и добавляем её к вершинам
        // получается, что нормали в вершинах вычисляются усреднением(будет ниже) нормалей смежных граней(как в презентации)
        for polygon in polygons {
            let poly_normal = polygon.get_normal(vertexes, Some(mesh_center));
            let vertex_indices = polygon.get_vertexes();

            for &vertex_index in vertex_indices {
                normals[vertex_index] = normals[vertex_index] + poly_normal;
                face_count[vertex_index] += 1;
            }
        }

        // Усредняем нормали и нормализуем
        for i in 0..normals.len() {
            if face_count[i] > 0 {
                normals[i] = normals[i] * (1.0 / face_count[i] as f32);
                normals[i] = normals[i].normalize();
            }
        }

        normals
    }

    /// Вычислить центр меша
    fn calculate_center(vertexes: &Vec<HVec3>) -> Vec3 {
        if vertexes.is_empty() {
            return Vec3::zero();
        }

        let sum: Vec3 = vertexes
            .iter()
            .map(|v| Vec3::from(*v))
            .fold(Vec3::zero(), |acc, v| acc + v);

        sum * (1.0 / vertexes.len() as f32)
    }

    /// Сгенерировать текстурные координаты по имеющимся полигонам.
    pub fn generate_texture_coord(
        vertexes: &Vec<HVec3>,
        polygons: &Vec<Polygon3>,
    ) -> Vec<(f32, f32)> {
        #[cfg(debug_assertions)]
        Self::assert_polygons(vertexes, polygons);

        // Автоматически выбираем метод развертки на основе геометрии(возможно, реализуем в будущем. Сейчас - planar)
        if Self::is_cylindrical_shape(vertexes) {
            Self::generate_texture_coord_cylindrical(vertexes, polygons)
        } else {
            Self::generate_texture_coord_planar(vertexes, polygons)
        }
    }

    /// Сгенерировать текстурные координаты с цилиндрической разверткой
    pub fn generate_texture_coord_cylindrical(
        vertexes: &Vec<HVec3>,
        polygons: &Vec<Polygon3>,
    ) -> Vec<(f32, f32)> {
        //todo
        todo!()
    }

    /// Вычислить ограничивающий параллелепипед вершин
    fn calculate_bounds(vertexes: &Vec<HVec3>) -> (Vec3, Vec3) {
        if vertexes.is_empty() {
            return (Vec3::zero(), Vec3::zero());
        }

        let first = Vec3::from(vertexes[0]);
        let mut min = first;
        let mut max = first;

        for vertex in vertexes.iter().skip(1) {
            let v = Vec3::from(*vertex);
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }

        (min, max)
    }

    /// Планарная развертка
    fn generate_texture_coord_planar(
        vertexes: &Vec<HVec3>,
        polygons: &Vec<Polygon3>,
    ) -> Vec<(f32, f32)> {
        let mut texture_coords = vec![(0.0, 0.0); vertexes.len()];
        let mut usage_count = vec![0; vertexes.len()];

        // Для каждого полигона вычисляем свою проекцию
        for polygon in polygons {
            let vertex_indices = polygon.get_vertexes();

            if vertex_indices.len() < 3 {
                continue;
            }

            // Вычисляем нормаль полигона для определения плоскости проекции
            let normal = polygon.get_normal(vertexes, None);
            let (u_axis, v_axis) = Self::get_projection_axes(normal);

            let (min_u, min_v, max_u, max_v) =
                Self::get_polygon_bounds(vertexes, vertex_indices, u_axis, v_axis);

            // Назначаем UV координаты для вершин этого полигона
            for &vertex_index in vertex_indices {
                let vertex = Vec3::from(vertexes[vertex_index]);
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

        texture_coords
    }

    /// Определяет оси проекции на основе нормали
    fn get_projection_axes(normal: Vec3) -> (Vec3, Vec3) {
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
        vertexes: &Vec<HVec3>,
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
    fn is_cylindrical_shape(vertexes: &Vec<HVec3>) -> bool {
        //TODO
        false
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
            Self::assert_normals(&normals);
            Self::assert_texture(&texture_coords);
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
    fn create_rotation_caps(
        polygons: &mut Vec<Polygon3>,
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
                    vertexes.push(HVec3::new(x, y, z));
                } else {
                    vertexes.push(HVec3::new(x, y, 0.0));
                }
            }
        }

        // Генерируем полигоны (треугольники)
        let mut polygons = Vec::new();
        for j in 0..y_steps {
            for i in 0..x_steps {
                let idx = |i: usize, j: usize| -> usize { j * (x_steps + 1) + i };

                // Первый треугольник
                polygons.push(Polygon3::triangle(
                    idx(i, j),
                    idx(i + 1, j),
                    idx(i + 1, j + 1),
                ));

                // Второй треугольник
                polygons.push(Polygon3::triangle(
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
        #[cfg(debug_assertions)]
        if self.normals.len() != self.vertexes.len() {
            eprintln!(
                "Warning: используются нормали модели, но их количество не совпадает с количеством вершин"
            );
        }

        &self.normals
    }

    pub fn get_texture_coords(&self) -> &Vec<(f32, f32)> {
        #[cfg(debug_assertions)]
        if self.texture_coords.len() != self.vertexes.len() {
            eprintln!(
                "Warning: используются текстурные координаты модели, но их количество не совпадает с количеством вершин"
            );
        }

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
    fn assert_normals(normals: &Vec<Vec3>) {
        for normal in normals {
            assert!(
                (normal.length() - 1.0).abs() < 2.0 * f32::EPSILON,
                "Нормаль вершины длиной {} должена быть нормированной",
                normal.length()
            );
        }
    }

    /// Проверка текстурных координат на корректность
    fn assert_texture(texture_coords: &Vec<(f32, f32)>) {
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

    /// Получить нормаль к полигону
    pub fn get_normal(&self, vertexes: &Vec<HVec3>, mesh_center: Option<Vec3>) -> Vec3 {
        let vertex_indices = self.get_vertexes();

        match vertex_indices.len() {
            0 | 1 | 2 => Vec3::new(0.0, 0.0, 1.0), // Недостаточно вершин
            _ => self.get_normal_polygon(vertexes, mesh_center),
        }
    }

    /// Нормаль для многоугольника
    fn get_normal_polygon(&self, vertexes: &Vec<HVec3>, mesh_center: Option<Vec3>) -> Vec3 {
        let poly_vertex_indices = self.get_vertexes();
        let mut normal = Vec3::zero();

        for i in 0..poly_vertex_indices.len() {
            let current = Vec3::from(vertexes[poly_vertex_indices[i]]);
            let next =
                Vec3::from(vertexes[poly_vertex_indices[(i + 1) % poly_vertex_indices.len()]]);

            normal.x += (current.y - next.y) * (current.z + next.z);
            normal.y += (current.z - next.z) * (current.x + next.x);
            normal.z += (current.x - next.x) * (current.y + next.y);
        }

        // Ориентируем нормаль от центра объекта, если центр предоставлен
        if let Some(center) = mesh_center {
            normal = self.orient_polygon_normal_from_center(normal, vertexes, center);
        }

        if normal.length() > 0.001 {
            normal.normalize()
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }

    /// Ориентирует нормаль многоугольника от центра
    fn orient_polygon_normal_from_center(
        &self,
        normal: Vec3,
        vertexes: &Vec<HVec3>,
        center: Vec3,
    ) -> Vec3 {
        let vertex_indices = self.get_vertexes();

        // Вычисляем центр многоугольника
        let mut face_center = Vec3::zero();
        for &index in vertex_indices {
            face_center = face_center + Vec3::from(vertexes[index]);
        }
        face_center = face_center * (1.0 / vertex_indices.len() as f32);

        // Вектор от центра объекта к центру грани
        let center_to_face = face_center - center;

        // Если нормаль направлена к центру объекта, разворачиваем её
        if normal.dot(center_to_face) < 0.0 {
            -normal
        } else {
            normal
        }
    }
}
