//! Конфигурация layout для Salt диаграмм

use crate::text::TextMeasurer;

/// Конфигурация для Salt layout engine
#[derive(Debug, Clone)]
pub struct SaltLayoutConfig {
    /// Отступ от краёв диаграммы
    pub padding: f64,
    /// Высота строки
    pub row_height: f64,
    /// Минимальная ширина ячейки
    pub min_cell_width: f64,
    /// Отступ между ячейками
    pub cell_padding: f64,
    /// Размер шрифта
    pub font_size: f64,
    /// Высота кнопки
    pub button_height: f64,
    /// Высота текстового поля
    pub textfield_height: f64,
    /// Ширина чекбокса/радио
    pub checkbox_size: f64,
    /// Толщина границы
    pub border_width: f64,
    /// Цвет фона
    pub background_color: &'static str,
    /// Цвет границы
    pub border_color: &'static str,
    /// Цвет кнопки
    pub button_color: &'static str,
    /// Цвет текстового поля
    pub textfield_color: &'static str,
    /// Измеритель текста (считает символы, а не байты)
    pub text: TextMeasurer,
    /// Доступная ширина для раскладки контейнеров.
    ///
    /// Раньше была жёстко зашита в `layout()` (800.0), из-за чего настройка
    /// через `with_config` не влияла на ширину.
    pub available_width: f64,
}

impl Default for SaltLayoutConfig {
    fn default() -> Self {
        Self {
            padding: 10.0,
            row_height: 28.0,
            min_cell_width: 60.0,
            cell_padding: 8.0,
            font_size: 13.0,
            button_height: 24.0,
            textfield_height: 22.0,
            checkbox_size: 14.0,
            border_width: 1.0,
            background_color: "#FFFFFF",
            border_color: "#888888",
            button_color: "#E0E0E0",
            textfield_color: "#FFFFFF",
            text: TextMeasurer::default(),
            available_width: 800.0,
        }
    }
}
