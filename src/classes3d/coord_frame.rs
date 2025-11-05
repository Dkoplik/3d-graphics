use crate::{CoordFrame, Point3, Transform3D, Vec3};

impl CoordFrame {
    /// Создать новую координатную систему по 3-м базисам и точке, из которой эти базисы выходят.
    /// Базисы должны быть **ортогональными**.
    pub fn new(x: Vec3, y: Vec3, z: Vec3, origin: Point3) -> Self {
        assert_eq!(x.dot(y), 0.0, "Базисы должны быть ортогональными");
        assert_eq!(x.dot(z), 0.0, "Базисы должны быть ортогональными");
        assert_eq!(z.dot(y), 0.0, "Базисы должны быть ортогональными");
        Self { x, y, z, origin }
    }

    /// Получить матрицу преобразования из текущих локальных координат в глобальные.
    ///
    /// Эта матрица преобразует вектор из локальной системы координат в глобальную.
    pub fn local_to_global_matrix(&self) -> Transform3D {
        // Предполагаем, что локальная система получается из глобальной
        // последовательностью поворотов. Для простоты используем повороты
        // которые выравнивают оси.

        // Вычисляем углы между осями
        let global_x = Vec3::new(1.0, 0.0, 0.0);
        let global_y = Vec3::new(0.0, 1.0, 0.0);
        let global_z = Vec3::new(0.0, 0.0, 1.0);

        // Угол поворота вокруг Z для выравнивания X осей в плоскости XY
        let angle_z = (self.x.y).atan2(self.x.x);

        // Временный вектор после поворота вокруг Z
        let temp_x = Transform3D::rotation_z_rad(angle_z).apply_to_hvec(&global_x.into());
        let temp_y = Transform3D::rotation_z_rad(angle_z).apply_to_hvec(&global_y.into());

        // Угол поворота вокруг Y для выравнивания с конечным X
        let angle_y = (self.x.z).atan2(temp_x.x);

        // Временный вектор после поворота вокруг Y
        let temp_x2 = Transform3D::rotation_y_rad(angle_y)
            .multiply(Transform3D::rotation_z_rad(angle_z))
            .apply_to_hvec(&global_x.into());

        // Угол поворота вокруг X для окончательного выравнивания
        let angle_x = (self.y.z - temp_x2.z).atan2(self.y.y - temp_x2.y);

        // Композиция поворотов в обратном порядке для обратного преобразования
        Transform3D::rotation_z_rad(-angle_z)
            .multiply(Transform3D::rotation_y_rad(-angle_y))
            .multiply(Transform3D::rotation_x_rad(-angle_x))
    }

    /// Получить матрицу преобразования из глобальных координат в текущие локальные.
    ///
    /// Эта матрица преобразует вектор из глобальной системы координат в локальную.
    pub fn global_to_local_matrix(&self) -> Transform3D {
        // Обратное преобразование - используем те же углы, но в прямом порядке
        let global_x = Vec3::new(1.0, 0.0, 0.0);

        let angle_z = (self.x.y).atan2(self.x.x);
        let temp_x = Transform3D::rotation_z_rad(angle_z).apply_to_hvec(&global_x.into());
        let angle_y = (self.x.z).atan2(temp_x.x);
        let temp_x2 = Transform3D::rotation_y_rad(angle_y)
            .multiply(Transform3D::rotation_z_rad(angle_z))
            .apply_to_hvec(&global_x.into());
        let angle_x = (self.y.z - temp_x2.z).atan2(self.y.y - temp_x2.y);

        // Прямые повороты в правильном порядке
        Transform3D::rotation_z_rad(angle_z)
            .multiply(Transform3D::rotation_y_rad(angle_y))
            .multiply(Transform3D::rotation_x_rad(angle_x))
    }

    /// Получить матрицу преобразования из текущих локальных координат в другие локальные координаты.
    pub fn local_to_other_local_matrix(&self, other: &CoordFrame) -> Transform3D {
        other
            .global_to_local_matrix()
            .multiply(self.local_to_global_matrix())
    }

    /// Получить матрицу преобразования из других локальных координат в текущие локальные.
    pub fn other_local_to_local_matrix(&self, other: &CoordFrame) -> Transform3D {
        self.global_to_local_matrix()
            .multiply(other.local_to_global_matrix())
    }
}
