//! Конфигурация для Class Layout Engine

use crate::text::TextMeasurer;

/// Конфигурация layout'а class diagrams
#[derive(Debug, Clone)]
pub struct ClassLayoutConfig {
    /// Горизонтальный отступ между узлами на одном слое
    pub node_horizontal_spacing: f64,
    /// Вертикальный отступ между слоями
    pub layer_vertical_spacing: f64,
    /// Минимальная ширина класса
    pub min_class_width: f64,
    /// Минимальная высота класса
    pub min_class_height: f64,
    /// Высота заголовка класса
    pub class_header_height: f64,
    /// Высота строки (для полей/методов)
    pub line_height: f64,
    /// Padding внутри класса
    pub class_padding: f64,
    /// Отступ от границ диаграммы
    pub margin: f64,
    /// Размер шрифта
    pub font_size: f64,
    /// Измеритель текста (считает символы, а не байты)
    pub text: TextMeasurer,
}

impl Default for ClassLayoutConfig {
    fn default() -> Self {
        Self {
            // Измерено по эталону class_inheritance: «Dog» на x=7 шириной
            // 68.3, «Cat» на x=110.5 — зазор 35.2; «Dog» на y=7, «Animal»
            // на y=131.29 — шаг 124.29 при высоте бокса 64.297, то есть
            // вертикальный отступ 59.99.
            node_horizontal_spacing: 35.2,
            layer_vertical_spacing: 60.0,
            // PlantUML задаёт ширину класса по содержимому: в эталонах
            // встречаются 68–80px для коротких имён. Константа 120
            // перебивала измерение и делала все классы одинаково широкими.
            min_class_width: 40.0,
            // Высота класса измерена по эталону class_inheritance: 64.297
            // при заголовке и одной строке содержимого.
            min_class_height: 64.297,
            // Измерено по эталону class_inheritance: бокс «Dog» занимает
            // y=7..71.297 (высота 64.297), разделители на 39 и 47.
            // Отсюда заголовок 32, строка члена 16.297, отступ секции 8:
            // 32 + (16.297 + 8) + 8 = 64.297.
            class_header_height: 32.0,
            line_height: 16.297,
            // Отступ секции члена. Измерено по эталону: 32 (заголовок)
            // + (16.297 + 8) + 8 = 64.297 — высота бокса «Dog».
            class_padding: 8.0,
            margin: 7.0,
            font_size: 14.0,
            text: TextMeasurer::default(),
        }
    }
}

impl ClassLayoutConfig {
    /// Создаёт новую конфигурацию
    pub fn new() -> Self {
        Self::default()
    }

    /// Устанавливает расстояние между узлами
    pub fn with_node_spacing(mut self, horizontal: f64, vertical: f64) -> Self {
        self.node_horizontal_spacing = horizontal;
        self.layer_vertical_spacing = vertical;
        self
    }
}
