use crate::{Mesh, Point3, Polygon};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SurfaceFunction {
    #[default]
    Paraboloid,
    Saddle,
    Wave,
    Ripple,
    Gaussian,
}

impl SurfaceFunction {
    pub fn evaluate(&self, x: f32, y: f32) -> f32 {
        match self {
            Self::Paraboloid => x * x + y * y,
            Self::Saddle => x * x - y * y,
            Self::Wave => x.sin() * y.cos(),
            Self::Ripple => {
                let r = (x * x + y * y).sqrt();
                if r.abs() < 1e-10 { 1.0 } else { r.sin() / r }
            }
            Self::Gaussian => (-(x * x + y * y)).exp(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Paraboloid => "Параболоид (z = x² + y²)",
            Self::Saddle => "Седло (z = x² - y²)",
            Self::Wave => "Волна (z = sin(x)·cos(y))",
            Self::Ripple => "Пульсация (z = sin(r)/r)",
            Self::Gaussian => "Гаусс (z = e^(-(x²+y²)))",
        }
    }

    /// Генерация сетки поверхности из функции f(x, y) = z
    pub fn generate_surface_mesh(
        &self,
        x_range: (f32, f32),
        y_range: (f32, f32),
        divisions: (usize, usize),
    ) -> Mesh {
        let (x0, x1) = x_range;
        let (y0, y1) = y_range;
        let (nx, ny) = divisions;

        let dx = (x1 - x0) / nx as f32;
        let dy = (y1 - y0) / ny as f32;

        let mut vertices = Vec::new();

        // Генерация вершин
        for j in 0..=ny {
            for i in 0..=nx {
                let x = x0 + i as f32 * dx;
                let y = y0 + j as f32 * dy;
                let z = self.evaluate(x, y);

                if z.is_finite() {
                    // HVec3::new() создаёт однородный вектор (x, y, z, 1.0)
                    vertices.push(Point3::new(x, y, z));
                } else {
                    vertices.push(Point3::new(x, y, 0.0));
                }
            }
        }

        // Генерация полигонов (треугольников)
        let mut polygons = Vec::new();
        for j in 0..ny {
            for i in 0..nx {
                let idx = |i: usize, j: usize| -> usize { j * (nx + 1) + i };

                // Первый треугольник
                polygons.push(Polygon::triangle(
                    idx(i, j),
                    idx(i + 1, j),
                    idx(i + 1, j + 1),
                ));

                // Второй треугольник
                polygons.push(Polygon::triangle(
                    idx(i, j),
                    idx(i + 1, j + 1),
                    idx(i, j + 1),
                ));
            }
        }

        // Используем from_polygons, которая сама сгенерирует нормали и текстуры
        Mesh::from_polygons(vertices, polygons)
    }
}

#[cfg(test)]
mod test_surface_generator {
    use crate::SurfaceFunction;

    #[test]
    fn test_mesh_is_inside_range() {
        let surface = SurfaceFunction::Gaussian;
        let x_range = (-10.0, 10.0);
        let y_range = (-5.0, 5.0);
        let mesh = surface.generate_surface_mesh(x_range, y_range, (10, 10));
        for vertex in mesh.get_local_vertex_iter() {
            assert!(
                x_range.0 <= vertex.x && vertex.x <= x_range.1,
                "все вершины графика должны соблюдать пределы, но {} не в пределах оси x [{}, {}]",
                vertex,
                x_range.0,
                x_range.1
            );

            assert!(
                y_range.0 <= vertex.y && vertex.y <= y_range.1,
                "все вершины графика должны соблюдать пределы, но {} не в пределах оси y [{}, {}]",
                vertex,
                y_range.0,
                y_range.1
            );
        }
    }
}
