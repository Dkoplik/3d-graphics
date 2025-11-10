use crate::{CoordFrame, Material, Mesh, Model3, Point3, Transform3D, Vec3};

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

    /// Поставить модель в новую позицию.
    ///
    /// Просто синтаксический сахар для более удобных операций над моделькой.
    pub fn set_position(&mut self, position: Point3) {
        let current_frame = *self.mesh.get_local_frame();
        let new_frame =
            CoordFrame::new(current_frame.x, current_frame.y, current_frame.z, position);
        *self.mesh.get_local_frame_mut() = new_frame;
    }

    /// Загузить и создать модель из .obj файла
    ///
    /// По идее, .obj файла должно хватить для всей информации о Mesh модели,
    /// но при этом материал и текстура там вроде не хранятся.
    fn load_from_obj(file_path: &str) -> Result<Model3, ObjLoadError> {
        // TODO
        todo!("Реализовать чтение из obj файла")
    }

    /// Сохранить текущую модель в .obj файл
    fn save_to_obj(&self, file_path: &str) -> Result<(), ObjSaveError> {
        // TODO
        todo!("Реализовать сохранение в obj файл")
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
