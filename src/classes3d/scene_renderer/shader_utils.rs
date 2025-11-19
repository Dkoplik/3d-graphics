use crate::{Canvas, Point3, Transform3D, Vec3};

/// Найти противополжный цвет.
pub fn opposite_color(color: egui::Color32) -> egui::Color32 {
    egui::Color32::from_rgb(255 - color.r(), 255 - color.g(), 255 - color.b())
}

pub fn is_inside_polygon(vertexes: &Vec<Vec3>, indexes: &Vec<usize>, pos: Vec3) -> bool {
    let mut sign = None;
    for i in 0..indexes.len() {
        let p0 = vertexes[i];
        let p1 = vertexes[(i + 1) % indexes.len()];
        let vec = p1 - p0;
        let normal = Vec3::new(-vec.y, vec.x, 0.0);
        let mut to_pos = pos - p0;
        to_pos.z = 0.0;

        if let Some(sign) = sign {
            if sign != (normal.dot(to_pos) > 0.0) {
                return false;
            }
        } else {
            sign = Some(normal.dot(to_pos) > 0.0)
        }
    }
    true
}

/// Рендерить линию, образованную точками `start` и `end`.
///
/// Сами точки `start` и `end` должны указываться в **глобальных** координатах
pub fn render_line(
    global_to_screen_transform: Transform3D,
    start: Point3,
    end: Point3,
    color: egui::Color32,
    canvas: &mut Canvas,
) {
    let start = Point3::from(global_to_screen_transform.apply_to_hvec(start.into()));
    let end = Point3::from(global_to_screen_transform.apply_to_hvec(end.into()));

    let start_pos = egui::Pos2::new(start.x, start.y);
    let end_pos = egui::Pos2::new(end.x, end.y);
    canvas.draw_sharp_line(start_pos, end_pos, color);
}

/// Находит барицентрические координаты по 3-м точкам.
/// `triangle` - полигон-треугольник, по которому строятся координаты
/// `point` - точка, для которой нужны координаты
///
/// Поскольку это уже в проекции на экран, z-координата не учитывается.
///
/// Возвращает координаты в виде Point3.
pub fn barycentric_coordinates(triangle: &[Vec3], point: Point3) -> Point3 {
    let mut v0 = triangle[1] - triangle[0];
    let mut v1 = triangle[2] - triangle[0];
    let mut v2 = Vec3::from(point) - triangle[0];

    // z-координата предозначеня для буфера, точки уже в проекции
    v0.z = 0.0;
    v1.z = 0.0;
    v2.z = 0.0;

    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);

    let denom = d00 * d11 - d01 * d01;
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    Point3::new(u, v, w)
}

/// Находит uv-координаты для билинейной интерполяции.
///
/// Все точки являются проекциями на экран, z-компонента не учитывается.
pub fn find_uv_for_bilerp(
    p0: Point3,
    p1: Point3,
    _p2: Point3,
    p3: Point3,
    cur: Point3,
) -> Option<(f32, f32)> {
    let p0p1 = p1 - p0;
    let p0p3 = p3 - p0;
    let det = p0p3.x * p0p1.y - p0p3.y * p0p1.x;
    if det.abs() <= f32::EPSILON {
        return None;
    }
    let det_u = (cur.x - p0.x) * p0p1.y - (cur.y - p0.y) * p0p1.x;
    let det_v = p0p3.x * (cur.y - p0.y) - p0p3.y * (cur.x - p0.x);
    Some((det_u / det, det_v / det))
}

/// Интерполяция вещественного числа через барицентрические координаты.
pub fn interpolate_float(bary: Point3, a: f32, b: f32, c: f32) -> f32 {
    let alpha = bary.x;
    let beta = bary.y;
    let gamma = bary.z;
    alpha * a + beta * b + gamma * c
}

/// Билинейная интерполяция вещественного числа.
pub fn bilerp_float(
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
    alpha: f32,
    beta: f32,
) -> f32 {
    let top = lerp_float(top_left, top_right, alpha);
    let bottom = lerp_float(bottom_left, bottom_right, alpha);
    lerp_float(top, bottom, beta)
}

/// Линейная интерполяция вещественного числа.
pub fn lerp_float(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Интерполяция цвета через барицентрические координаты.
pub fn interpolate_color(
    bary: Point3,
    a: egui::Color32,
    b: egui::Color32,
    c: egui::Color32,
) -> egui::Color32 {
    let alpha = bary.x;
    let beta = bary.y;
    let gamma = bary.z;
    a.gamma_multiply(alpha) + b.gamma_multiply(beta) + c.gamma_multiply(gamma)
}

/// Билинейная интерполяция цвета.
pub fn bilerp_color(
    top_left: egui::Color32,
    top_right: egui::Color32,
    bottom_left: egui::Color32,
    bottom_right: egui::Color32,
    alpha: f32,
    beta: f32,
) -> egui::Color32 {
    let top = lerp_color(top_left, top_right, alpha);
    let bottom = lerp_color(bottom_left, bottom_right, alpha);
    lerp_color(top, bottom, beta)
}

/// Линейная интерполяция цвета.
pub fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    egui::Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}

/// Интерполяция вектора через барицентрические координаты.
pub fn interpolate_vec(bary: Point3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let alpha = bary.x;
    let beta = bary.y;
    let gamma = bary.z;
    a * alpha + b * beta + c * gamma
}

/// Билинейная интерполяция вектора.
pub fn bilerp_vec(
    top_left: Vec3,
    top_right: Vec3,
    bottom_left: Vec3,
    bottom_right: Vec3,
    alpha: f32,
    beta: f32,
) -> Vec3 {
    let top = lerp_vec(top_left, top_right, alpha);
    let bottom = lerp_vec(bottom_left, bottom_right, alpha);
    lerp_vec(top, bottom, beta)
}

/// Линейная интерполяция вектора.
fn lerp_vec(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}

/// Триангуляция полигона.
/// `polygon` - полигон, заданный индексами вершин.
///
/// Пока что примитивная веерная триангуляция.
pub fn triangulate_polygon(polygon: &[usize]) -> Vec<[usize; 3]> {
    #[cfg(debug_assertions)]
    {
        if polygon.len() < 3 {
            eprintln!(
                "Warning: триангуляция полигона с {} вершинами",
                polygon.len()
            );
        }
    }

    let mut triangles = vec![];
    for i in 1..polygon.len() - 1 {
        triangles.push([polygon[0], polygon[i], polygon[i + 1]]);
    }
    triangles
}

#[cfg(test)]
mod render_tests {
    use crate::HVec3;

    use super::*;

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
    fn test_find_uv_square_center() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(0.0, 10.0, 0.0);
        let p2 = Point3::new(10.0, 10.0, 0.0);
        let p3 = Point3::new(10.0, 0.0, 0.0);
        // центер кавадрата
        let cur = Point3::new(5.0, 5.0, 0.0);

        let uv = find_uv_for_bilerp(p0, p1, p2, p3, cur);
        assert!(uv.is_some()); // коодринаты должны быть

        let (u, v) = uv.unwrap();
        assert!((u - 0.5).abs() < TOLERANCE);
        assert!((v - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn test_find_uv_square_left_lower() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(0.0, 10.0, 0.0);
        let p2 = Point3::new(10.0, 10.0, 0.0);
        let p3 = Point3::new(10.0, 0.0, 0.0);
        // левый нижний угол
        let cur = Point3::new(0.0, 0.0, 0.0);

        let uv = find_uv_for_bilerp(p0, p1, p2, p3, cur);
        assert!(uv.is_some()); // коодринаты должны быть

        let (u, v) = uv.unwrap();
        assert!((u - 0.0).abs() < TOLERANCE);
        assert!((v - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_find_uv_square_right_upper() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(0.0, 10.0, 0.0);
        let p2 = Point3::new(10.0, 10.0, 0.0);
        let p3 = Point3::new(10.0, 0.0, 0.0);
        // првый верхний угол
        let cur = Point3::new(10.0, 10.0, 0.0);

        let uv = find_uv_for_bilerp(p0, p1, p2, p3, cur);
        assert!(uv.is_some()); // коодринаты должны быть

        let (u, v) = uv.unwrap();
        assert!((u - 1.0).abs() < TOLERANCE);
        assert!((v - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_find_uv_beyong_borders_1() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(0.0, 10.0, 0.0);
        let p2 = Point3::new(10.0, 10.0, 0.0);
        let p3 = Point3::new(10.0, 0.0, 0.0);
        // за пределами квадрата
        let cur = Point3::new(15.0, 15.0, 0.0);

        let uv = find_uv_for_bilerp(p0, p1, p2, p3, cur);
        assert!(uv.is_some()); // коодринаты должны быть

        let (u, v) = uv.unwrap();
        assert!(u > 1.0 + TOLERANCE);
        assert!(v > 1.0 + TOLERANCE);
    }

    #[test]
    fn test_find_uv_beyong_borders_2() {
        // трапеция
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(0.0, 10.0, 0.0);
        let p2 = Point3::new(10.0, 8.0, 0.0);
        let p3 = Point3::new(10.0, 2.0, 0.0);
        // за пределами трапеции
        let cur = Point3::new(10.0, 10.0, 0.0);

        let uv = find_uv_for_bilerp(p0, p1, p2, p3, cur);
        assert!(uv.is_some()); // коодринаты должны быть

        let (u, v) = uv.unwrap();
        assert!(u > 1.0 + TOLERANCE);
        assert!(v > 1.0 + TOLERANCE);
    }

    #[test]
    fn test_find_uv_beyong_borders_3() {
        // трапеция
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(0.0, 10.0, 0.0);
        let p2 = Point3::new(10.0, 8.0, 0.0);
        let p3 = Point3::new(10.0, 2.0, 0.0);
        // за пределами трапеции
        let cur = Point3::new(10.0, 0.0, 0.0);

        let uv = find_uv_for_bilerp(p0, p1, p2, p3, cur);
        assert!(uv.is_some()); // коодринаты должны быть

        let (u, v) = uv.unwrap();
        assert!(u > 1.0 + TOLERANCE);
        assert!(v > 1.0 + TOLERANCE);
    }
}
