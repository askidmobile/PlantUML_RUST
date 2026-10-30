//! Разворачивает разметку подписей в готовом SVG.
//!
//! Текст с разметкой приходит сюда из любого типа диаграмм, поэтому это единая
//! точка: подпись из `<text>` разбирается и становится цепочкой элементов с
//! явными координатами. Так же выводит подписи и сам PlantUML.

use plantuml_layout::creole::{self, Span};
use plantuml_layout::text::TextMeasurer;

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    Some(&rest[..rest.find('"')?])
}

fn unescape(text: &str) -> String {
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
fn structural(line: &str) -> String {
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
        return format!("{}‣ {body}", "  ".repeat(hashes - 1));
    }
    line.to_string()
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
fn emit_line(out: &mut String, tag: &str, line: &str, x: f64, y: f64) {
    let base_size: f64 = attr(tag, "font-size")
        .and_then(|v| v.parse().ok())
        .unwrap_or(13.0);
    let base_bold = attr(tag, "font-weight").is_some_and(|w| w == "700" || w == "bold");
    let base_fill = attr(tag, "fill").unwrap_or("#000");
    let family = attr(tag, "font-family");
    let anchor = attr(tag, "text-anchor").unwrap_or("start");
    let class = attr(tag, "font-style");

    let measurer = TextMeasurer::default();
    let spans = creole::parse(&structural(line));
    let total: f64 = spans
        .iter()
        .map(|s| span_width(&measurer, s, base_size, base_bold))
        .sum();
    let mut cursor = match anchor {
        "middle" => x - total / 2.0,
        "end" => x - total,
        _ => x,
    };

    for span in &spans {
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

/// Разметка, из-за которой элемент надо разбирать.
fn needs_expansion(content: &str) -> bool {
    let text = unescape(content);
    if text.contains("\\n") {
        return true;
    }
    text.lines()
        .any(|line| creole::plain(line) != line || structural(line) != line)
}

/// Строки элемента: `<tspan x y>` либо текст с `\n`.
fn expand(out: &mut String, tag: &str, content: &str) {
    let x: f64 = attr(tag, "x").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let y: f64 = attr(tag, "y").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let size: f64 = attr(tag, "font-size")
        .and_then(|v| v.parse().ok())
        .unwrap_or(13.0);
    let line_height = size * 1.1641;

    if content.contains("<tspan") {
        let mut rest = content;
        let mut index = 0.0;
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
                .unwrap_or(y + index * line_height);
            emit_line(out, tag, &unescape(&tail[gt + 1..close]), span_x, span_y);
            rest = &tail[close + "</tspan>".len()..];
            index += 1.0;
        }
        return;
    }

    let text = unescape(content).replace("\\n", "\n");
    for (index, line) in text.lines().enumerate() {
        emit_line(out, tag, line, x, y + index as f64 * line_height);
    }
}

/// Разворачивает разметку всех подписей документа.
pub fn expand_markup(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        out.push_str(&rest[..open]);
        let tail = &rest[open..];
        let (Some(gt), Some(close)) = (tail.find('>'), tail.find("</text>")) else {
            out.push_str(tail);
            return out;
        };
        let tag = &tail[..gt];
        let content = &tail[gt + 1..close];
        if needs_expansion(content) || content.contains("<tspan") && has_tspan_markup(content) {
            expand(&mut out, tag, content);
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
        expand_markup(&format!("{HEAD}{content}</text>"))
    }

    #[test]
    fn test_plain_text_is_untouched() {
        let svg = format!("{HEAD}просто текст</text>");
        assert_eq!(expand_markup(&svg), svg);
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
    fn test_table_row_has_no_pipes() {
        let out = run("|= h1 |= h2 |");
        assert!(!out.contains("|="), "{out}");
        assert!(out.contains(">h1</text>"), "{out}");
    }
}
