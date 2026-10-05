//! Конфигурация layout для Timing Diagrams

use crate::text::TextMeasurer;

/// Конфигурация layout для Timing Diagrams
#[derive(Debug, Clone)]
pub struct TimingLayoutConfig {
    /// Измеритель текста.
    pub text: TextMeasurer,
    /// Отступ от краёв
    pub padding: f64,
    /// Ширина области имён участников
    pub participant_label_width: f64,
    /// Высота одного участника (lane)
    pub lane_height: f64,
    /// Вертикальный отступ между lanes
    pub lane_spacing: f64,
    /// Масштаб времени (пикселей на единицу времени)
    pub time_scale: f64,
    /// Высота состояния для robust
    pub robust_state_height: f64,
    /// Высота линии для concise
    pub concise_line_height: f64,
    /// Размер шрифта меток
    pub label_font_size: f64,
    /// Размер шрифта временных меток
    pub time_font_size: f64,
}

impl Default for TimingLayoutConfig {
    fn default() -> Self {
        Self {
            padding: 20.0,
            participant_label_width: 120.0,
            lane_height: 60.0,
            lane_spacing: 5.0,
            // Измерено по эталону PlantUML
            // (tests/golden/reference/timing_basic.svg): метка времени 0 на
            // x=88.2, метка 100 на x=131.2, то есть 0.43px на единицу.
            // Раньше стояло выдуманное 3.0, из-за чего диаграмма была
            // в семь раз шире эталона.
            time_scale: 0.43,
            robust_state_height: 30.0,
            concise_line_height: 20.0,
            label_font_size: 12.0,
            text: TextMeasurer::default(),
            time_font_size: 10.0,
        }
    }
}
