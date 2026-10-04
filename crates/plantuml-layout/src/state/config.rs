//! Конфигурация layout для State Diagrams

/// Конфигурация State Layout Engine
#[derive(Debug, Clone)]
pub struct StateLayoutConfig {
    /// Отступ от края диаграммы
    pub margin: f64,
    /// Ширина состояния
    pub state_width: f64,
    /// Минимальная высота состояния
    pub state_min_height: f64,
    /// Вертикальный отступ между состояниями
    pub vertical_spacing: f64,
    /// Горизонтальный отступ между состояниями
    pub horizontal_spacing: f64,
    /// Радиус начального/конечного круга
    pub node_radius: f64,
    /// Радиус скругления состояния
    pub state_corner_radius: f64,
    /// Ширина ромба choice
    pub choice_size: f64,
    /// Ширина полоски fork/join
    pub bar_width: f64,
    /// Высота полоски fork/join
    pub bar_height: f64,
    /// Размер стрелки
    pub arrow_size: f64,
    /// Отступ текста внутри состояния
    pub text_padding: f64,
}

impl Default for StateLayoutConfig {
    fn default() -> Self {
        Self {
            // Измерено по эталону PlantUML
            // (tests/golden/reference/state_simple.svg): начальный круг
            // занимает y=6..26 (margin 6), шаг от него до первого состояния
            // 61. Раньше стояло 30 и 60.
            margin: 6.0,
            state_width: 120.0,
            state_min_height: 50.0,
            vertical_spacing: 61.0,
            horizontal_spacing: 80.0,
            node_radius: 10.0,
            state_corner_radius: 10.0,
            choice_size: 20.0,
            bar_width: 80.0,
            bar_height: 6.0,
            arrow_size: 8.0,
            text_padding: 10.0,
        }
    }
}
