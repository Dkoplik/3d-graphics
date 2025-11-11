use crate::{
    Camera3, Canvas, HVec3, LightSource, Model3, Point3, Scene, Transform3D, Vec3, classes3d::mesh,
};
use egui::{Color32, Pos2};

/// Тип рендера (как отображать объекты?)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum RenderType {
    /// Отображение только каркасов (Mesh-ей) моделей.
    #[default]
    WireFrame,
    /// Полноценное отображение модели с материалом и текстурой.
    Solid,
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

/// Тип шейдинга.
///
/// Меняет отображение материала в зависимости от освещения.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum ShadingType {
    /// Отсутствие шейдинна
    #[default]
    None,
    /// Шейдинг Гуро для модели Ламберта
    Gouraud,
    /// Шейдинг Фонга для модели туншейдинг
    Phong,
}

/// Полные параметры рендера сцены.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct RenderOptions {
    pub render_type: RenderType,
    pub projection_type: ProjectionType,
    pub shading_type: ShadingType,
    pub backface_culling: bool,
    pub z_buffer_enabled: bool,
}

impl Scene {
    // --------------------------------------------------
    // Операции над сценой
    // --------------------------------------------------

    /// Создать пустую сцену.
    pub fn new(camera: Camera3) -> Self {
        Self {
            models: Vec::new(),
            camera,
            lights: Vec::new(),
            ambient_light: Color32::BLACK,
        }
    }

    /// Добавить новую модель на сцену.
    pub fn add_model(&mut self, model: Model3) {
        self.models.push(model);
    }

    /// Добавить новый источник света на сцену.
    pub fn add_light(&mut self, light: LightSource) {
        self.lights.push(light);
    }

    /// Изменить цвет глобального освещения.
    pub fn set_ambient_light(&mut self, color: Color32) {
        self.ambient_light = color;
    }

    /// Нарисовать сцену на холст со всеми нужными преобразованиями.
    pub fn render(
        &self,
        canvas: &mut Canvas,
        render_options: RenderOptions,
        show_custom_axis: bool,
        axis_point1: Point3,
        axis_point2: Point3,
    ) {
        // Стереть прошлый кадр.
        canvas.clear(Color32::GRAY);

        // Матрица преобразования из глобальных координат в экранные
        let global_to_screen_transform =
            self.get_view_projection_transform(render_options.projection_type, canvas);

        // Отрисовка глобальной координатной системы.
        self.draw_coordinate_axes(canvas, &global_to_screen_transform);

        // Отрисовка пользовательской оси вращения
        if show_custom_axis {
            self.draw_custom_axis_line(
                canvas,
                &global_to_screen_transform,
                axis_point1,
                axis_point2,
            );
        }

        // Отрисовка каждой модели
        for model in &self.models {
            // Проекция вершин модели
            let projected_vertexes = self.transform_model(global_to_screen_transform, model);

            // Полигоны к отрисовке
            let polygons = if render_options.backface_culling {
                self.model_backface_culling(model)
            } else {
                model.mesh.polygons.clone()
            };

            match render_options.render_type {
                RenderType::WireFrame => {
                    self.render_model_wireframe(&projected_vertexes, &polygons, canvas)
                }
                RenderType::Solid => {
                    self.render_solid(
                        &projected_vertexes,
                        &polygons,
                        model,
                        canvas,
                        render_options.z_buffer_enabled,
                    );
                    match render_options.shading_type {
                        ShadingType::None => (),
                        ShadingType::Gouraud => self.render_gouraud_lambert(
                            &projected_vertexes,
                            &polygons,
                            model,
                            canvas,
                            render_options.z_buffer_enabled,
                        ),
                        ShadingType::Phong => self.render_phong(
                            &projected_vertexes,
                            &polygons,
                            model,
                            canvas,
                            render_options.z_buffer_enabled,
                        ),
                    }
                }
            }
        }
    }

    // --------------------------------------------------
    // Вспомогательные методы для рендера
    // --------------------------------------------------

    /// Получить матрицу преобразования из глобальных координат в экранные (viewport, он же canvas)
    ///
    /// То есть, матрица производит следующие операции:
    /// глобальные координаты -> координаты камеры (view tranform) -> проекция на камеру в NDC -> растяжение NDC на размер canvas.
    fn get_view_projection_transform(
        &self,
        projection_type: ProjectionType,
        canvas: &Canvas,
    ) -> Transform3D {
        // Матрица проекции координат камеры в NDC
        let proj_matrix = match projection_type {
            ProjectionType::Parallel => Transform3D::parallel_symmetric(
                canvas.width as f32,
                canvas.height as f32,
                self.camera.get_near_plane(),
                self.camera.get_far_plane(),
            ),
            ProjectionType::Perspective => Transform3D::perspective(
                self.camera.get_fov(),
                self.camera.get_aspect_ratio(),
                self.camera.get_near_plane(),
                self.camera.get_far_plane(),
            ),
        };

        let scale_x = canvas.width as f32 / 2.0; // растянуть NDC по ширине
        let scale_y = canvas.height as f32 / 2.0; // растянуть NDC по высоте

        self.camera
            .get_local_frame()
            .global_to_local_matrix() // view transformation (локальные координаты камеры)
            .multiply(proj_matrix) // вот тут получается NDC с координатами [-1, +1]
            .multiply(Transform3D::translation_uniform(1.0)) // теперь координаты [0, +2]
            .multiply(Transform3D::scale(scale_x, scale_y, 1.0)) // теперь экранные
    }

    /// Отрисовка пользовательской оси для вращения
    fn draw_custom_axis_line(
        &self,
        canvas: &mut Canvas,
        view_proj_matrix: &Transform3D,
        point1: Point3,
        point2: Point3,
    ) {
        // Проецируем точки в 2D используя нашу систему проекций
        let screen_point1 = self.project_point(point1, view_proj_matrix);
        let screen_point2 = self.project_point(point2, view_proj_matrix);

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

    /// Отрисовка глобальной координатной системы.
    fn draw_coordinate_axes(&self, canvas: &mut Canvas, view_proj_matrix: &Transform3D) {
        // TODO нелпохо бы сделать полноценную отрисовку координатной сетки.
        let axis_length = 2.0; // Длина осей
        let origin = Point3::new(0.0, 0.0, 0.0);

        let x_axis_end = Point3::new(axis_length, 0.0, 0.0);
        let y_axis_end = Point3::new(0.0, axis_length, 0.0);
        let z_axis_end = Point3::new(0.0, 0.0, axis_length);

        let origin_2d = self.project_point(origin, view_proj_matrix);
        let x_end_2d = self.project_point(x_axis_end, view_proj_matrix);
        let y_end_2d = self.project_point(y_axis_end, view_proj_matrix);
        let z_end_2d = self.project_point(z_axis_end, view_proj_matrix);

        // Рисуем оси с разными цветами
        // Ось X - красная
        canvas.draw_sharp_line(origin_2d, x_end_2d, Color32::RED);

        // Ось Y - зелёная
        canvas.draw_sharp_line(origin_2d, y_end_2d, Color32::GREEN);

        // Ось Z - синяя
        canvas.draw_sharp_line(origin_2d, z_end_2d, Color32::BLUE);
    }

    /// Преобразует глобальные координаты точки в координаты экрана.
    fn project_point(&self, point: Point3, view_proj_matrix: &Transform3D) -> Pos2 {
        let proj_point: Point3 = view_proj_matrix.apply_to_hvec(point.into()).into();
        Pos2::new(proj_point.x, proj_point.y)
    }

    /// Проецирует вершины модели на экран.
    /// `view_proj_matrix` - матрица проекции из глобальных координат в экранные
    /// `model` - сама модель
    ///
    /// Важно: после этих преобразований вершины становится `Vec3`, то есть все преобразования в
    /// 4D векторах закончены и теперь 2D пространство представлено 3D векторами, где z-компонента
    /// нужна для порядка отрисовки на экране.
    fn transform_model(&self, view_proj_matrix: Transform3D, model: &Model3) -> Vec<Vec3> {
        // Преобразование из координат модели в проекцию NDC.
        let full_transform = model
            .mesh
            .get_local_frame()
            .local_to_global_matrix()
            .multiply(view_proj_matrix);

        // Проекция точек модели на экран.
        model
            .mesh
            .get_vertexes()
            .iter()
            .map(|vertex| Vec3::from(vertex.apply_transform(&full_transform)))
            .collect()
    }

    /// Отсечение нелицевых граней модели
    /// `model` - сама модель.
    ///
    /// Возвращает вектор полигонов только с лицевыми гранями.
    fn model_backface_culling(&self, model: &Model3) -> Vec<mesh::Polygon3> {
        let vertexes = model.mesh.get_vertexes();
        let polygons = model.mesh.get_polygons();
        let mut visible_polygons = Vec::new();

        let mesh_center = Self::calculate_mesh_center(vertexes);
        let camera_pos = self.camera.get_position();

        let view_direction = (mesh_center - Vec3::from(camera_pos)).normalize();

        for polygon in polygons {
            let vertex_indices = polygon.get_vertexes();
            if vertex_indices.len() < 3 {
                continue;
            }

            let polygon_normal = polygon.get_normal(vertexes, Some(mesh_center));
            let dot_product = polygon_normal.dot(view_direction);

            if dot_product > 0.0 {
                visible_polygons.push(polygon.clone());
            }
        }

        visible_polygons
    }

    fn calculate_mesh_center(vertexes: &Vec<HVec3>) -> Vec3 {
        if vertexes.is_empty() {
            return Vec3::zero();
        }

        let sum: Vec3 = vertexes
            .iter()
            .map(|v| Vec3::from(*v))
            .fold(Vec3::zero(), |acc, v| acc + v);

        sum * (1.0 / vertexes.len() as f32)
    }

    /// Реднер каркаса модели.
    fn render_model_wireframe(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<mesh::Polygon3>,
        canvas: &mut Canvas,
    ) {
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
                canvas.draw_sharp_line(start_pos, end_pos, Color32::WHITE);
            }
        }

        // Рисуем вершины
        for &vertex in projected_vertexes {
            let pos = Pos2::new(vertex.x, vertex.y);
            canvas.circle_filled(pos, 3.0, Color32::WHITE);
        }
    }

    fn render_solid(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<mesh::Polygon3>,
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        // TODO Тут просто отрисовка моделей без учёта освещения.
        // Обращаю внимание, что материал модели уже имеет удобный метод для получения цвета
        // с учётом материала и текстуры
        // Также обращаю внимание, что функция может применяться как с z_buffer'ом, так и без него. Сам z_buffer заложен в Canvas.
        todo!("рендер цельной модели");
    }

    /// Шейдинг Гуро для модели Ламберта.
    ///
    /// Применяется после отрисовки моделей без шейдинга. Иными словами, на холсте canvas уже нарисованы
    /// все модели, но без учёта освещения, поэтому теперь поверх этих цветов надо наложить сам шейдинг.
    fn render_gouraud_lambert(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<mesh::Polygon3>,
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        // TODO
        todo!("Шейдинг Гуро для модели Ламберта")
    }

    /// Шейдинг Фонга для модели туншейдинг.
    ///
    /// Применяется после отрисовки моделей без шейдинга. Иными словами, на холсте canvas уже нарисованы
    /// все модели, но без учёта освещения, поэтому теперь поверх этих цветов надо наложить сам шейдинг.
    fn render_phong(
        &self,
        projected_vertexes: &Vec<Vec3>,
        polygons: &Vec<mesh::Polygon3>,
        model: &Model3,
        canvas: &mut Canvas,
        z_buffer_enabled: bool,
    ) {
        // TODO
        todo!("Шейдинг Фонга для модели туншейдинг")
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(Camera3::default())
    }
}
