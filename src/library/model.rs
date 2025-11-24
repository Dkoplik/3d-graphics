use crate::{CoordFrame, UVec3};

use super::primitives::{Point3, Transform3D, Vec3};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

// составные части модели
mod material;
mod mesh;
mod surface_generator;
mod texture;

// re-export в модуль `model`
pub use material::*;
pub use mesh::*;
pub use surface_generator::*;
pub use texture::*;

/// Модель (объект) в 3D пространстве.
///
/// По сути просто контейнер для Mesh'а и его материала, где Mesh задаёт форму модели, а материал отображение (цвет).
#[derive(Debug, Clone)]
pub struct Model {
    /// Mesh модели.
    pub mesh: Mesh,
    /// Материал модели.
    pub material: Material,
}

impl Model {
    // --------------------------------------------------
    // Конструкторы
    // --------------------------------------------------

    /// Создать модель из Mesh'а, материал дефолтный.
    pub fn from_mesh(mesh: Mesh) -> Self {
        Self {
            mesh,
            material: Material::default(),
        }
    }

    /// Загузить и создать модель из .obj файла
    ///
    /// По идее, .obj файла должно хватить для всей информации о Mesh модели,
    /// но при этом материал и текстура там вроде не хранятся.
    pub fn load_from_obj(file_path: &str) -> Result<Self, ObjLoadError> {
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

                        vertexes.push(Point3::new(x, y, z));
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
                            polygons.push(Polygon::from_list(&face_vertex_indices));
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

        Ok(Self::from_mesh(mesh))
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
            self.mesh.vertex_count(),
            self.mesh.polygon_count(),
        )
        .map_err(|_| ObjSaveError::WriteError)?;
        writeln!(file).map_err(|_| ObjSaveError::WriteError)?;

        // Создаем карту для быстрого поиска индексов вершин по координатам
        let vertex_map = self.create_vertex_coordinate_map();

        // Записываем вершины
        for vertex in self.mesh.get_local_vertex_iter() {
            writeln!(file, "v {:.6} {:.6} {:.6}", vertex.x, vertex.y, vertex.z)
                .map_err(|_| ObjSaveError::WriteError)?;
        }

        writeln!(file).map_err(|_| ObjSaveError::WriteError)?;

        // Записываем полигоны
        for polygon in self.mesh.get_polygon_iter() {
            write!(file, "f").map_err(|_| ObjSaveError::WriteError)?;

            for vertex_index in polygon.get_mesh_vertex_index_iter() {
                if vertex_index < self.mesh.vertex_count() {
                    let vertex = self.mesh.get_local_vertex(vertex_index);

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

        for (i, vertex) in self.mesh.get_local_vertex_iter().enumerate() {
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

    // --------------------------------------------------
    // Синтаксический сахар для преобразований
    // --------------------------------------------------

    /// Сдвинуть Mesh на вектор `delta`.
    pub fn translate(&mut self, delta: Vec3) {
        self.mesh.local_frame.translate_vec(delta);
    }

    /// Сдвинуть Mesh по оси X.
    pub fn move_x(&mut self, dx: f32) {
        self.mesh.local_frame.translate_vec(Vec3::new(dx, 0.0, 0.0));
    }

    /// Сдвинуть Mesh по оси Y.
    pub fn move_y(&mut self, dy: f32) {
        self.mesh.local_frame.translate_vec(Vec3::new(0.0, dy, 0.0));
    }

    /// Сдвинуть Mesh по оси Z.
    pub fn move_z(&mut self, dz: f32) {
        self.mesh.local_frame.translate_vec(Vec3::new(0.0, 0.0, dz));
    }

    /// Повернуть модель из направления `from` в направление `to` в **локальных** координатах.
    ///
    /// Сами `from` и `to` указываются в **глобальных** координатах.
    pub fn rotate(&mut self, from: UVec3, to: UVec3) {
        // привести к локальным координатам модели
        self.mesh
            .local_frame
            .rotate(Transform3D::rotation_aligning(from, to));
    }

    /// Повернуть модель вокруг **локальной** оси X.
    pub fn rotate_local_x(&mut self, angle: f32) {
        let right = self.mesh.local_frame.right();
        self.mesh
            .local_frame
            .rotate(Transform3D::rotation_around_axis(right, angle));
    }

    /// Повернуть модель вокруг **локальной** оси Y.
    pub fn rotate_local_y(&mut self, angle: f32) {
        let up = self.mesh.local_frame.up();
        self.mesh
            .local_frame
            .rotate(Transform3D::rotation_around_axis(up, angle));
    }

    /// Повернуть модель вокруг **локальной** оси Z.
    pub fn rotate_local_z(&mut self, angle: f32) {
        let forward = self.mesh.local_frame.forward();
        self.mesh
            .local_frame
            .rotate(Transform3D::rotation_around_axis(forward, angle));
    }

    pub fn scale_vec(&mut self, vec: Vec3) {
        self.mesh.local_frame.scale_by_vec(vec);
    }

    pub fn uniform_scale(&mut self, scale: f32) {
        self.mesh
            .local_frame
            .scale_by_vec(Vec3::new(scale, scale, scale));
    }

    /// Отразить модель в плоскости XY относительно **локальных координат**.
    pub fn reflect_local_xy(&mut self) {
        self.mesh.local_frame.reflect_xy();
    }

    /// Отразить модель в плоскости XZ относительно **локальных координат**.
    pub fn reflect_local_xz(&mut self) {
        self.mesh.local_frame.reflect_xz();
    }

    /// Отразить модель в плоскости YZ относительно **локальных координат**.
    pub fn reflect_local_yz(&mut self) {
        self.mesh.local_frame.reflect_yz();
    }

    /// Текущая позиция модели
    pub fn get_position(&self) -> Point3 {
        self.mesh.local_frame.origin
    }

    /// Поставить модель в новую позицию.
    ///
    /// Просто синтаксический сахар для более удобных операций над моделькой.
    pub fn set_position(&mut self, position: Point3) {
        let current_frame = self.mesh.local_frame;
        let new_frame = CoordFrame::new(
            current_frame.forward(),
            current_frame.right(),
            current_frame.up(),
            position,
        );
        self.mesh.local_frame = new_frame;
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

#[cfg(test)]
mod model_tests {
    use super::*;
    use crate::HVec3;

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

    fn assert_uvecs(got: UVec3, expected: UVec3, tolerance: f32) {
        assert!(
            got.approx_equal(expected, tolerance),
            "ожидался unit-вектор {:?}, но получен unit-вектор {:?}, одна из координат которого отличается более чем на {}",
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
    fn test_move_x() {
        let mut model = Model::from_mesh(Mesh::dodecahedron());
        let mut expected_pos = model.get_position();

        model.move_x(5.0);
        expected_pos.x += 5.0;
        assert_points(model.get_position(), expected_pos, TOLERANCE);
    }

    #[test]
    fn test_move_y() {
        let mut model = Model::from_mesh(Mesh::dodecahedron());
        let mut expected_pos = model.get_position();

        model.move_y(5.0);
        expected_pos.y += 5.0;
        assert_points(model.get_position(), expected_pos, TOLERANCE);
    }

    #[test]
    fn test_move_z() {
        let mut model = Model::from_mesh(Mesh::dodecahedron());
        let mut expected_pos = model.get_position();

        model.move_z(5.0);
        expected_pos.z += 5.0;
        assert_points(model.get_position(), expected_pos, TOLERANCE);
    }

    #[test]
    fn test_rotate_1() {
        let mut model = Model::from_mesh(Mesh::dodecahedron());

        model.translate(Vec3::new(2.0, 2.0, 3.0));
        model.rotate(UVec3::forward(), UVec3::up());

        assert_points(model.get_position(), Point3::new(2.0, 2.0, 3.0), TOLERANCE);

        assert_uvecs(model.mesh.local_frame.forward(), UVec3::up(), TOLERANCE);
        assert_uvecs(model.mesh.local_frame.right(), UVec3::right(), TOLERANCE);
        assert_uvecs(model.mesh.local_frame.up(), UVec3::backward(), TOLERANCE);
    }

    #[test]
    fn test_rotate_2() {
        let mut model = Model::from_mesh(Mesh::dodecahedron());

        model.translate(Vec3::new(2.0, 2.0, 3.0));
        model.rotate(UVec3::forward(), UVec3::right());

        assert_points(model.get_position(), Point3::new(2.0, 2.0, 3.0), TOLERANCE);

        assert_uvecs(model.mesh.local_frame.forward(), UVec3::right(), TOLERANCE);
        assert_uvecs(model.mesh.local_frame.right(), UVec3::backward(), TOLERANCE);
        assert_uvecs(model.mesh.local_frame.up(), UVec3::up(), TOLERANCE);
    }

    #[test]
    fn test_rotate_x() {
        let mut model = Model::from_mesh(Mesh::dodecahedron());

        model.translate(Vec3::new(2.0, 2.0, 3.0));
        model.rotate_local_x((-90.0 as f32).to_radians());

        assert_points(model.get_position(), Point3::new(2.0, 2.0, 3.0), TOLERANCE);

        assert_uvecs(model.mesh.local_frame.forward(), UVec3::up(), TOLERANCE);
        assert_uvecs(model.mesh.local_frame.right(), UVec3::right(), TOLERANCE);
        assert_uvecs(model.mesh.local_frame.up(), UVec3::backward(), TOLERANCE);
    }

    #[test]
    fn test_translated() {
        let mut cube = Model::from_mesh(Mesh::hexahedron());
        // сдвиг по x
        cube.move_x(10.0);

        let global_normals: Vec<UVec3> = cube.mesh.get_global_normals_iter().unwrap().collect();
        let local_normals: Vec<UVec3> = cube.mesh.get_local_normals_iter().unwrap().collect();
        for i in 0..global_normals.len() {
            assert_uvecs(global_normals[i], local_normals[i], TOLERANCE);
        }

        let global_vertexes: Vec<Point3> = cube.mesh.get_global_vertex_iter().collect();
        let local_vertexes: Vec<Point3> = cube.mesh.get_local_vertex_iter().collect();
        for i in 0..global_normals.len() {
            assert_points(
                global_vertexes[i],
                local_vertexes[i] + Vec3::new(10.0, 0.0, 0.0),
                TOLERANCE,
            );
        }
    }
}
