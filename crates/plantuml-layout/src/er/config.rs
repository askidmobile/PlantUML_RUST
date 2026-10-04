//! Конфигурация layout для ER диаграмм

use crate::text::TextMeasurer;

/// Конфигурация для ER layout engine
#[derive(Debug, Clone)]
pub struct ErLayoutConfig {
    /// Отступ от краёв диаграммы
    pub padding: f64,
    /// Минимальная ширина сущности
    pub min_entity_width: f64,
    /// Высота заголовка сущности
    pub entity_header_height: f64,
    /// Высота строки атрибута
    pub attribute_height: f64,
    /// Горизонтальный отступ между сущностями
    pub horizontal_spacing: f64,
    /// Вертикальный отступ между сущностями
    pub vertical_spacing: f64,
    /// Padding внутри сущности
    pub entity_padding: f64,
    /// Размер шрифта
    pub font_size: f64,
    /// Измеритель текста (считает символы, а не байты)
    pub text: TextMeasurer,
    /// Цвет фона сущности
    pub entity_bg_color: &'static str,
    /// Цвет заголовка
    pub header_bg_color: &'static str,
}

impl Default for ErLayoutConfig {
    fn default() -> Self {
        Self {
            padding: 7.0,
            // Ширина сущности определяется содержимым. Фиксированный
            // минимум 150 делал диаграмму вдвое шире эталона (180 против 85):
            // в эталоне «User» занимает 63.9, «Order» — 32. Минимум оставлен
            // небольшим, только чтобы фигура не выродилась.
            min_entity_width: 32.0,
            entity_header_height: 30.0,
            attribute_height: 22.0,
            horizontal_spacing: 80.0,
            vertical_spacing: 60.0,
            entity_padding: 10.0,
            font_size: 13.0,
            text: TextMeasurer::default(),
            entity_bg_color: "#FEFECE",
            header_bg_color: "#E2E2F0",
        }
    }
}
