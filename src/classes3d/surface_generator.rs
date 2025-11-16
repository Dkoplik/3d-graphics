use crate::classes3d::mesh::Polygon3;
use crate::{HVec3, Mesh, SurfaceFunction};

impl SurfaceFunction {
    pub fn evaluate(&self, x: f64, y: f64) -> f64 {
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
}

/// Генерация сетки поверхности из функции f(x, y) = z
pub fn generate_surface_mesh(
    func: SurfaceFunction,
    x_range: (f64, f64),
    y_range: (f64, f64),
    divisions: (usize, usize),
) -> Mesh {
    let (x0, x1) = x_range;
    let (y0, y1) = y_range;
    let (nx, ny) = divisions;

    let dx = (x1 - x0) / nx as f64;
    let dy = (y1 - y0) / ny as f64;

    let mut vertices = Vec::new();

    // Генерация вершин
    for j in 0..=ny {
        for i in 0..=nx {
            let x = x0 + i as f64 * dx;
            let y = y0 + j as f64 * dy;
            let z = func.evaluate(x, y);

            if z.is_finite() {
                // HVec3::new() создаёт однородный вектор (x, y, z, 1.0)
                vertices.push(HVec3::new(x as f32, y as f32, z as f32));
            } else {
                vertices.push(HVec3::new(x as f32, y as f32, 0.0));
            }
        }
    }

    // Генерация полигонов (треугольников)
    let mut polygons = Vec::new();
    for j in 0..ny {
        for i in 0..nx {
            let idx = |i: usize, j: usize| -> usize { j * (nx + 1) + i };

            // Первый треугольник
            polygons.push(Polygon3::triangle(
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
            ));

            // Второй треугольник
            polygons.push(Polygon3::triangle(
                idx(i, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ));
        }
    }

    // Используем from_polygons, которая сама сгенерирует нормали и текстуры
    Mesh::from_polygons(vertices, polygons)
}
