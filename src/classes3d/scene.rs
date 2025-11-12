use crate::{
    Camera3, Canvas, HVec3, LightSource, Model3, Point3, Scene, Transform3D, Vec3, classes3d::mesh,
};
use egui::{Color32, Pos2};

/// Тип рендера (как отображать объекты?)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum RenderType {
    /// Отображение только каркасов (Mesh-ей) моделей.
    #[default]
    WireFrame,
    /// Полноценное отображение модели с материалом и текстурой.
    Solid,
}

/// Тип проекции на камеру.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType {
    /// Параллельная (ортографическая) проекция.
    Parallel,
    /// Перспективная проекция.
    #[default]
    Perspective,
    // /// Аксонометрическая проекция.
    // Axonimetrix,
}

/// Тип шейдинга.
///
/// Меняет отображение материала в зависимости от освещения.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum ShadingType {
    /// Отсутствие шейдинна
    #[default]
    None,
    /// Шейдинг Гуро для модели Ламберта
    Gouraud,
    /// Шейдинг Фонга для модели туншейдинг
    Phong,
}

/// Полные параметры рендера сцены.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct RenderOptions {
    pub render_type: RenderType,
    pub projection_type: ProjectionType,
    pub shading_type: ShadingType,
    pub backface_culling: bool,
    pub z_buffer_enabled: bool,
}

impl Scene {
    // --------------------------------------------------
    // Операции над сценой
    // --------------------------------------------------

    /// Создать пустую сцену.
    pub fn new(camera: Camera3) -> Self {
        Self {
            models: Vec::new(),
            camera,
            lights: Vec::new(),
            ambient_light: Color32::BLACK,
        }
    }

    /// Добавить новую модель на сцену.
    pub fn add_model(&mut self, model: Model3) {
        self.models.push(model);
    }

    /// Добавить новый источник света на сцену.
    pub fn add_light(&mut self, light: LightSource) {
        self.lights.push(light);
    }

    /// Изменить цвет глобального освещения.
    pub fn set_ambient_light(&mut self, color: Color32) {
        self.ambient_light = color;
    }

    /// Нарисовать сцену на холст со всеми нужными преобразованиями.
    ///
    /// Возвращает количество отрисованных вершин и полигонов.
    pub fn render(
        &self,
        canvas: &mut Canvas,
        render_options: RenderOptions,
        show_custom_axis: bool,
        axis_point1: Point3,
        axis_point2: Point3,
    ) -> (usize, usize) {
        // Стереть прошлый кадр.
        canvas.clear(Color32::GRAY);

        // Матрица преобразования из глобальных координат в экранные
        let global_to_screen_transform =
            self.get_view_projection_transform(render_options.projection_type, canvas);

        // Отрисовка глобальной координатной системы.
        self.draw_coordinate_axes(canvas, &global_to_screen_transform);

        // Отрисовка пользовательской оси вращения
        if show_custom_axis {
            self.draw_custom_axis_line(
                canvas,
                &global_to_screen_transform,
                axis_point1,
                axis_point2,
            );
        }

        // количество отрисованных вершин
        let mut vertex_count = 0;
        // количество отрисованных полигонов.
        let mut polygon_count = 0;

        // Отрисовка каждой модели
        for model in &self.models {
            // Проекция вершин модели
            let projected_vertexes = self.transform_model(global_to_screen_transform, model);

            // Полигоны к отрисовке
            let polygons = if render_options.backface_culling {
                self.model_backface_culling(model)
            } else {
                model.mesh.polygons.clone()
            };
            let polygons = self.skip_out_of_camera_polygons(model, polygons);

            let (model_vertexes, model_polygons) = match render_options.render_type {
                RenderType::WireFrame => {
                    self.render_model_wireframe(&projected_vertexes, &polygons, canvas)
                }
                RenderType::Solid => {
                    let (vertex_cnt, polygon_cnt) = self.render_solid(
                        &projected_vertexes,
                        &polygons,
                        model,
                        canvas,
                        render_options.z_buffer_enabled,
                    );
                    match render_options.shading_type {
                        ShadingType::None => (),
                        ShadingType::Gouraud => self.render_gouraud_lambert(
                            &projected_vertexes,
                            &polygons,
                            model,
                            canvas,
                            render_options.z_buffer_enabled,
                        ),
                        ShadingType::Phong => self.render_phong(
                            &projected_vertexes,
                            &polygons,
                            model,
                            canvas,
                            render_options.z_buffer_enabled,
                        ),
                    }
                    (vertex_cnt, polygon_cnt)
                }
            };
            vertex_count += model_vertexes;
            polygon_count += model_polygons;
        }

        (vertex_count, polygon_count)
    }

    // --------------------------------------------------
    // Вспомогательные методы для рендера
    // --------------------------------------------------

    /// Получить матрицу преобразования из глобальных координат в экранные (viewport, он же canvas)
    ///
    /// То есть, матрица производит следующие операции:
    /// глобальные координаты -> координаты камеры (view tranform) -> проекция на камеру в NDC -> растяжение NDC на размер canvas.
    fn get_view_projection_transform(
        &self,
        projection_type: ProjectionType,
        canvas: &Canvas,
    ) -> Transform3D {
        // Матрица проекции координат камеры в NDC
        let proj_matrix = match projection_type {
            ProjectionType::Parallel => Transform3D::parallel_symmetric(
                canvas.width as f32,
                canvas.height as f32,
                self.camera.get_near_plane(),
                self.camera.get_far_plane(),
            ),
            ProjectionType::Perspective => Transform3D::perspective(
                self.camera.get_fov(),
                self.camera.get_aspect_ratio(),
                self.camera.get_near_plane(),
                self.camera.get_far_plane(),
            ),
        };

        let scale_x = canvas.width as f32 / 2.0; // растянуть NDC по ширине
        let scale_y = canvas.height as f32 / 2.0; // растянуть NDC по высоте

        self.camera
            .get_local_frame()
            .global_to_local_matrix() // view transformation (локальные координаты камеры)
            .multiply(proj_matrix) // вот тут получается NDC с координатами [-1, +1]
            .multiply(Transform3D::translation_uniform(1.0)) // теперь координаты [0, +2]
            .multiply(Transform3D::scale(scale_x, scale_y, 1.0)) // теперь экранные
    }

    /// Отрисовка пользовательской оси для вращения
    fn draw_custom_axis_line(
        &self,
        canvas: &mut Canvas,
        view_proj_matrix: &Transform3D,
        point1: Point3,
        point2: Point3,
    ) {
        // Проецируем точки в 2D используя нашу систему проекций
        let screen_point1 = self.project_point(point1, view_proj_matrix);
        let screen_point2 = self.project_point(point2, view_proj_matrix);

        // Вычисляем направление линии
        let direction = (screen_point2 - screen_point1).normalized();

        // Удлиняем линию для лучшей видимости
        let extension_length = 500.0;
        let extended_start = screen_point1 - direction * extension_length;
        let extended_end = screen_point2 + direction * extension_length;

        let orange = Color32::from_rgb(255, 165, 0);
        canvas.draw_sharp_line(extended_start, extended_end, orange);

        canvas.circle_filled(screen_point1, 4.0, Color32::GREEN);
        canvas.circle_filled(screen_point2, 4.0, Color32::BLUE);
    }

    /// Отрисовка глобальной координатной системы.
    fn draw_coordinate_axes(&self, canvas: &mut Canvas, view_proj_matrix: &Transform3D) {
        // TODO нелпохо бы сделать полноценную отрисовку координатной сетки.
        let axis_length = 2.0; // Длина осей
        let origin = Point3::new(0.0, 0.0, 0.0);

        let x_axis_end = Point3::new(axis_length, 0.0, 0.0);
        let y_axis_end = Point3::new(0.0, axis_length, 0.0);
        let z_axis_end = Point3::new(0.0, 0.0, axis_length);

        let origin_2d = self.project_point(origin, view_proj_matrix);
        let x_end_2d = self.project_point(x_axis_end, view_proj_matrix);
        let y_end_2d = self.project_point(y_axis_end, view_proj_matrix);
        let z_end_2d = self.project_point(z_axis_end, view_proj_matrix);

        // Рисуем оси с разными цветами
        // Ось X - красная
        canvas.draw_sharp_line(origin_2d, x_end_2d, Color32::RED);

        // Ось Y - зелёная
        canvas.draw_sharp_line(origin_2d, y_end_2d, Color32::GREEN);

        // Ось Z - синяя
        canvas.draw_sharp_line(origin_2d, z_end_2d, Color32::BLUE);
    }

    /// Преобразует глобальные координаты точки в координаты экрана.
    fn project_point(&self, point: Point3, view_proj_matrix: &Transform3D) -> Pos2 {
        let proj_point: Point3 = view_proj_matrix.apply_to_hvec(point.into()).into();
        Pos2::new(proj_point.x, proj_point.y)
    }

    /// Проецирует вершины модели на экран.
    /// `view_proj_matrix` - матрица проекции из глобальных координат в экранные
    /// `model` - сама модель
    ///
    /// Важно: после этих преобразований вершины становится `Vec3`, то есть все преобразования в
    /// 4D векторах закончены и теперь 2D пространство представлено 3D векторами, где z-компонента
    /// нужна для порядка отрисовки на экране.
    fn transform_model(&self, view_proj_matrix: Transform3D, model: &Model3) -> Vec<Vec3> {
        // Преобразование из координат модели в проекцию NDC.
        let full_transform = model
            .mesh
            .get_local_frame()
            .local_to_global_matrix()
            .multiply(view_proj_matrix);

        // Проекция точек модели на экран.
        model
            .mesh
            .get_vertexes()
            .iter()
            .map(|vertex| Vec3::from(vertex.apply_transform(&full_transform)))
            .collect()
    }

    /// Удаляет из вектора полигонов те полигоны, которые полностью (со всеми вершинами)
    /// находятся вне камеры.
    fn skip_out_of_camera_polygons(
        &self,
        model: &Model3,
        mut polygons: Vec<mesh::Polygon3>,
    ) -> Vec<mesh::Polygon3> {
        let vertexes = model.mesh.get_vertexes();
        polygons.retain(|polygon| {
            for &index in polygon.get_vertexes() {
                let vertex = vertexes[index];
                if (vertex.x < -1.0 || vertex.x > 1.0)
                    && (vertex.y < -1.0 || vertex.y > 1.0)
                    && (vertex.z < -1.0 || vertex.z > 1.0)
                {
                    return false;
                }
            }
            true
        });
        polygons
    }

    /// Отсечение нелицевых граней модели
    /// `model` - сама модель.
    ///
    /// Возвращает вектор полигонов только с лицевыми гранями.
    fn model_backface_culling(&self, model: &Model3) -> Vec<mesh::Polygon3> {
        let vertexes = model.mesh.get_vertexes();
        let polygons = model.mesh.get_polygons();
        let mut visible_polygons = Vec::new();

        let mesh_center = Self::calculate_mesh_center(vertexes);
        let camera_pos = self.camera.get_position();

        let view_direction = (mesh_center - Vec3::from(camera_pos)).normalize();

        for polygon in polygons {
            let vertex_indices = polygon.get_vertexes();
            if vertex_indices.len() < 3 {
                continue;
            }

            let polygon_normal = polygon.get_normal(vertexes, Some(mesh_center));
            let dot_product = polygon_normal.dot(view_direction);

            if dot_product > 0.0 {
                visible_polygons.push(polygon.clone());
            }
        }

        visible_polygons
    }

    fn calculate_mesh_center(vertexes: &Vec<HVec3>) -> Vec3 {
        if vertexes.is_empty() {
            return Vec3::zero();
        }

        let sum: Vec3 = vertexes
            .iter()
            .map(|v| Vec3::from(*v))
            .fold(Vec3::zero(), |acc, v| acc + v);

        sum * (1.0 / vertexes.len() as f32)
    }

    /// Реднер каркаса модели.
    fn render_model_wireframe(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<mesh::Polygon3>,
        canvas: &mut Canvas,
    ) -> (usize, usize) {
        let mut model_vertexes = 0;
        let model_polygons = polygons.len();
        // Рисуем рёбра
        for polygon in polygons {
            // Вершины полигона
            model_vertexes += polygon.get_vertexes().len();
            let points: Vec<Vec3> = polygon
                .get_vertexes()
                .iter()
                .map(|&index| projected_vertexes[index])
                .collect();

            for i in 0..points.len() {
                let start = points[i];
                let end = points[(i + 1) % points.len()];

                let start_pos = Pos2::new(start.x, start.y);
                let end_pos = Pos2::new(end.x, end.y);
                canvas.draw_sharp_line(start_pos, end_pos, Color32::WHITE);
            }
        }

        // Рисуем вершины
        for &vertex in projected_vertexes {
            let pos = Pos2::new(vertex.x, vertex.y);
            canvas.circle_filled(pos, 3.0, Color32::WHITE);
        }

        (model_vertexes, model_polygons)
    }

    /// Рендер цельного объекта, с гранями вместо границ и с учётом материала и текстуры.
    ///
    /// Этот этап рендера не учитывает освещение, поэтому без шейдинга.
    fn render_solid(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<mesh::Polygon3>,
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) -> (usize, usize) {
        let mut model_vertexes = 0;
        let model_polygons = polygons.len();
        for polygon in polygons {
            let vertex_indices = polygon.get_vertexes();
            model_vertexes += vertex_indices.len();

            // Вершины полигона
            let poly_vertices: Vec<Vec3> = vertex_indices
                .iter()
                .map(|&idx| projected_vertexes[idx])
                .collect();

            // Текстурные координаты полигона
            let texture_coords: Vec<(f32, f32)> = vertex_indices
                .iter()
                .map(|&idx| model.mesh.get_texture_coords()[idx])
                .collect();

            // TODO это костыль, надо сделать потом нормальное наложение текстуры без освещения
            let base_color = if let Some((u, v)) = texture_coords.first() {
                model.material.get_uv_color(*u, *v)
            } else {
                model.material.color
            };

            // Заполнение треугольников
            if vertex_indices.len() == 3 {
                self.fill_triangle(&poly_vertices, base_color, canvas, z_buffer_enabled);
            } else {
                self.triangulate_and_fill_polygon(
                    &poly_vertices,
                    base_color,
                    canvas,
                    z_buffer_enabled,
                );
            }
        }

        (model_vertexes, model_polygons)
    }

    /// Шейдинг Гуро для модели Ламберта.
    ///
    /// Применяется после отрисовки моделей без шейдинга. Иными словами, на холсте canvas уже нарисованы
    /// все модели, но без учёта освещения, поэтому теперь поверх этих цветов надо наложить сам шейдинг.
    fn render_gouraud_lambert(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<mesh::Polygon3>,
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        // освещённость вершин
        let vertex_intensities = self.calculate_vertex_lighting(model);

        for polygon in polygons {
            let vertex_indices = polygon.get_vertexes();

            if vertex_indices.len() < 3 {
                continue;
            }

            // вершины полигона
            let poly_vertices: Vec<Vec3> = vertex_indices
                .iter()
                .map(|&idx| projected_vertexes[idx])
                .collect();

            let poly_intensities: Vec<f32> = vertex_indices
                .iter()
                .map(|&idx| vertex_intensities[idx])
                .collect();

            // текстурные координаты
            let texture_coords: Vec<(f32, f32)> = vertex_indices
                .iter()
                .map(|&idx| model.mesh.get_texture_coords()[idx])
                .collect();

            // отрисовка треугольников
            if vertex_indices.len() == 3 {
                self.fill_triangle_gouraud(
                    &poly_vertices,
                    &poly_intensities,
                    &texture_coords,
                    model,
                    canvas,
                    z_buffer_enabled,
                );
            } else {
                self.triangulate_and_fill_gouraud(
                    &poly_vertices,
                    &poly_intensities,
                    &texture_coords,
                    model,
                    canvas,
                    z_buffer_enabled,
                );
            }
        }
    }

    /// Шейдинг Фонга для модели туншейдинг.
    ///
    /// Применяется после отрисовки моделей без шейдинга. Иными словами, на холсте canvas уже нарисованы
    /// все модели, но без учёта освещения, поэтому теперь поверх этих цветов надо наложить сам шейдинг.
    fn render_phong(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<mesh::Polygon3>,
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        // нормали вершин и их позиции
        let vertex_normals = model.mesh.get_normals();
        let vertex_positions: Vec<Vec3> = model
            .mesh
            .get_vertexes()
            .iter()
            .map(|&v| Vec3::from(v))
            .collect();

        for polygon in polygons {
            let vertex_indices = polygon.get_vertexes();

            if vertex_indices.len() < 3 {
                continue;
            }

            // вершины полигона
            let poly_vertices: Vec<Vec3> = vertex_indices
                .iter()
                .map(|&idx| projected_vertexes[idx])
                .collect();

            let poly_normals: Vec<Vec3> = vertex_indices
                .iter()
                .map(|&idx| vertex_normals[idx])
                .collect();

            let poly_positions: Vec<Vec3> = vertex_indices
                .iter()
                .map(|&idx| vertex_positions[idx])
                .collect();

            let texture_coords: Vec<(f32, f32)> = vertex_indices
                .iter()
                .map(|&idx| model.mesh.get_texture_coords()[idx])
                .collect();

            if vertex_indices.len() == 3 {
                self.fill_triangle_phong(
                    &poly_vertices,
                    &poly_normals,
                    &poly_positions,
                    &texture_coords,
                    model,
                    canvas,
                    z_buffer_enabled,
                );
            } else {
                self.triangulate_and_fill_phong(
                    &poly_vertices,
                    &poly_normals,
                    &poly_positions,
                    &texture_coords,
                    model,
                    canvas,
                    z_buffer_enabled,
                );
            }
        }
    }

    fn calculate_vertex_lighting(&self, model: &Model3) -> Vec<f32> {
        let vertex_normals = model.mesh.get_normals();
        let vertex_positions: Vec<Vec3> = model
            .mesh
            .get_vertexes()
            .iter()
            .map(|&v| Vec3::from(v))
            .collect();

        let mut intensities = Vec::with_capacity(vertex_normals.len());

        for i in 0..vertex_normals.len() {
            let normal = vertex_normals[i];
            let position = vertex_positions[i];

            // глобальный свет
            let mut total_intensity = 0.3; // TODO костыль?

            // Влияние каждого источника
            for light in &self.lights {
                let light_dir = (light.position - Point3::from(position)).normalize();
                let distance = (light.position - Point3::from(position)).length();

                let diffuse = normal.dot(light_dir).max(0.0);
                let attenuation = 1.0 / (1.0 + distance * 0.1);

                total_intensity += diffuse * attenuation * light.intensity;
            }

            intensities.push(total_intensity.min(1.0).max(0.0));
        }

        intensities
    }

    fn fill_triangle(
        &self,
        vertices: &[Vec3],
        color: Color32,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        let [v0, v1, v2] = [vertices[0], vertices[1], vertices[2]];

        let min_x = v0.x.min(v1.x.min(v2.x)) as i32;
        let max_x = v0.x.max(v1.x.max(v2.x)) as i32;
        let min_y = v0.y.min(v1.y.min(v2.y)) as i32;
        let max_y = v0.y.max(v1.y.max(v2.y)) as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x < 0 || y < 0 || x >= canvas.width as i32 || y >= canvas.height as i32 {
                    continue;
                }

                let p = Pos2::new(x as f32, y as f32);
                let bary = self.barycentric_coordinates(p, v0, v1, v2);

                if bary.0 >= 0.0 && bary.1 >= 0.0 && bary.2 >= 0.0 {
                    let z = v0.z * bary.0 + v1.z * bary.1 + v2.z * bary.2;

                    if z_buffer_enabled {
                        if canvas.test_and_set_z(x as usize, y as usize, z) {
                            canvas[(x as usize, y as usize)] = color;
                        }
                    } else {
                        canvas[(x as usize, y as usize)] = color;
                    }
                }
            }
        }
    }

    fn fill_triangle_gouraud(
        &self,
        vertices: &[Vec3],
        intensities: &[f32],
        texture_coords: &[(f32, f32)],
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        let [v0, v1, v2] = [vertices[0], vertices[1], vertices[2]];
        let [i0, i1, i2] = [intensities[0], intensities[1], intensities[2]];
        let [uv0, uv1, uv2] = [texture_coords[0], texture_coords[1], texture_coords[2]];

        let min_x = v0.x.min(v1.x.min(v2.x)) as i32;
        let max_x = v0.x.max(v1.x.max(v2.x)) as i32;
        let min_y = v0.y.min(v1.y.min(v2.y)) as i32;
        let max_y = v0.y.max(v1.y.max(v2.y)) as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x < 0 || y < 0 || x >= canvas.width as i32 || y >= canvas.height as i32 {
                    continue;
                }

                let p = Pos2::new(x as f32, y as f32);
                let bary = self.barycentric_coordinates(p, v0, v1, v2);

                if bary.0 >= 0.0 && bary.1 >= 0.0 && bary.2 >= 0.0 {
                    let z = v0.z * bary.0 + v1.z * bary.1 + v2.z * bary.2;
                    let intensity = i0 * bary.0 + i1 * bary.1 + i2 * bary.2;
                    let u = uv0.0 * bary.0 + uv1.0 * bary.1 + uv2.0 * bary.2;
                    let v = uv0.1 * bary.0 + uv1.1 * bary.1 + uv2.1 * bary.2;

                    let base_color = model.material.get_uv_color(u, v);
                    let shaded_color = self.apply_lighting_color(base_color, intensity);

                    if z_buffer_enabled {
                        if canvas.test_z(x as usize, y as usize, z) {
                            canvas[(x as usize, y as usize)] = shaded_color;
                        }
                    } else {
                        canvas[(x as usize, y as usize)] = shaded_color;
                    }
                }
            }
        }
    }

    fn fill_triangle_phong(
        &self,
        vertices: &[Vec3],
        normals: &[Vec3],
        positions: &[Vec3],
        texture_coords: &[(f32, f32)],
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        let [v0, v1, v2] = [vertices[0], vertices[1], vertices[2]];
        let [n0, n1, n2] = [normals[0], normals[1], normals[2]];
        let [p0, p1, p2] = [positions[0], positions[1], positions[2]];
        let [uv0, uv1, uv2] = [texture_coords[0], texture_coords[1], texture_coords[2]];

        let min_x = v0.x.min(v1.x.min(v2.x)) as i32;
        let max_x = v0.x.max(v1.x.max(v2.x)) as i32;
        let min_y = v0.y.min(v1.y.min(v2.y)) as i32;
        let max_y = v0.y.max(v1.y.max(v2.y)) as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x < 0 || y < 0 || x >= canvas.width as i32 || y >= canvas.height as i32 {
                    continue;
                }

                let screen_pos = Pos2::new(x as f32, y as f32);
                let bary = self.barycentric_coordinates(screen_pos, v0, v1, v2);

                if bary.0 >= 0.0 && bary.1 >= 0.0 && bary.2 >= 0.0 {
                    let z = v0.z * bary.0 + v1.z * bary.1 + v2.z * bary.2;
                    let normal = (n0 * bary.0 + n1 * bary.1 + n2 * bary.2).normalize();
                    let world_pos = p0 * bary.0 + p1 * bary.1 + p2 * bary.2;
                    let u = uv0.0 * bary.0 + uv1.0 * bary.1 + uv2.0 * bary.2;
                    let v = uv0.1 * bary.0 + uv1.1 * bary.1 + uv2.1 * bary.2;

                    let base_color = model.material.get_uv_color(u, v);
                    let shaded_color =
                        self.calculate_phong_lighting(base_color, normal, world_pos, model);

                    if z_buffer_enabled {
                        if canvas.test_z(x as usize, y as usize, z) {
                            canvas[(x as usize, y as usize)] = shaded_color;
                        }
                    } else {
                        canvas[(x as usize, y as usize)] = shaded_color;
                    }
                }
            }
        }
    }

    /// Находит барицентрические координаты по 3-м точкам.
    fn barycentric_coordinates(&self, p: Pos2, a: Vec3, b: Vec3, c: Vec3) -> (f32, f32, f32) {
        let v0 = Vec3::new(b.x - a.x, b.y - a.y, 0.0);
        let v1 = Vec3::new(c.x - a.x, c.y - a.y, 0.0);
        let v2 = Vec3::new(p.x - a.x, p.y - a.y, 0.0);

        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);
        let denom = d00 * d11 - d01 * d01;

        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;

        (u, v, w)
    }

    fn apply_lighting_color(&self, color: Color32, intensity: f32) -> Color32 {
        let r = (color.r() as f32 * intensity) as u8;
        let g = (color.g() as f32 * intensity) as u8;
        let b = (color.b() as f32 * intensity) as u8;
        Color32::from_rgb(r, g, b)
    }

    fn calculate_phong_lighting(
        &self,
        base_color: Color32,
        normal: Vec3,
        position: Vec3,
        model: &Model3,
    ) -> Color32 {
        let mut final_color = base_color;

        let ambient_strength = 0.1;
        final_color = self.mix_colors(final_color, self.ambient_light, ambient_strength);

        for light in &self.lights {
            let light_dir = (light.position - Point3::from(position)).normalize();
            let view_dir = (self.camera.get_position() - Point3::from(position)).normalize();
            let reflect_dir = reflect(-light_dir, normal);

            let diff = normal.dot(light_dir).max(0.0);
            let diffuse = self.mix_colors(Color32::BLACK, light.color, diff);

            let spec = reflect_dir
                .dot(view_dir)
                .max(0.0)
                .powf(model.material.shininess);
            let specular = self.mix_colors(
                Color32::BLACK,
                light.color,
                spec * model.material.specular_strength,
            );

            // Combine
            final_color = self.add_colors(final_color, diffuse);
            final_color = self.add_colors(final_color, specular);
        }

        final_color
    }

    fn mix_colors(&self, a: Color32, b: Color32, factor: f32) -> Color32 {
        let r = (a.r() as f32 * (1.0 - factor) + b.r() as f32 * factor) as u8;
        let g = (a.g() as f32 * (1.0 - factor) + b.g() as f32 * factor) as u8;
        let b_val = (a.b() as f32 * (1.0 - factor) + b.b() as f32 * factor) as u8;
        Color32::from_rgb(r, g, b_val)
    }

    fn add_colors(&self, a: Color32, b: Color32) -> Color32 {
        let r = (a.r() as u16 + b.r() as u16).min(255) as u8;
        let g = (a.g() as u16 + b.g() as u16).min(255) as u8;
        let b_val = (a.b() as u16 + b.b() as u16).min(255) as u8;
        Color32::from_rgb(r, g, b_val)
    }

    fn triangulate_and_fill_polygon(
        &self,
        vertices: &[Vec3],
        color: Color32,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        for i in 1..vertices.len() - 1 {
            let triangle = [vertices[0], vertices[i], vertices[i + 1]];
            self.fill_triangle(&triangle, color, canvas, z_buffer_enabled);
        }
    }

    fn triangulate_and_fill_gouraud(
        &self,
        vertices: &[Vec3],
        intensities: &[f32],
        texture_coords: &[(f32, f32)],
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        for i in 1..vertices.len() - 1 {
            let triangle_verts = [vertices[0], vertices[i], vertices[i + 1]];
            let triangle_ints = [intensities[0], intensities[i], intensities[i + 1]];
            let triangle_uvs = [texture_coords[0], texture_coords[i], texture_coords[i + 1]];
            self.fill_triangle_gouraud(
                &triangle_verts,
                &triangle_ints,
                &triangle_uvs,
                model,
                canvas,
                z_buffer_enabled,
            );
        }
    }

    fn triangulate_and_fill_phong(
        &self,
        vertices: &[Vec3],
        normals: &[Vec3],
        positions: &[Vec3],
        texture_coords: &[(f32, f32)],
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        for i in 1..vertices.len() - 1 {
            let triangle_verts = [vertices[0], vertices[i], vertices[i + 1]];
            let triangle_norms = [normals[0], normals[i], normals[i + 1]];
            let triangle_pos = [positions[0], positions[i], positions[i + 1]];
            let triangle_uvs = [texture_coords[0], texture_coords[i], texture_coords[i + 1]];
            self.fill_triangle_phong(
                &triangle_verts,
                &triangle_norms,
                &triangle_pos,
                &triangle_uvs,
                model,
                canvas,
                z_buffer_enabled,
            );
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(Camera3::default())
    }
}

fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - normal * 2.0 * incident.dot(normal)
}
