use crate::{Mesh, Model3};

impl Model3 {
    pub fn new(mesh: Mesh) -> Self {
        Self { mesh }
    }

    /// Возвращает Mesh модели.
    pub fn get_mesh(&self) -> &Mesh {
        &self.mesh
    }

    /// Возвращает изменяемый Mesh модели.
    pub fn get_mesh_mut(&mut self) -> &mut Mesh {
        &mut self.mesh
    }
}
