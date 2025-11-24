use crate::{
    Camera, Canvas, LightSource, Model, Point3, Polygon, ProjectionType, Shader, library::utils,
};

pub struct WireframeShader;

impl WireframeShader {
    pub fn new() -> Self {
        Self
    }
}

impl Shader for WireframeShader {
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
        // выбираем цвет для каркаса (чтобы потом не сливался с основной моделью)
        let model_color = model.material.color;
        let wireframe_color = utils::opposite_color(model_color);

        // проекция вершин на экран
        let projected_vertexes: Vec<Point3> = model
            .mesh
            .get_global_vertex_iter()
            .map(|v| v.apply_transform(global_to_screen_transform).unwrap())
            .collect();

        // Рисуем рёбра
        for polygon in polygons {
            // Вершины полигона
            let indexes: Vec<usize> = polygon.get_mesh_vertex_index_iter().collect();
            let vertexes: Vec<Point3> = indexes
                .iter()
                .map(|&index| projected_vertexes[index])
                .collect();

            // рисуем рёбра полигона
            for i in 0..vertexes.len() {
                let start = vertexes[i];
                let end = vertexes[(i + 1) % vertexes.len()];

                let start_pos = egui::Pos2::new(start.x, start.y);
                let end_pos = egui::Pos2::new(end.x, end.y);
                canvas.draw_sharp_line(start_pos, end_pos, wireframe_color);
            }

            // рисуем вершины полигона
            for i in 0..vertexes.len() {
                let vertex = vertexes[i];
                let pos = egui::Pos2::new(vertex.x, vertex.y);
                canvas.circle_filled(pos, 3.0, wireframe_color);
            }
        }
    }
}
