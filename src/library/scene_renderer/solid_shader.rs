use crate::{
    Camera, Canvas, LightSource, Model, Point3, Polygon, ProjectionType, Shader, library::utils,
};

pub struct SolidShader {
    z_buffer_enabled: bool,
}

impl SolidShader {
    pub fn new(z_buffer_enabled: bool) -> Self {
        Self { z_buffer_enabled }
    }
}

impl Shader for SolidShader {
    fn shade_model(
        &self,
        model: &Model,
        polygons: &Vec<Polygon>,
        camera: &Camera,
        projection_type: ProjectionType,
        _lights: &Vec<LightSource>,
        canvas: &mut Canvas,
    ) {
        // матрица преобразования на экран
        let global_to_screen_transform = camera.global_to_screen_transform(projection_type, canvas);
        // проекция вершин на экран
        let projected_vertexes: Vec<Point3> = model
            .mesh
            .get_global_vertex_iter()
            .map(|v| v.apply_transform(global_to_screen_transform).unwrap())
            .collect();

        // отрисовка каждого полигона
        for polygon in polygons {
            // если четырёхугольник - билинейная интерполяция
            if polygon.is_quad() {
                // индексы вершин
                let i0 = polygon.get_mesh_vertex_index(0);
                let i1 = polygon.get_mesh_vertex_index(1);
                let i2 = polygon.get_mesh_vertex_index(2);
                let i3 = polygon.get_mesh_vertex_index(3);

                // проекция вершин треугольника
                let v0 = projected_vertexes[i0];
                let v1 = projected_vertexes[i1];
                let v2 = projected_vertexes[i2];
                let v3 = projected_vertexes[i3];

                // текстурные UV-координаты вершин треугольника
                let tx0 = model.mesh.get_texture_coord(i0).unwrap();
                let tx1 = model.mesh.get_texture_coord(i1).unwrap();
                let tx2 = model.mesh.get_texture_coord(i2).unwrap();
                let tx3 = model.mesh.get_texture_coord(i3).unwrap();

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
                        if x >= canvas.width() || y >= canvas.height() {
                            continue;
                        }

                        let cur_point = Point3::new(x as f32, y as f32, 0.0);
                        if let Some((alpha, beta)) = utils::find_uv_for_bilerp(
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

                            // z-буфер, если есть
                            if self.z_buffer_enabled {
                                let z = utils::bilerp_float(v0.z, v1.z, v2.z, v3.z, alpha, beta);
                                if !canvas.test_and_set_z(x, y, z) {
                                    continue;
                                }
                            }

                            // текстурные координаты пикселя
                            let u = utils::bilerp_float(tx0.0, tx1.0, tx2.0, tx3.0, alpha, beta);
                            let v = utils::bilerp_float(tx0.1, tx1.1, tx2.0, tx3.0, alpha, beta);

                            let base_color = model.material.get_uv_color(u, v);
                            canvas[(x, y)] = base_color;
                        }
                    }
                }
            } else {
                // иначе барицентрическая интерполяция с триангуляцией
                let indexes: Vec<usize> = polygon.get_mesh_vertex_index_iter().collect();
                for triangle in utils::triangulate_polygon(&indexes) {
                    // индексы вершин
                    let i0 = triangle[0];
                    let i1 = triangle[1];
                    let i2 = triangle[2];

                    // проекция вершин треугольника
                    let v0 = projected_vertexes[i0];
                    let v1 = projected_vertexes[i1];
                    let v2 = projected_vertexes[i2];

                    // текстурные UV-координаты вершин треугольника
                    let tx0 = model.mesh.get_texture_coord(i0).unwrap();
                    let tx1 = model.mesh.get_texture_coord(i1).unwrap();
                    let tx2 = model.mesh.get_texture_coord(i2).unwrap();

                    // ограничивающий прямоугольник
                    let min_x = v0.x.min(v1.x.min(v2.x)) as usize;
                    let max_x = v0.x.max(v1.x.max(v2.x)) as usize;
                    let min_y = v0.y.min(v1.y.min(v2.y)) as usize;
                    let max_y = v0.y.max(v1.y.max(v2.y)) as usize;

                    for y in min_y..=max_y {
                        for x in min_x..=max_x {
                            if x >= canvas.width() || y >= canvas.height() {
                                continue;
                            }

                            let p = Point3::new(x as f32, y as f32, 0.0);
                            let bary = utils::barycentric_coordinates(&[v0, v1, v2], p);

                            // точка на полигоне?
                            if bary.x < 0.0 || bary.y < 0.0 || bary.z < 0.0 {
                                continue;
                            }

                            // z-буфер, если есть
                            if self.z_buffer_enabled {
                                let z = utils::interpolate_float(bary, v0.z, v1.z, v2.z);
                                if !canvas.test_and_set_z(x, y, z) {
                                    continue;
                                }
                            }

                            // текстурные коодринаты пикселя
                            let u = utils::interpolate_float(bary, tx0.0, tx1.0, tx2.0);
                            let v = utils::interpolate_float(bary, tx0.1, tx1.1, tx2.1);

                            let base_color = model.material.get_uv_color(u, v);
                            canvas[(x, y)] = base_color;
                        }
                    }
                }
            }
        }
    }
}
