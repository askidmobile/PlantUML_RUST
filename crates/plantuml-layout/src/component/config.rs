//! Конфигурация layout для Component Diagrams

use crate::text::TextMeasurer;

/// Конфигурация Component Layout Engine
#[derive(Debug, Clone)]
pub struct ComponentLayoutConfig {
    /// Отступ от края диаграммы
    pub margin: f64,
    /// Ширина компонента
    pub component_width: f64,
    /// Высота компонента
    pub component_height: f64,
    /// Вертикальный отступ между элементами
    pub vertical_spacing: f64,
    /// Горизонтальный отступ между элементами
    pub horizontal_spacing: f64,
    /// Радиус интерфейса (кружок)
    pub interface_radius: f64,
    /// Ширина пакета
    pub package_padding: f64,
    /// Высота заголовка пакета
    pub package_header_height: f64,
    /// Радиус скругления
    pub corner_radius: f64,
    /// Размер иконки компонента
    pub icon_size: f64,
    /// Измеритель текста (считает символы, а не байты)
    pub text: TextMeasurer,
    /// Размер шрифта подписей (в эталоне 14).
    pub font_size: f64,
}

impl Default for ComponentLayoutConfig {
    fn default() -> Self {
        Self {
            // Измерено по эталону: тела компонентов начинаются на x=7 и y=7.
            margin: 7.0,
            // Размеры измерены по эталону PlantUML
            // (tests/golden/reference/component_basic.svg): компонент 40x40,
            // шаг по вертикали 112 (40 + отступ 72), подпись рисуется над
            // фигурой. Раньше было 140x60, из-за чего диаграмма получалась
            // широкой и низкой: 410x240 против эталонных 62x295.
            component_width: 40.0,
            // Высота задаётся контентно в движке (COMPONENT_HEIGHT = 46.297).
            component_height: 46.297,
            // Вертикальный шаг измерен по эталону: тела компонентов стоят на
            // y=7 и 130.297, то есть шаг 123.297 при высоте тела 46.297,
            // значит отступ между ними 77.
            vertical_spacing: 77.0,
            horizontal_spacing: 30.0,
            interface_radius: 10.0,
            // Измерено по эталону deployment_basic: контейнер «Сервер
            // приложений» имеет высоту 95.29 = 40 (заголовок с отступом)
            // + 39.297 (содержимое) + 16 (нижний отступ). Отсюда заголовок
            // 25 при отступе 15 и нижнем отступе 16.
            package_padding: 15.0,
            package_header_height: 25.0,
            corner_radius: 5.0,
            icon_size: 16.0,
            text: TextMeasurer::default(),
            font_size: 14.0,
        }
    }
}
