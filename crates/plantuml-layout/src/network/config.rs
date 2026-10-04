//! Конфигурация layout для Network диаграмм

/// Конфигурация для Network layout engine
#[derive(Debug, Clone)]
pub struct NetworkLayoutConfig {
    /// Отступ от краёв диаграммы
    pub padding: f64,
    /// Высота полосы сети
    pub network_band_height: f64,
    /// Вертикальный отступ между сетями
    pub network_spacing: f64,
    /// Ширина сервера
    pub server_width: f64,
    /// Высота сервера
    pub server_height: f64,
    /// Горизонтальный отступ между серверами
    pub server_spacing: f64,
    /// Зазор между нижним краем полосы сети и верхом серверов.
    ///
    /// Ранее `network_band_height` использовался сразу в двух смыслах —
    /// как высота видимой полосы и как высота всего блока сети, — поэтому
    /// при уменьшении полосы до эталонных 5px серверы налезали на неё.
    pub server_top_offset: f64,
    /// Высота заголовка сети
    pub network_header_height: f64,
    /// Размер шрифта
    pub font_size: f64,
    /// Цвет фона сети
    pub network_bg_color: &'static str,
    /// Цвет фона сервера
    pub server_bg_color: &'static str,
    /// Цвет фона группы
    pub group_bg_color: &'static str,
}

impl Default for NetworkLayoutConfig {
    fn default() -> Self {
        Self {
            // Измерено по эталону PlantUML
            // (tests/golden/reference/network_nwdiag.svg, 117x129):
            // отступ 10, полоса сети 100x5 на y=12.5, серверы 20x30 на y=52.5
            // с шагом 50. Раньше стояли 20/120/40/100/60/30, из-за чего
            // диаграмма была почти втрое шире и выше эталона.
            padding: 10.0,
            network_band_height: 5.0,
            network_spacing: 35.0,
            server_width: 20.0,
            server_height: 30.0,
            server_spacing: 30.0,
            server_top_offset: 35.0,
            network_header_height: 15.0,
            font_size: 12.0,
            network_bg_color: "#E2E2F0",
            server_bg_color: "#FEFECE",
            group_bg_color: "#FFAAAA33",
        }
    }
}
