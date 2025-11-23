use std::fmt::Display;

use crate::{
    Camera, Canvas, LightSource, Model, Point3, Polygon, ProjectionType, Scene, Transform3D, UVec3,
    Vec3, library::utils,
};
use egui::{Color32, Pos2};

mod gouraud_lambert_shader;
mod normals_shader;
mod phong_toon_shader;
mod solid_shader;
mod wireframe_shader;

pub trait Shader {
    /// Применить шейдинг к модели.
    ///
    /// `model` - модель, к которой применяется шейдинг;
    /// `polygons` - набор полигонов к отрисовке;
    /// `camera` - камера, на которую присходит проекция;
    /// `lights` - освещение на сцене;
    /// `canvas` - холст, на котором отрисовывается сцена;
    fn shade_model(
        &self,
        model: &Model,
        polygons: &Vec<Polygon>,
        camera: &Camera,
        projection_type: ProjectionType,
        lights: &Vec<LightSource>,
        canvas: &mut Canvas,
    );
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

/// Структура для отрисовки сцены. Содержит в себе параметры рендера.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneRenderer {
    /// Отрисовывать ли каркас модели.
    pub render_wireframe: bool,
    /// Отрисовывать ли нормали вершин.
    pub render_normals: bool,
    /// Отрисовывать ли грани модели.
    pub render_solid: bool,
    /// Тип проекции на камеру.
    pub projection_type: ProjectionType,
    /// Тип шейдинга. Ни на что не влияет, если `render_solid = false`.
    pub shading_type: ShadingType,
    /// Производить ли отсечение нелицевых граней.
    pub backface_culling: bool,
    /// Использовать ли z-buffer для упорядочивания граней.
    pub z_buffer_enabled: bool,
}

impl Default for SceneRenderer {
    fn default() -> Self {
        Self {
            render_wireframe: true,
            render_normals: false,
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
        let global_to_screen_transform = scene
            .camera
            .global_to_screen_transform(self.projection_type, canvas);

        // Отрисовка глобальной координатной системы.
        self.draw_coordinate_axes(canvas, global_to_screen_transform);

        // Отрисовка пользовательской оси вращения, если имеется
        if show_custom_axis {
            self::draw_custom_axis_line(
                canvas,
                global_to_screen_transform,
                axis_point1,
                axis_point2,
            );
        }

        draw_lights(&scene.lights, global_to_screen_transform, canvas);

        // количество отрисованных полигонов.
        let mut polygon_count: usize = 0;

        // отрисовка моделей
        for model in &scene.models {
            // Полигоны к отрисовке
            let polygons = if self.backface_culling {
                // только видимые
                self.model_backface_culling(scene.camera, model)
            } else {
                // все
                model.mesh.get_polygon_iter().cloned().collect()
            };

            polygon_count = polygons.len();

            // заполнить модель
            if self.render_solid {
                match self.shading_type {
                    ShadingType::None => {
                        let shader = solid_shader::SolidShader::new(self.z_buffer_enabled);
                        shader.shade_model(
                            model,
                            &polygons,
                            &scene.camera,
                            self.projection_type,
                            &scene.lights,
                            canvas,
                        );
                    }
                    ShadingType::GouraudLambert => {
                        let shader = gouraud_lambert_shader::GouraudLambertShader::new(
                            self.z_buffer_enabled,
                        );
                        shader.shade_model(
                            model,
                            &polygons,
                            &scene.camera,
                            self.projection_type,
                            &scene.lights,
                            canvas,
                        );
                    }
                    ShadingType::PhongToonShading(bands) => {
                        let shader =
                            phong_toon_shader::PhongToonShading::new(self.z_buffer_enabled, bands);
                        shader.shade_model(
                            model,
                            &polygons,
                            &scene.camera,
                            self.projection_type,
                            &scene.lights,
                            canvas,
                        );
                    }
                };
            }

            // каркас модели
            if self.render_wireframe {
                let shader = wireframe_shader::WireframeShader::new();
                shader.shade_model(
                    model,
                    &polygons,
                    &scene.camera,
                    self.projection_type,
                    &scene.lights,
                    canvas,
                );
            }

            // нормали модели
            if self.render_normals {
                let shader = normals_shader::NormalsShader::new();
                shader.shade_model(
                    model,
                    &polygons,
                    &scene.camera,
                    self.projection_type,
                    &scene.lights,
                    canvas,
                );
            }
        }
        canvas.invert_y();
        polygon_count
    }

    /// Отрисовка глобальной координатной системы.
    fn draw_coordinate_axes(&self, canvas: &mut Canvas, global_to_screen_transform: Transform3D) {
        let axis_length = 5.0; // Длина осей
        let origin = Point3::zero();

        let x_axis_end = Point3::new(axis_length, 0.0, 0.0);
        let y_axis_end = Point3::new(0.0, axis_length, 0.0);
        let z_axis_end = Point3::new(0.0, 0.0, axis_length);

        // Рисуем оси с разными цветами
        // Ось X - красная
        utils::render_line(
            global_to_screen_transform,
            origin,
            x_axis_end,
            Color32::RED,
            canvas,
        );

        // Ось Y - зелёная
        utils::render_line(
            global_to_screen_transform,
            origin,
            y_axis_end,
            Color32::GREEN,
            canvas,
        );

        // Ось Z - синяя
        utils::render_line(
            global_to_screen_transform,
            origin,
            z_axis_end,
            Color32::BLUE,
            canvas,
        );
    }

    /// Отсечение нелицевых граней модели
    ///
    /// Возвращает вектор полигонов только с лицевыми гранями.
    fn model_backface_culling(&self, camera: Camera, model: &Model) -> Vec<Polygon> {
        let global_normals: Vec<UVec3> = model.mesh.get_global_normals_iter().unwrap().collect();
        let global_vertexes: Vec<Point3> = model.mesh.get_global_vertex_iter().collect();
        let mut visible_polygons = Vec::new();
        for polygon in model.mesh.get_polygon_iter() {
            let mut polygon_normal = Vec3::zero();
            let indexes: Vec<usize> = polygon.get_mesh_vertex_index_iter().collect();
            for vertex_index in indexes.clone() {
                polygon_normal += global_normals[vertex_index];
            }

            // Если нормаль есть, производим отсечение
            if polygon_normal.length_squared() > 0.0 {
                let polygon_normal = (polygon_normal / polygon.vertex_count() as f32)
                    .normalize()
                    .unwrap();

                let camera_direction = match self.projection_type {
                    ProjectionType::Parallel => camera.get_direction(),
                    ProjectionType::Perspective => {
                        let mut polygon_pos = Point3::zero();
                        for vertex_index in indexes {
                            polygon_pos += Vec3::from(global_vertexes[vertex_index]);
                        }
                        polygon_pos =
                            Point3::from(Vec3::from(polygon_pos) / polygon.vertex_count() as f32);
                        (polygon_pos - camera.get_position()).normalize().unwrap()
                    }
                };

                // Если нормаль направлена в сторону камеры, то оставляем полигон
                let dot_product = polygon_normal.dot(camera_direction);
                if dot_product < 0.0 {
                    visible_polygons.push(polygon.clone());
                }
            }
        }

        visible_polygons
    }
}

// --------------------------------------------------
// Вспомогательные методы
// --------------------------------------------------

/// Преобразует глобальные координаты точки в координаты экрана.
fn project_point(point: Point3, view_proj_matrix: Transform3D) -> Pos2 {
    let proj_point: Point3 = point.apply_transform(view_proj_matrix).unwrap();
    Pos2::new(proj_point.x, proj_point.y)
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

fn draw_lights(
    lights: &Vec<LightSource>,
    global_to_screen_transform: Transform3D,
    canvas: &mut Canvas,
) {
    for light in lights {
        let light_pos = light
            .position
            .apply_transform(global_to_screen_transform)
            .unwrap();
        let pos = Pos2::new(light_pos.x, light_pos.y);
        let radius = utils::lerp_float(15.0, 3.0, (light_pos.z + 1.0) / 2.0);
        if pos.x < canvas.width() as f32 && pos.y < canvas.height() as f32 {
            canvas.circle_filled(pos, radius, light.color);
        }
    }
}

#[cfg(test)]
mod render_tests {
    use crate::HVec3;

    use super::*;

    const TOLERANCE: f32 = 1e-6;

    fn assert_vecs(got: Vec3, expected: Vec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn assert_points(got: Point3, expected: Point3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидалась точка {:?}, но получена {:?}, одна из координат которой отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    fn assert_hvecs(got: HVec3, expected: HVec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался вектор {:?}, но получен вектор {:?}, одна из координат которого отличается более чем на {}",
            expected,
            got,
            tolerance
        );
    }

    #[test]
    fn test_global_to_screen_transform_1() {
        let camera = Camera::default();
        let canvas = Canvas::new(900, 600);
        let transform = camera.global_to_screen_transform(ProjectionType::Perspective, &canvas);

        // точка по центру камеры
        let camera_pos = camera.get_position();
        let z_depth = (camera.get_near_plane() + camera.get_far_plane()) / 2.0;
        let point = Point3::new(camera_pos.x, camera_pos.y, camera_pos.z + z_depth);

        // точка должна быть где-то по центру экрана
        let proj_point = point.apply_transform(transform).unwrap();
        assert!((proj_point.x - canvas.width() as f32 / 2.0).abs() < TOLERANCE);
        assert!((proj_point.y - canvas.height() as f32 / 2.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_global_to_screen_transform_2() {
        let camera = Camera::default();
        let canvas = Canvas::new(900, 600);
        let transform = camera.global_to_screen_transform(ProjectionType::Perspective, &canvas);

        // точка слева снизу от центра камеры
        let camera_pos = camera.get_position();
        let z_depth = (camera.get_near_plane() + camera.get_far_plane()) / 2.0;
        let point = Point3::new(
            camera_pos.x - 15.0,
            camera_pos.y - 15.0,
            camera_pos.z + z_depth,
        );

        // точка должна быть где-то слева снизу от центра экрана
        let proj_point = point.apply_transform(transform).unwrap();
        assert!(proj_point.x < canvas.width() as f32 / 2.0 - TOLERANCE);
        assert!(proj_point.y < canvas.height() as f32 / 2.0 - TOLERANCE);
    }

    #[test]
    fn test_global_to_screen_transform_3() {
        let camera = Camera::default();
        let canvas = Canvas::new(900, 600);
        let transform = camera.global_to_screen_transform(ProjectionType::Perspective, &canvas);

        // точка справа сверху от центра камеры
        let camera_pos = camera.get_position();
        let z_depth = (camera.get_near_plane() + camera.get_far_plane()) / 2.0;
        let point = Point3::new(
            camera_pos.x + 15.0,
            camera_pos.y + 15.0,
            camera_pos.z + z_depth,
        );

        // точка должна быть где-то справа сверху от центра экрана
        let proj_point = point.apply_transform(transform).unwrap();
        assert!(proj_point.x > canvas.width() as f32 / 2.0 + TOLERANCE);
        assert!(proj_point.y > canvas.height() as f32 / 2.0 + TOLERANCE);
    }
}
