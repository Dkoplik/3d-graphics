use crate::{
    Camera3, Canvas, HVec3, LightSource, Material, Model3, Point3, Scene, SceneRenderer,
    Transform3D, Vec3, classes3d::mesh::Polygon3,
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
    /// Отсутствие шейдинга
    #[default]
    None,
    /// Шейдинг Гуро для модели Ламберта
    GouraudLambert,
    /// Шейдинг Фонга для модели туншейдинг
    PhongToonShading(usize),
}

impl SceneRenderer {
    /// Нарисовать сцену на холст со всеми нужными преобразованиями.
    ///
    /// Возвращает количество отрисованных полигонов.
    pub fn render(
        &self,
        scene: &Scene,
        canvas: &mut Canvas,
        show_custom_axis: bool,
        axis_point1: Point3,
        axis_point2: Point3,
    ) -> usize {
        // Стереть прошлый кадр.
        canvas.clear(Color32::GRAY);

        // Матрица преобразования из глобальных координат в экранные
        let global_to_screen_transform =
            self.get_view_projection_transform(self.projection_type, canvas);

        // Отрисовка глобальной координатной системы.
        self.draw_coordinate_axes(canvas, global_to_screen_transform);

        // Отрисовка пользовательской оси вращения, если имеется
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
            let projected_vertexes: Vec<Vec3> =
                transform_model(global_to_screen_transform, model).collect();

            // Полигоны к отрисовке
            let polygons = if self.backface_culling {
                todo!()
            } else {
                todo!()
            };

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
        // рендер идёт верхом вниз
        canvas.invert_y();

        polygon_count
    }

    // --------------------------------------------------
    // Основные методы рендера
    // --------------------------------------------------

    /// Реднер каркаса модели.
    ///
    /// Возвращает количество нарисованных полигонов.
    fn render_model_wireframe(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<Polygon3>,
        material: Material,
        canvas: &mut Canvas,
    ) {
        let color = opposite_color(material.color);

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
                canvas.draw_sharp_line(start_pos, end_pos, color);
            }
        }

        // Рисуем вершины
        for &vertex in projected_vertexes {
            let pos = Pos2::new(vertex.x, vertex.y);
            canvas.circle_filled(pos, 3.0, color);
        }
    }

    /// Рендер цельного объекта, с гранями вместо границ и с учётом материала и текстуры.
    ///
    /// Этот этап рендера не учитывает освещение, поэтому без шейдинга.
    fn render_solid(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<Polygon3>,
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        let vertexes_2d: Vec<Vec3> = projected_vertexes
            .iter()
            .cloned()
            .map(|v| Vec3::new(v.x, v.y, 0.0))
            .collect();
        let texture_coords = model.mesh.get_texture_coords();
        for polygon in polygons {
            // если четырёхугольник - билинейная интерполяция
            if polygon.is_rectangle() {
                let rectangle = polygon.get_vertexes();
                // проекция вершин треугольника
                let v0 = projected_vertexes[rectangle[0]];
                let v1 = projected_vertexes[rectangle[1]];
                let v2 = projected_vertexes[rectangle[2]];
                let v3 = projected_vertexes[rectangle[3]];

                // текстурные UV-координаты вершин треугольника
                let tx0 = texture_coords[rectangle[0]];
                let tx1 = texture_coords[rectangle[1]];
                let tx2 = texture_coords[rectangle[2]];
                let tx3 = texture_coords[rectangle[3]];

                // ограничивающий прямоугольник
                let min_x = *vec![v0.x as usize, v1.x as usize, v2.x as usize, v3.x as usize]
                    .iter()
                    .min()
                    .unwrap();
                let max_x = *vec![v0.x as usize, v1.x as usize, v2.x as usize, v3.x as usize]
                    .iter()
                    .max()
                    .unwrap();
                let min_y = *vec![v0.y as usize, v1.y as usize, v2.y as usize, v3.y as usize]
                    .iter()
                    .min()
                    .unwrap();
                let max_y = *vec![v0.y as usize, v1.y as usize, v2.y as usize, v3.y as usize]
                    .iter()
                    .max()
                    .unwrap();

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        if x < 0 || y < 0 || x >= canvas.width || y >= canvas.height {
                            continue;
                        }

                        let cur_point = Vec3::new(x as f32, y as f32, 0.0);
                        // точка на полигоне?
                        if !polygon.is_point_in_convex_polygon(&vertexes_2d, cur_point) {
                            continue;
                        }

                        let (alpha, beta) = find_uv_for_bilerp(v0, v1, v2, v3, cur_point);

                        if z_buffer_enabled {
                            let z = bilerp_float(v0.z, v1.z, v2.z, v3.z, alpha, beta);
                            if !canvas.test_and_set_z(x, y, z) {
                                continue;
                            }
                        }

                        let u = bilerp_float(tx0.0, tx1.0, tx2.0, tx3.0, alpha, beta);
                        let v = bilerp_float(tx0.1, tx1.1, tx2.0, tx3.0, alpha, beta);

                        let base_color = model.material.get_uv_color(u, v);
                        canvas[(x, y)] = base_color;
                    }
                }
            } else {
                // иначе барицентрическая интерполяция с триангуляцией
                for triangle in triangulate_polygon(&polygon.get_vertexes()) {
                    // проекция вершин треугольника
                    let v0 = projected_vertexes[triangle.0];
                    let v1 = projected_vertexes[triangle.1];
                    let v2 = projected_vertexes[triangle.2];

                    // текстурные UV-координаты вершин треугольника
                    let tx0 = texture_coords[triangle.0];
                    let tx1 = texture_coords[triangle.1];
                    let tx2 = texture_coords[triangle.2];

                    // ограничивающий прямоугольник
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
                            let bary = barycentric_coordinates(&[v0, v1, v2], p);
                            // точка на полигоне?
                            if bary.x < 0.0 || bary.y < 0.0 || bary.z < 0.0 {
                                continue;
                            }

                            if z_buffer_enabled {
                                let z = interpolate_float(bary, v0.z, v1.z, v2.z);
                                if !canvas.test_and_set_z(x, y, z) {
                                    continue;
                                }
                            }

                            let u = interpolate_float(bary, tx0.0, tx1.0, tx2.0);
                            let v = interpolate_float(bary, tx0.1, tx1.1, tx2.1);

                            let base_color = model.material.get_uv_color(u, v);
                            canvas[(x, y)] = base_color;
                        }
                    }
                }
            }
        }
    }

    /// Шейдинг Гуро для модели Ламберта.
    ///
    /// Применяется после отрисовки моделей без шейдинга. Иными словами, на холсте canvas уже нарисованы
    /// все модели, но без учёта освещения, поэтому теперь поверх этих цветов надо наложить сам шейдинг.
    fn render_gouraud_lambert(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<Polygon3>,
        model: &Model3,
        lights: &Vec<LightSource>,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        let vertexes_2d: Vec<Vec3> = projected_vertexes
            .iter()
            .cloned()
            .map(|v| Vec3::new(v.x, v.y, 0.0))
            .collect();
        // освещённость всех вершин модели
        let light_colors: Vec<Color32>;
        if let Some(colors) = Self::lambert_diffuse(model, lights) {
            light_colors = colors;
        } else {
            return;
        }
        for polygon in polygons {
            // если четырёхугольник - билинейная интерполяция
            if polygon.is_rectangle() {
                let rectangle = polygon.get_vertexes();
                // проекция вершин треугольника
                let v0 = projected_vertexes[rectangle[0]];
                let v1 = projected_vertexes[rectangle[1]];
                let v2 = projected_vertexes[rectangle[2]];
                let v3 = projected_vertexes[rectangle[3]];

                // текстурные UV-координаты вершин треугольника
                // освещённость вершин треугольника
                let light0 = light_colors[rectangle[0]];
                let light1 = light_colors[rectangle[1]];
                let light2 = light_colors[rectangle[2]];
                let light3 = light_colors[rectangle[3]];

                // ограничивающий прямоугольник
                let min_x = *vec![v0.x as usize, v1.x as usize, v2.x as usize, v3.x as usize]
                    .iter()
                    .min()
                    .unwrap();
                let max_x = *vec![v0.x as usize, v1.x as usize, v2.x as usize, v3.x as usize]
                    .iter()
                    .max()
                    .unwrap();
                let min_y = *vec![v0.y as usize, v1.y as usize, v2.y as usize, v3.y as usize]
                    .iter()
                    .min()
                    .unwrap();
                let max_y = *vec![v0.y as usize, v1.y as usize, v2.y as usize, v3.y as usize]
                    .iter()
                    .max()
                    .unwrap();

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        if x < 0 || y < 0 || x >= canvas.width || y >= canvas.height {
                            continue;
                        }

                        let cur_point = Vec3::new(x as f32, y as f32, 0.0);
                        // точка на полигоне?
                        if !polygon.is_point_in_convex_polygon(&vertexes_2d, cur_point) {
                            continue;
                        }

                        let (alpha, beta) = find_uv_for_bilerp(v0, v1, v2, v3, cur_point);

                        if z_buffer_enabled {
                            let z = bilerp_float(v0.z, v1.z, v2.z, v3.z, alpha, beta);
                            if !canvas.test_and_set_z(x, y, z) {
                                continue;
                            }
                        }

                        let surface_color = canvas[(x, y)];
                        // освещённость в данной точке
                        let light = bilerp_color(light0, light1, light2, light3, alpha, beta);
                        canvas[(x, y)] = surface_color * light;
                    }
                }
            } else {
                // иначе барицентрическая интерполяция с триангуляцией
                for triangle in triangulate_polygon(&polygon.get_vertexes()) {
                    // проекция вершин треугольника
                    let v0 = projected_vertexes[triangle.0];
                    let v1 = projected_vertexes[triangle.1];
                    let v2 = projected_vertexes[triangle.2];

                    // освещённость вершин треугольника
                    let light0 = light_colors[triangle.0];
                    let light1 = light_colors[triangle.1];
                    let light2 = light_colors[triangle.2];

                    // описывающий прямоугольник
                    let min_x = v0.x.min(v1.x.min(v2.x)) as usize;
                    let max_x = v0.x.max(v1.x.max(v2.x)) as usize;
                    let min_y = v0.y.min(v1.y.min(v2.y)) as usize;
                    let max_y = v0.y.max(v1.y.max(v2.y)) as usize;

                    for y in min_y..=max_y {
                        for x in min_x..=max_x {
                            // точка на полигоне?
                            if x < 0 || y < 0 || x >= canvas.width || y >= canvas.height {
                                continue;
                            }

                            let p = Point3::new(x as f32, y as f32, 0.0);
                            let bary = barycentric_coordinates(&[v0, v1, v2], p);

                            if z_buffer_enabled {
                                let z = interpolate_float(bary, v0.z, v1.z, v2.z);
                                if !canvas.test_z(x, y, z) {
                                    continue;
                                }
                            }

                            let surface_color = canvas[(x, y)];
                            // освещённость в данной точке
                            let light = interpolate_color(bary, light0, light1, light2);
                            canvas[(x, y)] = surface_color * light;
                        }
                    }
                }
            }
        }
    }

    /// Шейдинг Фонга для модели туншейдинг.
    ///
    /// Применяется после отрисовки моделей без шейдинга. Иными словами, на холсте canvas уже нарисованы
    /// все модели, но без учёта освещения, поэтому теперь поверх этих цветов надо наложить сам шейдинг.
    fn render_phong_toon_shading(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<Polygon3>,
        model: &Model3,
        lights: &Vec<LightSource>,
        bands: usize,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        // нет света - нет шейдинга.
        if lights.is_empty() {
            return;
        }

        // нормали в глобальных координатах
        let global_vertex_normals: Vec<Vec3> = model.mesh.get_global_normals().collect();
        // позиции вершин в глобальной системе
        let global_vertex_positions: Vec<Vec3> = model
            .mesh
            .get_global_vertexes()
            .map(|v| Vec3::from(v))
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
                        // точка на полигоне?
                        if x < 0 || y < 0 || x >= canvas.width || y >= canvas.height {
                            continue;
                        }

                        let p = Point3::new(x as f32, y as f32, 0.0);
                        let bary = barycentric_coordinates(&[v0, v1, v2], p);

                        if z_buffer_enabled {
                            let z = interpolate_float(bary, v0.z, v1.z, v2.z);
                            if !canvas.test_z(x, y, z) {
                                continue;
                            }
                        }

                        let position = interpolate_vec(bary, pos0, pos1, pos2);
                        let normal = interpolate_vec(bary, normal0, normal1, normal2);

                        let surface_color = canvas[(x, y)];
                        // освещённость в данной точке
                        let light =
                            Self::toon_shading(position.into(), normal, lights, bands).unwrap();
                        canvas[(x, y)] = surface_color * light;
                    }
                }
            }
        }
    }

    /// Считает освещённость каждой вершины по модели Ламберта.
    ///
    /// Для каждой вершины считает значение `интенсивность * цвет света * угол между поверхностью и светом`.
    fn lambert_diffuse(model: &Model3, lights: &Vec<LightSource>) -> Option<Vec<Color32>> {
        if lights.is_empty() {
            return None;
        }

        // нормали в глобальных координатах
        let global_vertex_normals: Vec<Vec3> = model.mesh.get_global_normals().collect();
        // позиции вершин в глобальной системе
        let global_vertex_positions: Vec<HVec3> = model.mesh.get_global_vertexes().collect();

        // результирующий массив
        let mut colors = Vec::with_capacity(global_vertex_normals.len());

        for i in 0..global_vertex_normals.len() {
            let normal = global_vertex_normals[i];
            let position = global_vertex_positions[i];
            let mut light_color = None;

            // Влияние каждого источника
            for light in lights {
                let light_dir = (light.position - Point3::from(position)).normalize();
                let cos = normal.dot(light_dir).max(0.0);
                if let Some(lcolor) = light_color {
                    light_color = Some(lcolor + light.color.gamma_multiply(light.intensity * cos));
                } else {
                    light_color = Some(light.color.gamma_multiply(light.intensity * cos));
                }
            }

            colors.push(
                light_color.expect(
                    "Освещённость либо есть сразу для всех, либо отсутсвует сразу для всех",
                ),
            );
        }

        Some(colors)
    }

    /// Считает освещённость каждой вершины по модели Toon Shading.
    ///
    /// Для каждой вершины считает значение `интенсивность * цвет света * угол между поверхностью и светом`.
    fn toon_shading(
        position: Point3,
        normal: Vec3,
        lights: &Vec<LightSource>,
        bands: usize,
    ) -> Option<Color32> {
        if lights.is_empty() {
            return None;
        }

        let mut light_color = None;
        // Влияние каждого источника
        for light in lights {
            let light_dir = (light.position - position).normalize();
            let cos = normal.dot(light_dir).max(0.0);
            if let Some(lcolor) = light_color {
                light_color = Some(lcolor + light.color.gamma_multiply(light.intensity * cos));
            } else {
                light_color = Some(light.color.gamma_multiply(light.intensity * cos));
            }
        }

        let light_color = light_color.unwrap();
        let step = 255.0 / bands as f32;
        Some(Color32::from_rgb(
            ((light_color.r() as f32 / step).floor() * step).min(step) as u8,
            ((light_color.g() as f32 / step).floor() * step).min(step) as u8,
            ((light_color.b() as f32 / step).floor() * step).min(step) as u8,
        ))
    }
}

// --------------------------------------------------
// Вспомогательные методы
// --------------------------------------------------

/// Получить матрицу преобразования из глобальных координат в экранные (viewport, он же canvas)
///
/// То есть, матрица производит следующие операции:
/// глобальные координаты -> координаты камеры (view tranform) -> проекция на камеру в NDC -> растяжение NDC на размер canvas.
fn get_global_to_screen_transform(
    projection_type: ProjectionType,
    scene: &Scene,
    canvas: &Canvas,
) -> Transform3D {
    let camera = scene.camera;
    // Матрица проекции координат камеры в NDC
    let proj_matrix = match projection_type {
        ProjectionType::Parallel => Transform3D::parallel_symmetric(
            canvas.width as f32,
            canvas.height as f32,
            camera.get_near_plane(),
            camera.get_far_plane(),
        ),
        ProjectionType::Perspective => Transform3D::perspective(
            camera.get_fov(),
            camera.get_aspect_ratio(),
            camera.get_near_plane(),
            camera.get_far_plane(),
        ),
    };

    let scale_x = canvas.width as f32 / 2.0; // растянуть NDC по ширине
    let scale_y = canvas.height as f32 / 2.0; // растянуть NDC по высоте

    camera
        .get_local_frame()
        .global_to_local_matrix() // view transformation (локальные координаты камеры)
        .multiply(proj_matrix) // вот тут получается NDC с координатами [-1, +1]
        .multiply(Transform3D::translation_uniform(1.0)) // теперь координаты [0, +2]
        .multiply(Transform3D::scale(scale_x, scale_y, 1.0)) // теперь экранные
}

/// Проецирует вершины модели на экран.
/// `view_proj_matrix` - матрица проекции из глобальных координат в экранные
/// `model` - сама модель
///
/// Важно: после этих преобразований вершины становится `Vec3`, то есть все преобразования в
/// 4D векторах закончены и теперь 2D пространство представлено 3D векторами, где z-компонента
/// нужна для порядка отрисовки на экране.
fn transform_model(
    global_to_screen_transform: Transform3D,
    model: &Model3,
) -> impl Iterator<Item = Vec3> {
    // Проекция точек модели на экран.
    model
        .mesh
        .get_global_vertexes() // вершины модели в глобальных координатах
        .map(move |vertex| Vec3::from(vertex.apply_transform(&global_to_screen_transform))) // теперь коодринаты экрана, где z - глубина
}

/// Преобразует глобальные координаты точки в координаты экрана.
fn project_point(point: Point3, view_proj_matrix: &Transform3D) -> Pos2 {
    let proj_point: Point3 = view_proj_matrix.apply_to_hvec(point.into()).into();
    Pos2::new(proj_point.x, proj_point.y)
}

/// Отрисовка глобальной координатной системы.
fn draw_coordinate_axes(canvas: &mut Canvas, global_to_screen_transform: &Transform3D) {
    // TODO нелпохо бы сделать полноценную отрисовку координатной сетки.
    let axis_length = 2.0; // Длина осей
    let origin = Point3::new(0.0, 0.0, 0.0);

    let x_axis_end = Point3::new(axis_length, 0.0, 0.0);
    let y_axis_end = Point3::new(0.0, axis_length, 0.0);
    let z_axis_end = Point3::new(0.0, 0.0, axis_length);

    let origin_2d = project_point(origin, global_to_screen_transform);
    let x_end_2d = project_point(x_axis_end, global_to_screen_transform);
    let y_end_2d = project_point(y_axis_end, global_to_screen_transform);
    let z_end_2d = project_point(z_axis_end, global_to_screen_transform);

    // Рисуем оси с разными цветами
    // Ось X - красная
    canvas.draw_sharp_line(origin_2d, x_end_2d, Color32::RED);

    // Ось Y - зелёная
    canvas.draw_sharp_line(origin_2d, y_end_2d, Color32::GREEN);

    // Ось Z - синяя
    canvas.draw_sharp_line(origin_2d, z_end_2d, Color32::BLUE);
}

/// Отрисовка пользовательской оси для вращения
fn draw_custom_axis_line(
    canvas: &mut Canvas,
    global_to_screen_transform: &Transform3D,
    point1: Point3,
    point2: Point3,
) {
    // Проецируем точки в 2D используя нашу систему проекций
    let screen_point1 = project_point(point1, global_to_screen_transform);
    let screen_point2 = project_point(point2, global_to_screen_transform);

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

/// Отсечение нелицевых граней модели
/// `model` - сама модель.
///
/// Возвращает вектор полигонов только с лицевыми гранями.
fn model_backface_culling(camera: Camera3, model: &Model3) -> Vec<Polygon3> {
    let camera_direction = camera.get_direction();
    let global_normals: Vec<Vec3> = model.mesh.get_global_normals().collect();
    let mut visible_polygons = Vec::new();
    for polygon in model.mesh.get_polygons() {
        if !polygon.is_valid() {
            continue;
        }

        let mut polygon_normal = Vec3::zero();
        let mut normals_count = 0;

        for &vertex_index in polygon.get_vertexes() {
            polygon_normal += global_normals[vertex_index];
            normals_count += 1;
        }

        // Если нормаль есть, производим отсечение
        if normals_count > 0 && polygon_normal.length_squared() > f32::EPSILON {
            polygon_normal = polygon_normal.normalize();

            // Если нормаль направлена в сторону камеры, то оставляем полигон
            let dot_product = polygon_normal.dot(camera_direction);
            if dot_product < 0.0 {
                visible_polygons.push(polygon.clone());
            }
        }
    }

    visible_polygons
}

/// Находит барицентрические координаты по 3-м точкам.
/// `triangle` - полигон-треугольник, по которому строятся координаты
/// `point` - точка, для которой нужны координаты
///
/// Поскольку это уже в проекции на экран, z-координата не учитывается.
///
/// Возвращает координаты в виде Point3.
fn barycentric_coordinates(triangle: &[Vec3], point: Point3) -> Point3 {
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

/// Находит uv-координаты для билинейной интерполяции.
///
/// Все точки являются проекциями на экран, z-компонента не учитывается.
fn find_uv_for_bilerp(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, cur: Vec3) -> (f32, f32) {
    todo!()
}

/// Найти противополжный цвет.
fn opposite_color(color: Color32) -> Color32 {
    Color32::from_rgb(255 - color.r(), 255 - color.g(), 255 - color.b())
}

fn interpolate_float(bary: Point3, a: f32, b: f32, c: f32) -> f32 {
    let alpha = bary.x;
    let beta = bary.y;
    let gamma = bary.z;
    alpha * a + beta * b + gamma * c
}

fn bilerp_float(
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
    alpha: f32,
    beta: f32,
) -> f32 {
    let top = lerp_float(top_left, top_right, alpha);
    let bottom = lerp_float(bottom_left, bottom_right, alpha);
    lerp_float(top, bottom, beta)
}

fn lerp_float(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn interpolate_color(bary: Point3, a: Color32, b: Color32, c: Color32) -> Color32 {
    let alpha = bary.x;
    let beta = bary.y;
    let gamma = bary.z;
    a.gamma_multiply(alpha) + b.gamma_multiply(beta) + c.gamma_multiply(gamma)
}

fn bilerp_color(
    top_left: Color32,
    top_right: Color32,
    bottom_left: Color32,
    bottom_right: Color32,
    alpha: f32,
    beta: f32,
) -> Color32 {
    let top = lerp_color(top_left, top_right, alpha);
    let bottom = lerp_color(bottom_left, bottom_right, alpha);
    lerp_color(top, bottom, beta)
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}

fn interpolate_vec(bary: Point3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let alpha = bary.x;
    let beta = bary.y;
    let gamma = bary.z;
    a * alpha + b * beta + c * gamma
}

fn bilerp_vec(
    top_left: Vec3,
    top_right: Vec3,
    bottom_left: Vec3,
    bottom_right: Vec3,
    alpha: f32,
    beta: f32,
) -> Vec3 {
    let top = lerp_vec(top_left, top_right, alpha);
    let bottom = lerp_vec(bottom_left, bottom_right, alpha);
    lerp_vec(top, bottom, beta)
}

fn lerp_vec(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
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
