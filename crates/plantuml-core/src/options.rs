//! Опции рендеринга

use plantuml_themes::Theme;

/// Опции рендеринга диаграмм
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Тема оформления
    pub theme: Theme,

    /// Формат вывода
    pub format: OutputFormat,

    /// Масштаб (1.0 = 100%)
    pub scale: f64,

    /// Добавить XML заголовок в SVG
    pub xml_header: bool,

    /// Цвет фона (None = из темы)
    pub background_color: Option<String>,

    /// Максимальная ширина (None = без ограничений)
    pub max_width: Option<f64>,

    /// Максимальная высота (None = без ограничений)
    pub max_height: Option<f64>,

    /// Тёмная тема
    pub dark: bool,
}

/// Формат вывода
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// SVG (по умолчанию)
    #[default]
    Svg,
    // PNG будет добавлен позже через resvg
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            format: OutputFormat::default(),
            scale: 1.0,
            xml_header: true,
            background_color: None,
            max_width: None,
            max_height: None,
            dark: false,
        }
    }
}

impl RenderOptions {
    /// Создаёт опции по умолчанию
    pub fn new() -> Self {
        Self::default()
    }

    /// Включает тёмную тему
    pub fn with_dark(mut self, dark: bool) -> Self {
        self.dark = dark;
        self
    }

    /// Устанавливает тему
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Устанавливает тему по имени
    pub fn with_theme_name(mut self, name: &str) -> Self {
        if let Some(theme) = Theme::by_name(name) {
            self.theme = theme;
        }
        self
    }

    /// Устанавливает масштаб
    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    /// Устанавливает цвет фона
    pub fn with_background(mut self, color: impl Into<String>) -> Self {
        self.background_color = Some(color.into());
        self
    }

    /// Прозрачный фон
    pub fn with_transparent_background(mut self) -> Self {
        self.background_color = None;
        self
    }

    /// Без XML заголовка
    pub fn without_xml_header(mut self) -> Self {
        self.xml_header = false;
        self
    }

    /// Устанавливает максимальную ширину
    pub fn with_max_width(mut self, width: f64) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Устанавливает максимальную высоту
    pub fn with_max_height(mut self, height: f64) -> Self {
        self.max_height = Some(height);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = RenderOptions::default();
        assert_eq!(opts.scale, 1.0);
        assert!(opts.xml_header);
    }

    #[test]
    fn test_builder_pattern() {
        let opts = RenderOptions::new()
            .with_theme_name("dark")
            .with_scale(2.0)
            .without_xml_header();

        assert_eq!(opts.theme.name, "dark");
        assert_eq!(opts.scale, 2.0);
        assert!(!opts.xml_header);
    }
}
