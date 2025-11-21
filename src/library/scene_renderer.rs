use std::fmt::Display;

use crate::{
    Camera3, Canvas, LightSource, Model3, Point3, Scene, SceneRenderer, Transform3D, Vec3,
    classes3d::mesh::Polygon3,
};
use egui::{Color32, Pos2};

mod gouraud_lambert_shader;
mod normals_shader;
mod phong_toon_shader;
mod shader_utils;
mod solid_shader;
mod wireframe_shader;

pub trait Shader {
    fn shade_model(
        &self,
        model: &Model3,
        polygons: &Vec<Polygon3>,
        global_to_screen_transform: Transform3D,
        lights: &Vec<LightSource>,
        canvas: &mut Canvas,
    );
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
    pub projection_type: classes3d::scene_renderer::ProjectionType,
    /// Тип шейдинга. Ни на что не влияет, если `render_solid = false`.
    pub shading_type: classes3d::scene_renderer::ShadingType,
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
    /// Возвращает матрицу преобразований из экранных координат в глобальные.
    pub fn screen_to_global_transform(&self, scene: &Scene, canvas: &Canvas) -> Transform3D {
        let camera = scene.camera;

        let scale_x = canvas.width as f32 / 2.0;
        let scale_y = canvas.height as f32 / 2.0;

        let width = 2.0 * (camera.get_fov() / 2.0).sin();
        let height = width / camera.get_aspect_ratio();

        Transform3D::scale(1.0 / scale_x, -1.0 / scale_y, 1.0) // экранные в [0, +2]
            .multiply(Transform3D::translation(-1.0, -1.0, 0.0)) // в NDC [-1, +1]
            .multiply(Transform3D::scale(width, height, 1.0)) // локальные координаты камеры
            .multiply(camera.get_local_frame().local_to_global_matrix()) // глобальные
    }

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
            self::get_global_to_screen_transform(self.projection_type, &scene.camera, &canvas);

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
                self.model_backface_culling(scene.camera, model)
            } else {
                model.mesh.get_polygons().cloned().collect()
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
                            global_to_screen_transform,
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
                            global_to_screen_transform,
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
                            global_to_screen_transform,
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
                    global_to_screen_transform,
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
                    global_to_screen_transform,
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
        shader_utils::render_line(
            global_to_screen_transform,
            origin,
            x_axis_end,
            Color32::RED,
            canvas,
        );

        // Ось Y - зелёная
        shader_utils::render_line(
            global_to_screen_transform,
            origin,
            y_axis_end,
            Color32::GREEN,
            canvas,
        );

        // Ось Z - синяя
        shader_utils::render_line(
            global_to_screen_transform,
            origin,
            z_axis_end,
            Color32::BLUE,
            canvas,
        );
    }

    /// Отсечение нелицевых граней модели
    /// `model` - сама модель.
    ///
    /// Возвращает вектор полигонов только с лицевыми гранями.
    fn model_backface_culling(&self, camera: Camera3, model: &Model3) -> Vec<Polygon3> {
        let global_normals: Vec<Vec3> = model.mesh.get_global_normals().collect();
        let global_vertexes: Vec<Vec3> = model
            .mesh
            .get_global_vertexes()
            .map(|v| Vec3::from(v))
            .collect();
        let mut visible_polygons = Vec::new();
        for polygon in model.mesh.get_polygons() {
            if !polygon.is_valid() {
                continue;
            }

            let mut polygon_normal = Vec3::zero();
            for &vertex_index in polygon.get_vertexes() {
                polygon_normal += global_normals[vertex_index];
            }

            // Если нормаль есть, производим отсечение
            if polygon_normal.length_squared() > 0.0 {
                polygon_normal /= polygon.get_vertexes().len() as f32;

                let camera_direction = match self.projection_type {
                    ProjectionType::Parallel => camera.get_direction(),
                    ProjectionType::Perspective => {
                        let mut polygon_pos = Vec3::zero();
                        for &vertex_index in polygon.get_vertexes() {
                            polygon_pos += global_vertexes[vertex_index];
                        }
                        polygon_pos /= polygon.get_vertexes().len() as f32;
                        Point3::from(polygon_pos) - camera.get_position()
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

/// Получить матрицу преобразования из глобальных координат в экранные (viewport, он же canvas)
///
/// То есть, матрица производит следующие операции:
/// глобальные координаты -> координаты камеры (view tranform) -> проекция на камеру в NDC -> растяжение NDC на размер canvas.
fn get_global_to_screen_transform(
    projection_type: ProjectionType,
    camera: &Camera3,
    canvas: &Canvas,
) -> Transform3D {
    // Матрица проекции координат камеры в NDC
    let proj_matrix = match projection_type {
        ProjectionType::Parallel => {
            let width = 2.0 * (camera.get_fov() / 2.0).sin();
            let height = width / camera.get_aspect_ratio();
            Transform3D::parallel_symmetric(
                width,
                height,
                camera.get_near_plane(),
                camera.get_far_plane(),
            )
        }
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
        .multiply(Transform3D::translation(-1.0, 1.0, 0.0))
        .multiply(Transform3D::scale(-scale_x, scale_y, 1.0)) // теперь экранные
}

/// Преобразует глобальные координаты точки в координаты экрана.
fn project_point(point: Point3, view_proj_matrix: Transform3D) -> Pos2 {
    let proj_point: Point3 = view_proj_matrix.apply_to_hvec(point.into()).into();
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
        let light_pos = light.position * global_to_screen_transform;
        let pos = Pos2::new(light_pos.x, light_pos.y);
        if pos.x < canvas.width as f32 && pos.y < canvas.height as f32 {
            canvas.circle_filled(pos, 3.0, light.color);
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
        let camera = Camera3::default();
        let canvas = Canvas::new(900, 600);
        let transform =
            get_global_to_screen_transform(ProjectionType::Perspective, &camera, &canvas);

        // точка по центру камеры
        let camera_pos = camera.get_position();
        let z_depth = (camera.get_near_plane() + camera.get_far_plane()) / 2.0;
        let point = Point3::new(camera_pos.x, camera_pos.y, camera_pos.z + z_depth);

        // точка должна быть где-то по центру экрана
        let proj_point = point * transform;
        assert!((proj_point.x - canvas.width as f32 / 2.0).abs() < TOLERANCE);
        assert!((proj_point.y - canvas.height as f32 / 2.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_global_to_screen_transform_2() {
        let camera = Camera3::default();
        let canvas = Canvas::new(900, 600);
        let transform =
            get_global_to_screen_transform(ProjectionType::Perspective, &camera, &canvas);

        // точка слева снизу от центра камеры
        let camera_pos = camera.get_position();
        let z_depth = (camera.get_near_plane() + camera.get_far_plane()) / 2.0;
        let point = Point3::new(
            camera_pos.x - 15.0,
            camera_pos.y - 15.0,
            camera_pos.z + z_depth,
        );

        // точка должна быть где-то слева снизу от центра экрана
        let proj_point = point * transform;
        assert!(proj_point.x < canvas.width as f32 / 2.0 - TOLERANCE);
        assert!(proj_point.y < canvas.height as f32 / 2.0 - TOLERANCE);
    }

    #[test]
    fn test_global_to_screen_transform_3() {
        let camera = Camera3::default();
        let canvas = Canvas::new(900, 600);
        let transform =
            get_global_to_screen_transform(ProjectionType::Perspective, &camera, &canvas);

        // точка справа сверху от центра камеры
        let camera_pos = camera.get_position();
        let z_depth = (camera.get_near_plane() + camera.get_far_plane()) / 2.0;
        let point = Point3::new(
            camera_pos.x + 15.0,
            camera_pos.y + 15.0,
            camera_pos.z + z_depth,
        );

        // точка должна быть где-то справа сверху от центра экрана
        let proj_point = point * transform;
        assert!(proj_point.x > canvas.width as f32 / 2.0 + TOLERANCE);
        assert!(proj_point.y > canvas.height as f32 / 2.0 + TOLERANCE);
    }
}
