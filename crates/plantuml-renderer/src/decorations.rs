//! Заголовок, колонтитулы, подпись, легенда и рамка вокруг готовой диаграммы.
//!
//! Эти элементы не зависят от типа схемы и не участвуют в её раскладке, поэтому
//! рисуются один раз здесь, поверх готового SVG. Содержимое диаграммы сдвигается
//! группой `<g transform>`, а корневой элемент получает новые размеры.
//!
//! Отступы сняты с эталонов PlantUML: `feat_seq_title_header_footer`,
//! `feat_seq_legend_caption`, `real_styling` в `tests/golden/corpus_reference`.

use plantuml_ast::common::{DiagramMetadata, LegendAlign};
use plantuml_layout::text::TextMeasurer;

const TITLE_SIZE: f64 = 14.0;
const HEADER_SIZE: f64 = 10.0;
const HEADER_COLOR: &str = "#888";
const LEGEND_FILL: &str = "#DDD";
const LEGEND_RADIUS: f64 = 7.5;
const LEGEND_PADDING: f64 = 5.0;

/// Отступ заголовка сверху без шапки и с ней.
const TITLE_TOP_ALONE: f64 = 15.0;
const TITLE_TOP_AFTER_HEADER: f64 = 6.0;
const TITLE_BOTTOM: f64 = 6.0;
const HEADER_PADDING: f64 = 5.0;
/// Нижнее поле самой диаграммы: его заменяют нижние блоки.
const MAIN_BOTTOM_MARGIN: f64 = 9.0;
const LEGEND_TOP: f64 = 17.0;
const CAPTION_GAP: f64 = 14.0;
const CAPTION_BOTTOM: f64 = 8.0;
const FOOTER_TOP: f64 = 5.0;
const FOOTER_BOTTOM: f64 = 7.0;
const SIDE_MARGIN: f64 = 7.0;
const FRAME_MARGIN: f64 = 6.0;
const FRAME_TAB_HEIGHT: f64 = 18.0;
const FRAME_TAB_PADDING: f64 = 8.0;

fn line_height(size: f64) -> f64 {
    size * 1.1641
}

fn ascent(size: f64) -> f64 {
    size * 0.9282
}

/// Строки подписи: настоящие переводы и запись `\n`.
fn lines_of(text: &str) -> Vec<String> {
    text.replace("\\n", "\n")
        .lines()
        .map(str::to_string)
        .collect()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn fmt(value: f64) -> String {
    format!("{value:.3}")
}

/// Блок текста из нескольких строк с заданной позицией по горизонтали.
struct Block {
    lines: Vec<String>,
    size: f64,
    bold: bool,
    color: &'static str,
}

impl Block {
    fn new(text: &str, size: f64, bold: bool, color: &'static str) -> Self {
        Self {
            lines: lines_of(text),
            size,
            bold,
            color,
        }
    }

    fn width(&self) -> f64 {
        let metrics = TextMeasurer::default();
        self.lines
            .iter()
            .map(|line| {
                if self.bold {
                    metrics.width_bold(line, self.size)
                } else {
                    metrics.width(line, self.size)
                }
            })
            .fold(0.0, f64::max)
    }

    fn height(&self) -> f64 {
        self.lines.len() as f64 * line_height(self.size)
    }

    /// Рисует блок: `align` — доля свободной ширины слева (0 — влево, 1 — вправо).
    fn draw(&self, out: &mut String, left: f64, avail: f64, align: f64, top: f64) {
        let metrics = TextMeasurer::default();
        for (index, line) in self.lines.iter().enumerate() {
            let width = if self.bold {
                metrics.width_bold(line, self.size)
            } else {
                metrics.width(line, self.size)
            };
            let x = left + (avail - width) * align;
            let y = top + ascent(self.size) + index as f64 * line_height(self.size);
            let weight = if self.bold {
                r#" font-weight="700""#
            } else {
                ""
            };
            out.push_str(&format!(
                r#"<text x="{}" y="{}" fill="{}" font-size="{}" font-family="sans-serif" textLength="{}" lengthAdjust="spacing"{weight}>{}</text>"#,
                fmt(x),
                fmt(y),
                self.color,
                self.size,
                fmt(width),
                escape(line),
            ));
        }
    }
}

/// Значение числового атрибута корневого элемента.
fn number_attr(root: &str, name: &str) -> Option<f64> {
    let needle = format!(" {name}=\"");
    let start = root.find(&needle)? + needle.len();
    let rest = &root[start..];
    rest[..rest.find('"')?].trim_end_matches("px").parse().ok()
}

/// Значения `viewBox`.
fn view_box(root: &str) -> Option<[f64; 4]> {
    let needle = " viewBox=\"";
    let start = root.find(needle)? + needle.len();
    let rest = &root[start..];
    let raw = &rest[..rest.find('"')?];
    let mut parts = raw.split_whitespace().filter_map(|v| v.parse::<f64>().ok());
    Some([parts.next()?, parts.next()?, parts.next()?, parts.next()?])
}

/// Заменяет значение атрибута в корневом элементе.
fn with_attr(root: &str, name: &str, value: &str) -> String {
    let needle = format!(" {name}=\"");
    let Some(start) = root.find(&needle).map(|i| i + needle.len()) else {
        return root.to_string();
    };
    let Some(len) = root[start..].find('"') else {
        return root.to_string();
    };
    format!("{}{}{}", &root[..start], value, &root[start + len..])
}

/// Добавляет к готовому SVG элементы из метаданных. Без них SVG не меняется.
pub fn decorate(svg: String, meta: &DiagramMetadata) -> String {
    let has_any = meta.title.is_some()
        || meta.header.is_some()
        || meta.footer.is_some()
        || meta.caption.is_some()
        || meta.legend.is_some()
        || meta.mainframe.is_some();
    if !has_any {
        return svg;
    }

    // Разбираем только свой же корневой элемент; при неожиданной форме
    // возвращаем диаграмму как есть, без украшений.
    let Some(root_start) = svg.find("<svg") else {
        return svg;
    };
    let Some(root_len) = svg[root_start..].find('>') else {
        return svg;
    };
    let root_end = root_start + root_len + 1;
    let Some(inner_end) = svg.rfind("</svg>") else {
        return svg;
    };
    let root = &svg[root_start..root_end];
    let (Some(main_w), Some(main_h), Some(vb)) = (
        number_attr(root, "width"),
        number_attr(root, "height"),
        view_box(root),
    ) else {
        return svg;
    };

    let header = meta
        .header
        .as_deref()
        .map(|t| Block::new(t, HEADER_SIZE, false, HEADER_COLOR));
    let title = meta
        .title
        .as_deref()
        .map(|t| Block::new(t, TITLE_SIZE, true, "#000"));
    let caption = meta
        .caption
        .as_deref()
        .map(|t| Block::new(t, TITLE_SIZE, false, "#000"));
    let legend = meta
        .legend
        .as_deref()
        .map(|t| Block::new(t, TITLE_SIZE, false, "#000"));
    let footer = meta
        .footer
        .as_deref()
        .map(|t| Block::new(t, HEADER_SIZE, false, HEADER_COLOR));
    let frame = meta
        .mainframe
        .as_deref()
        .map(|t| Block::new(t, TITLE_SIZE, true, "#000"));

    let text_width = [&header, &title, &caption, &footer]
        .into_iter()
        .flatten()
        .map(Block::width)
        .chain(legend.iter().map(|b| b.width() + LEGEND_PADDING * 2.0))
        .chain(frame.iter().map(|b| b.width() + FRAME_TAB_PADDING * 2.0))
        .fold(0.0, f64::max);
    let frame_pad = if frame.is_some() { FRAME_MARGIN } else { 0.0 };
    let inner_w = main_w.max(text_width + SIDE_MARGIN * 2.0);
    let width = inner_w + frame_pad * 2.0;
    let left = frame_pad + SIDE_MARGIN;
    let avail = inner_w - SIDE_MARGIN * 2.0;

    let mut body = String::new();
    let mut y = frame_pad;

    if frame.is_some() {
        y += FRAME_TAB_HEIGHT;
    }
    if let Some(block) = &header {
        y += HEADER_PADDING;
        block.draw(&mut body, left, avail, 1.0, y);
        y += block.height() + HEADER_PADDING;
    }
    if let Some(block) = &title {
        y += if header.is_some() {
            TITLE_TOP_AFTER_HEADER
        } else {
            TITLE_TOP_ALONE
        };
        block.draw(&mut body, left, avail, 0.5, y);
        y += block.height() + TITLE_BOTTOM;
    }

    let dx = frame_pad + (inner_w - main_w) / 2.0 - vb[0];
    let dy = y - vb[1];
    let main_bottom = y + main_h;
    y = main_bottom;

    if legend.is_some() || caption.is_some() || footer.is_some() {
        y -= MAIN_BOTTOM_MARGIN;
    }
    if let Some(block) = &legend {
        y += LEGEND_TOP;
        let box_w = block.width() + LEGEND_PADDING * 2.0;
        let box_h = block.height() + LEGEND_PADDING * 2.0;
        let align = match meta.legend_align {
            LegendAlign::Left => 0.0,
            LegendAlign::Center => 0.5,
            LegendAlign::Right => 1.0,
        };
        let box_x = left + (avail - box_w) * align;
        body.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{LEGEND_FILL}" style="stroke:#000;stroke-width:1;" rx="{LEGEND_RADIUS}" ry="{LEGEND_RADIUS}"/>"#,
            fmt(box_x),
            fmt(y),
            fmt(box_w),
            fmt(box_h),
        ));
        block.draw(
            &mut body,
            box_x + LEGEND_PADDING,
            box_w,
            0.0,
            y + LEGEND_PADDING,
        );
        y += box_h;
    }
    if let Some(block) = &caption {
        y += CAPTION_GAP;
        block.draw(&mut body, left, avail, 0.5, y);
        y += block.height();
        if footer.is_none() {
            y += CAPTION_BOTTOM;
        }
    }
    if let Some(block) = &footer {
        y += FOOTER_TOP;
        block.draw(&mut body, left, avail, 0.5, y);
        y += block.height() + FOOTER_BOTTOM;
    }

    let height = y + frame_pad;
    if let Some(block) = &frame {
        let tab_w = block.width() + FRAME_TAB_PADDING * 2.0;
        body.push_str(&format!(
            r#"<rect x="{0}" y="{0}" width="{1}" height="{2}" fill="none" style="stroke:#000;stroke-width:1.5;"/><path d="M{0},{3} L{4},{3} L{5},{6} L{5},{0}" fill="none" style="stroke:#000;stroke-width:1.5;"/>"#,
            fmt(FRAME_MARGIN / 2.0),
            fmt(width - FRAME_MARGIN),
            fmt(height - FRAME_MARGIN),
            fmt(FRAME_MARGIN / 2.0 + FRAME_TAB_HEIGHT),
            fmt(FRAME_MARGIN / 2.0 + tab_w - 6.0),
            fmt(FRAME_MARGIN / 2.0 + tab_w),
            fmt(FRAME_MARGIN / 2.0 + FRAME_TAB_HEIGHT - 6.0),
        ));
        block.draw(
            &mut body,
            FRAME_MARGIN / 2.0 + FRAME_TAB_PADDING,
            tab_w,
            0.0,
            FRAME_MARGIN / 2.0 + 1.0,
        );
    }

    let width_s = format!("{}px", width.ceil());
    let height_s = format!("{}px", height.ceil());
    let mut new_root = with_attr(root, "width", &width_s);
    new_root = with_attr(&new_root, "height", &height_s);
    new_root = with_attr(
        &new_root,
        "viewBox",
        &format!("0 0 {} {}", width.ceil(), height.ceil()),
    );
    new_root = with_attr(
        &new_root,
        "style",
        &format!("width:{width_s};height:{height_s};background:#FFFFFF;"),
    );

    let body = crate::rich_text::expand_markup(&body);
    format!(
        "{}{new_root}<g transform=\"translate({} {})\">{}</g>{body}</svg>{}",
        &svg[..root_start],
        fmt(dx),
        fmt(dy),
        &svg[root_end..inner_end],
        &svg[inner_end + "</svg>".len()..],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str =
        r#"<svg width="100px" height="50px" viewBox="0 0 100 50" style="x"><rect/></svg>"#;

    fn meta() -> DiagramMetadata {
        DiagramMetadata::default()
    }

    #[test]
    fn test_no_metadata_keeps_svg() {
        assert_eq!(decorate(SVG.to_string(), &meta()), SVG);
    }

    #[test]
    fn test_title_grows_canvas_and_shifts_content() {
        let mut m = meta();
        m.title = Some("Тема <x>".to_string());
        let out = decorate(SVG.to_string(), &m);
        assert!(out.contains("Тема &lt;x&gt;"));
        assert!(
            out.contains(r#"<g transform="translate(0.000 37.297)"><rect/></g>"#),
            "{out}"
        );
        assert!(!out.contains(r#"height="50px""#));
    }

    #[test]
    fn test_all_blocks_are_drawn() {
        let mut m = meta();
        m.header = Some("Ш".to_string());
        m.footer = Some("П".to_string());
        m.caption = Some("Ц".to_string());
        m.legend = Some("Л".to_string());
        m.mainframe = Some("Р".to_string());
        let out = decorate(SVG.to_string(), &m);
        for word in ["Ш", "П", "Ц", "Л", "Р"] {
            assert!(out.contains(&format!(">{word}</text>")), "нет {word}");
        }
    }

    #[test]
    fn test_multiline_title_uses_escape_form() {
        let mut m = meta();
        m.title = Some("а\\nб".to_string());
        let out = decorate(SVG.to_string(), &m);
        assert!(out.contains(">а</text>") && out.contains(">б</text>"));
    }
}
