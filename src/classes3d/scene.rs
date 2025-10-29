use egui::Painter;

use crate::{Camera3, Model3, RenderStyle, Scene, Transformable3, Point3};

impl Scene {
    /// Создать пустую сцену.
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    /// Добавить новую модель на сцену.
    pub fn add_model(&mut self, model: Model3) {
        self.models.push(model);
    }

    //Нарисовать сцену на экран со всеми нужными преобразованиями.
    //pub fn render(&self, camera: Camera3, painter: &mut Painter, style: &RenderStyle) {
    // Изменить на &mut
    //   self.models
    //       .iter()
    //      .cloned()
    //     .map(|model| {
    //         model
    //              .to_world_coordinates()
    //              .transform(camera.view_projection_matrix())
    //     })
    //      .for_each(|model| model.draw(painter, style));
    // }
    pub fn render(&self, camera: Camera3, painter: &mut Painter, style: &RenderStyle) {
        for model in &self.models {
            self.render_simple(model, painter, style, &camera);
        }
    }

    fn render_simple(&self, model: &Model3, painter: &mut Painter, style: &RenderStyle, camera: &Camera3) {
        let projected_points: Vec<egui::Pos2> = model
            .get_vertexes()
            .iter()
            .map(|vertex| {
                // Применяем мировое преобразование модели
                let world_vertex = *vertex + model.get_origin().into();
                
                self.project_point(world_vertex, camera)
            })
            .collect();

        // Рисуем рёбра
        for polygon in model.get_polygons() {
            if polygon.vertexes.len() >= 2 {
                let points: Vec<egui::Pos2> = polygon
                    .vertexes
                    .iter()
                    .map(|&index| projected_points[index])
                    .collect();

                for i in 0..points.len() {
                    let start = points[i];
                    let end = points[(i + 1) % points.len()];
                    painter.line_segment([start, end], (style.edge_width, style.edge_color));
                }
            }
        }

        // Рисуем вершины
        for &point in &projected_points {
            painter.circle_filled(point, style.vertex_radius, style.vertex_color);
        }
    }

    /// Проецирование 3D точки в 2D в зависимости от типа проекции
    fn project_point(&self, point: Point3, camera: &Camera3) -> egui::Pos2 {
        match camera.projection_type {
            crate::ProjectionType::Perspective => {
                // Перспективная проекция
                let distance = 5.0; // Расстояние от камеры
                let scale = 100.0;
                let center_x = 450.0;
                let center_y = 300.0;
                
                let factor = distance / (distance + point.z);
                let x_proj = point.x * factor;
                let y_proj = point.y * factor;
                
                egui::Pos2::new(
                    center_x + x_proj * scale,
                    center_y - y_proj * scale,
                )
            }
            crate::ProjectionType::Isometric => {
                // Изометрическая проекция
                let scale = 80.0;
                let center_x = 450.0;
                let center_y = 300.0;
                
                // Стандартная изометрия: углы 30° для X и Z
                let x_proj = point.x * 0.866 - point.z * 0.866; // cos(30°) = 0.866
                let y_proj = point.y + (point.x + point.z) * 0.5; // sin(30°) = 0.5
                
                egui::Pos2::new(
                    center_x + x_proj * scale,
                    center_y - y_proj * scale,
                )
            }
            _ => {
                // По умолчанию - ортографическая проекция
                let scale = 100.0;
                let center_x = 450.0;
                let center_y = 300.0;
                
                egui::Pos2::new(
                    center_x + point.x * scale,
                    center_y - point.y * scale,
                )
            }
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

// Создать сцену из модели
impl From<Model3> for Scene {
    fn from(value: Model3) -> Self {
        Self {
            models: vec![value],
        }
    }
}
