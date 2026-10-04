//! Конфигурация layout для Gantt Diagrams

/// Конфигурация layout для Gantt Diagrams
#[derive(Debug, Clone)]
pub struct GanttLayoutConfig {
    /// Отступ от краёв
    pub padding: f64,
    /// Ширина области имён задач
    pub task_label_width: f64,
    /// Высота одной задачи (строки)
    pub row_height: f64,
    /// Высота бара задачи
    pub bar_height: f64,
    /// Вертикальный отступ между строками
    pub row_spacing: f64,
    /// Ширина одного дня
    pub day_width: f64,
    /// Высота заголовка с датами
    pub header_height: f64,
    /// Размер шрифта меток
    pub label_font_size: f64,
    /// Размер шрифта дат
    pub date_font_size: f64,
}

impl Default for GanttLayoutConfig {
    fn default() -> Self {
        Self {
            padding: 20.0,
            task_label_width: 150.0,
            row_height: 30.0,
            bar_height: 20.0,
            row_spacing: 5.0,
            // Измерено по эталону PlantUML
            // (tests/golden/reference/gantt_basic.svg): полоса задачи
            // длительностью 10 дней имеет ширину 156px, то есть 15.6px на
            // день. Раньше стояло 20.0, из-за чего диаграмма расходилась.
            day_width: 15.6,
            header_height: 40.0,
            label_font_size: 12.0,
            date_font_size: 10.0,
        }
    }
}
