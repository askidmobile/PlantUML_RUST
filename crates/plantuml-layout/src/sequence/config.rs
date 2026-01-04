//! Конфигурация layout для Sequence Diagrams

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
    /// Примерная ширина символа (для расчёта ширины текста)
    pub char_width: f64,
    /// Высота строки текста
    pub line_height: f64,
    /// Высота заголовка бокса (participant box)
    pub box_title_height: f64,
}

impl Default for SequenceLayoutConfig {
    fn default() -> Self {
        Self {
            // PlantUML стиль: более компактные размеры
            participant_spacing: 50.0,    // дефолт для пар без сообщений
            message_spacing: 28.0,        // PlantUML ~28px между однострочными сообщениями
            participant_width: 50.0,      // уменьшено (было 80) - PlantUML адаптивный
            participant_height: 30.0,     // уменьшено (было 40)
            activation_width: 10.0,       // уменьшено (было 12)
            fragment_padding: 10.0,       // уменьшено (было 15)
            fragment_header_height: 22.0, // уменьшено (было 25)
            divider_height: 25.0,         // уменьшено (было 30)
            delay_height: 20.0,           // уменьшено (было 25)
            note_height: 30.0,            // уменьшено (было 35)
            note_width: 100.0,            // уменьшено (было 120)
            margin: 15.0,                 // уменьшено (было 20)
            font_size: 13.0,
            char_width: 7.0,
            line_height: 16.0, // уменьшено (было 18)
            box_title_height: 30.0, // высота для заголовка бокса (отступ от верха box до участников)
        }
    }
}

impl SequenceLayoutConfig {
    /// Создаёт конфигурацию по умолчанию
    pub fn new() -> Self {
        Self::default()
    }

    /// Вычисляет примерную ширину текста (с учётом Unicode)
    pub fn text_width(&self, text: &str) -> f64 {
        // Считаем символы, а не байты (для корректной работы с кириллицей)
        let char_count = text.chars().count();
        char_count as f64 * self.char_width
    }

    /// Вычисляет ширину участника с учётом имени
    pub fn participant_width_for_name(&self, name: &str) -> f64 {
        let text_width = self.text_width(name) + 20.0; // padding
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
