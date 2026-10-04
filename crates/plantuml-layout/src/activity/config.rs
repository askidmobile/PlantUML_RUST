//! Конфигурация layout для Activity Diagrams

/// Конфигурация Activity Layout Engine
#[derive(Debug, Clone)]
pub struct ActivityLayoutConfig {
    /// Отступ от края диаграммы
    pub margin: f64,
    /// Ширина блока действия
    pub action_width: f64,
    /// Высота блока действия
    pub action_height: f64,
    /// Вертикальный отступ между элементами
    pub vertical_spacing: f64,
    /// Горизонтальный отступ между ветками
    pub horizontal_spacing: f64,
    /// Радиус начального/конечного круга
    pub node_radius: f64,
    /// Ширина ромба условия
    pub diamond_width: f64,
    /// Высота ромба условия
    pub diamond_height: f64,
    /// Высота полоски fork/join
    pub bar_height: f64,
    /// Ширина полоски fork/join
    pub bar_width: f64,
    /// Радиус скругления действий
    pub action_corner_radius: f64,
    /// Размер стрелки
    pub arrow_size: f64,
    /// Ширина swimlane
    pub swimlane_width: f64,
    /// Высота заголовка swimlane
    pub swimlane_header_height: f64,
    /// Отступ между swimlanes
    pub swimlane_spacing: f64,
}

impl Default for ActivityLayoutConfig {
    fn default() -> Self {
        Self {
            margin: 20.0,
            action_width: 120.0,
            action_height: 40.0,
            vertical_spacing: 30.0,
            horizontal_spacing: 60.0,
            node_radius: 10.0,
            diamond_width: 30.0,
            diamond_height: 30.0,
            bar_height: 5.0,
            bar_width: 50.0,
            action_corner_radius: 10.0,
            arrow_size: 8.0,
            swimlane_width: 180.0,
            swimlane_header_height: 30.0,
            swimlane_spacing: 10.0,
        }
    }
}
