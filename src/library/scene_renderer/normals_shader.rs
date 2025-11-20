use crate::{
    Point3, Vec3,
    classes3d::{
        mesh::Polygon3,
        scene_renderer::{Shader, shader_utils},
    },
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
        model: &crate::Model3,
        polygons: &Vec<Polygon3>,
        global_to_screen_transform: crate::Transform3D,
        _lights: &Vec<crate::LightSource>,
        canvas: &mut crate::Canvas,
    ) {
        let global_normals: Vec<Vec3> = model.mesh.get_global_normals().collect();
        let global_positions: Vec<Vec3> = model
            .mesh
            .get_global_vertexes()
            .map(|v| Vec3::from(v))
            .collect();

        // индексы используемых нормалей
        let mut indexes: Vec<usize> = polygons
            .iter()
            .flat_map(|polygon| polygon.get_vertexes().clone())
            .collect();
        indexes.sort();
        indexes.dedup();

        // рисуем нормали вершин
        for index in indexes {
            let origin = global_positions[index];
            let direction = global_normals[index];

            let start = Point3::from(origin);
            shader_utils::render_line(
                global_to_screen_transform,
                start,
                start + direction,
                self.vertex_normal_color,
                canvas,
            );
        }

        // рисуем нормали полигонов
        for polygon in polygons {
            let mut polygon_normal = Vec3::zero();
            let mut polygon_pos = Vec3::zero();
            for &index in polygon.get_vertexes() {
                polygon_normal += global_normals[index];
                polygon_pos += global_normals[index];
            }
            polygon_normal /= polygon.get_vertexes().len() as f32;
            polygon_pos /= polygon.get_vertexes().len() as f32;
            shader_utils::render_line(
                global_to_screen_transform,
                polygon_pos.into(),
                (polygon_pos + polygon_normal).into(),
                self.polygon_normal_color,
                canvas,
            );
        }
    }
}
