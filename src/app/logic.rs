use crate::app::AthenianApp;
use egui::{Color32, Painter, Pos2, Response, Ui};
use g3d::{Model3, Transform3D, Point3, Vec3, Transformable3};


// --------------------------------------------------
// Обработка области рисования (холст)
// --------------------------------------------------

impl AthenianApp {
    /// Выделить egui::painter на всю свободную область указанного UI элемента.
    pub fn allocate_painter(&mut self, ui: &mut Ui) -> (Response, Painter) {
        let available_size = ui.available_size();
        self.painter_width = available_size.x;
        self.painter_height = available_size.y;

        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(self.painter_width, self.painter_height),
            egui::Sense::click_and_drag(),
        );

        // цвет холста
        painter.rect_filled(response.rect, 0.0, Color32::WHITE);
        // границы
        // painter.rect_stroke(
        //     response.rect,
        //     0,
        //     Stroke::new(1.0, Color32::GRAY),
        //     StrokeKind::Inside,
        // );

        (response, painter)
    }

    /// Очистить холст от полигонов.
    pub fn clear_canvas(&mut self) {
// TODO
    }

    /// Нарисовать текущий якорь.
    fn draw_anchor(&self, painter: &Painter) {
        // if let Some(anchor) = self.selected_polygon_anchor {
        //     painter.circle_filled(anchor, 5.0, Color32::RED);
        // }
    }

    /// Нарисовать выбранную точку.
    fn draw_point(&self, painter: &Painter) {
        // if let Some(anchor) = self.selected_point {
        //     painter.circle_filled(anchor, 5.0, Color32::GREEN);
        // }
    }

    /// Нарисовать холст.
    pub fn draw_canvas(&mut self, painter: &Painter) {
        // for i in 0..self.polygons.len() {
        //     if self.selected_polygon_index.is_some() && i == self.selected_polygon_index.unwrap() {
        //         self.polygons[i].draw(&painter, &PolygonStyle::selected(), self.selected_point);
        //     } else {
        //         self.polygons[i].draw(&painter, &PolygonStyle::standard(), self.selected_point);
        //     }
        // }
        // self.draw_anchor(painter);
        // self.draw_point(painter);
    }
}

// --------------------------------------------------
// Обработка управления
// --------------------------------------------------

impl AthenianApp {
    /// Обработать взаимодействие с холстом.
    pub fn handle_input(&mut self, response: &Response) {
        self.handle_click(response);
        self.handle_drag(response);
    }

    /// Обработать клики по холсту.
    fn handle_click(&mut self, response: &Response) {
        if response.clicked_by(egui::PointerButton::Primary) {
            let pos = response.hover_pos().unwrap();
            match &self.instrument {
                Instrument::SetPoint => self.change_point(pos),
                _ => self.handle_3d_click(pos),
            }
        }
    }

    /// Обработать перетаскивание по холсту.
    fn handle_drag(&mut self, response: &Response) {
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            self.drag_prev_pos = None;
            return;
        }

        if !response.dragged_by(egui::PointerButton::Primary) {
            return;
        }

        if let Some(drag_start) = self.drag_prev_pos
            && let Some(drag_cur) = response.hover_pos()
        {
            match &self.instrument {
                _ => self.handle_3d_drag(drag_start, drag_cur),
            }
        }

        self.drag_prev_pos = response.hover_pos();
    }

    /// Обработать клики по холсту для 3D.
    fn handle_3d_click(&mut self, pos: egui::Pos2) {

    }

    /// Обработать перетаскивание для 3D.
    fn handle_3d_drag(&mut self, start: egui::Pos2, end: egui::Pos2) {

    }

}

// --------------------------------------------------
// Взаимодействие с полигонами
// --------------------------------------------------

impl AthenianApp {
    /// Добавить новую вершину к текущему полигону.
    fn add_vertex_to_selected_polygon(&mut self, pos: Pos2) {
        // if let Some(index) = self.selected_polygon_index {
        //     let polygon = &mut self.polygons[index];
        //     polygon.add_vertex_pos(pos);
        // }
        // // Новый полигон
        // else {
        //     let polygon = Polygon::from_pos(pos);
        //     self.polygons.push(polygon);
        //     self.selected_polygon_index = Some(self.polygons.len() - 1);
        // }
    }

    /// Выбрать полигон в указанной точке.
    fn select_polygon(&mut self, pos: Pos2) {
        // // обнулить прошлый якорь
        // self.selected_polygon_anchor = None;

        // for i in 0..self.polygons.len() {
        //     if self.polygons[i].contains_pos(pos) {
        //         self.selected_polygon_index = Some(i);
        //         return;
        //     }
        // }
        // self.selected_polygon_index = None;
    }

    /// Выбрать якорь для операций над полигоном.
    fn change_anchor(&mut self, pos: Pos2) {
        // self.selected_polygon_anchor = Some(pos);
    }

    /// Выбрать точку для проверки положения относительно ребёр.
    fn change_point(&mut self, pos: Pos2) {
        // self.selected_point = Some(pos);
    }

    /// Переместить выбранный полигон параллельно координатным осям.
    fn drag_selected_polygon(&mut self, start: Pos2, end: Pos2) {
        // if let Some(index) = self.selected_polygon_index {
        //     let delta = end - start;
        //     let polygon = &mut self.polygons[index];
        //     polygon.apply_transform(Transform2D::translation(delta.x, delta.y));

        //     #[cfg(debug_assertions)]
        //     println!("drag with start {:#?} end {:#?}", start, end);
        // }
    }

    /// Повернуть выбранный полигон через вектор смещения.
    fn rotate_selected_polygon(&mut self, start: Pos2, end: Pos2) {
        // if let Some(index) = self.selected_polygon_index {
        //     let polygon = &mut self.polygons[index];

        //     // Задан якорь для вращения
        //     if let Some(anchor) = self.selected_polygon_anchor {
        //         let angle = calculate_rotation_angle(anchor, start, end);
        //         polygon.apply_transform(Transform2D::rotation_around_pos(angle, anchor));

        //         #[cfg(debug_assertions)]
        //         println!("rotate relative to {:#?} with angle {:#?}", anchor, angle);
        //     }
        //     // Просто повернуть относительно центра
        //     else {
        //         let center = polygon.get_center();
        //         let angle = calculate_rotation_angle(center, start, end);
        //         polygon.apply_transform(Transform2D::rotation_around_pos(angle, center));

        //         #[cfg(debug_assertions)]
        //         println!(
        //             "rotate relative to center {:#?} with angle {:#?}",
        //             center, angle
        //         );
        //     }
        // }
    }

    /// Изменить размер полигона через вектор смещения.
    fn scale_selected_polygon(&mut self, start: Pos2, end: Pos2) {
        // if let Some(index) = self.selected_polygon_index {
        //     let polygon = &mut self.polygons[index];

        //     // Задан якорь для изменения размера
        //     if let Some(anchor) = self.selected_polygon_anchor {
        //         let (sx, sy) = calculate_scale(anchor, start, end);
        //         polygon.apply_transform(Transform2D::scaling_around_pos(sx, sy, anchor));

        //         #[cfg(debug_assertions)]
        //         println!(
        //             "scale relative to {:#?} with scale x:{} y:{}",
        //             anchor, sx, sy
        //         );
        //     }
        //     // Просто растянуть относительно центра
        //     else {
        //         let center = polygon.get_center();
        //         let (sx, sy) = calculate_scale(center, start, end);
        //         polygon.apply_transform(Transform2D::scaling_around_pos(sx, sy, center));

        //         #[cfg(debug_assertions)]
        //         println!(
        //             "scale relative to center {:#?} with scale x:{} y:{}",
        //             center, sx, sy
        //         );
        //     }
        // }
    }


    ///////////////////////////////
    // 3D
    //////////////////////////////
    ///////////

    /// Выбрать 3D модель в указанной точке.
    fn select_3d_model(&mut self, pos: egui::Pos2) {
    }

    /// Переместить 3D модель.
    fn move_3d_model(&mut self, index: usize, start: egui::Pos2, end: egui::Pos2) {
    }

    /// Повернуть 3D модель.
    fn rotate_3d_model(&mut self, index: usize, start: egui::Pos2, end: egui::Pos2) {
    }

    /// Масштабировать 3D модель.
    fn scale_3d_model(&mut self, index: usize, start: egui::Pos2, end: egui::Pos2) {

    }

    /// Установить первую точку оси.
    fn set_axis_point1(&mut self, pos: egui::Pos2) {
        // TODO: преобразовать 2D позицию в 3D точку
    }

    /// Установить вторую точку оси.
    fn set_axis_point2(&mut self, pos: egui::Pos2) {
        // TODO: преобразовать 2D позицию в 3D точку

    }

    /// Применить поворот вокруг произвольной оси.
    fn apply_axis_rotation(&mut self) {
    }



    // Простые методы для добавления многогранников
    pub fn add_tetrahedron(&mut self) {
        self.scene.add_model(Model3::tetrahedron());
    }

    pub fn add_hexahedron(&mut self) {
        self.scene.add_model(Model3::hexahedron());
    }

    pub fn add_octahedron(&mut self) {
        self.scene.add_model(Model3::octahedron());
    }

    /// Установить перспективную проекцию
    pub fn set_perspective_projection(&mut self) {
        self.current_projection = Projection::Perspective;
        // Используем стандартную перспективную проекцию камеры
        self.camera.set_projection(
            std::f32::consts::PI / 3.0, // 60 degrees FOV
            self.painter_width / self.painter_height,
            0.1,
            100.0
        );
    }

    /// Установить изометрическую проекцию
    pub fn set_isometric_projection(&mut self) {
        self.current_projection = Projection::Isometric;
        let isometric_transform = Transform3D::isometric();
        self.apply_projection_transform(isometric_transform);
    }

    /// Установить диметрическую проекцию
    pub fn set_dimetric_projection(&mut self) {
        self.current_projection = Projection::Dimetric;
        // Стандартные углы для диметрической проекции
        let dimetric_transform = Transform3D::dimetric(20.0, 30.0, 0.5);
        self.apply_projection_transform(dimetric_transform);
    }

    /// Установить триметрическую проекцию
    pub fn set_trimetric_projection(&mut self) {
        self.current_projection = Projection::Trimetric;
        // Произвольные углы для триметрической проекции
        let trimetric_transform = Transform3D::trimetric(15.0, 25.0, 1.0, 0.7, 0.9);
        self.apply_projection_transform(trimetric_transform);
    }

    /// Применить матрицу проекции ко всем моделям на сцене
    fn apply_projection_transform(&mut self, projection: Transform3D) {
        // Для аксонометрических проекций устанавливаем камеру
        self.camera.position = Point3::new(0.0, 0.0, 5.0);
        self.camera.direction = Vec3::new(0.0, 0.0, -1.0);
        self.camera.up = Vec3::new(0.0, 1.0, 0.0);
        
        // Применяем проекцию ко всем моделям на сцене
        for i in 0..self.scene.models.len() {
            let model = self.scene.models[i].clone();
            let transformed_model = model.transform(projection);
            self.scene.models[i] = transformed_model;
        }
    }
}

#[derive(Default)]
pub enum Instrument {
    #[default]
    SetPoint,
    Move3D,
    Rotate3D,
    Scale3D,
    ReflectXY,
    ReflectXZ,
    ReflectYZ,
    RotateAroundAxis,
    SetAxisPoint1,
    SetAxisPoint2,
}

impl ToString for Instrument {
    fn to_string(&self) -> String {
        match self {
            Self::SetPoint => String::from("изменить точку"),
            Self::Move3D => String::from("переместить 3D модель"),
            Self::Rotate3D => String::from("повернуть 3D модель"),
            Self::Scale3D => String::from("масштабировать 3D модель"),
            Self::ReflectXY => String::from("отражение XY"),
            Self::ReflectXZ => String::from("отражение XZ"),
            Self::ReflectYZ => String::from("отражение YZ"),
            Self::RotateAroundAxis => String::from("поворот вокруг оси"),
            Self::SetAxisPoint1 => String::from("установить точку оси 1"),
            Self::SetAxisPoint2 => String::from("установить точку оси 2"),
        }
    }
}

/// Считает угол поворота в раиданах на основе смещения относительно какого-то центра.
fn calculate_rotation_angle(center: Pos2, start: Pos2, end: Pos2) -> f32 {
    let start_vec = (start.x - center.x, start.y - center.y);
    let end_vec = (end.x - center.x, end.y - center.y);

    let start_angle = start_vec.1.atan2(start_vec.0);
    let end_angle = end_vec.1.atan2(end_vec.0);

    let mut angle = start_angle - end_angle;
    let pi = std::f32::consts::PI;
    while angle > pi {
        angle -= 2.0 * pi;
    }
    while angle < -pi {
        angle += 2.0 * pi;
    }

    angle
}

/// Считает растяжение на основе смещения относительно какого-то центра.
fn calculate_scale(center: Pos2, start: Pos2, end: Pos2) -> (f32, f32) {
    let start_vec = start - center;
    let end_vec = end - center;

    let scale_x = if start_vec.x.abs() < f32::EPSILON {
        1.0
    } else {
        end_vec.x / start_vec.x
    };

    let scale_y = if start_vec.y.abs() < f32::EPSILON {
        1.0
    } else {
        end_vec.y / start_vec.y
    };

    (scale_x, scale_y)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Projection {
    #[default]
    Perspective,
    Isometric,
    Dimetric,
    Trimetric,
    Cabinet,
    Cavalier,
}

impl ToString for Projection {
    fn to_string(&self) -> String {
        match self {
            Self::Perspective => "Perspective".to_string(),
            Self::Isometric => "Isometric".to_string(),
            Self::Dimetric => "Dimetric".to_string(),
            Self::Trimetric => "Trimetric".to_string(),
            Self::Cabinet => "Cabinet".to_string(),
            Self::Cavalier => "Cavalier".to_string(),
        }
    }
}