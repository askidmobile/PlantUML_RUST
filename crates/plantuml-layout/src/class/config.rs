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
            node_horizontal_spacing: 50.0,
            layer_vertical_spacing: 80.0,
            // PlantUML задаёт ширину класса по содержимому: в эталонах
            // встречаются 68–80px для коротких имён. Константа 120
            // перебивала измерение и делала все классы одинаково широкими.
            min_class_width: 40.0,
            min_class_height: 60.0,
            class_header_height: 30.0,
            line_height: 20.0,
            class_padding: 10.0,
            margin: 20.0,
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
