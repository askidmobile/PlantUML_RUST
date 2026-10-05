//! Конфигурация layout для JSON диаграмм

use crate::text::TextMeasurer;
/// Конфигурация для JSON layout engine
#[derive(Debug, Clone)]
pub struct JsonLayoutConfig {
    /// Отступ от краёв диаграммы
    pub padding: f64,
    /// Отступ для вложенных элементов
    pub indent: f64,
    /// Высота строки
    pub line_height: f64,
    /// Минимальная ширина ключа
    pub min_key_width: f64,
    /// Размер шрифта
    pub font_size: f64,
    /// Измеритель текста (считает символы, а не байты)
    pub text: TextMeasurer,
    /// Радиус скругления для объектов/массивов
    pub corner_radius: f64,
    /// Цвет фона объекта
    pub object_bg_color: &'static str,
    /// Цвет фона массива
    pub array_bg_color: &'static str,
    /// Цвет ключа
    pub key_color: &'static str,
    /// Цвет строки
    pub string_color: &'static str,
    /// Цвет числа
    pub number_color: &'static str,
    /// Рисовать boolean обычным текстом, а не флажком.
    ///
    /// PlantUML ставит «☑ true» только в JSON; в YAML то же значение
    /// выводится как `true`. Проверено на сервере для обеих диаграмм.
    pub bool_as_text: bool,
    /// Цвет boolean/null
    pub keyword_color: &'static str,
}

impl Default for JsonLayoutConfig {
    fn default() -> Self {
        Self {
            // Измерено по эталону: таблица начинается на x=10, y=10.
            padding: 10.0,
            indent: 20.0,
            // Измерено по эталону PlantUML
            // (tests/golden/reference/json_basic.svg): строки идут с шагом
            // 20.297 (y = 24.995, 45.292, 65.589, 85.886), высота диаграммы
            // 103 при четырёх строках. Раньше стояло 24.0.
            line_height: 20.297,
            min_key_width: 60.0,
            // В эталоне весь текст таблицы набран размером 14.
            font_size: 14.0,
            text: TextMeasurer::default(),
            corner_radius: 3.0,
            // Эталон `json_basic`: заливка таблицы `#F1F1F1` — та же,
            // что у блоков activity и классов, а не «жёлтая» тема.
            object_bg_color: "#F1F1F1",
            array_bg_color: "#E8F4E8",
            key_color: "#000080",
            string_color: "#008000",
            number_color: "#0000FF",
            keyword_color: "#800080",
            bool_as_text: false,
        }
    }
}
