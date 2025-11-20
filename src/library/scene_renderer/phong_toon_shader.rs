use crate::{
    LightSource, Point3, Vec3,
    classes3d::{
        mesh::Polygon3,
        scene_renderer::{Shader, shader_utils},
    },
};

pub struct PhongToonShading {
    z_buffer_enabled: bool,
    bands: usize,
}

impl PhongToonShading {
    pub fn new(z_buffer_enabled: bool, bands: usize) -> Self {
        Self {
            z_buffer_enabled,
            bands,
        }
    }

    /// Считает освещённость каждой вершины по модели Toon Shading.
    ///
    /// Для каждой вершины считает значение `интенсивность * цвет света * угол между поверхностью и светом`.
    fn toon_shading(
        position: Point3,
        normal: Vec3,
        lights: &Vec<LightSource>,
        bands: usize,
    ) -> egui::Color32 {
        if lights.is_empty() {
            return egui::Color32::BLACK;
        }

        let mut light_color = egui::Color32::BLACK;
        // Влияние каждого источника
        for light in lights {
            let light_dir = (light.position - position).normalize();
            let cos = normal.dot(light_dir).max(0.0);
            light_color = light_color + light.color.gamma_multiply(light.intensity * cos);
        }
        let step = 255.0 / bands as f32;
        egui::Color32::from_rgb(
            ((light_color.r() as f32 / step).floor() * step).min(step) as u8,
            ((light_color.g() as f32 / step).floor() * step).min(step) as u8,
            ((light_color.b() as f32 / step).floor() * step).min(step) as u8,
        )
    }
}

impl Shader for PhongToonShading {
    fn shade_model(
        &self,
        model: &crate::Model3,
        polygons: &Vec<Polygon3>,
        global_to_screen_transform: crate::Transform3D,
        lights: &Vec<crate::LightSource>,
        canvas: &mut crate::Canvas,
    ) {
        // проекция вершин на экран
        let projected_vertexes: Vec<Vec3> = model
            .mesh
            .get_global_vertexes()
            .map(|v| Vec3::from(v * global_to_screen_transform))
            .collect();
        // текстурные координаты модели
        let texture_coords = model.mesh.get_texture_coords();
        // глобальные позиции вершин
        let global_vertex_poses: Vec<Vec3> = model
            .mesh
            .get_global_vertexes()
            .map(|v| Vec3::from(v))
            .collect();
        // глобальные нормали
        let global_normals: Vec<Vec3> = model.mesh.get_global_normals().collect();

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

                // глобальные нормали вершин треугольника
                let normal0 = global_normals[rectangle[0]];
                let normal1 = global_normals[rectangle[1]];
                let normal2 = global_normals[rectangle[2]];
                let normal3 = global_normals[rectangle[3]];

                // глобальные позиции вершин треугольника
                let pos0 = global_vertex_poses[rectangle[0]];
                let pos1 = global_vertex_poses[rectangle[1]];
                let pos2 = global_vertex_poses[rectangle[2]];
                let pos3 = global_vertex_poses[rectangle[3]];

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
                        if let Some((alpha, beta)) = shader_utils::find_uv_for_bilerp(
                            v0.into(),
                            v1.into(),
                            v2.into(),
                            v3.into(),
                            cur_point,
                        ) {
                            // точка за границами полигона
                            if alpha < 0.0 || 1.0 < alpha || beta < 0.0 || 1.0 < beta {
                                continue;
                            }

                            if self.z_buffer_enabled {
                                let z =
                                    shader_utils::bilerp_float(v0.z, v1.z, v2.z, v3.z, alpha, beta);
                                if !canvas.test_and_set_z(x, y, z) {
                                    continue;
                                }
                            }

                            let position =
                                shader_utils::bilerp_vec(pos0, pos1, pos2, pos3, alpha, beta);
                            let normal = shader_utils::bilerp_vec(
                                normal0, normal1, normal2, normal3, alpha, beta,
                            );

                            // текстурные координаты пикселя
                            let u =
                                shader_utils::bilerp_float(tx0.0, tx1.0, tx2.0, tx3.0, alpha, beta);
                            let v =
                                shader_utils::bilerp_float(tx0.1, tx1.1, tx2.0, tx3.0, alpha, beta);
                            let base_color = model.material.get_uv_color(u, v);

                            // освещённость в данной точке
                            let light =
                                Self::toon_shading(position.into(), normal, lights, self.bands);
                            canvas[(x, y)] = base_color * light;
                        }
                    }
                }
            } else {
                for triangle in shader_utils::triangulate_polygon(&polygon.get_vertexes()) {
                    // проекция вершин треугольника
                    let v0 = projected_vertexes[triangle[0]];
                    let v1 = projected_vertexes[triangle[1]];
                    let v2 = projected_vertexes[triangle[2]];

                    // текстурные UV-координаты вершин треугольника
                    let tx0 = texture_coords[triangle[0]];
                    let tx1 = texture_coords[triangle[1]];
                    let tx2 = texture_coords[triangle[2]];

                    // глобальные нормали вершин треугольника
                    let normal0 = global_normals[triangle[0]];
                    let normal1 = global_normals[triangle[1]];
                    let normal2 = global_normals[triangle[2]];

                    // глобальные позиции вершин треугольника
                    let pos0 = global_vertex_poses[triangle[0]];
                    let pos1 = global_vertex_poses[triangle[1]];
                    let pos2 = global_vertex_poses[triangle[2]];

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
                            let bary = shader_utils::barycentric_coordinates(&[v0, v1, v2], p);

                            if self.z_buffer_enabled {
                                let z = shader_utils::interpolate_float(bary, v0.z, v1.z, v2.z);
                                if !canvas.test_z(x, y, z) {
                                    continue;
                                }
                            }

                            let position = shader_utils::interpolate_vec(bary, pos0, pos1, pos2);
                            let normal =
                                shader_utils::interpolate_vec(bary, normal0, normal1, normal2);

                            // текстурные коодринаты пикселя
                            let u = shader_utils::interpolate_float(bary, tx0.0, tx1.0, tx2.0);
                            let v = shader_utils::interpolate_float(bary, tx0.1, tx1.1, tx2.1);
                            let base_color = model.material.get_uv_color(u, v);

                            // освещённость в данной точке
                            let light =
                                Self::toon_shading(position.into(), normal, lights, self.bands);
                            canvas[(x, y)] = base_color * light;
                        }
                    }
                }
            }
        }
    }
}
