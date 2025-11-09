use crate::app::AthenianApp;

// --------------------------------------------------
// Построение UI приложения
// --------------------------------------------------

impl eframe::App for AthenianApp {
    /// Главный цикл UI.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_top_panel(ctx);
        self.show_left_panel(ctx);
        self.show_bottom_panel(ctx);
        self.show_central_panel(ctx);
    }
}

impl AthenianApp {
    /// Показать верхную панель приложения.
    fn show_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Projection", |ui| {
                    if ui.button("Perspective").clicked() {
                        self.set_perspective_projection();
                    }
                    if ui.button("Isometric").clicked() {
                        self.set_isometric_projection();
                    }
                });
            });
        });
    }

    /// Показать левую панель приложения.
    fn show_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    if ui.button("Стереть всё").clicked() {
                        self.clear_canvas();
                    }

                    self.show_model_settings(ui);
                });
            });
    }

    /// Показать панель в контексте операций над моделями.
    fn show_model_settings(&mut self, ui: &mut egui::Ui) {
        // Display the ComboBox
        egui::ComboBox::from_label("<- 3D операции")
            .selected_text(format!("{}", self.instrument.to_string()))
            .show_ui(ui, |ui| {
                // Опции
                ui.selectable_value(
                    &mut self.instrument,
                    crate::app::logic::Instrument::Move3D,
                    "Двигать",
                );
                ui.selectable_value(
                    &mut self.instrument,
                    crate::app::logic::Instrument::Rotate3D,
                    "Вращать",
                );
                ui.selectable_value(
                    &mut self.instrument,
                    crate::app::logic::Instrument::Scale3D,
                    "Масштабирование",
                );
                ui.selectable_value(
                    &mut self.instrument,
                    crate::app::logic::Instrument::RotateAroundX,
                    "Вращать вокруг X",
                );
                ui.selectable_value(
                    &mut self.instrument,
                    crate::app::logic::Instrument::RotateAroundY,
                    "Вращать вокруг Y",
                );
                ui.selectable_value(
                    &mut self.instrument,
                    crate::app::logic::Instrument::RotateAroundZ,
                    "Вращать вокруг Z",
                );
            });

        if ui.button("Отразить по XY").clicked() {
            self.apply_reflection(crate::app::logic::ReflectionPlane::XY);
        }

        if ui.button("Отразить по XZ").clicked() {
            self.apply_reflection(crate::app::logic::ReflectionPlane::XZ);
        }

        if ui.button("Отразить по YZ").clicked() {
            self.apply_reflection(crate::app::logic::ReflectionPlane::YZ);
        }

        ui.separator();

        // Выбор модели
        ui.label("Многогранники:");
        if ui.button("тетраэдр").clicked() {
            self.add_tetrahedron();
        }
        if ui.button("гексаэдр").clicked() {
            self.add_hexahedron();
        }
        if ui.button("октаэдр").clicked() {
            self.add_octahedron();
        }
        if ui.button("икосаэдр").clicked() {
            self.add_icosahedron();
        }
        if ui.button("додекаэдр").clicked() {
            self.add_dodecahedron();
        }

        ui.separator();

        // Ввод координат для произвольной оси
        ui.label("Custom Axis Rotation:");

        ui.label("Axis Point 1:");
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(&mut self.axis_point1.x)
                    .speed(0.1)
                    .prefix("X:"),
            );
            ui.add(
                egui::DragValue::new(&mut self.axis_point1.y)
                    .speed(0.1)
                    .prefix("Y:"),
            );
            ui.add(
                egui::DragValue::new(&mut self.axis_point1.z)
                    .speed(0.1)
                    .prefix("Z:"),
            );
        });

        ui.label("Axis Point 2:");
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(&mut self.axis_point2.x)
                    .speed(0.1)
                    .prefix("X:"),
            );
            ui.add(
                egui::DragValue::new(&mut self.axis_point2.y)
                    .speed(0.1)
                    .prefix("Y:"),
            );
            ui.add(
                egui::DragValue::new(&mut self.axis_point2.z)
                    .speed(0.1)
                    .prefix("Z:"),
            );
        });

        // Кнопка для вращения вокруг произвольной линии с перетаскиванием
        if ui.button("Rotate Line").clicked() {
            self.instrument = crate::app::logic::Instrument::RotateAroundCustomLine;
        }
    }

    /// Показать панель в контексте операций над камерой.
    fn show_camera_settings(&mut self, ui: &mut egui::Ui) {}

    /// Показать нижнюю панель приложения.
    fn show_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("инструмент: {}", self.instrument.to_string()));
                ui.separator();
                // ui.label(format!(
                //     "размер холста: {:.1} x {:.1}",
                //     self.painter_width, self.painter_height
                // ));
            });
        });
    }

    /// Показать центральную (основную) панель приложения.
    fn show_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Выделить область под холст
            let (canvas_response, painter) = self.allocate_canvas(ui);
            let canvas_rect = canvas_response.rect;

            self.display_canvas_width = canvas_rect.width();
            self.display_canvas_height = canvas_rect.height();

            // Вывести текущую сцену на экран
            self.render_scene();
            self.update_texture(ctx);
            if let Some(texture) = &self.texture_handle {
                painter.image(
                    texture.id(),
                    canvas_rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        });
    }
}
