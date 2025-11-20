//! Модуль с примитивами для 3D графики по типу точек, векторов и подобных объектов.

// объявление модулей-примитивов
mod hvec3;
mod line3;
mod plane;
mod point3;
mod transform3;
mod uvec3;
mod vec3;

// re-export модулей в этот модуль
pub use hvec3::HVec3;
pub use line3::Line3;
pub use plane::Plane;
pub use point3::Point3;
pub use transform3::Transform3D;
pub use uvec3::UVec3;
pub use vec3::Vec3;
