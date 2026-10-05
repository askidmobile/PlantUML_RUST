//! Конфигурация layout для MindMap диаграмм

use crate::text::TextMeasurer;

/// Конфигурация для MindMap layout engine
#[derive(Debug, Clone)]
pub struct MindMapLayoutConfig {
    /// Измеритель текста.
    pub text: TextMeasurer,
    /// Отступ от краёв диаграммы
    pub padding: f64,
    /// Горизонтальный отступ между уровнями
    pub level_spacing: f64,
    /// Вертикальный отступ между узлами одного уровня
    pub sibling_spacing: f64,
    /// Минимальная ширина узла
    pub min_node_width: f64,
    /// Высота узла
    pub node_height: f64,
    /// Горизонтальный padding внутри узла
    pub node_padding_x: f64,
    /// Вертикальный padding внутри узла
    pub node_padding_y: f64,
    /// Размер шрифта
    pub font_size: f64,
    /// Радиус скругления узлов
    pub corner_radius: f64,
}

impl Default for MindMapLayoutConfig {
    fn default() -> Self {
        Self {
            // Значения измерены по эталону PlantUML
            // (tests/golden/reference/mindmap_basic.svg, латинские подписи):
            // высота узла 36.297, шаг между уровнями ~130, отступ от края 10.
            // Раньше стояли 80/20/30, из-за чего диаграмма была заметно шире.
            // Измерено по эталону mindmap_basic (387x246): узел — это текст
            // плюс 20 («Project» 48.453 → 68.5; «Implementation» 111.166 →
            // 131.2), высота 36.297; зазор между уровнями 50, между
            // соседями 20.
            padding: 10.0,
            text: TextMeasurer::default(),
            level_spacing: 50.0,
            sibling_spacing: 20.0,
            min_node_width: 20.0,
            node_height: 36.297,
            node_padding_x: 10.0,
            node_padding_y: 6.0,
            font_size: 14.0,
            corner_radius: 5.0,
        }
    }
}
