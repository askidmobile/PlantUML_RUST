//! # plantuml-wasm
//!
//! WASM биндинги для использования plantuml-rs в браузере.
//!
//! ## Использование в JavaScript
//!
//! Блок помечен `javascript`, поэтому rustdoc его не проверяет. Проверку
//! делает шаг «Verify WASM module» в `.github/workflows/deploy-pages.yml`:
//! он требует, чтобы модуль оставался валидным, а экспорты — на месте.
//!
//! ```javascript
//! import init, { render, render_with_theme, parse_to_json, version } from 'plantuml-wasm';
//!
//! async function main() {
//!     await init();
//!
//!     const source = `
//! @startuml
//! Alice -> Bob: Hello
//! @enduml
//! `;
//!
//!     // render бросает исключение при ошибке разбора — оборачиваем в try.
//!     try {
//!         const svg = render(source);
//!         document.getElementById('diagram').innerHTML = svg;
//!     } catch (error) {
//!         console.error('не удалось построить диаграмму:', error);
//!     }
//! }
//! ```
//!
//! ## Что экспортируется
//!
//! | Экспорт | Назначение |
//! |---|---|
//! | `render(source)` | исходный код → SVG |
//! | `render_with_theme(source, theme)` | то же с указанной темой |
//! | `parse_to_json(source)` | исходный код → JSON с AST |
//! | `version()` | версия библиотеки |
//! | `available_themes()` | список имён тем |
//!
//! Все функции кроме `version` и `available_themes` бросают `JsValue`
//! с текстом ошибки, если разбор не удался.

use plantuml_core::RenderOptions;
use wasm_bindgen::prelude::*;

/// Инициализация panic hook для лучших сообщений об ошибках
#[cfg(feature = "console_error_panic_hook")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Рендерит PlantUML исходный код в SVG
///
/// @param source - PlantUML исходный код
/// @returns SVG строка или ошибка
#[wasm_bindgen]
pub fn render(source: &str) -> Result<String, JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();

    plantuml_core::render(source, &RenderOptions::default())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Рендерит с указанной темой
///
/// @param source - PlantUML исходный код
/// @param theme_name - имя темы (default, dark, minimal, sketchy, cerulean)
/// @returns SVG строка или ошибка
#[wasm_bindgen]
pub fn render_with_theme(source: &str, theme_name: &str) -> Result<String, JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();

    let options = RenderOptions::new().with_theme_name(theme_name);

    plantuml_core::render(source, &options).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Парсит PlantUML и возвращает JSON представление AST
///
/// @param source - PlantUML исходный код
/// @returns JSON строка с AST
#[wasm_bindgen]
pub fn parse_to_json(source: &str) -> Result<String, JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();

    let diagram =
        plantuml_core::parse_diagram(source).map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_json::to_string(&diagram)
        .map_err(|e: serde_json::Error| JsValue::from_str(&e.to_string()))
}

/// Возвращает версию библиотеки
#[wasm_bindgen]
pub fn version() -> String {
    plantuml_core::version().to_string()
}

/// Возвращает список доступных тем
#[wasm_bindgen]
pub fn available_themes() -> Vec<JsValue> {
    plantuml_core::available_themes()
        .into_iter()
        .map(JsValue::from_str)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
