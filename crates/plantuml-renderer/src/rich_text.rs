//! Разворачивает разметку подписей в готовом SVG.
//!
//! Текст с разметкой приходит сюда из любого типа диаграмм, поэтому это единая
//! точка: подпись из `<text>` разбирается и становится цепочкой элементов с
//! явными координатами. Так же выводит подписи и сам PlantUML.

use plantuml_ast::class::Sprite;
use plantuml_layout::creole::{self, Span};
use plantuml_layout::text::TextMeasurer;

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    Some(&rest[..rest.find('"')?])
}

pub(crate) fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Строки таблицы и списка creole.
fn structural(line: &str, number: usize) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with('|') && trimmed.trim_end().ends_with('|') {
        let cells: Vec<String> = trimmed
            .trim()
            .trim_matches('|')
            .split('|')
            .map(|cell| match cell.trim().strip_prefix('=') {
                Some(head) => format!("**{}**", head.trim()),
                None => cell.trim().to_string(),
            })
            .collect();
        return cells.join("  │  ");
    }
    let stars = trimmed.chars().take_while(|c| *c == '*').count();
    if stars > 0 && trimmed.chars().nth(stars) == Some(' ') {
        let body = &trimmed[stars + 1..];
        return format!("{}• {body}", "  ".repeat(stars - 1));
    }
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if hashes > 0 && trimmed.chars().nth(hashes) == Some(' ') {
        let body = &trimmed[hashes + 1..];
        return format!("{}{number}. {body}", "  ".repeat(hashes - 1));
    }
    line.to_string()
}

/// Размер спрайта подписи; неизвестный спрайт места не занимает.
fn sprite_size(span: &Span, sprites: &[Sprite]) -> Option<(f64, f64, f64)> {
    let (name, scale) = span.sprite.as_ref()?;
    let sprite = sprites.iter().find(|s| &s.name == name)?;
    Some((
        sprite.width as f64 * scale,
        sprite.height as f64 * scale,
        *scale,
    ))
}

fn span_width(measurer: &TextMeasurer, span: &Span, base_size: f64, base_bold: bool) -> f64 {
    let size = span.size.unwrap_or(base_size);
    if span.bold || base_bold {
        measurer.width_bold(&span.text, size)
    } else {
        measurer.width(&span.text, size)
    }
}

/// Выводит одну строку цепочкой элементов.
fn emit_line(
    out: &mut String,
    tag: &str,
    line: &str,
    at: (f64, f64),
    sprites: &[Sprite],
    number: usize,
) {
    let (x, y) = at;
    let base_size: f64 = attr(tag, "font-size")
        .and_then(|v| v.parse().ok())
        .unwrap_or(13.0);
    let base_bold = attr(tag, "font-weight").is_some_and(|w| w == "700" || w == "bold");
    let base_fill = attr(tag, "fill").unwrap_or("#000");
    let family = attr(tag, "font-family");
    let anchor = attr(tag, "text-anchor").unwrap_or("start");
    let class = attr(tag, "font-style");

    let measurer = TextMeasurer::default();
    let spans = creole::parse(&structural(line, number));
    let width_of = |s: &Span| match sprite_size(s, sprites) {
        Some((width, _, _)) => width,
        None if s.sprite.is_some() => 0.0,
        None => span_width(&measurer, s, base_size, base_bold),
    };
    let total: f64 = spans.iter().map(width_of).sum();
    let mut cursor = match anchor {
        "middle" => x - total / 2.0,
        "end" => x - total,
        _ => x,
    };

    for span in &spans {
        if let Some((width, height, scale)) = sprite_size(span, sprites) {
            if let Some(sprite) = sprites
                .iter()
                .find(|s| span.sprite.as_ref().is_some_and(|(n, _)| &s.name == n))
            {
                draw_sprite(
                    out,
                    sprite,
                    cursor,
                    y - height + base_size * 0.2,
                    scale,
                    base_fill,
                );
            }
            cursor += width;
            continue;
        }
        if span.sprite.is_some() {
            continue;
        }
        let width = span_width(&measurer, span, base_size, base_bold);
        let size = span.size.unwrap_or(base_size);
        let mut attrs = format!(
            r#" x="{cursor:.3}" y="{y:.3}" fill="{}" font-size="{size}""#,
            span.color.as_deref().unwrap_or(base_fill)
        );
        if span.mono {
            attrs.push_str(r#" font-family="monospace""#);
        } else if let Some(family) = family {
            attrs.push_str(&format!(r#" font-family="{family}""#));
        }
        if span.bold || base_bold {
            attrs.push_str(r#" font-weight="700""#);
        }
        if span.italic || class == Some("italic") {
            attrs.push_str(r#" font-style="italic""#);
        }
        match (span.underline, span.strike) {
            (true, true) => attrs.push_str(r#" text-decoration="underline line-through""#),
            (true, false) => attrs.push_str(r#" text-decoration="underline""#),
            (false, true) => attrs.push_str(r#" text-decoration="line-through""#),
            _ => {}
        }
        if !span.text.trim().is_empty() {
            attrs.push_str(&format!(
                r#" textLength="{width:.3}" lengthAdjust="spacing""#
            ));
        }
        let element = format!("<text{attrs}>{}</text>", escape(&span.text));
        match &span.href {
            Some(href) => {
                let href = escape(href);
                out.push_str(&format!(
                    r#"<a target="_top" href="{href}" xlink:href="{href}">{element}</a>"#
                ));
            }
            None => out.push_str(&element),
        }
        cursor += width;
    }
}

/// Рисует растровый спрайт цветом текста: цифра задаёт непрозрачность пикселя.
fn draw_sprite(out: &mut String, sprite: &Sprite, x: f64, y: f64, scale: f64, fill: &str) {
    for (row_index, row) in sprite.rows.iter().enumerate() {
        let cells: Vec<u32> = row.chars().map(|c| c.to_digit(16).unwrap_or(0)).collect();
        let mut col = 0;
        while col < cells.len() {
            let digit = cells[col];
            let mut run = 1;
            while col + run < cells.len() && cells[col + run] == digit {
                run += 1;
            }
            if digit > 0 {
                out.push_str(&format!(
                    r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{fill}" fill-opacity="{:.2}"/>"#,
                    x + col as f64 * scale,
                    y + row_index as f64 * scale,
                    run as f64 * scale,
                    scale,
                    f64::from(digit) / 15.0,
                ));
            }
            col += run;
        }
    }
}

/// Насколько спрайт выше обычной строки.
fn line_extra(line: &str, line_height: f64, sprites: &[Sprite]) -> f64 {
    creole::parse(line)
        .iter()
        .filter_map(|span| sprite_size(span, sprites))
        .map(|(_, height, _)| (height - line_height).max(0.0))
        .fold(0.0, f64::max)
}

/// Разметка, из-за которой элемент надо разбирать.
fn needs_expansion(content: &str) -> bool {
    let text = unescape(content);
    if text.contains("\\n") {
        return true;
    }
    text.lines()
        .any(|line| creole::plain(line) != line || line.trim_start().starts_with(['*', '#', '|']))
}

/// Строки элемента: `<tspan x y>` либо текст с `\n`.
fn expand(out: &mut String, tag: &str, content: &str, sprites: &[Sprite], number: usize) {
    let x: f64 = attr(tag, "x").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let y: f64 = attr(tag, "y").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let size: f64 = attr(tag, "font-size")
        .and_then(|v| v.parse().ok())
        .unwrap_or(13.0);
    let line_height = size * 1.1641;

    // Строка, позиция по x и y.
    let mut lines: Vec<(String, f64, f64)> = Vec::new();
    if content.contains("<tspan") {
        let mut rest = content;
        while let Some(open) = rest.find("<tspan") {
            let tail = &rest[open..];
            let Some(gt) = tail.find('>') else { break };
            let Some(close) = tail.find("</tspan>") else {
                break;
            };
            let span_tag = &tail[..gt];
            let span_x = attr(span_tag, "x")
                .and_then(|v| v.parse().ok())
                .unwrap_or(x);
            let span_y = attr(span_tag, "y")
                .and_then(|v| v.parse().ok())
                .unwrap_or(y + lines.len() as f64 * line_height);
            lines.push((unescape(&tail[gt + 1..close]), span_x, span_y));
            rest = &tail[close + "</tspan>".len()..];
        }
    } else {
        let text = unescape(content).replace("\\n", "\n");
        for (index, line) in text.lines().enumerate() {
            lines.push((line.to_string(), x, y + index as f64 * line_height));
        }
    }

    // Высокий спрайт раздвигает строки; блок остаётся по центру своей фигуры.
    let extras: Vec<f64> = lines
        .iter()
        .map(|(line, _, _)| line_extra(line, line_height, sprites))
        .collect();
    let mut shift = -extras.iter().sum::<f64>() / 2.0;
    for ((line, line_x, line_y), extra) in lines.iter().zip(&extras) {
        shift += extra;
        emit_line(out, tag, line, (*line_x, line_y + shift), sprites, number);
    }
}

/// Разворачивает разметку всех подписей документа.
pub fn expand_markup(svg: &str, sprites: &[Sprite]) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    let mut number = 0;
    while let Some(open) = rest.find("<text") {
        out.push_str(&rest[..open]);
        let tail = &rest[open..];
        let (Some(gt), Some(close)) = (tail.find('>'), tail.find("</text>")) else {
            out.push_str(tail);
            return out;
        };
        let tag = &tail[..gt];
        let content = &tail[gt + 1..close];
        // Пункты списка лежат в отдельных элементах: считаем их здесь.
        if content.trim_start().starts_with("# ") {
            number += 1;
        } else {
            number = 0;
        }
        if needs_expansion(content) || content.contains("<tspan") && has_tspan_markup(content) {
            expand(&mut out, tag, content, sprites, number);
        } else {
            out.push_str(&tail[..close + "</text>".len()]);
        }
        rest = &tail[close + "</text>".len()..];
    }
    out.push_str(rest);
    out
}

/// Разметка внутри строк `<tspan>`.
fn has_tspan_markup(content: &str) -> bool {
    let mut rest = content;
    while let Some(open) = rest.find("<tspan") {
        let tail = &rest[open..];
        let (Some(gt), Some(close)) = (tail.find('>'), tail.find("</tspan>")) else {
            return false;
        };
        if needs_expansion(&tail[gt + 1..close]) {
            return true;
        }
        rest = &tail[close + "</tspan>".len()..];
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = r##"<text x="10" y="20" fill="#000" font-size="13">"##;

    fn run(content: &str) -> String {
        expand_markup(&format!("{HEAD}{content}</text>"), &[])
    }

    #[test]
    fn test_plain_text_is_untouched() {
        let svg = format!("{HEAD}просто текст</text>");
        assert_eq!(expand_markup(&svg, &[]), svg);
    }

    #[test]
    fn test_bold_becomes_own_element() {
        let out = run("a **b** c");
        assert!(out.contains(r#"font-weight="700""#), "{out}");
        assert!(out.contains(">b</text>"), "{out}");
        assert!(!out.contains("**"), "{out}");
    }

    #[test]
    fn test_literal_newline_splits_lines() {
        let out = run("а\\nб");
        assert!(
            out.contains(">а</text>") && out.contains(">б</text>"),
            "{out}"
        );
        assert!(!out.contains("\\n"), "{out}");
    }

    #[test]
    fn test_safe_link_gets_anchor_and_unsafe_does_not() {
        assert!(run("[[https://e.example x]]").contains(r#"href="https://e.example""#));
        let bad = run("[[javascript:alert(1) x]]");
        assert!(!bad.contains("href"), "{bad}");
        assert!(!bad.contains("javascript"), "{bad}");
    }

    #[test]
    fn test_escaped_input_stays_escaped() {
        let out = run("&lt;script&gt;alert(1)&lt;/script&gt; **x**");
        assert!(!out.contains("<script"), "{out}");
        assert!(out.contains("&lt;script&gt;"), "{out}");
    }

    #[test]
    fn test_sprite_is_drawn_and_unknown_is_hidden() {
        let sprite = Sprite {
            name: "s".to_string(),
            rows: vec!["0F".to_string()],
            width: 2,
            height: 1,
            palette: Vec::new(),
            svg: None,
        };
        let out = expand_markup(&format!("{HEAD}a&lt;$s&gt;</text>"), &[sprite]);
        assert!(out.contains("<rect"), "{out}");
        let hidden = run("a&lt;$нет&gt;");
        assert!(!hidden.contains("$"), "{hidden}");
    }

    #[test]
    fn test_table_row_has_no_pipes() {
        let out = run("|= h1 |= h2 |");
        assert!(!out.contains("|="), "{out}");
        assert!(out.contains(">h1</text>"), "{out}");
    }
}
