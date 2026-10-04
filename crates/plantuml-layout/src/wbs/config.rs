//! Конфигурация layout для WBS диаграмм

use crate::text::TextMeasurer;

/// Конфигурация для WBS layout engine
#[derive(Debug, Clone)]
pub struct WbsLayoutConfig {
    /// Отступ от краёв диаграммы
    pub padding: f64,
    /// Вертикальный отступ между уровнями
    pub level_spacing: f64,
    /// Горизонтальный отступ между узлами одного уровня
    pub sibling_spacing: f64,
    /// Минимальная ширина узла
    pub min_node_width: f64,
    /// Высота узла
    pub node_height: f64,
    /// Горизонтальный padding внутри узла
    pub node_padding_x: f64,
    /// Размер шрифта
    pub font_size: f64,
    /// Измеритель текста (считает символы, а не байты)
    pub text: TextMeasurer,
}

impl Default for WbsLayoutConfig {
    fn default() -> Self {
        Self {
            padding: 20.0,
            level_spacing: 60.0,
            sibling_spacing: 20.0,
            min_node_width: 80.0,
            node_height: 30.0,
            node_padding_x: 12.0,
            font_size: 13.0,
            text: TextMeasurer::default(),
        }
    }
}
