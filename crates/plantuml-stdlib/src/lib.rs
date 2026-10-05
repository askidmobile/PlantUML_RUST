//! # plantuml-stdlib
//!
//! Стандартная библиотека PlantUML для Rust: иконки, спрайты, макросы.
//!
//! Эта библиотека предоставляет встроенную поддержку стандартных включений PlantUML,
//! таких как `<C4/C4_Context>`, `<aws/...>`, `<azure/...>` и других.
//!
//! ## Поддерживаемые библиотеки
//!
//! - **C4** — C4 Model (Context, Container, Component, Code)
//! - **tupadr3** — Базовые спрайты и иконки
//! - **office** — Microsoft Office иконки
//! - **logos** — Логотипы популярных технологий
//!
//! ## Использование
//!
//! ```rust,ignore
//! use plantuml_stdlib::{get_include, exists};
//!
//! // Проверить существование включения
//! if exists("C4/C4_Context") {
//!     // Получить содержимое
//!     let content = get_include("C4/C4_Context").unwrap();
//!     println!("{}", content);
//! }
//! ```
//!
//! ## Структура путей
//!
//! Пути следуют формату PlantUML stdlib:
//! - `<C4/C4_Context>` → C4 Model контекстная диаграмма
//! - `<tupadr3/common>` → Общие определения tupadr3
//! - `<logos/rust>` → Логотип Rust

mod c4;
mod common;
pub mod inflate;
mod kubernetes;
mod logos;
mod office;
mod tupadr3;

use std::collections::HashMap;
use std::sync::LazyLock;

/// Реестр всех включений стандартной библиотеки
static STDLIB_REGISTRY: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut registry = HashMap::new();

    // Добавляем все включения из модулей
    c4::register(&mut registry);
    kubernetes::register(&mut registry);
    tupadr3::register(&mut registry);
    logos::register(&mut registry);
    office::register(&mut registry);
    common::register(&mut registry);

    registry
});

/// Получает содержимое включения по пути
///
/// # Аргументы
///
/// * `path` - путь к включению (например, "C4/C4_Context")
///
/// # Возвращает
///
/// * `Some(&str)` - содержимое файла включения
/// * `None` - если включение не найдено
///
/// # Пример
///
/// ```rust
/// use plantuml_stdlib::get_include;
///
/// if let Some(content) = get_include("C4/C4_Context") {
///     println!("C4 Context macros loaded!");
/// }
/// ```
pub fn get_include(path: &str) -> Option<&'static str> {
    // Нормализуем путь: убираем начальный слеш если есть
    let normalized = path.trim_start_matches('/');

    // Пробуем точное совпадение
    if let Some(content) = STDLIB_REGISTRY.get(normalized) {
        return Some(content);
    }

    // Пробуем с .puml расширением
    let with_puml = format!("{}.puml", normalized);
    if let Some(content) = STDLIB_REGISTRY.get(with_puml.as_str()) {
        return Some(content);
    }

    // Пробуем с .iuml расширением
    let with_iuml = format!("{}.iuml", normalized);
    if let Some(content) = STDLIB_REGISTRY.get(with_iuml.as_str()) {
        return Some(content);
    }

    None
}

/// Проверяет существование включения в стандартной библиотеке
///
/// # Аргументы
///
/// * `path` - путь к включению
///
/// # Возвращает
///
/// `true` если включение существует, `false` иначе
///
/// # Пример
///
/// ```rust
/// use plantuml_stdlib::exists;
///
/// assert!(exists("C4/C4_Context"));
/// assert!(!exists("nonexistent/path"));
/// ```
pub fn exists(path: &str) -> bool {
    get_include(path).is_some()
}

/// Получает спрайт по имени (устаревший API)
///
/// Используйте `get_include` вместо этой функции.
#[deprecated(
    since = "0.2.0",
    note = "используйте get_include() вместо этой функции"
)]
pub fn get_sprite(name: &str) -> Option<&'static str> {
    // Ищем в разделах спрайтов
    let sprite_path = format!("sprites/{}", name);
    get_include(&sprite_path)
}

/// Получает макрос по имени (устаревший API)
///
/// Используйте `get_include` вместо этой функции.
#[deprecated(
    since = "0.2.0",
    note = "используйте get_include() вместо этой функции"
)]
pub fn get_macro(name: &str) -> Option<&'static str> {
    get_include(name)
}

/// Возвращает список всех доступных путей в стандартной библиотеке
///
/// # Пример
///
/// ```rust
/// use plantuml_stdlib::list_all;
///
/// for path in list_all() {
///     println!("Available: {}", path);
/// }
/// ```
pub fn list_all() -> Vec<&'static str> {
    let mut paths: Vec<_> = STDLIB_REGISTRY.keys().copied().collect();
    paths.sort();
    paths
}

/// Возвращает список путей, начинающихся с указанного префикса
///
/// # Аргументы
///
/// * `prefix` - префикс пути (например, "C4" или "aws")
///
/// # Пример
///
/// ```rust
/// use plantuml_stdlib::list_by_prefix;
///
/// for path in list_by_prefix("C4") {
///     println!("C4 include: {}", path);
/// }
/// ```
pub fn list_by_prefix(prefix: &str) -> Vec<&'static str> {
    let normalized = prefix.trim_start_matches('/');
    let mut paths: Vec<_> = STDLIB_REGISTRY
        .keys()
        .copied()
        .filter(|p| p.starts_with(normalized))
        .collect();
    paths.sort();
    paths
}

/// Информация о стандартной библиотеке
pub struct StdlibInfo {
    /// Версия библиотеки
    pub version: &'static str,
    /// Количество включений
    pub include_count: usize,
    /// Поддерживаемые категории
    pub categories: Vec<&'static str>,
}

/// Возвращает информацию о стандартной библиотеке
pub fn info() -> StdlibInfo {
    StdlibInfo {
        version: env!("CARGO_PKG_VERSION"),
        include_count: STDLIB_REGISTRY.len(),
        categories: vec!["C4", "tupadr3", "logos", "office", "common"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c4_context_exists() {
        assert!(exists("C4/C4_Context"));
    }

    #[test]
    fn test_get_c4_include() {
        let content = get_include("C4/C4_Context");
        assert!(content.is_some());
        let text = content.unwrap();
        assert!(text.contains("!define"));
    }

    #[test]
    fn test_nonexistent() {
        assert!(!exists("nonexistent/path/to/file"));
        assert!(get_include("nonexistent").is_none());
    }

    #[test]
    fn test_list_all() {
        let all = list_all();
        assert!(!all.is_empty());
        assert!(all.iter().any(|p| p.starts_with("C4")));
    }

    #[test]
    fn test_list_by_prefix() {
        let c4_includes = list_by_prefix("C4");
        assert!(!c4_includes.is_empty());
        for path in c4_includes {
            assert!(path.starts_with("C4"));
        }
    }

    #[test]
    fn test_info() {
        let info = info();
        assert!(!info.version.is_empty());
        assert!(info.include_count > 0);
        assert!(!info.categories.is_empty());
    }
}
