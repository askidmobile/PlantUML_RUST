//! Конфигурация layout для Sequence Diagrams

use crate::text::TextMeasurer;

/// Внутренний отступ бокса участника с каждой стороны.
///
/// Измерено по эталону PlantUML: `textLength="33.667"` при `width="47.667"`,
/// то есть (47.667 − 33.667) / 2 = 7.
pub const PARTICIPANT_PADDING: f64 = 7.0;

/// Конфигурация layout sequence diagram
#[derive(Debug, Clone)]
pub struct SequenceLayoutConfig {
    /// Отступ между участниками по горизонтали
    pub participant_spacing: f64,
    /// Отступ между сообщениями по вертикали
    pub message_spacing: f64,
    /// Ширина блока участника
    pub participant_width: f64,
    /// Высота блока участника  
    pub participant_height: f64,
    /// Ширина блока активации
    pub activation_width: f64,
    /// Отступ внутри фрагмента
    pub fragment_padding: f64,
    /// Высота заголовка фрагмента
    pub fragment_header_height: f64,
    /// Высота разделителя (==)
    pub divider_height: f64,
    /// Высота задержки (...)
    pub delay_height: f64,
    /// Высота заметки
    pub note_height: f64,
    /// Ширина заметки
    pub note_width: f64,
    /// Отступ от края диаграммы
    pub margin: f64,
    /// Размер шрифта
    pub font_size: f64,
    /// Измеритель текста (считает символы, а не байты)
    pub text: TextMeasurer,
    /// Высота строки текста
    pub line_height: f64,
    /// Высота заголовка бокса (participant box)
    pub box_title_height: f64,
}

impl Default for SequenceLayoutConfig {
    fn default() -> Self {
        Self {
            // Значения измерены по эталону PlantUML 1.2026.9beta4
            // (tests/golden/reference/sequence_simple.svg).
            // Раньше стояли «примерно» подобранные числа, дававшие
            // расхождение габаритов на 10–15%.
            participant_spacing: 80.0, // минимальное расстояние между участниками
            // Шаг между сообщениями: в эталоне Y сообщений 70.43 и 99.563,
            // разница 29.133
            message_spacing: 29.133,
            // Базовая ширина; расширяется по тексту
            participant_width: 50.0,
            // Высота бокса участника: в эталоне height="30.297"
            participant_height: 30.297,
            activation_width: 10.0,       // ширина блока активации
            fragment_padding: 10.0,       // отступ внутри фрагментов
            fragment_header_height: 22.0, // высота заголовка фрагмента
            divider_height: 25.0,         // высота разделителя
            delay_height: 20.0,           // высота задержки
            note_height: 30.0,            // высота заметки
            note_width: 100.0,            // ширина заметки
            // Отступ от края: в эталоне участник стоит на x=10, y=10
            margin: 10.0,
            font_size: 13.0,
            text: TextMeasurer::default(),
            line_height: 18.0,      // высота строки
            box_title_height: 30.0, // высота заголовка бокса
        }
    }
}

impl SequenceLayoutConfig {
    /// Создаёт конфигурацию по умолчанию
    pub fn new() -> Self {
        Self::default()
    }

    /// Ширина текста через общий измеритель.
    pub fn text_width(&self, text: &str) -> f64 {
        self.text.width(text, self.font_size)
    }

    /// Ширина участника с учётом имени.
    ///
    /// В эталоне PlantUML: textLength="33.667" при width="47.667", то есть
    /// по 7px внутреннего отступа с каждой стороны.
    pub fn participant_width_for_name(&self, name: &str) -> f64 {
        let text_width = self.text_width(name) + PARTICIPANT_PADDING * 2.0;
        self.participant_width.max(text_width)
    }

    /// Вычисляет ширину текста сообщения с отступами
    /// Для многострочного текста возвращает ширину самой длинной строки
    pub fn message_label_width(&self, label: &str) -> f64 {
        // Разбиваем на строки (учитываем и \n и \\n)
        let processed = label.replace("\\n", "\n");
        let max_line_width = processed
            .split('\n')
            .map(|line| self.text_width(line))
            .fold(0.0_f64, f64::max);
        max_line_width + 16.0 // padding с обеих сторон
    }
}
