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

                    ui.separator();

                    // Выбор многогранников
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

                    ui.label("3D инструменты:");

                    if ui.button("3D Move").clicked() {
                        self.instrument = crate::app::logic::Instrument::Move3D;
                    }

                    if ui.button("3D Rotate").clicked() {
                        self.instrument = crate::app::logic::Instrument::Rotate3D;
                    }

                    if ui.button("3D Scale").clicked() {
                        self.instrument = crate::app::logic::Instrument::Scale3D;
                    }

                    if ui.button("Reflect XY").clicked() {
                        self.apply_reflection(crate::app::logic::ReflectionPlane::XY);
                    }

                    if ui.button("Reflect XZ").clicked() {
                        self.apply_reflection(crate::app::logic::ReflectionPlane::XZ);
                    }

                    if ui.button("Reflect YZ").clicked() {
                        self.apply_reflection(crate::app::logic::ReflectionPlane::YZ);
                    }

                    if ui.button("Rotate X").clicked() {
                        self.instrument = crate::app::logic::Instrument::RotateAroundX;
                    }

                    if ui.button("Rotate Y").clicked() {
                        self.instrument = crate::app::logic::Instrument::RotateAroundY;
                    }

                    if ui.button("Rotate Z").clicked() {
                        self.instrument = crate::app::logic::Instrument::RotateAroundZ;
                    }

                    ui.separator();

                    // Ввод координат для произвольной оси
                    ui.label("Custom Axis Rotation:");

                    ui.label("Axis Point 1:");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.axis_point1_x)
                                .speed(0.1)
                                .prefix("X:"),
                        );
                        ui.add(
                            egui::DragValue::new(&mut self.axis_point1_y)
                                .speed(0.1)
                                .prefix("Y:"),
                        );
                        ui.add(
                            egui::DragValue::new(&mut self.axis_point1_z)
                                .speed(0.1)
                                .prefix("Z:"),
                        );
                    });

                    ui.label("Axis Point 2:");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.axis_point2_x)
                                .speed(0.1)
                                .prefix("X:"),
                        );
                        ui.add(
                            egui::DragValue::new(&mut self.axis_point2_y)
                                .speed(0.1)
                                .prefix("Y:"),
                        );
                        ui.add(
                            egui::DragValue::new(&mut self.axis_point2_z)
                                .speed(0.1)
                                .prefix("Z:"),
                        );
                    });

                    // Кнопка для вращения вокруг произвольной линии с перетаскиванием
                    if ui.button("Rotate Line").clicked() {
                        self.instrument = crate::app::logic::Instrument::RotateAroundCustomLine;
                    }
                });
            });
    }

    /// Показать нижнюю панель приложения.
    fn show_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("инструмент: {}", self.instrument.to_string()));
                ui.separator();
                ui.label(format!(
                    "размер холста: {:.1} x {:.1}",
                    self.painter_width, self.painter_height
                ));
            });
        });
    }

    /// Показать центральную (основную) панель приложения.
    fn show_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Resize::default()
                .default_size(egui::Vec2 { x: 900.0, y: 600.0 })
                .show(ui, |ui| {
                    let (response, painter) = self.allocate_painter(ui);

                    // Создаем mutable reference из painter
                    let mut painter_mut = painter;
                    self.draw_canvas(&mut painter_mut);

                    self.handle_input(&response);
                });
        });
    }
}
