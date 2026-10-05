//! # plantuml-core
//!
//! Pure Rust библиотека для рендеринга PlantUML диаграмм.
//!
//! Это главный фасад библиотеки, предоставляющий простой API для:
//! - Парсинга PlantUML исходного кода
//! - Рендеринга диаграмм в SVG
//! - Настройки тем и стилей
//!
//! ## Быстрый старт
//!
//! ```rust
//! use plantuml_core::{render, RenderOptions};
//!
//! let source = r#"
//! @startuml
//! Alice -> Bob: Hello
//! Bob --> Alice: Hi
//! @enduml
//! "#;
//!
//! let svg = render(source, &RenderOptions::default()).unwrap();
//! println!("{}", svg);
//! ```
//!
//! ## Архитектура
//!
//! Библиотека состоит из нескольких crates:
//!
//! - `plantuml-ast` — AST типы для всех диаграмм
//! - `plantuml-parser` — лексер и парсер
//! - `plantuml-preprocessor` — препроцессор (!include, !define, etc.)
//! - `plantuml-layout` — алгоритмы размещения элементов
//! - `plantuml-renderer` — SVG/PNG рендеринг
//! - `plantuml-themes` — темы и skinparam

pub mod alloc;
mod error;
mod options;
mod pipeline;

pub use alloc::{memory_limit, memory_used, set_memory_limit, DEFAULT_MEMORY_LIMIT};
pub use error::{Error, Result};
pub use options::RenderOptions;

/// Жёсткий предел памяти приложения: 1 ГиБ.
///
/// Защищает машину от исчерпания памяти, если макрос PlantUML уйдёт в
/// бесконечный рост. Обоснование и способы настройки — в модуле [`alloc`].
#[global_allocator]
static ALLOCATOR: alloc::LimitedAllocator = alloc::LimitedAllocator;

// Re-exports для удобства
pub use plantuml_ast::Diagram;
pub use plantuml_parser::parse;
pub use plantuml_preprocessor::FsFileResolver;
pub use plantuml_themes::Theme;

// PNG рендеринг (требует feature "png")
#[cfg(feature = "png")]
pub use plantuml_renderer::{PngError, PngOptions, PngRenderer};

/// Рендерит PlantUML диаграмму в SVG.
///
/// Это основная функция библиотеки. Она выполняет полный pipeline:
/// 1. Препроцессинг (обработка директив)
/// 2. Парсинг (построение AST)
/// 3. Layout (расчёт позиций элементов)
/// 4. Рендеринг (генерация SVG)
///
/// # Аргументы
///
/// * `source` - исходный код PlantUML
/// * `options` - опции рендеринга
///
/// # Возвращает
///
/// * `Ok(String)` - SVG строка
/// * `Err(Error)` - ошибка на любом этапе
///
/// # Пример
///
/// ```rust
/// use plantuml_core::{render, RenderOptions};
///
/// let source = "@startuml\nAlice -> Bob\n@enduml";
/// let svg = render(source, &RenderOptions::default()).unwrap();
/// assert!(svg.contains("<svg"));
/// ```
pub fn render(source: &str, options: &RenderOptions) -> Result<String> {
    pipeline::render_pipeline(source, options)
}

/// Рендерит PlantUML диаграмму из файла с поддержкой !include.
///
/// Эта функция аналогична `render`, но позволяет использовать
/// директивы `!include` и `!include_once` для включения внешних файлов.
///
/// # Аргументы
///
/// * `source` - исходный код PlantUML
/// * `base_path` - базовая директория для относительных путей !include
/// * `options` - опции рендеринга
///
/// # Пример
///
/// ```rust,ignore
/// use plantuml_core::{render_with_includes, RenderOptions};
/// use std::path::Path;
///
/// let source = r#"
/// @startuml
/// !include "common.puml"
/// Alice -> Bob
/// @enduml
/// "#;
///
/// let svg = render_with_includes(source, Path::new("/path/to/project"), &RenderOptions::default()).unwrap();
/// ```
pub fn render_with_includes(
    source: &str,
    base_path: &std::path::Path,
    options: &RenderOptions,
) -> Result<String> {
    pipeline::render_pipeline_with_includes(source, base_path, options)
}

/// Рендерит PlantUML диаграмму в PNG.
///
/// Требует feature `png`.
///
/// # Аргументы
///
/// * `source` - исходный код PlantUML
/// * `options` - опции рендеринга SVG
/// * `png_options` - опции рендеринга PNG
///
/// # Пример
///
/// ```rust,ignore
/// use plantuml_core::{render_png, RenderOptions, PngOptions};
///
/// let source = "@startuml\nAlice -> Bob\n@enduml";
/// let png_bytes = render_png(source, &RenderOptions::default(), &PngOptions::default())?;
/// std::fs::write("diagram.png", png_bytes)?;
/// ```
#[cfg(feature = "png")]
pub fn render_png(
    source: &str,
    options: &RenderOptions,
    png_options: &PngOptions,
) -> Result<Vec<u8>> {
    // Сначала рендерим в SVG
    let svg = render(source, options)?;

    // Затем конвертируем в PNG
    let png_renderer = PngRenderer::new(png_options.clone());
    png_renderer
        .render_svg(&svg)
        .map_err(|e| Error::Render(e.to_string()))
}

/// Парсит PlantUML и возвращает AST без рендеринга.
///
/// Полезно для анализа структуры диаграммы или для собственного рендеринга.
///
/// # Пример
///
/// ```rust
/// use plantuml_core::parse_diagram;
///
/// let source = "@startuml\nAlice -> Bob: Hello\n@enduml";
/// let diagram = parse_diagram(source).unwrap();
/// ```
pub fn parse_diagram(source: &str) -> Result<Diagram> {
    // Препроцессинг
    let processed = plantuml_preprocessor::preprocess(source)
        .map_err(|e: plantuml_preprocessor::PreprocessError| Error::Preprocess(e.to_string()))?;

    // Парсинг
    let diagram = plantuml_parser::parse(&processed)
        .map_err(|e: plantuml_parser::ParseError| Error::Parse(e.to_string()))?;

    Ok(diagram)
}

/// Возвращает список поддерживаемых тем.
pub fn available_themes() -> Vec<&'static str> {
    vec!["default", "minimal", "dark", "sketchy", "cerulean"]
}

/// Информация о версии библиотеки.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_sequence() {
        let source = "@startuml\nAlice -> Bob: Hello\n@enduml";
        let result = render(source, &RenderOptions::default());
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_parse_diagram() {
        let source = "@startuml\nAlice -> Bob\n@enduml";
        let result = parse_diagram(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_available_themes() {
        let themes = available_themes();
        assert!(themes.contains(&"default"));
        assert!(themes.contains(&"dark"));
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }
}
