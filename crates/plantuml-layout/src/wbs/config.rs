//! Конфигурация layout для WBS диаграмм

use crate::text::TextMeasurer;

/// Конфигурация для WBS layout engine
#[derive(Debug, Clone)]
pub struct WbsLayoutConfig {
    /// Отступ от краёв диаграммы
    pub padding: f64,
    /// Вертикальный отступ между уровнями
    pub level_spacing: f64,
    /// Отступ между РЯДАМИ внутри одного уровня.
    ///
    /// Измерено по эталону: шаги рядов 73.97 (первый) и 48.97 (далее)
    /// при высоте узла 33.969, то есть отступы 40.0 и 15.0.
    pub row_spacing: f64,
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
            // Измерено по эталону wbs_basic (289x246): узел — это текст
            // плюс 20 («Phase 1» 47.279 → 67.3, «Project» 41.531 → 61.5),
            // высота 34; шаг соседей по вертикали 48.968; корень стоит
            // на x=90.1 при левом крае 20.
            padding: 10.0,
            // Вертикальный зазор между уровнями. Измерено по эталону:
            // корень на y=20, его дети на y=94 при высоте узла 34, то есть
            // зазор 40.
            level_spacing: 40.0,
            row_spacing: 15.0,
            // Зазор между соседними поддеревьями. Измерено по эталону:
            // поддерево «Phase 1» занимает 20..134.423, «Phase 2»
            // начинается на 154.423, то есть зазор 20.
            sibling_spacing: 20.0,
            min_node_width: 20.0,
            node_height: 34.0,
            node_padding_x: 10.0,
            font_size: 12.0,
            text: TextMeasurer::default(),
        }
    }
}
