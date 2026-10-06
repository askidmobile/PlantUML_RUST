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

/// Логика `render` без привязки к WASM.
///
/// `JsValue` на нативной платформе использовать нельзя, поэтому весь
/// реальный код вынесен сюда и возвращает обычную строку с ошибкой.
/// Иначе весь публичный API крейта остался бы непроверяемым обычными
/// тестами: покрытие файла было 16.7% при 520 тестах в проекте.
fn render_impl(source: &str, options: &RenderOptions) -> Result<String, String> {
    plantuml_core::render(source, options).map_err(|e| e.to_string())
}

/// Логика `parse_to_json` без привязки к WASM.
fn parse_to_json_impl(source: &str) -> Result<String, String> {
    let diagram = plantuml_core::parse_diagram(source).map_err(|e| e.to_string())?;
    serde_json::to_string(&diagram).map_err(|e| e.to_string())
}

/// Рендерит PlantUML исходный код в SVG
///
/// @param source - PlantUML исходный код
/// @returns SVG строка или ошибка
#[wasm_bindgen]
pub fn render(source: &str) -> Result<String, JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();

    render_impl(source, &RenderOptions::default()).map_err(|e| JsValue::from_str(&e))
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

    render_impl(source, &options).map_err(|e| JsValue::from_str(&e))
}

/// Парсит PlantUML и возвращает JSON представление AST
///
/// @param source - PlantUML исходный код
/// @returns JSON строка с AST
#[wasm_bindgen]
pub fn parse_to_json(source: &str) -> Result<String, JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();

    parse_to_json_impl(source).map_err(|e| JsValue::from_str(&e))
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

    const SEQUENCE: &str = "@startuml\nAlice -> Bob: Привет\n@enduml";

    /// Экспорт `render` обязан отдавать готовый SVG.
    #[test]
    fn test_render_impl_produces_svg() {
        let svg = render_impl(SEQUENCE, &RenderOptions::default()).expect("разбор должен пройти");
        assert!(svg.starts_with("<?xml"), "нет заголовка XML");
        assert!(svg.contains("<svg"), "нет корневого элемента");
        assert!(
            svg.contains("Alice") && svg.contains("Bob"),
            "потеряны подписи"
        );
    }

    /// Ошибка разбора приходит строкой, а не паникой.
    ///
    /// В браузере эта строка попадёт в `JsValue` и станет исключением,
    /// поэтому сообщение должно быть непустым и читаемым.
    #[test]
    fn test_render_impl_reports_error_as_string() {
        let error = render_impl("мусор без маркеров", &RenderOptions::default())
            .expect_err("мусор обязан отвергаться");
        assert!(!error.is_empty(), "сообщение об ошибке пустое");
    }

    /// Пустой исходник отвергается, а не рисует пустую картинку.
    #[test]
    fn test_render_impl_rejects_empty_source() {
        assert!(render_impl("", &RenderOptions::default()).is_err());
    }

    /// Смена темы не ломает отрисовку.
    #[test]
    fn test_render_impl_accepts_theme() {
        for theme in plantuml_core::available_themes() {
            let svg = render_impl(SEQUENCE, &RenderOptions::new().with_theme_name(theme))
                .unwrap_or_else(|e| panic!("тема {theme} сломала отрисовку: {e}"));
            assert!(svg.contains("<svg"), "тема {theme} дала пустой SVG");
        }
    }

    /// Несуществующая тема не должна ронять вызов.
    #[test]
    fn test_render_impl_with_unknown_theme_still_renders() {
        let result = render_impl(
            SEQUENCE,
            &RenderOptions::new().with_theme_name("темы-не-существует"),
        );
        assert!(result.is_ok(), "неизвестная тема не должна ломать вызов");
    }

    /// JSON разбора обязан быть валидным и упоминать элементы диаграммы.
    #[test]
    fn test_parse_to_json_impl_returns_valid_json() {
        let json = parse_to_json_impl(SEQUENCE).expect("разбор должен пройти");
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("результат не является JSON");
        // Имена участников обязаны дойти до JSON: этим полем нашёлся бы
        // потребитель, читающий AST из браузера.
        assert!(
            json.contains("Alice") && json.contains("Bob"),
            "в JSON потерялись участники: {}",
            &json[..json.len().min(200)]
        );
        assert!(
            parsed.is_object() || parsed.is_array(),
            "корень JSON должен быть объектом или массивом"
        );
    }

    /// Мусорный исходник даёт ошибку, а не пустой JSON.
    #[test]
    fn test_parse_to_json_impl_rejects_garbage() {
        assert!(parse_to_json_impl("мусор").is_err());
    }
}
