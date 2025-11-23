//! Этот модуль нужен чисто для того, чтобы основную логику библиотеки `g3d` вынести в отдельную папку проекта
//!
//! В какой-то степени это костыль, но теперь вся логика библиотеки находится в `./library`

// примитивы графики
pub mod primitives;

// модель
pub mod model;

// прочие структуры
pub mod camera;
pub mod canvas;
pub mod coord_frame;
pub mod light_source;
pub mod scene;
pub mod scene_renderer;

// вспомогательные методы
pub mod utils;
