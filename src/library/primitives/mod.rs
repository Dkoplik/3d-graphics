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
pub use hvec3::*;
pub use line3::*;
pub use plane::*;
pub use point3::*;
pub use transform3::*;
pub use uvec3::*;
pub use vec3::*;
