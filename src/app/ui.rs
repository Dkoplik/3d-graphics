use crate::app::{AthenianApp, logic, ui};

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
            });
        });
    }

    /// Показать левую панель приложения.
    fn show_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    if ui.button("Стереть всё").clicked() {
                        self.clear_canvas();
                    }

                    ui.separator();

                    // Вкладки для разных категорий
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::CollapsingHeader::new("Модели")
                            .default_open(true)
                            .show(ui, |ui| {
                                self.show_model_controls(ui);
                            });

                        if let Some(_) = self.get_selected_model_mut() {
                            egui::CollapsingHeader::new("Текущая модель").show(ui, |ui| {
                                self.show_current_model_controls(ui);
                            });
                        }

                        egui::CollapsingHeader::new("Рендер").show(ui, |ui| {
                            self.show_rendering_controls(ui);
                        });

                        egui::CollapsingHeader::new("Освещение").show(ui, |ui| {
                            self.show_lighting_controls(ui);
                        });

                        egui::CollapsingHeader::new("Камера").show(ui, |ui| {
                            self.show_camera_controls(ui);
                        });
                    });
                });
            });
    }

    /// Показать создание моделей.
    fn show_model_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Примитивы:");
        ui.horizontal(|ui| {
            if ui.button("Тетраэдр").clicked() {
                self.add_tetrahedron();
            }
            if ui.button("Куб").clicked() {
                self.add_hexahedron();
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Октаэдр").clicked() {
                self.add_octahedron();
            }
            if ui.button("Икосаэдр").clicked() {
                self.add_icosahedron();
            }
        });
        if ui.button("Додекаэдр").clicked() {
            self.add_dodecahedron();
        }

        ui.separator();

        ui.label("Загрузка моделей:");
        if ui.button("Загрузить OBJ").clicked() {
            self.load_obj_file();
        }
        if ui.button("Сохранить OBJ").clicked() {
            self.save_obj_file();
        }

        ui.separator();

        egui::CollapsingHeader::new("Создание модели вращения")
            .default_open(false)
            .show(ui, |ui| {
                self.show_rotation_model_controls(ui);
            });

        ui.separator();

        egui::CollapsingHeader::new("График функции двух переменных").show(ui, |ui| {
            self.show_function_model_controls(ui);
        });

        ui.separator();

        // Выбор текущей модели
        if !self.scene.models.is_empty() {
            ui.label("Выбранная модель:");
            let cur_model = if let Some(index) = self.selected_3d_model_index {
                format!("{}", index)
            } else {
                "не выбрана".into()
            };
            egui::ComboBox::from_label("")
                .selected_text(format!("Модель {}", cur_model))
                .show_ui(ui, |ui| {
                    for (i, _) in self.scene.models.iter().enumerate() {
                        ui.selectable_value(
                            &mut self.selected_3d_model_index,
                            Some(i),
                            format!("Модель {}", i),
                        );
                    }
                });
        }
    }

    /// Показать управление созданием моделей вращения
    fn show_rotation_model_controls(&mut self, ui: &mut egui::Ui) {
        // Сначала обрабатываем UI элементы, которые не требуют вызовов self
        self.show_rotation_params_ui(ui);

        // Затем обрабатываем кнопки действий
        self.show_rotation_actions(ui);
    }

    fn show_rotation_params_ui(&mut self, ui: &mut egui::Ui) {
        let rotation_params = &mut self.rotation_params;

        ui.label("Модель вращения:");

        // Выбор оси вращения
        ui.label("Ось вращения:");
        egui::ComboBox::from_label("")
            .selected_text(rotation_params.axis_type.name())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut rotation_params.axis_type,
                    logic::AxisType::Center(logic::CenterAxis::X),
                    "Ось X",
                );
                ui.selectable_value(
                    &mut rotation_params.axis_type,
                    logic::AxisType::Center(logic::CenterAxis::Y),
                    "Ось Y",
                );
                ui.selectable_value(
                    &mut rotation_params.axis_type,
                    logic::AxisType::Center(logic::CenterAxis::Z),
                    "Ось Z",
                );
                ui.selectable_value(
                    &mut rotation_params.axis_type,
                    logic::AxisType::Custom,
                    "Произвольная ось",
                );
            });

        if let logic::AxisType::Custom = rotation_params.axis_type {
            ui.label("Произвольная ось:");
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Начало:");
                    ui.add(
                        egui::DragValue::new(&mut rotation_params.custom_axis_start.x)
                            .speed(0.1)
                            .prefix("X:"),
                    );
                    ui.add(
                        egui::DragValue::new(&mut rotation_params.custom_axis_start.y)
                            .speed(0.1)
                            .prefix("Y:"),
                    );
                    ui.add(
                        egui::DragValue::new(&mut rotation_params.custom_axis_start.z)
                            .speed(0.1)
                            .prefix("Z:"),
                    );
                });
                ui.vertical(|ui| {
                    ui.label("Конец:");
                    ui.add(
                        egui::DragValue::new(&mut rotation_params.custom_axis_end.x)
                            .speed(0.1)
                            .prefix("X:"),
                    );
                    ui.add(
                        egui::DragValue::new(&mut rotation_params.custom_axis_end.y)
                            .speed(0.1)
                            .prefix("Y:"),
                    );
                    ui.add(
                        egui::DragValue::new(&mut rotation_params.custom_axis_end.z)
                            .speed(0.1)
                            .prefix("Z:"),
                    );
                });
            });
        }

        ui.label("Количество сегментов:");
        ui.add(egui::Slider::new(&mut rotation_params.segments, 4..=64).text("Сегментов"));

        if !rotation_params.profile_points.is_empty() {
            ui.label("Редактирование точек:");
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for (i, point) in rotation_params.profile_points.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Точка {}:", i));
                            ui.add(egui::DragValue::new(&mut point.x).speed(0.1).prefix("X:"));
                            ui.add(egui::DragValue::new(&mut point.y).speed(0.1).prefix("Y:"));
                            ui.add(egui::DragValue::new(&mut point.z).speed(0.1).prefix("Z:"));
                        });
                    }
                });
        }

        // Информация о профиле
        ui.label(format!(
            "Количество точек профиля: {}",
            rotation_params.profile_points.len()
        ));

        // Предупреждение если недостаточно точек
        if rotation_params.profile_points.len() < 2 {
            ui.colored_label(
                egui::Color32::RED,
                "Профиль должен содержать хотя бы 2 точки",
            );
        }
    }

    fn show_rotation_actions(&mut self, ui: &mut egui::Ui) {
        // Управление точками профиля
        ui.label("Точки профиля:");
        ui.horizontal(|ui| {
            if ui.button("Добавить точку").clicked() {
                if let Some(last_point) = self.rotation_params.profile_points.last() {
                    self.add_profile_point(*last_point);
                } else {
                    self.add_profile_point(g3d::Point3::new(0.0, 0.0, 0.0));
                }
            }
            if ui.button("Удалить точку").clicked() {
                self.remove_last_profile_point();
            }
            if ui.button("Очистить").clicked() {
                self.clear_profile();
            }
        });

        // Кнопки создания и сохранения
        ui.horizontal(|ui| {
            if ui.button("Создать модель вращения").clicked() {
                self.create_rotation_model();
            }

            if ui.button("Сохранить модель").clicked() {
                self.save_rotation_model();
            }
        });
    }

    fn show_function_model_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Функция:");
            egui::ComboBox::from_id_salt("surface_function")
                .selected_text(match self.selected_surface_function {
                    g3d::SurfaceFunction::Paraboloid => "Параболоид",
                    g3d::SurfaceFunction::Saddle => "Седло",
                    g3d::SurfaceFunction::Wave => "Волна",
                    g3d::SurfaceFunction::Ripple => "Пульсация",
                    g3d::SurfaceFunction::Gaussian => "Гаусс",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.selected_surface_function,
                        g3d::SurfaceFunction::Paraboloid,
                        "Параболоид (z = x² + y²)",
                    );
                    ui.selectable_value(
                        &mut self.selected_surface_function,
                        g3d::SurfaceFunction::Saddle,
                        "Седло (z = x² - y²)",
                    );
                    ui.selectable_value(
                        &mut self.selected_surface_function,
                        g3d::SurfaceFunction::Wave,
                        "Волна (z = sin(x)·cos(y))",
                    );
                    ui.selectable_value(
                        &mut self.selected_surface_function,
                        g3d::SurfaceFunction::Ripple,
                        "Пульсация (z = sin(r)/r)",
                    );
                    ui.selectable_value(
                        &mut self.selected_surface_function,
                        g3d::SurfaceFunction::Gaussian,
                        "Гаусс (z = e^(-(x²+y²)))",
                    );
                });
        });

        ui.horizontal(|ui| {
            ui.label("X:");
            ui.add(
                egui::DragValue::new(&mut self.surface_x_min)
                    .speed(0.1)
                    .prefix("от "),
            );
            ui.add(
                egui::DragValue::new(&mut self.surface_x_max)
                    .speed(0.1)
                    .prefix("до "),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Y:");
            ui.add(
                egui::DragValue::new(&mut self.surface_y_min)
                    .speed(0.1)
                    .prefix("от "),
            );
            ui.add(
                egui::DragValue::new(&mut self.surface_y_max)
                    .speed(0.1)
                    .prefix("до "),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Разбиений:");
            ui.add(egui::Slider::new(&mut self.surface_divisions, 10..=200).step_by(5.0));
        });

        if ui.button("Построить график").clicked() {
            self.create_function_model();
        }
    }

    /// Показать управление выбранной моделью
    fn show_current_model_controls(&mut self, ui: &mut egui::Ui) {
        self.show_transform_controls(ui);
        self.show_material_controls(ui);
    }

    /// Показать элементы управления преобразованиями.
    fn show_transform_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Преобразования:");

        egui::ComboBox::from_label("Инструмент")
            .selected_text(self.instrument.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.instrument,
                    logic::Instrument::Move3D,
                    "Перемещение",
                );
                ui.selectable_value(
                    &mut self.instrument,
                    logic::Instrument::Rotate3D,
                    "Вращение",
                );
                ui.selectable_value(&mut self.instrument, logic::Instrument::Scale3D, "Масштаб");
                ui.selectable_value(
                    &mut self.instrument,
                    logic::Instrument::RotateAroundX,
                    "Вращение X",
                );
                ui.selectable_value(
                    &mut self.instrument,
                    logic::Instrument::RotateAroundY,
                    "Вращение Y",
                );
                ui.selectable_value(
                    &mut self.instrument,
                    logic::Instrument::RotateAroundZ,
                    "Вращение Z",
                );
                ui.selectable_value(
                    &mut self.instrument,
                    logic::Instrument::RotateAroundCustomLine,
                    "Вращение линии",
                );
            });

        // Перемещение
        let mut new_pos = self.get_selected_model().unwrap().get_position();
        ui.label("Перемещение:");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut new_pos.x).speed(0.1).prefix("X"));
            ui.add(egui::DragValue::new(&mut new_pos.y).speed(0.1).prefix("Y"));
            ui.add(egui::DragValue::new(&mut new_pos.z).speed(0.1).prefix("Z"));
        });
        let delta_vec = new_pos - self.get_selected_model().unwrap().get_position();
        self.translate_model(delta_vec);

        ui.horizontal(|ui| {
            if ui.button("Масштаб +").clicked() {
                self.scale_model(1.2);
            }
            if ui.button("Масштаб -").clicked() {
                self.scale_model(0.8);
            }
        });

        // Отражения
        ui.label("Отражения:");
        ui.horizontal(|ui| {
            if ui.button("XY").clicked() {
                self.apply_reflection(logic::ReflectionPlane::XY);
            }
            if ui.button("XZ").clicked() {
                self.apply_reflection(logic::ReflectionPlane::XZ);
            }
            if ui.button("YZ").clicked() {
                self.apply_reflection(logic::ReflectionPlane::YZ);
            }
        });

        // Произвольная ось вращения
        if self.instrument == logic::Instrument::RotateAroundCustomLine {
            ui.label("Произвольная ось:");
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Точка 1:");
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
                });
            });
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Точка 2:");
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
                });
            });
            ui.add(
                egui::DragValue::new(&mut self.angle_of_rotate)
                    .range(0.0..=360.0)
                    .suffix("°"),
            );
            if ui.button("Применить вращение").clicked() {
                self.apply_custom_rotation();
            }
        }
    }

    /// Показать управление материалами и текстурами.
    fn show_material_controls(&mut self, ui: &mut egui::Ui) {
        let material = &mut self.get_selected_model_mut().unwrap().material;

        ui.label("Материал:");

        ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut material.color);
            ui.label("Цвет");
        });

        ui.separator();

        ui.label("Тип совмещения:");
        egui::ComboBox::from_id_salt("blending")
            .selected_text(material.blend_mode.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut material.blend_mode,
                    g3d::classes3d::material::TextureBlendMode::Replace,
                    g3d::classes3d::material::TextureBlendMode::Replace.to_string(),
                );
                ui.selectable_value(
                    &mut material.blend_mode,
                    g3d::classes3d::material::TextureBlendMode::Modulate,
                    g3d::classes3d::material::TextureBlendMode::Modulate.to_string(),
                );
                ui.selectable_value(
                    &mut material.blend_mode,
                    g3d::classes3d::material::TextureBlendMode::Additive,
                    g3d::classes3d::material::TextureBlendMode::Additive.to_string(),
                );
            });

        ui.separator();

        ui.label("Текстуры:");
        if ui.button("Загрузить текстуру...").clicked() {
            self.load_texture();
        }
        if ui.button("Удалить текстуру").clicked() {
            self.remove_texture();
        }
    }

    /// Показать настройки рендеринга.
    fn show_rendering_controls(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.scene_renderer.render_wireframe, "Рендер каркаса");
        ui.checkbox(&mut self.scene_renderer.render_normals, "Рендер нормалей");
        ui.checkbox(&mut self.scene_renderer.render_solid, "Рендер полигонов");
        ui.checkbox(
            &mut self.scene_renderer.backface_culling,
            "Отсечение задних граней",
        );
        ui.checkbox(&mut self.scene_renderer.z_buffer_enabled, "Z-буфер");

        ui.label("Шейдинг:");
        egui::ComboBox::from_label("Модель")
            .selected_text(self.scene_renderer.shading_type.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.scene_renderer.shading_type,
                    g3d::classes3d::scene_renderer::ShadingType::None,
                    g3d::classes3d::scene_renderer::ShadingType::None.to_string(),
                );
                ui.selectable_value(
                    &mut self.scene_renderer.shading_type,
                    g3d::classes3d::scene_renderer::ShadingType::GouraudLambert,
                    g3d::classes3d::scene_renderer::ShadingType::GouraudLambert.to_string(),
                );
                ui.selectable_value(
                    &mut self.scene_renderer.shading_type,
                    g3d::classes3d::scene_renderer::ShadingType::PhongToonShading(3),
                    g3d::classes3d::scene_renderer::ShadingType::PhongToonShading(0).to_string(),
                );
            });

        match self.scene_renderer.shading_type {
            g3d::classes3d::scene_renderer::ShadingType::PhongToonShading(mut bands) => {
                ui.add(egui::Slider::new(&mut bands, 1..=256).text("Групп:"));
            }
            _ => (),
        }

        ui.label("Проекция:");
        egui::ComboBox::from_label("Тип проекции")
            .selected_text(self.scene_renderer.projection_type.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.scene_renderer.projection_type,
                    g3d::classes3d::scene_renderer::ProjectionType::Perspective,
                    g3d::classes3d::scene_renderer::ProjectionType::Perspective.to_string(),
                );
                ui.selectable_value(
                    &mut self.scene_renderer.projection_type,
                    g3d::classes3d::scene_renderer::ProjectionType::Parallel,
                    g3d::classes3d::scene_renderer::ProjectionType::Perspective.to_string(),
                );
            });
    }

    /// Показать управление освещением.
    fn show_lighting_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Источники света:");

        if ui.button("Добавить свет").clicked() {
            self.add_light_source();
        }

        if !self.scene.lights.is_empty() {
            egui::ComboBox::from_label("Выбранный свет")
                .selected_text(if let Some(light_index) = self.selected_light_index {
                    format!("Свет {}", light_index)
                } else {
                    "Отсутствует".to_owned()
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_light_index, None, "None".to_owned());
                    for (i, _) in self.scene.lights.iter().enumerate() {
                        ui.selectable_value(
                            &mut self.selected_light_index,
                            Some(i),
                            format!("Свет {}", i),
                        );
                    }
                });

            if let Some(index) = self.selected_light_index {
                if let Some(light) = self.scene.lights.get_mut(index) {
                    ui.label("Позиция:");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut light.position.x)
                                .speed(0.1)
                                .prefix("X:"),
                        );
                        ui.add(
                            egui::DragValue::new(&mut light.position.y)
                                .speed(0.1)
                                .prefix("Y:"),
                        );
                        ui.add(
                            egui::DragValue::new(&mut light.position.z)
                                .speed(0.1)
                                .prefix("Z:"),
                        );
                    });

                    ui.label("Интенсивность:");
                    ui.add(egui::Slider::new(&mut light.intensity, 0.0..=2.0));

                    ui.label("Цвет:");
                    ui.color_edit_button_srgba(&mut light.color);

                    if ui.button("Удалить свет").clicked() {
                        self.scene.lights.remove(index);
                        self.selected_light_index = None;
                    }
                }
            }
        }
    }

    /// Показать управление камерой.
    fn show_camera_controls(&mut self, ui: &mut egui::Ui) {
        let camera = &mut self.scene.camera;

        ui.label("Позиция камеры:");
        ui.horizontal(|ui| {
            let mut new_pos = camera.get_position();
            ui.add(egui::DragValue::new(&mut new_pos.x).speed(0.5).prefix("X:"));
            ui.add(egui::DragValue::new(&mut new_pos.y).speed(0.5).prefix("Y:"));
            ui.add(egui::DragValue::new(&mut new_pos.z).speed(0.5).prefix("Z:"));
            camera.set_position(new_pos);
        });

        let mut new_fov = camera.get_fov_degrees();
        ui.add(egui::Slider::new(&mut new_fov, 30.0..=120.0).text("Поле зрения"));
        camera.set_fov_degrees(new_fov);

        ui.add(
            egui::Slider::new(&mut self.camera_controls.move_speed, 0.1..=2.0)
                .text("Скорость движения"),
        );
        ui.add(
            egui::Slider::new(&mut self.camera_controls.rotate_speed, 0.001..=0.1)
                .text("Скорость вращения"),
        );

        if ui.button("Сброс камеры").clicked() {
            self.reset_camera();
        }
    }

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

    fn show_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Выделить область под холст
            let (canvas_response, painter) = self.allocate_canvas(ui);
            self.display_canvas_width = canvas_response.rect.width();
            self.display_canvas_height = canvas_response.rect.height();

            // Обработка ввода
            self.handle_input(&canvas_response, ctx);

            // Рендеринг сцены
            self.render_scene();
            self.update_texture(ctx);

            // Отображение текстуры
            if let Some(texture) = &self.texture_handle {
                painter.image(
                    texture.id(),
                    canvas_response.rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        });
    }
}
