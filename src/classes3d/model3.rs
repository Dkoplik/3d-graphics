use crate::{CoordFrame, HVec3, Material, Mesh, Model3, Point3, Transform3D, Vec3};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

impl Model3 {
    /// Создать модель из Mesh'а, материал дефолтный.
    pub fn from_mesh(mesh: Mesh) -> Self {
        Model3 {
            mesh,
            material: Material::default(),
        }
    }

    /// Применить трансформацию к объекту.
    ///
    /// Так как за положение в пространстве отвечает Mesh, по факту, этот метод просто
    /// передаёт трансформацию в Mesh.
    pub fn apply_transform(&mut self, transform: &Transform3D) {
        self.mesh.get_local_frame_mut().apply_transform(transform);
    }

    /// Сместить модель на вектор `delta`.
    ///
    /// Просто синтаксический сахар для более удобных операций над моделькой.
    pub fn translate(&mut self, delta: Vec3) {
        let translation = Transform3D::translation_vec(delta);
        self.apply_transform(&translation);
    }

    /// Повернуть модель относительно оси.
    ///
    /// Просто синтаксический сахар для более удобных операций над моделькой.
    pub fn rotate_around_axis(&mut self, axis: Vec3, angle_rad: f32) {
        let rotation = Transform3D::rotation_around_axis(axis, angle_rad);
        self.apply_transform(&rotation);
    }

    /// Масштабирование модели.
    ///
    /// Просто синтаксический сахар для более удобных операций над моделькой.
    pub fn scale(&mut self, sx: f32, sy: f32, sz: f32) {
        let scale_transform = Transform3D::scale(sx, sy, sz);
        self.apply_transform(&scale_transform);
    }

    /// Текущая позиция модели
    pub fn get_position(&self) -> Point3 {
        self.mesh.get_local_frame().origin
    }

    /// Поставить модель в новую позицию.
    ///
    /// Просто синтаксический сахар для более удобных операций над моделькой.
    pub fn set_position(&mut self, position: Point3) {
        let current_frame = *self.mesh.get_local_frame();
        let new_frame = CoordFrame::new(
            current_frame.forward(),
            current_frame.right(),
            current_frame.up(),
            position,
        );
        *self.mesh.get_local_frame_mut() = new_frame;
    }

    /// Загузить и создать модель из .obj файла
    ///
    /// По идее, .obj файла должно хватить для всей информации о Mesh модели,
    /// но при этом материал и текстура там вроде не хранятся.
    pub fn load_from_obj(file_path: &str) -> Result<Model3, ObjLoadError> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(ObjLoadError::FileNotFound);
        }

        let file = File::open(file_path).map_err(|_| ObjLoadError::FileNotFound)?;
        let reader = BufReader::new(file);

        let mut vertexes = Vec::new();
        let mut polygons = Vec::new();
        // let mut current_line = 0;

        for line in reader.lines() {
            // current_line += 1;
            let line = line.map_err(|_| ObjLoadError::InvalidFormat)?;
            let trimmed = line.trim();

            // Пропускаем комментарии и пустые строки
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            match parts[0] {
                "v" => {
                    // Vertex: v x y z [w]
                    if parts.len() >= 4 {
                        let x = parts[1]
                            .parse::<f32>()
                            .map_err(|_| ObjLoadError::InvalidFormat)?;
                        let y = parts[2]
                            .parse::<f32>()
                            .map_err(|_| ObjLoadError::InvalidFormat)?;
                        let z = parts[3]
                            .parse::<f32>()
                            .map_err(|_| ObjLoadError::InvalidFormat)?;

                        vertexes.push(HVec3::new(x, y, z));
                    }
                }
                "f" => {
                    // Face: f v1 v2 v3 ...
                    if parts.len() >= 4 {
                        let mut face_vertex_indices = Vec::new();

                        for i in 1..parts.len() {
                            // OBJ формат может быть: "v", "v/vt", или "v/vt/vn"
                            // Нас интересует только индекс вершины
                            let vertex_part = parts[i].split('/').next().unwrap();
                            let vertex_index = vertex_part
                                .parse::<i32>()
                                .map_err(|_| ObjLoadError::InvalidFormat)?;

                            // OBJ индексы начинаются с 1, наши с 0
                            if vertex_index > 0 {
                                if (vertex_index as usize) <= vertexes.len() {
                                    face_vertex_indices.push((vertex_index - 1) as usize);
                                } else {
                                    return Err(ObjLoadError::InvalidFormat);
                                }
                            } else if vertex_index < 0 {
                                // Отрицательные индексы (относительные)
                                let actual_index = (vertexes.len() as i32 + vertex_index) as usize;
                                if actual_index < vertexes.len() {
                                    face_vertex_indices.push(actual_index);
                                } else {
                                    return Err(ObjLoadError::InvalidFormat);
                                }
                            }
                        }

                        if face_vertex_indices.len() >= 3 {
                            polygons.push(crate::classes3d::mesh::Polygon3::from_list(
                                &face_vertex_indices,
                            ));
                        }
                    }
                }
                "vt" | "vn" | "vp" => {
                    // Пока игнорируем текстурные координаты, нормали и параметрические вершины
                    continue;
                }
                _ => {
                    // Игнорируем неизвестные типы
                    continue;
                }
            }
        }

        if vertexes.is_empty() || polygons.is_empty() {
            return Err(ObjLoadError::InvalidFormat);
        }

        // Создаем Mesh из вершин и полигонов
        let mesh = Mesh::from_polygons(vertexes, polygons);

        Ok(Model3::from_mesh(mesh))
    }

    /// Сохранить текущую модель в .obj файл
    pub fn save_to_obj(&self, file_path: &str) -> Result<(), ObjSaveError> {
        let mut file = File::create(file_path).map_err(|_| ObjSaveError::WriteError)?;

        // Записываем заголовок
        writeln!(file, "# Wavefront OBJ file exported from AthenianApp")
            .map_err(|_| ObjSaveError::WriteError)?;
        writeln!(
            file,
            "# Vertices: {}, Faces: {}",
            self.mesh.vertexes.len(),
            self.mesh.polygons.len()
        )
        .map_err(|_| ObjSaveError::WriteError)?;
        writeln!(file).map_err(|_| ObjSaveError::WriteError)?;

        // Создаем карту для быстрого поиска индексов вершин по координатам
        let vertex_map = self.create_vertex_coordinate_map();

        // Записываем вершины
        for vertex in &self.mesh.vertexes {
            writeln!(file, "v {:.6} {:.6} {:.6}", vertex.x, vertex.y, vertex.z)
                .map_err(|_| ObjSaveError::WriteError)?;
        }

        writeln!(file).map_err(|_| ObjSaveError::WriteError)?;

        // Записываем полигоны
        for polygon in &self.mesh.polygons {
            write!(file, "f").map_err(|_| ObjSaveError::WriteError)?;

            for &vertex_index in polygon.get_vertexes() {
                if vertex_index < self.mesh.vertexes.len() {
                    let vertex = &self.mesh.vertexes[vertex_index];

                    // Ищем соответствующий индекс в сохраненных вершинах
                    if let Some(&saved_index) =
                        vertex_map.get(&Self::quantize_coordinates(vertex.x, vertex.y, vertex.z))
                    {
                        write!(file, " {}", saved_index + 1)
                            .map_err(|_| ObjSaveError::WriteError)?;
                    } else {
                        return Err(ObjSaveError::InvalidData);
                    }
                } else {
                    return Err(ObjSaveError::InvalidData);
                }
            }

            writeln!(file).map_err(|_| ObjSaveError::WriteError)?;
        }

        Ok(())
    }

    /// Создает карту координат вершин для быстрого поиска
    fn create_vertex_coordinate_map(&self) -> HashMap<(i32, i32, i32), usize> {
        let mut map = HashMap::new();

        for (i, vertex) in self.mesh.vertexes.iter().enumerate() {
            let key = Self::quantize_coordinates(vertex.x, vertex.y, vertex.z);
            map.insert(key, i);
        }

        map
    }

    /// Квантует координаты для сравнения с допуском
    fn quantize_coordinates(x: f32, y: f32, z: f32) -> (i32, i32, i32) {
        let scale = 10000.0; // Для точности до 0.0001
        (
            (x * scale).round() as i32,
            (y * scale).round() as i32,
            (z * scale).round() as i32,
        )
    }
}

/// Ошибки при чтении obj файлов
#[derive(Debug)]
pub enum ObjLoadError {
    FileNotFound,
    InvalidFormat,
    UnsupportedFeature,
}

/// Ошибки при сохранении (записи) в obj файлы
#[derive(Debug)]
pub enum ObjSaveError {
    WriteError,
    InvalidData,
}
