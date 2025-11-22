//! Объявление и реализация `Polygon` для `Mesh`.

use crate::{Mesh, Point3, UVec3, Vec3};

/// Представление одного полигона модели. Дабы избежать копирования вершин,
/// полигоны только хранят индексы вершин из Mesh'а и ссылку на родительский Mesh.
#[derive(Debug, Clone)]
pub struct Polygon {
    /// Индексы вершин, которые соединяет этот полигон.
    vertex_indexes: Vec<usize>,
    /// Mesh, к которому этот полигон принадлежит.
    parent_mesh: &Mesh,
}

impl Polygon {
    // --------------------------------------------------
    // Конструкторы
    // --------------------------------------------------

    /// Создать треугольник.
    pub fn triangle(parent_mesh: &Mesh, p1: usize, p2: usize, p3: usize) -> Self {
        Self {
            vertex_indexes: vec![p1, p2, p3],
            parent_mesh,
        }
    }

    /// Создать полигон из списка индексов вершин.
    pub fn from_list(parent_mesh: &Mesh, vertex_indexes: &[usize]) -> Self {
        Self {
            vertex_indexes: vertex_indexes.into(),
            parent_mesh,
        }
    }

    /// Создать полигон из вектора индексов.
    pub fn from_vec(parent_mesh: &Mesh, vertex_indexes: Vec<usize>) -> Self {
        Self {
            vertex_indexes,
            parent_mesh,
        }
    }

    // --------------------------------------------------
    // Доступ к элементам
    // --------------------------------------------------

    /// Количество вершин в полигоне.
    pub fn vertex_count(&self) -> usize {
        self.vertex_indexes.len()
    }

    /// Для i-ой вершины полигона возвращает номер этой вершины во всём Mesh'э.
    pub fn get_mesh_vertex_index(&self, i: usize) -> usize {
        debug_assert!(
            i < self.vertex_count(),
            "Попытка получить вершину {} в полигоне из {} вершин",
            i,
            self.vertex_count()
        );
        self.vertex_indexes[i]
    }

    /// Получить i-ую вершину полигона в **локальных** координатах.
    pub fn get_local_vertex(&self, i: usize) -> Point3 {
        debug_assert!(
            i < self.vertex_count(),
            "Попытка получить вершину {} в полигоне из {} вершин",
            i,
            self.vertex_count()
        );
        self.parent_mesh.get_local_vertex(self.vertex_indexes[i])
    }

    /// Получить i-ую вершину полигона в **глобальных** координатах.
    pub fn get_global_vertex(&self, i: usize) -> Point3 {
        debug_assert!(
            i < self.vertex_count(),
            "Попытка получить вершину {} в полигоне из {} вершин",
            i,
            self.vertex_count()
        );
        self.parent_mesh.get_global_vertex(self.vertex_indexes[i])
    }

    /// Получить нормаль i-ой вершины полигона в **локальных** координатах.
    pub fn get_local_normal(&self, i: usize) -> UVec3 {
        debug_assert!(
            i < self.vertex_count(),
            "Попытка получить вершину {} в полигоне из {} вершин",
            i,
            self.vertex_count()
        );
        self.parent_mesh.get_local_normal(self.vertex_indexes[i])
    }

    /// Получить нормаль i-ой вершины полигона в **глобальных** координатах.
    pub fn get_global_normal(&self, i: usize) -> UVec3 {
        debug_assert!(
            i < self.vertex_count(),
            "Попытка получить вершину {} в полигоне из {} вершин",
            i,
            self.vertex_count()
        );
        self.parent_mesh.get_global_normal(self.vertex_indexes[i])
    }

    /// Получить текстурные координаты i-ой вершины полигона.
    pub fn get_texture_coord(&self, i: usize) -> (f32, f32) {
        debug_assert!(
            i < self.vertex_count(),
            "Попытка получить вершину {} в полигоне из {} вершин",
            i,
            self.vertex_count()
        );
        self.parent_mesh.get_texture_coord(self.vertex_indexes[i])
    }

    /// Возвращает итератор по номерам вершин полигона в нумерации из всего Mesh'а.
    pub fn get_mesh_vertex_index_iter(&self) -> impl Iterator<Item = usize> {
        self.vertex_indexes.iter().copied()
    }

    /// Получить итератор по всем вершинам полигона в **локальных** координатах.
    pub fn get_local_vertex_iter(&self) -> impl Iterator<Item = Point3> {
        self.vertex_indexes
            .iter()
            .map(|&i| self.parent_mesh.get_local_vertex(i))
    }

    /// Получить итератор по всем вершинам полигона в **глобальных** координатах.
    pub fn get_global_vertex_iter(&self) -> impl Iterator<Item = Point3> {
        self.vertex_indexes
            .iter()
            .map(|&i| self.parent_mesh.get_global_vertex(i))
    }

    /// Получить итератор по всем нормалям полигона в **локальных** координатах.
    ///
    /// Нормали идут в порядке соответствующих им вершин
    pub fn get_local_normals_iter(&self) -> impl Iterator<Item = UVec3> {
        self.vertex_indexes
            .iter()
            .map(|&i| self.parent_mesh.get_local_normal(i))
    }

    /// Получить итератор по всем нормалям полигона в **глобальных** координатах.
    ///
    /// Нормали идут в порядке соответствующих им вершин
    pub fn get_global_normals_iter(&self) -> impl Iterator<Item = UVec3> {
        self.vertex_indexes
            .iter()
            .map(|&i| self.parent_mesh.get_global_normal(i))
    }

    /// Получить итератор по всем текстурным координатам полигона.
    ///
    /// Текстурные координаты идут в порядке соответсвующих им вершин.
    pub fn get_texture_coord_iter(&self) -> impl Iterator<Item = (f32, f32)> {
        self.vertex_indexes
            .iter()
            .map(|&i| self.parent_mesh.get_texture_coord(i))
    }

    // --------------------------------------------------
    // Вспомогательные методы
    // --------------------------------------------------

    /// Состоит ли полигон только из одной вершины?
    pub fn is_vertex(&self) -> bool {
        self.vertex_indexes.len() == 1
    }

    /// Состоит ли полигон только из одного ребра?
    pub fn is_edge(&self) -> bool {
        self.vertex_indexes.len() == 2
    }

    /// Полигон является треугольником?
    pub fn is_triangle(&self) -> bool {
        self.vertex_indexes.len() == 3
    }

    /// Полигон является четырёхугольником?
    pub fn is_quad(&self) -> bool {
        self.vertex_indexes.len() == 0
    }

    /// Полигон является хотя бы треугольником.
    ///
    /// Иными словами, в нём хотя бы 3 вершины.
    pub fn is_valid(&self) -> bool {
        self.vertex_indexes.len() >= 3
    }

    /// Находится ли точка внутри полигона?
    pub fn is_point_in_convex_polygon(&self, point: Point3) -> bool {
        let n = self.vertex_indexes.len();
        let mut sign = 0.0;

        for i in 0..n {
            let j = (i + 1) % n;
            let vi = self.get_local_vertex(i);
            let vj = self.get_local_vertex(j);
            let edge = vj - vi;
            let to_point = point - vi;

            let cross = edge.dot(to_point);

            if i == 0 {
                sign = cross;
            } else if cross * sign < 0.0 {
                return false; // Разные знаки - точка снаружи
            }
        }

        true
    }

    /// Считает нормаль к полигону как к плоскости в **локальных** координатах.
    ///
    /// Этому методу нужны только позиции вершин.
    pub fn plane_normal(&self, mesh_center: Option<Point3>) -> UVec3 {
        // Необходимо хотя бы 3 вершины для образования плоскости
        if !self.is_valid() {
            return UVec3::new(0.0, 0.0, 1.0);
        }

        // Берем первые 3 вершины (можно любые 3 неколлинеарные), так как мы ищем нормаль к плоскости полигона
        let p0 = self.get_local_vertex(0);
        let p1 = self.get_local_vertex(1);
        let p2 = self.get_local_vertex(2);

        let edge1 = p1 - p0;
        let edge2 = p2 - p0;

        // Векторное произведение дает нормаль к плоскости
        let mut normal = edge1.cross(edge2);

        // Ориентируем нормаль ВНЕ объекта
        if let Some(center) = mesh_center {
            let v0 = Vec3::from(p0);
            let v1 = Vec3::from(p1);
            let v2 = Vec3::from(p2);
            let face_center: Point3 = ((v0 + v1 + v2) / 3.0).into();
            let center_to_face = face_center - center;

            if normal.dot(center_to_face) < 0.0 {
                normal = -normal;
            }
        }

        normal.normalize().unwrap_or(UVec3::new(0.0, 0.0, 1.0))
    }

    /// Считает нормаль к полигону через нормали вершин в **локальных** координатах.
    ///
    /// Соответственно, вершины полигона должны содежрать свои нормали перед использованием метода.
    pub fn smoothed_local_normal(&self) -> UVec3 {
        let normals_sum = self
            .get_local_normals_iter()
            .fold(Vec3::zero(), |acc, n| acc + n);

        (normals_sum / self.vertex_count() as f32)
            .normalize()
            .unwrap_or(UVec3::new(0.0, 0.0, 1.0))
    }

    /// Считает нормаль к полигону через нормали вершин в **глобальных** координатах.
    ///
    /// Соответственно, вершины полигона должны содежрать свои нормали перед использованием метода.
    pub fn smoothed_global_normal(&self) -> UVec3 {
        let normals_sum = self
            .get_global_normals_iter()
            .fold(Vec3::zero(), |acc, n| acc + n);

        (normals_sum / self.vertex_count() as f32)
            .normalize()
            .unwrap_or(UVec3::new(0.0, 0.0, 1.0))
    }
}
