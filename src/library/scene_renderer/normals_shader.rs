use crate::{
    Camera, Canvas, LightSource, Model, Point3, Polygon, ProjectionType, Shader, UVec3, Vec3,
    library::utils,
};

pub struct NormalsShader {
    vertex_normal_color: egui::Color32,
    polygon_normal_color: egui::Color32,
}

impl NormalsShader {
    pub fn new() -> Self {
        Self {
            vertex_normal_color: egui::Color32::PURPLE,
            polygon_normal_color: egui::Color32::ORANGE,
        }
    }
}

impl Shader for NormalsShader {
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
        let global_normals: Vec<UVec3> = model.mesh.get_global_normals_iter().unwrap().collect();
        let global_positions: Vec<Point3> = model.mesh.get_global_vertex_iter().collect();

        // индексы используемых нормалей
        let mut indexes: Vec<usize> = polygons
            .iter()
            .flat_map(|polygon| {
                let indexes: Vec<usize> = polygon.get_mesh_vertex_index_iter().collect();
                indexes
            })
            .collect();
        indexes.sort();
        indexes.dedup();

        // рисуем нормали вершин
        for index in indexes {
            let origin = global_positions[index];
            let direction = global_normals[index];

            let start = Point3::from(origin);
            utils::render_line(
                global_to_screen_transform,
                start,
                start + direction,
                self.vertex_normal_color,
                canvas,
            );
        }

        // рисуем нормали полигонов
        for polygon in polygons {
            let mut polygon_normal: Vec3 = Vec3::zero();
            let mut polygon_pos = Point3::zero();
            let indexes: Vec<usize> = polygon.get_mesh_vertex_index_iter().collect();
            for index in indexes {
                polygon_normal += global_normals[index];
                polygon_pos += global_normals[index];
            }
            polygon_normal /= polygon.vertex_count() as f32;
            polygon_pos = (Vec3::from(polygon_pos) / polygon.vertex_count() as f32).into();
            utils::render_line(
                global_to_screen_transform,
                polygon_pos.into(),
                (polygon_pos + polygon_normal).into(),
                self.polygon_normal_color,
                canvas,
            );
        }
    }
}
