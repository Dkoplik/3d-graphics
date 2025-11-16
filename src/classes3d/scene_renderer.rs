use std::fmt::Display;

use crate::{
    Camera3, Canvas, HVec3, LightSource, Material, Model3, Point3, Scene, SceneRenderer,
    Transform3D, Vec3, classes3d::mesh::Polygon3,
};
use egui::{Color32, Pos2};

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

impl Display for ProjectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parallel => f.write_str("Параллельная"),
            Self::Perspective => f.write_str("Перспективная"),
        }
    }
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

impl Display for ShadingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_str("Отсутсвует"),
            Self::GouraudLambert => f.write_str("Гуро для модели Ламберта"),
            Self::PhongToonShading(_) => f.write_str("Фонга для модели туншейдинг"),
        }
    }
}

impl Default for SceneRenderer {
    fn default() -> Self {
        Self {
            render_wireframe: true,
            render_solid: false,
            projection_type: Default::default(),
            shading_type: Default::default(),
            backface_culling: false,
            z_buffer_enabled: true,
        }
    }
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
            self::get_global_to_screen_transform(self.projection_type, scene, &canvas);

        // Отрисовка глобальной координатной системы.
        self::draw_coordinate_axes(canvas, global_to_screen_transform);

        // Отрисовка пользовательской оси вращения, если имеется
        if show_custom_axis {
            self::draw_custom_axis_line(
                canvas,
                global_to_screen_transform,
                axis_point1,
                axis_point2,
            );
        }

        // количество отрисованных полигонов.
        let mut polygon_count: usize = 0;

        // отрисовка моделей
        for model in &scene.models {
            // Проекция вершин модели
            let projected_vertexes: Vec<Vec3> =
                transform_model(global_to_screen_transform, model).collect();

            // Полигоны к отрисовке
            let polygons = if self.backface_culling {
                self::model_backface_culling(scene.camera, model)
            } else {
                model.mesh.get_polygons().cloned().collect()
            };

            polygon_count = polygons.len();

            // заполнить модель
            if self.render_solid {
                // заполнение без шейдинга
                self.render_solid(
                    &projected_vertexes,
                    &polygons,
                    model,
                    canvas,
                    self.z_buffer_enabled,
                );

                // шейдинг, если имеется
                match self.shading_type {
                    ShadingType::None => (),
                    ShadingType::GouraudLambert => {
                        self.render_gouraud_lambert(
                            &projected_vertexes,
                            &polygons,
                            model,
                            &scene.lights,
                            canvas,
                            self.z_buffer_enabled,
                        );
                    }
                    ShadingType::PhongToonShading(bands) => {
                        self.render_phong_toon_shading(
                            &projected_vertexes,
                            &polygons,
                            model,
                            &scene.lights,
                            bands,
                            canvas,
                            self.z_buffer_enabled,
                        );
                    }
                };
            }

            // каркас модели
            if self.render_wireframe {
                self.render_model_wireframe(
                    &projected_vertexes,
                    &polygons,
                    &model.material,
                    canvas,
                );
            }
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
        material: &Material,
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
                        if x >= canvas.width || y >= canvas.height {
                            continue;
                        }

                        let cur_point = Point3::new(x as f32, y as f32, 0.0);
                        if let Some((alpha, beta)) = find_uv_for_bilerp(
                            v0.into(),
                            v1.into(),
                            v2.into(),
                            v3.into(),
                            cur_point,
                        ) {
                            if alpha < 0.0 || beta < 0.0 {
                                continue;
                            }
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
                            if x >= canvas.width || y >= canvas.height {
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
                        if x >= canvas.width || y >= canvas.height {
                            continue;
                        }

                        let cur_point = Point3::new(x as f32, y as f32, 0.0);
                        if let Some((alpha, beta)) = find_uv_for_bilerp(
                            v0.into(),
                            v1.into(),
                            v2.into(),
                            v3.into(),
                            cur_point,
                        ) {
                            if alpha < 0.0 || beta < 0.0 {
                                continue;
                            }

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
                            if x >= canvas.width || y >= canvas.height {
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
            // если четырёхугольник - билинейная интерполяция
            if polygon.is_rectangle() {
                let rectangle = polygon.get_vertexes();
                // проекция вершин треугольника
                let v0 = projected_vertexes[rectangle[0]];
                let v1 = projected_vertexes[rectangle[1]];
                let v2 = projected_vertexes[rectangle[2]];
                let v3 = projected_vertexes[rectangle[3]];

                // глобальные нормали вершин треугольника
                let normal0 = global_vertex_normals[rectangle[0]];
                let normal1 = global_vertex_normals[rectangle[1]];
                let normal2 = global_vertex_normals[rectangle[2]];
                let normal3 = global_vertex_normals[rectangle[3]];

                // глобальные позиции вершин треугольника
                let pos0 = global_vertex_positions[rectangle[0]];
                let pos1 = global_vertex_positions[rectangle[1]];
                let pos2 = global_vertex_positions[rectangle[2]];
                let pos3 = global_vertex_positions[rectangle[3]];

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
                        if x >= canvas.width || y >= canvas.height {
                            continue;
                        }

                        let cur_point = Point3::new(x as f32, y as f32, 0.0);
                        if let Some((alpha, beta)) = find_uv_for_bilerp(
                            v0.into(),
                            v1.into(),
                            v2.into(),
                            v3.into(),
                            cur_point,
                        ) {
                            if alpha < 0.0 || beta < 0.0 {
                                continue;
                            }

                            if z_buffer_enabled {
                                let z = bilerp_float(v0.z, v1.z, v2.z, v3.z, alpha, beta);
                                if !canvas.test_and_set_z(x, y, z) {
                                    continue;
                                }
                            }

                            let position = bilerp_vec(pos0, pos1, pos2, pos3, alpha, beta);
                            let normal =
                                bilerp_vec(normal0, normal1, normal2, normal3, alpha, beta);

                            let surface_color = canvas[(x, y)];
                            // освещённость в данной точке
                            let light =
                                Self::toon_shading(position.into(), normal, lights, bands).unwrap();
                            canvas[(x, y)] = surface_color * light;
                        }
                    }
                }
            } else {
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
                            if x >= canvas.width || y >= canvas.height {
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
        let step = 256.0 / bands as f32;
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
fn project_point(point: Point3, view_proj_matrix: Transform3D) -> Pos2 {
    let proj_point: Point3 = view_proj_matrix.apply_to_hvec(point.into()).into();
    Pos2::new(proj_point.x, proj_point.y)
}

/// Отрисовка глобальной координатной системы.
fn draw_coordinate_axes(canvas: &mut Canvas, global_to_screen_transform: Transform3D) {
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
    global_to_screen_transform: Transform3D,
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
fn find_uv_for_bilerp(
    p0: Point3,
    p1: Point3,
    p2: Point3,
    p3: Point3,
    cur: Point3,
) -> Option<(f32, f32)> {
    let p0p1 = p1 - p0;
    let p0p3 = p1 - p3;
    let det = p0p3.x * p0p1.y - p0p3.y * p0p1.x;
    if det.abs() <= f32::EPSILON {
        return None;
    }
    let det_u = (cur.x - p0.x) * p0p1.y - (cur.y - p0.y) * p0p1.x;
    let det_v = p0p3.x * (cur.y - p0.y) - p0p3.y * (cur.x - p0.x);
    Some((det_u / det, det_v / det))
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
