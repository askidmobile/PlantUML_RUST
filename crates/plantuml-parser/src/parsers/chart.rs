//! Парсер `@startchart`: оси, ряды `bar`/`line`/`area`, легенда.
//!
//! Строки команд короткие и однотипные, поэтому pest не нужен. Неизвестные
//! команды (`scatter`, `annotation`, `<style>`) пропускаются: диаграмма
//! остаётся читаемой и без них.

use plantuml_ast::chart::{ChartDiagram, LegendPosition, Series, SeriesKind};
use plantuml_ast::common::DiagramMetadata;

use crate::error::ParseError;

/// Парсит диаграмму `@startchart`.
pub fn parse_chart(source: &str) -> crate::Result<ChartDiagram> {
    let mut chart = ChartDiagram {
        metadata: DiagramMetadata::default(),
        ..ChartDiagram::default()
    };
    let mut in_style = false;
    for (index, raw) in source.lines().enumerate() {
        let line = raw.trim();
        let lower = line.to_lowercase();
        if in_style {
            in_style = !lower.starts_with("</style>");
            continue;
        }
        if lower.starts_with("<style>") {
            in_style = !lower.contains("</style>");
            continue;
        }
        if line.is_empty() || line.starts_with('\'') || line.starts_with('@') {
            continue;
        }
        let (command, rest) = line
            .split_once(char::is_whitespace)
            .map_or((line, ""), |(c, r)| (c, r.trim()));
        let fail = |message: &str| ParseError::SyntaxError {
            line: index + 1,
            message: format!("{message} в «{line}»"),
        };
        match command.to_lowercase().as_str() {
            "h-axis" => {
                chart.h_axis.title = quoted(rest).map(|(t, _)| t);
                if let Some(list) = bracketed(rest) {
                    chart.h_axis.categories = split_list(list);
                }
            }
            "v-axis" => {
                chart.v_axis.title = quoted(rest).map(|(t, _)| t);
                chart.v_axis.range = axis_range(rest);
                chart.v_axis.spacing = word_after(rest, "spacing").and_then(|w| w.parse().ok());
            }
            "bar" | "line" | "area" => {
                let kind = match command.to_lowercase().as_str() {
                    "bar" => SeriesKind::Bar,
                    "line" => SeriesKind::Line,
                    _ => SeriesKind::Area,
                };
                let (name, after) = quoted(rest).unwrap_or((String::new(), rest));
                let Some(list) = bracketed(after) else {
                    return Err(fail("у ряда нет списка значений [..]"));
                };
                let mut values = Vec::new();
                for item in split_list(list) {
                    // Пара `(x,y)`: для категориальной оси важен только y
                    let number = item.trim_matches(|c| c == '(' || c == ')');
                    let number = number.rsplit(',').next().unwrap_or(number).trim();
                    values.push(number.parse::<f64>().map_err(|_| fail("не число"))?);
                }
                let tail = after.rsplit(']').next().unwrap_or("");
                chart.series.push(Series {
                    kind,
                    name: if name.is_empty() {
                        format!("{} {}", command, chart.series.len() + 1)
                    } else {
                        name
                    },
                    values,
                    color: tail
                        .split_whitespace()
                        .find(|w| w.starts_with('#'))
                        .map(str::to_string),
                    labels: tail
                        .split_whitespace()
                        .any(|w| w.eq_ignore_ascii_case("labels")),
                });
            }
            "legend" => {
                chart.legend = match rest.to_lowercase().as_str() {
                    "left" => Some(LegendPosition::Left),
                    "top" => Some(LegendPosition::Top),
                    "bottom" => Some(LegendPosition::Bottom),
                    _ => Some(LegendPosition::Right),
                };
            }
            "title" => chart.metadata.title = Some(rest.trim_start_matches(':').trim().to_string()),
            "stackmode" => chart.stacked = rest.eq_ignore_ascii_case("stacked"),
            // Молча пропущенная строка дала бы неполный график, который выглядит верным.
            _ => return Err(fail("неподдержанная команда графика")),
        }
    }
    if chart.series.is_empty() && chart.h_axis.categories.is_empty() {
        return Err(ParseError::EmptyDiagram);
    }
    Ok(chart)
}

/// Строка в двойных кавычках в начале текста: (содержимое, остаток).
fn quoted(text: &str) -> Option<(String, &str)> {
    let body = text.strip_prefix('"')?;
    let end = body.find('"')?;
    Some((body[..end].to_string(), body[end + 1..].trim_start()))
}

/// Содержимое первых квадратных скобок.
fn bracketed(text: &str) -> Option<&str> {
    let start = text.find('[')? + 1;
    let end = start + text[start..].find(']')?;
    Some(&text[start..end])
}

/// Элементы списка через запятую; запятая внутри `(x,y)` и кавычек не делит.
fn split_list(list: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let (mut depth, mut quote) = (0, false);
    for c in list.chars() {
        match c {
            '"' => quote = !quote,
            '(' if !quote => depth += 1,
            ')' if !quote => depth -= 1,
            ',' if depth == 0 && !quote => {
                items.push(std::mem::take(&mut current).trim().to_string());
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    items.push(current.trim().to_string());
    items.retain(|i| !i.is_empty());
    items
        .into_iter()
        .map(|i| i.trim_matches('"').to_string())
        .collect()
}

/// `min --> max` на оси.
fn axis_range(text: &str) -> Option<(f64, f64)> {
    let (left, right) = text.split_once("-->")?;
    let min = left.split_whitespace().last()?.parse().ok()?;
    let max = right.split_whitespace().next()?.parse().ok()?;
    Some((min, max))
}

/// Слово, стоящее сразу за ключевым.
fn word_after<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let mut words = text.split_whitespace();
    words.find(|w| w.eq_ignore_ascii_case(key))?;
    words.next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reference_case() {
        let c = parse_chart("@startchart\nh-axis [a1,b1]\nbar \"s1\" [1,2]\n@endchart").unwrap();
        assert_eq!(c.h_axis.categories, vec!["a1", "b1"]);
        assert_eq!(c.series[0].name, "s1");
        assert_eq!(c.series[0].kind, SeriesKind::Bar);
        assert_eq!(c.series[0].values, vec![1.0, 2.0]);
        assert_eq!(c.legend, None);
    }

    #[test]
    fn axes_series_options_and_legend() {
        let c = parse_chart(
            "@startchart\nh-axis \"Квартал\" [Q1, Q2]\nv-axis \"Выручка\" 0 --> 80 spacing 20\n\
             line \"План\" [10, 20.5] #e74c3c labels\narea \"Факт\" [(0,5), (1,7)]\n\
             legend bottom\nstackMode stacked\n@endchart",
        )
        .unwrap();
        assert_eq!(c.h_axis.title.as_deref(), Some("Квартал"));
        assert_eq!(c.v_axis.range, Some((0.0, 80.0)));
        assert_eq!(c.v_axis.spacing, Some(20.0));
        assert_eq!(c.series[0].color.as_deref(), Some("#e74c3c"));
        assert!(c.series[0].labels);
        assert_eq!(c.series[1].values, vec![5.0, 7.0]);
        assert_eq!(c.legend, Some(LegendPosition::Bottom));
        assert!(c.stacked);
    }

    #[test]
    fn broken_series_is_an_error_and_empty_is_empty() {
        assert!(parse_chart("@startchart\nbar \"x\" [1,abc]\n@endchart").is_err());
        assert!(parse_chart("@startchart\nbar \"x\"\n@endchart").is_err());
        assert!(matches!(
            parse_chart("@startchart\n@endchart"),
            Err(ParseError::EmptyDiagram)
        ));
    }
}
