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
                    if ui.button("Dimetric").clicked() {
                        self.set_dimetric_projection();
                    }
                    if ui.button("Trimetric").clicked() {
                        self.set_trimetric_projection();
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
                    ui.label("Polyhedra:");
                    if ui.button("Tetrahedron").clicked() {
                        self.add_tetrahedron();
                    }
                    if ui.button("Hexahedron").clicked() {
                        self.add_hexahedron();
                    }
                    if ui.button("Octahedron").clicked() {
                        self.add_octahedron();
                    }

                    ui.separator();

                    ui.label("3D Tools:");

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
                        self.instrument = crate::app::logic::Instrument::ReflectXY;
                    }

                    if ui.button("Reflect XZ").clicked() {
                        self.instrument = crate::app::logic::Instrument::ReflectXZ;
                    }

                    if ui.button("Reflect YZ").clicked() {
                        self.instrument = crate::app::logic::Instrument::ReflectYZ;
                    }

                    if ui.button("Rotate Around Axis").clicked() {
                        self.instrument = crate::app::logic::Instrument::RotateAroundAxis;
                    }

                    if ui.button("Set Axis Point 1").clicked() {
                        self.instrument = crate::app::logic::Instrument::SetAxisPoint1;
                    }

                    if ui.button("Set Axis Point 2").clicked() {
                        self.instrument = crate::app::logic::Instrument::SetAxisPoint2;
                    }

                    if ui.button("Set Point").clicked() {
                        self.instrument = crate::app::logic::Instrument::SetPoint;
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
                ui.label(format!("Projection: {}", self.current_projection.to_string()));
                ui.separator();
                ui.label(format!("размер холста: {:.1} x {:.1}", self.painter_width, self.painter_height));
            });
        });
    }

    /// Показать центральную (основную) панель приложения.
    fn show_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Resize::default()
                .default_size(egui::Vec2 {x: 900.0, y: 600.0})
                .show(ui, |ui| {
                    let (response, painter) = self.allocate_painter(ui);
                    self.draw_canvas(&painter);
                    self.handle_input(&response);
                });
        });
    }
}