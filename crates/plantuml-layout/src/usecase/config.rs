//! Конфигурация layout для Use Case Diagrams

/// Конфигурация Use Case Layout Engine
#[derive(Debug, Clone)]
pub struct UseCaseLayoutConfig {
    /// Отступ от края диаграммы
    pub margin: f64,
    /// Ширина эллипса use case
    pub usecase_width: f64,
    /// Высота эллипса use case
    pub usecase_height: f64,
    /// Ширина актёра
    pub actor_width: f64,
    /// Высота актёра
    pub actor_height: f64,
    /// Вертикальный отступ между элементами
    pub vertical_spacing: f64,
    /// Горизонтальный отступ между элементами
    pub horizontal_spacing: f64,
    /// Отступ внутри пакета/системы
    pub package_padding: f64,
    /// Высота заголовка пакета
    pub package_header_height: f64,
}

impl Default for UseCaseLayoutConfig {
    fn default() -> Self {
        Self {
            margin: 20.0,
            usecase_width: 160.0,    // Wider for longer text
            usecase_height: 40.0,    // PlantUML style
            actor_width: 40.0,       // Narrow actor
            actor_height: 70.0,      // Stick figure height
            vertical_spacing: 25.0,  // Compact
            horizontal_spacing: 80.0, // Increased for actor labels
            package_padding: 25.0,
            package_header_height: 30.0,
        }
    }
}
