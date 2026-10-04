//! PNG рендеринг через resvg и tiny-skia
//!
//! Этот модуль доступен только при включённом feature `png`.
//!
//! # Пример
//!
//! ```rust,ignore
//! use plantuml_renderer::{PngRenderer, PngOptions};
//!
//! let svg_content = "<svg>...</svg>";
//! let renderer = PngRenderer::new(PngOptions::default());
//! let png_bytes = renderer.render_svg(svg_content)?;
//! std::fs::write("diagram.png", png_bytes)?;
//! ```

use thiserror::Error;

/// Ошибки PNG рендеринга
#[derive(Error, Debug)]
pub enum PngError {
    /// Ошибка парсинга SVG
    #[error("ошибка парсинга SVG: {0}")]
    SvgParseError(String),

    /// Ошибка создания pixmap
    #[error("не удалось создать pixmap: размер {width}x{height}")]
    PixmapCreationError { width: u32, height: u32 },

    /// Ошибка кодирования PNG
    #[error("ошибка кодирования PNG: {0}")]
    EncodingError(String),

    /// Некорректные размеры
    #[error("некорректные размеры изображения")]
    InvalidDimensions,
}

/// Опции PNG рендеринга
#[derive(Debug, Clone)]
pub struct PngOptions {
    /// Масштаб (1.0 = 100%)
    pub scale: f32,
    /// Цвет фона (None = прозрачный)
    pub background_color: Option<tiny_skia::Color>,
}

impl Default for PngOptions {
    fn default() -> Self {
        Self {
            scale: 1.0,
            background_color: Some(tiny_skia::Color::WHITE),
        }
    }
}

impl PngOptions {
    /// Создаёт опции с прозрачным фоном
    pub fn transparent() -> Self {
        Self {
            background_color: None,
            ..Default::default()
        }
    }

    /// Создаёт опции с указанным масштабом
    pub fn with_scale(scale: f32) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Устанавливает цвет фона из CSS-строки
    pub fn with_background(mut self, color: &str) -> Self {
        self.background_color = parse_css_color(color);
        self
    }
}

/// PNG рендерер
pub struct PngRenderer {
    options: PngOptions,
}

impl PngRenderer {
    /// Создаёт новый PNG рендерер с опциями по умолчанию
    pub fn new(options: PngOptions) -> Self {
        Self { options }
    }

    /// Рендерит SVG строку в PNG байты
    pub fn render_svg(&self, svg_content: &str) -> Result<Vec<u8>, PngError> {
        // Настройки парсинга SVG (без fontdb для WASM-совместимости)
        let usvg_options = resvg::usvg::Options::default();

        // Парсим SVG
        let tree = resvg::usvg::Tree::from_str(svg_content, &usvg_options)
            .map_err(|e| PngError::SvgParseError(e.to_string()))?;

        // Получаем размеры с учётом масштаба
        let size = tree.size();
        let width = (size.width() * self.options.scale) as u32;
        let height = (size.height() * self.options.scale) as u32;

        if width == 0 || height == 0 {
            return Err(PngError::InvalidDimensions);
        }

        // Создаём pixmap
        let mut pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or(PngError::PixmapCreationError { width, height })?;

        // Заполняем фоном
        if let Some(bg) = self.options.background_color {
            pixmap.fill(bg);
        }

        // Создаём трансформацию масштаба
        let transform = tiny_skia::Transform::from_scale(self.options.scale, self.options.scale);

        // Рендерим
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // Кодируем в PNG
        pixmap
            .encode_png()
            .map_err(|e| PngError::EncodingError(e.to_string()))
    }

    /// Рендерит SVG и сохраняет в файл
    pub fn render_to_file(
        &self,
        svg_content: &str,
        path: &std::path::Path,
    ) -> Result<(), PngError> {
        let png_data = self.render_svg(svg_content)?;
        std::fs::write(path, png_data)
            .map_err(|e| PngError::EncodingError(format!("не удалось записать файл: {}", e)))
    }
}

impl Default for PngRenderer {
    fn default() -> Self {
        Self::new(PngOptions::default())
    }
}

/// Парсит CSS цвет в tiny_skia::Color
fn parse_css_color(css: &str) -> Option<tiny_skia::Color> {
    let css = css.trim();

    // Формат #RRGGBB или #RGB
    if let Some(hex) = css.strip_prefix('#') {
        let hex = hex.to_uppercase();

        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(tiny_skia::Color::from_rgba8(r, g, b, 255));
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some(tiny_skia::Color::from_rgba8(r, g, b, 255));
        }
    }

    // Именованные цвета
    match css.to_lowercase().as_str() {
        "white" => Some(tiny_skia::Color::WHITE),
        "black" => Some(tiny_skia::Color::BLACK),
        "transparent" => Some(tiny_skia::Color::TRANSPARENT),
        "red" => Some(tiny_skia::Color::from_rgba8(255, 0, 0, 255)),
        "green" => Some(tiny_skia::Color::from_rgba8(0, 128, 0, 255)),
        "blue" => Some(tiny_skia::Color::from_rgba8(0, 0, 255, 255)),
        "yellow" => Some(tiny_skia::Color::from_rgba8(255, 255, 0, 255)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_css_color_hex6() {
        let color = parse_css_color("#FF0000").unwrap();
        assert!((color.red() - 1.0).abs() < 0.01); // 255/255 = 1.0
        assert!(color.green() < 0.01);
        assert!(color.blue() < 0.01);
    }

    #[test]
    fn test_parse_css_color_hex3() {
        let color = parse_css_color("#F00").unwrap();
        assert!((color.red() - 1.0).abs() < 0.01);
        assert!(color.green() < 0.01);
        assert!(color.blue() < 0.01);
    }

    #[test]
    fn test_parse_css_color_named() {
        assert!(parse_css_color("white").is_some());
        assert!(parse_css_color("BLACK").is_some());
        assert!(parse_css_color("unknown").is_none());
    }

    #[test]
    fn test_png_options_default() {
        let opts = PngOptions::default();
        assert_eq!(opts.scale, 1.0);
        assert!(opts.background_color.is_some());
    }

    #[test]
    fn test_png_options_transparent() {
        let opts = PngOptions::transparent();
        assert!(opts.background_color.is_none());
    }

    #[test]
    fn test_render_simple_svg() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
        </svg>"#;

        let renderer = PngRenderer::default();
        let result = renderer.render_svg(svg);

        assert!(result.is_ok());
        let png_data = result.unwrap();

        // PNG magic bytes
        assert!(png_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
    }

    #[test]
    fn test_render_with_scale() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="0" y="0" width="100" height="100" fill="red"/>
        </svg>"#;

        let renderer = PngRenderer::new(PngOptions::with_scale(2.0));
        let result = renderer.render_svg(svg);

        assert!(result.is_ok());
    }

    #[test]
    fn test_render_invalid_svg() {
        let svg = "not valid svg";

        let renderer = PngRenderer::default();
        let result = renderer.render_svg(svg);

        assert!(result.is_err());
        assert!(matches!(result, Err(PngError::SvgParseError(_))));
    }
}
