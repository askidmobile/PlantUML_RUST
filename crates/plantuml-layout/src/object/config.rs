//! Конфигурация layout для Object Diagrams

/// Конфигурация layout для Object Diagrams
#[derive(Debug, Clone)]
pub struct ObjectLayoutConfig {
    /// Ширина объекта
    pub object_width: f64,
    /// Минимальная высота объекта
    pub object_min_height: f64,
    /// Высота строки поля
    pub field_height: f64,
    /// Отступ между объектами по горизонтали
    pub horizontal_spacing: f64,
    /// Отступ между объектами по вертикали
    pub vertical_spacing: f64,
    /// Отступ от края диаграммы
    pub padding: f64,
}

impl Default for ObjectLayoutConfig {
    fn default() -> Self {
        Self {
            object_width: 140.0,
            object_min_height: 36.3,
            field_height: 16.3,
            horizontal_spacing: 35.0,
            vertical_spacing: 77.0,
            padding: 7.0,
        }
    }
}
