//! Конфигурация layout для Gantt Diagrams

use crate::text::TextMeasurer;

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
    /// Измеритель текста (считает символы, а не байты)
    pub text: TextMeasurer,
    /// Размер шрифта дат
    pub date_font_size: f64,
}

impl Default for GanttLayoutConfig {
    fn default() -> Self {
        Self {
            // Измерено по эталону: первая полоса начинается на x=138.069
            // при task_label_width 133.07, то есть padding 5.
            padding: 5.0,
            // Измерено по эталону PlantUML: первая полоса задачи
            // начинается на x=138.069 при padding 5, то есть ширина блока
            // подписи 133.07. Раньше стояло 150.0.
            task_label_width: 133.07,
            row_height: 30.0,
            bar_height: 20.0,
            row_spacing: 5.0,
            // Измерено по эталону PlantUML
            // (tests/golden/reference/gantt_basic.svg): полоса задачи
            // длительностью 10 дней имеет ширину 156px, то есть 15.6px на
            // день. Раньше стояло 20.0, из-за чего диаграмма расходилась.
            // 10 дней дают полосу 156px, 20 дней — 316px, 5 дней — 76px,
            // то есть около 15.7px на день.
            day_width: 15.7,
            header_height: 36.0,
            label_font_size: 12.0,
            // Размер подписи задачи. В эталоне gantt_basic она нарисована
            // font-size=11 (textLength «Тестирование» = 81.136). Раньше
            // стояло 10, и подпись ошибочно помещалась ВНУТРЬ полосы;
            // ширина диаграммы сходилась лишь потому, что измерение текста
            // было завышено и случайно попадало в нужную ветку.
            date_font_size: 11.0,
            text: TextMeasurer::default(),
        }
    }
}
