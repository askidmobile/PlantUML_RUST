//! # plantuml-renderer
//!
//! SVG и PNG рендеринг диаграмм PlantUML.
//!
//! ## Features
//!
//! - `png` - включает PNG рендеринг через resvg (требует дополнительные зависимости)
//!
//! ## Пример
//!
//! ```rust,ignore
//! use plantuml_renderer::{SvgRenderer, RenderOptions};
//!
//! let renderer = SvgRenderer::with_options(RenderOptions::default());
//! let svg = renderer.render(&layout, &theme);
//!
//! // С PNG (требует feature "png"):
//! #[cfg(feature = "png")]
//! {
//!     use plantuml_renderer::{PngRenderer, PngOptions};
//!     let png_renderer = PngRenderer::default();
//!     let png_bytes = png_renderer.render_svg(&svg)?;
//! }
//! ```

pub mod shapes;
pub mod svg_renderer;

#[cfg(feature = "png")]
pub mod png_renderer;

pub use plantuml_layout::{
    ClassMember, ClassifierKind, EdgeType, ElementType, FragmentSection, 
    LayoutElement, LayoutResult, MemberVisibility, Point, Rect, ZLayer,
};
pub use plantuml_themes::Theme;
pub use svg_renderer::SvgRenderer;

#[cfg(feature = "png")]
pub use png_renderer::{PngError, PngOptions, PngRenderer};

/// Трейт для рендереров
pub trait Renderer {
    /// Тип выходных данных
    type Output;

    /// Рендерит результат layout
    fn render(&self, layout: &LayoutResult, theme: &Theme) -> Self::Output;
}

/// Опции рендеринга
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Добавить XML заголовок
    pub xml_header: bool,
    /// Масштаб
    pub scale: f64,
    /// Цвет фона (None = прозрачный)
    pub background_color: Option<String>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            xml_header: true,
            scale: 1.0,
            background_color: None, // None = PlantUML default (#FEFECE)
        }
    }
}
