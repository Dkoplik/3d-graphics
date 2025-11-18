use crate::Model3;
use crate::classes3d::scene_renderer::shader_utils;
use crate::{Canvas, LightSource, Transform3D, Vec3, classes3d::scene_renderer::Shader};

pub struct WireframeShader;

impl WireframeShader {
    pub fn new() -> Self {
        Self
    }
}

impl Shader for WireframeShader {
    fn shade_model(
        &self,
        model: &Model3,
        global_to_screen_transform: Transform3D,
        _lights: &Vec<LightSource>,
        canvas: &mut Canvas,
    ) {
        // выбираем цвет для каркаса (чтобы потом не сливался с основной моделью)
        let model_color = model.material.color;
        let wireframe_color = shader_utils::opposite_color(model_color);

        // проекция вершин на экран
        let projected_vertexes: Vec<Vec3> = model
            .mesh
            .get_global_vertexes()
            .map(|v| Vec3::from(v * global_to_screen_transform))
            .collect();

        // Рисуем рёбра
        for polygon in model.mesh.get_polygons() {
            // Вершины полигона
            let points: Vec<Vec3> = polygon
                .get_vertexes()
                .iter()
                .map(|&index| projected_vertexes[index])
                .collect();

            // рёбра полигона
            for i in 0..points.len() {
                let start = points[i];
                let end = points[(i + 1) % points.len()];

                let start_pos = egui::Pos2::new(start.x, start.y);
                let end_pos = egui::Pos2::new(end.x, end.y);
                canvas.draw_sharp_line(start_pos, end_pos, wireframe_color);
            }
        }

        // Рисуем вершины
        for vertex in projected_vertexes {
            let pos = egui::Pos2::new(vertex.x, vertex.y);
            canvas.circle_filled(pos, 3.0, wireframe_color);
        }
    }
}
