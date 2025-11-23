//! Небольшая библиотека для работы с 3D графикой.
//!
//! Содержит все необходимые классы для представления 3D моделей в пространстве,
//! а так же вспомогательные классы по типу освещения и камеры для отрисовки этих
//! моделей. Модели поддерживают шейдинг и текстурирование.

// Модуль с реализациями заданных структур. Он не pub, так как ниже идёт re-export для более удобного API.
mod library;

// re-export всех примитивов в корень библиотеки
pub use library::primitives::*;

// re-export модели в корень библиотеки
pub use library::model::*;

// re-export прочих структур в корень библиотеки
pub use library::camera::*;
pub use library::canvas::*;
pub use library::coord_frame::*;
pub use library::light_source::*;
pub use library::scene::*;
pub use library::scene_renderer::*;
