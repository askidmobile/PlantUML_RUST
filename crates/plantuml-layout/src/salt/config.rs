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
            // Измерено по эталону PlantUML
            // (tests/golden/reference/salt_basic.svg, 113x71): строки идут
            // с шагом 17.968 (y = 17.139, 35.107), кнопки высотой 17.969
            // со скруглением 5 и обводкой 2.5, размер шрифта 12.
            padding: 6.0,
            row_height: 17.968,
            min_cell_width: 52.07,
            cell_padding: 6.0,
            font_size: 12.0,
            button_height: 17.969,
            textfield_height: 17.969,
            checkbox_size: 11.0,
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
