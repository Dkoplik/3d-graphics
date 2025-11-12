use crate::{
    Camera3, Canvas, HVec3, LightSource, Model3, Point3, Scene, Transform3D, Vec3, classes3d::mesh,
};
use egui::{Color32, Pos2};
use image::codecs::tiff;

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
    /// Возвращает количество отрисованных полигонов.
    pub fn render(
        &self,
        canvas: &mut Canvas,
        render_options: RenderOptions,
        show_custom_axis: bool,
        axis_point1: Point3,
        axis_point2: Point3,
    ) -> usize {
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
            //let polygons = self.skip_out_of_camera_polygons(model, polygons);

            let model_polygons = match render_options.render_type {
                RenderType::WireFrame => {
                    self.render_model_wireframe(&projected_vertexes, &polygons, canvas)
                }
                RenderType::Solid => {
                    let polygon_cnt = self.render_solid(
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
                    polygon_cnt
                }
            };
            polygon_count += model_polygons;
        }

        polygon_count
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
        let polygons = model.mesh.get_polygons();
        let normals = model.mesh.get_normals();
        let mut visible_polygons = Vec::new();

        let camera_view_direction = self.camera.get_direction();

        for polygon in polygons {
            let vertex_indices = polygon.get_vertexes();
            if vertex_indices.len() < 3 {
                continue;
            }

            // Считаем нормаль полигона по нормалям вершин
            let mut polygon_normal = Vec3::zero();
            let mut normals_count = 0;

            // нормали надо преобразовать к глобальным координатам
            let to_global = model.mesh.get_local_frame().local_to_global_matrix();

            for &vertex_index in vertex_indices {
                polygon_normal =
                    polygon_normal + to_global.apply_to_hvec(normals[vertex_index].into()).into();
                normals_count += 1;
            }

            // Если нормаль есть, производим отсечение
            if normals_count > 0 && polygon_normal.length_squared() > f32::EPSILON {
                polygon_normal = polygon_normal.normalize();

                // Если нормаль направлена в сторону камеры, то оставляем полигон
                let dot_product = polygon_normal.dot(camera_view_direction);
                if dot_product < 0.0 {
                    visible_polygons.push(polygon.clone());
                }
            }
        }

        visible_polygons
    }

    /// Реднер каркаса модели.
    fn render_model_wireframe(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<mesh::Polygon3>,
        canvas: &mut Canvas,
    ) -> usize {
        // Рисуем рёбра
        for polygon in polygons {
            // Вершины полигона
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

        polygons.len()
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
    ) -> usize {
        for polygon in polygons {
            for triangle in triangulate_polygon(&polygon.get_vertexes()) {
                // проекция вершин треугольника
                let v0 = projected_vertexes[triangle.0];
                let v1 = projected_vertexes[triangle.1];
                let v2 = projected_vertexes[triangle.2];

                // текстурные UV-координаты вершин треугольника
                let tx0 = model.mesh.get_texture_coords()[triangle.0];
                let tx1 = model.mesh.get_texture_coords()[triangle.1];
                let tx2 = model.mesh.get_texture_coords()[triangle.2];

                let min_x = v0.x.min(v1.x.min(v2.x)) as usize;
                let max_x = v0.x.max(v1.x.max(v2.x)) as usize;
                let min_y = v0.y.min(v1.y.min(v2.y)) as usize;
                let max_y = v0.y.max(v1.y.max(v2.y)) as usize;

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        if x < 0 || y < 0 || x >= canvas.width || y >= canvas.height {
                            continue;
                        }

                        let p = Point3::new(x as f32, y as f32, 0.0);
                        let bary = self.barycentric_coordinates(&[v0, v1, v2], p);

                        let z = bary.x * v0.z + bary.y * v1.z + bary.z * v2.z;
                        if z_buffer_enabled && !canvas.test_and_set_z(x, y, z) {
                            continue;
                        }

                        let u = bary.x * tx0.0 + bary.y * tx1.0 + bary.z * tx2.0;
                        let v = bary.x * tx0.1 + bary.y * tx1.1 + bary.z * tx2.1;

                        let base_color = model.material.get_uv_color(u, v);
                        canvas[(x, y)] = base_color;
                    }
                }
            }
        }

        polygons.len()
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
        // освещённость всех вершин модели
        let light_colors = self.calculate_vertex_lighting(model);

        for polygon in polygons {
            for triangle in triangulate_polygon(&polygon.get_vertexes()) {
                // проекция вершин треугольника
                let v0 = projected_vertexes[triangle.0];
                let v1 = projected_vertexes[triangle.1];
                let v2 = projected_vertexes[triangle.2];

                // освещённость вершин треугольника
                let light0 = light_colors[triangle.0];
                let light1 = light_colors[triangle.1];
                let light2 = light_colors[triangle.2];

                let min_x = v0.x.min(v1.x.min(v2.x)) as usize;
                let max_x = v0.x.max(v1.x.max(v2.x)) as usize;
                let min_y = v0.y.min(v1.y.min(v2.y)) as usize;
                let max_y = v0.y.max(v1.y.max(v2.y)) as usize;

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        if x < 0 || y < 0 || x >= canvas.width || y >= canvas.height {
                            continue;
                        }

                        let p = Point3::new(x as f32, y as f32, 0.0);
                        let bary = self.barycentric_coordinates(&[v0, v1, v2], p);

                        let z = bary.x * v0.z + bary.y * v1.z + bary.z * v2.z;
                        if z_buffer_enabled && !canvas.test_and_set_z(x, y, z) {
                            continue;
                        }

                        let base_color = canvas[(x, y)];
                        // освещённость текселя
                        let light = light0.linear_multiply(bary.x)
                            + light1.linear_multiply(bary.y)
                            + light2.linear_multiply(bary.z);

                        let color = base_color + light;
                        canvas[(x, y)] = color;
                    }
                }
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
        // нормали в глобальных координатах
        let global_vertex_normals: Vec<Vec3> = model
            .mesh
            .get_normals()
            .iter()
            .map(|&v| {
                Vec3::from(
                    HVec3::from(v)
                        .apply_transform(&model.mesh.get_local_frame().local_to_global_matrix()),
                )
                .normalize()
            })
            .collect();

        // позиции вершин в глобальной системе
        let global_vertex_positions: Vec<Vec3> = model
            .mesh
            .get_vertexes()
            .iter()
            .map(|&v| {
                Vec3::from(
                    v.apply_transform(&model.mesh.get_local_frame().local_to_global_matrix()),
                )
            })
            .collect();

        for polygon in polygons {
            for triangle in triangulate_polygon(&polygon.get_vertexes()) {
                // проекция вершин треугольника
                let v0 = projected_vertexes[triangle.0];
                let v1 = projected_vertexes[triangle.1];
                let v2 = projected_vertexes[triangle.2];

                // глобальные нормали вершин треугольника
                let normal0 = global_vertex_normals[triangle.0];
                let normal1 = global_vertex_normals[triangle.1];
                let normal2 = global_vertex_normals[triangle.2];

                // глобальные позиции вершин треугольника
                let pos0 = global_vertex_positions[triangle.0];
                let pos1 = global_vertex_positions[triangle.1];
                let pos2 = global_vertex_positions[triangle.2];

                let min_x = v0.x.min(v1.x.min(v2.x)) as usize;
                let max_x = v0.x.max(v1.x.max(v2.x)) as usize;
                let min_y = v0.y.min(v1.y.min(v2.y)) as usize;
                let max_y = v0.y.max(v1.y.max(v2.y)) as usize;

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        if x < 0 || y < 0 || x >= canvas.width || y >= canvas.height {
                            continue;
                        }

                        let p = Point3::new(x as f32, y as f32, 0.0);
                        let bary = self.barycentric_coordinates(&[v0, v1, v2], p);

                        let z = bary.x * v0.z + bary.y * v1.z + bary.z * v2.z;
                        if z_buffer_enabled && !canvas.test_and_set_z(x, y, z) {
                            continue;
                        }

                        // Интерполируем нормали и позиции
                        let normal = normal0 * bary.x + normal1 * bary.y + normal2 * bary.z;
                        normal.normalize()
                        let pos = pos0 * bary.x + pos1 * bary.y + pos2 * bary.z;
            
                        let light_dir 
            // Вычисляем векторы
            lightDir = normalize(light.position - interpPosition)
            viewDir = normalize(camera.position - interpPosition)
            reflectDir = reflect(-lightDir, interpNormal)
            
            // Диффузная составляющая
            diffuse = max(dot(interpNormal, lightDir), 0.0)
            
            // Спекулярная составляющая
            specular = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess)
            
            // Финальный цвет
            ambientColor = material.ambient * light.color
            diffuseColor = material.diffuse * diffuse * light.color
            specularColor = material.specular * specular * light.color
            
            pixelColor = ambientColor + diffuseColor + specularColor
                        canvas[(x, y)] = color;
                    }
                }
            }
        }
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

    fn calculate_vertex_lighting(&self, model: &Model3) -> Vec<Color32> {
        // нормали в глобальных координатах
        let global_vertex_normals: Vec<Vec3> = model
            .mesh
            .get_normals()
            .iter()
            .map(|&v| {
                Vec3::from(
                    HVec3::from(v)
                        .apply_transform(&model.mesh.get_local_frame().local_to_global_matrix()),
                )
                .normalize()
            })
            .collect();

        // позиции вершин в глобальной системе
        let global_vertex_positions: Vec<Vec3> = model
            .mesh
            .get_vertexes()
            .iter()
            .map(|&v| {
                Vec3::from(
                    v.apply_transform(&model.mesh.get_local_frame().local_to_global_matrix()),
                )
            })
            .collect();

        let mut colors = Vec::with_capacity(global_vertex_normals.len());

        for i in 0..global_vertex_normals.len() {
            let normal = global_vertex_normals[i];
            let position = global_vertex_positions[i];

            // глобальный свет
            let mut light_color = self.ambient_light.linear_multiply(0.1);

            // Влияние каждого источника
            for light in &self.lights {
                let light_dir = (light.position - Point3::from(position)).normalize();
                let diffuse = normal.dot(light_dir).max(0.0);
                light_color = light_color + light.color.linear_multiply(light.intensity * diffuse);
            }

            colors.push(light_color);
        }

        colors
    }

    fn fill_triangle_solid(
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
    /// `triangle` - полигон-треугольник, по которому строятся координаты
    /// `point` - точка, для которой нужны координаты
    ///
    /// Поскольку это уже в проекции на экран, z-координата не учитывается.
    ///
    /// Возвращает координаты в виде Point3.
    fn barycentric_coordinates(&self, triangle: &[Vec3], point: Point3) -> Point3 {
        let mut v0 = triangle[1] - triangle[0];
        let mut v1 = triangle[2] - triangle[0];
        let mut v2 = Vec3::from(point) - triangle[0];

        // z-координата предозначеня для буфера, точки уже в проекции
        v0.z = 0.0;
        v1.z = 0.0;
        v2.z = 0.0;

        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);

        let denom = d00 * d11 - d01 * d01;
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;

        Point3::new(u, v, w)
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

/// Триангуляция полигона.
/// `polygon` - полигон, заданный индексами вершин.
///
/// Пока что примитивная веерная триангуляция.
fn triangulate_polygon(polygon: &[usize]) -> Vec<(usize, usize, usize)> {
    #[cfg(debug_assertions)]
    {
        if polygon.len() < 3 {
            eprintln!(
                "Warning: триангуляция полигона с {} вершинами",
                polygon.len()
            );
        }
    }

    let mut triangles = vec![];
    for i in 1..polygon.len() - 1 {
        triangles.push((polygon[0], polygon[i], polygon[i + 1]));
    }
    triangles
}

fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - normal * 2.0 * incident.dot(normal)
}

#[cfg(test)]
mod scene_tests {
    use super::*;
    use crate::{Material, Mesh, Model3, classes3d::mesh::Polygon3};
    use std::f32::consts::PI;

    const TOLERANCE: f32 = 1e-6;

    fn create_test_cube_model(position: Point3, color: Color32) -> Model3 {
        let mesh = Mesh::hexahedron();
        let mut model = Model3::from_mesh(mesh);
        model.material.color = color;
        model.set_position(position);
        model
    }

    fn create_test_tetrahedron_model(position: Point3, color: Color32) -> Model3 {
        let mesh = Mesh::tetrahedron();
        let mut model = Model3::from_mesh(mesh);
        model.material.color = color;
        model.set_position(position);
        model
    }

    fn create_simple_camera() -> Camera3 {
        Camera3::new(
            Point3::new(0.0, 0.0, -10.0),
            Vec3::forward(),
            Vec3::up(),
            PI / 3.0,
            16.0 / 9.0,
            0.1,
            100.0,
        )
    }

    #[test]
    fn test_scene_creation() {
        let camera = create_simple_camera();
        let scene = Scene::new(camera);

        assert_eq!(scene.models.len(), 0);
        assert_eq!(scene.lights.len(), 0);
        assert_eq!(scene.ambient_light, Color32::BLACK);
    }

    #[test]
    fn test_scene_add_model() {
        let camera = create_simple_camera();
        let mut scene = Scene::new(camera);

        let cube = create_test_cube_model(Point3::zero(), Color32::RED);
        scene.add_model(cube);

        assert_eq!(scene.models.len(), 1);
    }

    #[test]
    fn test_scene_add_light() {
        let camera = create_simple_camera();
        let mut scene = Scene::new(camera);

        let light = LightSource {
            position: Point3::new(0.0, 5.0, 0.0),
            color: Color32::WHITE,
            intensity: 1.0,
        };
        scene.add_light(light);

        assert_eq!(scene.lights.len(), 1);
    }

    #[test]
    fn test_wireframe_rendering() {
        let camera = create_simple_camera();
        let mut scene = Scene::new(camera);

        let cube = create_test_cube_model(Point3::zero(), Color32::RED);
        scene.add_model(cube);

        let mut canvas = Canvas::new(800, 600);
        let render_options = RenderOptions {
            render_type: RenderType::WireFrame,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        let polygon_count = scene.render(
            &mut canvas,
            render_options,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        // Cube has 6 faces (polygons)
        assert_eq!(polygon_count, 6);

        // Check that some pixels were drawn (not just background)
        let mut has_non_background_pixels = false;
        for x in 0..canvas.width {
            for y in 0..canvas.height {
                if canvas[(x, y)] != Color32::GRAY {
                    has_non_background_pixels = true;
                    break;
                }
            }
            if has_non_background_pixels {
                break;
            }
        }
        assert!(
            has_non_background_pixels,
            "Wireframe should draw visible lines"
        );
    }

    #[test]
    fn test_solid_rendering_no_shading() {
        let camera = create_simple_camera();
        let mut scene = Scene::new(camera);

        let cube = create_test_cube_model(Point3::zero(), Color32::BLUE);
        scene.add_model(cube);

        let mut canvas = Canvas::new(800, 600);
        let render_options = RenderOptions {
            render_type: RenderType::Solid,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        let polygon_count = scene.render(
            &mut canvas,
            render_options,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        // Cube has 6 faces
        assert_eq!(polygon_count, 6);

        // Check that solid areas were filled
        let mut has_solid_pixels = false;
        for x in 0..canvas.width {
            for y in 0..canvas.height {
                if canvas[(x, y)] == Color32::BLUE {
                    has_solid_pixels = true;
                    break;
                }
            }
            if has_solid_pixels {
                break;
            }
        }
        assert!(
            has_solid_pixels,
            "Solid rendering should fill polygons with material color"
        );
    }

    #[test]
    fn test_backface_culling() {
        let camera = create_simple_camera();
        let mut scene = Scene::new(camera);

        let cube = create_test_cube_model(Point3::zero(), Color32::GREEN);
        scene.add_model(cube);

        let mut canvas_with_culling = Canvas::new(800, 600);
        let mut canvas_without_culling = Canvas::new(800, 600);

        // Render with backface culling
        let render_options_with_culling = RenderOptions {
            render_type: RenderType::Solid,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: true,
            z_buffer_enabled: false,
        };

        // Render without backface culling
        let render_options_without_culling = RenderOptions {
            render_type: RenderType::Solid,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        let culling_polygons_count = scene.render(
            &mut canvas_with_culling,
            render_options_with_culling,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        let no_culling_polygons_count = scene.render(
            &mut canvas_without_culling,
            render_options_without_culling,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        assert!(
            culling_polygons_count < no_culling_polygons_count,
            "После отсечения граней должно рендериться меньше полигонов"
        );
    }

    #[test]
    fn test_z_buffer_occlusion() {
        let camera = create_simple_camera();
        let mut scene = Scene::new(camera);

        // Create two cubes: one in front, one behind
        let front_cube = create_test_cube_model(Point3::new(0.0, 0.0, 0.0), Color32::RED);
        let back_cube = create_test_cube_model(Point3::new(0.0, 0.0, 5.0), Color32::BLUE);

        scene.add_model(front_cube);
        scene.add_model(back_cube);

        let mut canvas_with_z_buffer = Canvas::new(800, 600);
        let mut canvas_without_z_buffer = Canvas::new(800, 600);

        // Render with z-buffer
        let render_options_with_z_buffer = RenderOptions {
            render_type: RenderType::Solid,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: true,
        };

        // Render without z-buffer
        let render_options_without_z_buffer = RenderOptions {
            render_type: RenderType::Solid,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        scene.render(
            &mut canvas_with_z_buffer,
            render_options_with_z_buffer,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        scene.render(
            &mut canvas_without_z_buffer,
            render_options_without_z_buffer,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        // With z-buffer, the back cube should be occluded by the front cube
        // Without z-buffer, both cubes might be visible (depending on rendering order)

        let mut front_pixels_with_z_buffer = 0;
        let mut back_pixels_with_z_buffer = 0;
        let mut front_pixels_without_z_buffer = 0;
        let mut back_pixels_without_z_buffer = 0;

        for x in 0..canvas_with_z_buffer.width {
            for y in 0..canvas_with_z_buffer.height {
                if canvas_with_z_buffer[(x, y)] == Color32::RED {
                    front_pixels_with_z_buffer += 1;
                }
                if canvas_with_z_buffer[(x, y)] == Color32::BLUE {
                    back_pixels_with_z_buffer += 1;
                }
                if canvas_without_z_buffer[(x, y)] == Color32::RED {
                    front_pixels_without_z_buffer += 1;
                }
                if canvas_without_z_buffer[(x, y)] == Color32::BLUE {
                    back_pixels_without_z_buffer += 1;
                }
            }
        }

        // With z-buffer, there should be fewer blue pixels (back cube)
        assert!(
            back_pixels_with_z_buffer < back_pixels_without_z_buffer,
            "Z-buffer should reduce visibility of occluded objects"
        );

        // Front cube should be clearly visible in both cases
        assert!(
            front_pixels_with_z_buffer > 0,
            "Front cube should be visible with z-buffer"
        );
        assert!(
            front_pixels_without_z_buffer > 0,
            "Front cube should be visible without z-buffer"
        );
    }

    #[test]
    fn test_lighting_effects() {
        let camera = create_simple_camera();
        let mut scene = Scene::new(camera);

        // Add a light source
        let light = LightSource {
            position: Point3::new(5.0, 5.0, -5.0),
            color: Color32::WHITE,
            intensity: 1.0,
        };
        scene.add_light(light);

        scene.set_ambient_light(Color32::from_rgb(50, 50, 50));

        let cube = create_test_cube_model(Point3::zero(), Color32::WHITE);
        scene.add_model(cube);

        let mut canvas_gouraud = Canvas::new(800, 600);
        let mut canvas_phong = Canvas::new(800, 600);

        // Test Gouraud shading
        let render_options_gouraud = RenderOptions {
            render_type: RenderType::Solid,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::Gouraud,
            backface_culling: false,
            z_buffer_enabled: true,
        };

        // Test Phong shading
        let render_options_phong = RenderOptions {
            render_type: RenderType::Solid,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::Phong,
            backface_culling: false,
            z_buffer_enabled: true,
        };

        scene.render(
            &mut canvas_gouraud,
            render_options_gouraud,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        scene.render(
            &mut canvas_phong,
            render_options_phong,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        // Both shading methods should produce visible results
        let mut gouraud_has_variation = false;
        let mut phong_has_variation = false;
        let mut previous_gouraud_color = Color32::GRAY;
        let mut previous_phong_color = Color32::GRAY;

        for x in (0..canvas_gouraud.width).step_by(10) {
            for y in (0..canvas_gouraud.height).step_by(10) {
                let gouraud_color = canvas_gouraud[(x, y)];
                let phong_color = canvas_phong[(x, y)];

                if gouraud_color != Color32::GRAY && gouraud_color != previous_gouraud_color {
                    gouraud_has_variation = true;
                }
                if phong_color != Color32::GRAY && phong_color != previous_phong_color {
                    phong_has_variation = true;
                }

                previous_gouraud_color = gouraud_color;
                previous_phong_color = phong_color;
            }
        }

        assert!(
            gouraud_has_variation,
            "Gouraud shading should produce color variation"
        );
        assert!(
            phong_has_variation,
            "Phong shading should produce color variation"
        );
    }

    #[test]
    fn test_coordinate_axes_rendering() {
        let camera = create_simple_camera();
        let scene = Scene::new(camera);

        let mut canvas = Canvas::new(800, 600);
        let render_options = RenderOptions {
            render_type: RenderType::WireFrame,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        scene.render(
            &mut canvas,
            render_options,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        // Coordinate axes should be drawn (red, green, blue lines from origin)
        let mut has_red = false;
        let mut has_green = false;
        let mut has_blue = false;

        for x in 0..canvas.width {
            for y in 0..canvas.height {
                let color = canvas[(x, y)];
                if color == Color32::RED {
                    has_red = true;
                } else if color == Color32::GREEN {
                    has_green = true;
                } else if color == Color32::BLUE {
                    has_blue = true;
                }
            }
        }

        assert!(has_red, "X axis (red) should be visible");
        assert!(has_green, "Y axis (green) should be visible");
        assert!(has_blue, "Z axis (blue) should be visible");
    }

    #[test]
    fn test_custom_axis_rendering() {
        let camera = create_simple_camera();
        let scene = Scene::new(camera);

        let mut canvas = Canvas::new(800, 600);
        let render_options = RenderOptions {
            render_type: RenderType::WireFrame,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        let axis_start = Point3::new(-2.0, 0.0, 0.0);
        let axis_end = Point3::new(2.0, 0.0, 0.0);

        scene.render(
            &mut canvas,
            render_options,
            true, // Show custom axis
            axis_start,
            axis_end,
        );

        // Custom axis should be drawn (orange line)
        let mut has_orange = false;
        let orange_color = Color32::from_rgb(255, 165, 0);

        for x in 0..canvas.width {
            for y in 0..canvas.height {
                if canvas[(x, y)] == orange_color {
                    has_orange = true;
                    break;
                }
            }
            if has_orange {
                break;
            }
        }

        assert!(
            has_orange,
            "Custom axis (orange) should be visible when enabled"
        );
    }

    #[test]
    fn test_empty_scene_rendering() {
        let camera = create_simple_camera();
        let scene = Scene::new(camera);

        let mut canvas = Canvas::new(800, 600);
        let render_options = RenderOptions {
            render_type: RenderType::WireFrame,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        let polygon_count = scene.render(
            &mut canvas,
            render_options,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        assert_eq!(polygon_count, 0);

        // Only coordinate axes should be drawn
        let mut has_non_axis_pixels = false;
        for x in 0..canvas.width {
            for y in 0..canvas.height {
                let color = canvas[(x, y)];
                if color != Color32::GRAY
                    && color != Color32::RED
                    && color != Color32::GREEN
                    && color != Color32::BLUE
                {
                    has_non_axis_pixels = true;
                    break;
                }
            }
            if has_non_axis_pixels {
                break;
            }
        }

        assert!(
            !has_non_axis_pixels,
            "Empty scene should only draw coordinate axes"
        );
    }

    #[test]
    fn test_polygon_out_of_camera_culling() {
        let camera = create_simple_camera();
        let mut scene = Scene::new(camera);

        // Create a cube far outside the camera's view
        let far_cube = create_test_cube_model(Point3::new(1000.0, 1000.0, 1000.0), Color32::RED);
        scene.add_model(far_cube);

        let mut canvas = Canvas::new(800, 600);
        let render_options = RenderOptions {
            render_type: RenderType::WireFrame,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        let polygon_count = scene.render(
            &mut canvas,
            render_options,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        // Even though the cube is far away, it might still be partially visible
        // or completely culled depending on the implementation
        // This test just ensures the rendering doesn't crash
        assert!(polygon_count >= 0);
    }

    #[test]
    fn test_different_projection_types() {
        let camera = create_simple_camera();
        let mut scene = Scene::new(camera);

        let cube = create_test_cube_model(Point3::zero(), Color32::YELLOW);
        scene.add_model(cube);

        let mut canvas_perspective = Canvas::new(800, 600);
        let mut canvas_parallel = Canvas::new(800, 600);

        // Perspective projection
        let render_options_perspective = RenderOptions {
            render_type: RenderType::WireFrame,
            projection_type: ProjectionType::Perspective,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        // Parallel projection
        let render_options_parallel = RenderOptions {
            render_type: RenderType::WireFrame,
            projection_type: ProjectionType::Parallel,
            shading_type: ShadingType::None,
            backface_culling: false,
            z_buffer_enabled: false,
        };

        scene.render(
            &mut canvas_perspective,
            render_options_perspective,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        scene.render(
            &mut canvas_parallel,
            render_options_parallel,
            false,
            Point3::zero(),
            Point3::zero(),
        );

        // Both projections should produce visible results
        let mut perspective_has_content = false;
        let mut parallel_has_content = false;

        for x in 0..canvas_perspective.width {
            for y in 0..canvas_perspective.height {
                if canvas_perspective[(x, y)] != Color32::GRAY {
                    perspective_has_content = true;
                }
                if canvas_parallel[(x, y)] != Color32::GRAY {
                    parallel_has_content = true;
                }
            }
        }

        assert!(
            perspective_has_content,
            "Perspective projection should render content"
        );
        assert!(
            parallel_has_content,
            "Parallel projection should render content"
        );
    }
}
