use egui::Painter;

use crate::{Model3, Point3, Polygon3, RenderStyle, Transform3D, Transformable3};

impl Model3 {
    pub fn new(origin: Point3, vertexes: Vec<Point3>, polygons: Vec<Polygon3>) -> Self {
        Model3 {
            origin,
            vertexes,
            polygons,
        }
    }

    /// Создание тетраэдра со сторонами единичной длины.
    pub fn tetrahedron() -> Self {
        // TODO
        todo!("Сделать тетраэдр");
    }

    /// Создание гексаэдра со сторонами единичной длины.
    pub fn hexahedron() -> Self {
        // TODO
        todo!("Сделать гексаэдр");
    }

    /// Создание октаэдра со сторонами единичной длины.
    pub fn octahedron() -> Self {
        // TODO
        todo!("Сделать октаэдр");
    }

    /// Создание икосаэдр со сторонами единичной длины.
    pub fn icosahedron() -> Self {
        // TODO
        todo!("Сделать икосаэдр");
    }

    /// Создание тетраэдра со сторонами единичной длины.
    pub fn dodecahedron() -> Self {
        // TODO
        todo!("Сделать додекаэдр");
    }

    /// Получить матрицу преобразования к мировым координатам.
    pub fn get_world_transform(&self) -> Transform3D {
        Transform3D::translation(self.origin.x, self.origin.y, self.origin.z)
    }

    /// Получить копию модели, но в глобальных координатах.
    pub fn to_world_coordinates(self) -> Self {
        let transform = self.get_world_transform();
        self.transform(transform)
    }

    /// Нарисовать модель.
    pub fn draw(&self, painter: &mut Painter, style: &RenderStyle) {
        self.polygons.iter().for_each(|polygon| {
            polygon.draw(&self.vertexes, painter, style.edge_color, style.edge_width)
        });
        self.vertexes
            .iter()
            .for_each(|vertex| vertex.draw(painter, style.vertex_color, style.vertex_radius));
    }
}

impl Transformable3 for Model3 {
    fn transform(self, transform: crate::Transform3D) -> Self {
        let origin = self.origin.transform(transform);
        let vertexes = self
            .vertexes
            .iter()
            .cloned()
            .map(|vertex| vertex.transform(transform))
            .collect();
        Self::new(origin, vertexes, self.polygons)
    }

    fn apply_transform(&mut self, transform: crate::Transform3D) {
        self.origin.apply_transform(transform);
        self.vertexes
            .iter_mut()
            .for_each(|vertex| vertex.apply_transform(transform));
    }
}
