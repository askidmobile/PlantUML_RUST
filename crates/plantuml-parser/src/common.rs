//! Команды, общие для всех типов диаграмм.
//!
//! `title`, `header`, `footer`, `caption`, `legend`, `mainframe`, `scale`,
//! направление раскладки и блок `<style>` не зависят от типа схемы. Раньше
//! каждая грамматика описывала их по-своему или не описывала вовсе, и такие
//! схемы либо не разбирались, либо теряли подпись. Теперь они снимаются до
//! разбора, а грамматика видит только содержимое.
//!
//! Снятая строка заменяется пустой: номера строк в ошибках остаются верными.

use plantuml_ast::common::{DiagramMetadata, Direction, LegendAlign};

/// Диаграммы со своим языком: `title: x` в YAML — это обычный ключ.
fn has_own_language(source: &str) -> bool {
    let head = source.trim_start().to_lowercase();
    ["@startjson", "@startyaml", "@startsalt"]
        .iter()
        .any(|tag| head.starts_with(tag))
}

/// Текст после ключевого слова, если строка им начинается.
///
/// Как и PlantUML, принимает `title текст` и `title: текст`. Запись вида
/// `title -> B` — это сообщение от участника, а не заголовок.
fn after_keyword<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let head = line.get(..keyword.len())?;
    if !head.eq_ignore_ascii_case(keyword) {
        return None;
    }
    let rest = &line[keyword.len()..];
    if rest.is_empty() {
        return Some("");
    }
    if !rest.starts_with([' ', '\t', ':']) {
        return None;
    }
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim_start();
    let is_message = rest.starts_with("->")
        || rest.starts_with("-->")
        || rest.starts_with("<-")
        || rest.starts_with("..");
    (!is_message).then_some(rest)
}

/// Конец многострочного блока: `end title` или `endtitle`.
fn is_block_end(line: &str, keyword: &str) -> bool {
    let lower = line.to_lowercase();
    let lower = lower.trim();
    lower == format!("end{keyword}") || lower == format!("end {keyword}")
}

/// Блоки, внутри которых ключевые слова — это просто текст.
fn opens_text_block(lower: &str) -> Option<&'static str> {
    let is_note_head =
        (lower.starts_with("note ") || lower.starts_with("hnote ") || lower.starts_with("rnote "))
            && !lower.contains(':');
    if is_note_head {
        return Some(if lower.starts_with("hnote") {
            "hnote"
        } else if lower.starts_with("rnote") {
            "rnote"
        } else {
            "note"
        });
    }
    if lower.starts_with("ref ") && !lower.contains(':') {
        return Some("ref");
    }
    None
}

/// Разбирает `legend [left|right|center] [top|bottom]`.
fn legend_align(rest: &str) -> Option<LegendAlign> {
    let mut align = LegendAlign::Center;
    for word in rest.split_whitespace() {
        match word.to_lowercase().as_str() {
            "left" => align = LegendAlign::Left,
            "right" => align = LegendAlign::Right,
            "center" | "top" | "bottom" => {}
            _ => return None,
        }
    }
    Some(align)
}

/// Числовой `scale 2`. Формы вроде `scale 200 width` и шкала timing остаются
/// в тексте: у них другой смысл.
fn plain_scale(rest: &str) -> Option<f64> {
    rest.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v > 0.0)
}

/// Снимает общие команды. Возвращает текст для грамматики той же длины в строках.
pub fn extract(source: &str) -> (String, DiagramMetadata) {
    let mut common = DiagramMetadata::default();
    if has_own_language(source) {
        return (source.to_string(), common);
    }
    let mut sprites: Vec<String> = Vec::new();

    let lines: Vec<&str> = source.lines().collect();
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut depth = 0usize;
    let mut text_block: Option<&'static str> = None;
    let mut i = 0;

    while i < lines.len() {
        let raw = lines[i];
        let line = raw.trim();
        let lower = line.to_lowercase();

        if let Some(kind) = text_block {
            if is_block_end(line, kind) {
                text_block = None;
            }
            out.push(raw);
            i += 1;
            continue;
        }

        // Внутри фигурных скобок `title : String` — это поле класса.
        if depth == 0 {
            if lower.starts_with("<style>") || lower == "<style" {
                while i < lines.len() {
                    let closed = lines[i].to_lowercase().contains("</style>");
                    out.push("");
                    i += 1;
                    if closed {
                        break;
                    }
                }
                continue;
            }

            if let Some(consumed) = sprite_block_len(&lines[i..]) {
                sprites.push(lines[i..i + consumed].join("\n"));
                out.extend(std::iter::repeat_n("", consumed));
                i += consumed;
                continue;
            }

            if let Some(consumed) = take_command(&lines[i..], &mut common) {
                out.extend(std::iter::repeat_n("", consumed));
                i += consumed;
                continue;
            }

            if let Some(kind) = opens_text_block(&lower) {
                text_block = Some(kind);
            }
        }

        depth += line.matches('{').count();
        depth = depth.saturating_sub(line.matches('}').count());
        out.push(raw);
        i += 1;
    }

    let mut text = out.join("\n");
    if source.ends_with('\n') {
        text.push('\n');
    }
    common.sprites = sprites
        .iter()
        .filter_map(|raw| crate::parsers::class::parse_sprite_text(raw))
        .collect();
    (text, common)
}

/// Длина объявления спрайта в начале `lines`.
fn sprite_block_len(lines: &[&str]) -> Option<usize> {
    let first = lines.first()?.trim();
    if !first.starts_with("sprite ") {
        return None;
    }
    let closes_at = |marker: &str| {
        lines
            .iter()
            .position(|l| l.contains(marker))
            .map(|index| index + 1)
    };
    if first.contains('{') && !first.contains('}') {
        return closes_at("}");
    }
    if first.contains("<svg") && !first.contains("</svg>") {
        return closes_at("</svg>");
    }
    Some(1)
}

/// Пробует снять команду с начала `lines`; возвращает число снятых строк.
fn take_command(lines: &[&str], common: &mut DiagramMetadata) -> Option<usize> {
    let line = lines.first()?.trim();
    let lower = line.to_lowercase();

    match lower.as_str() {
        "left to right direction" => {
            common.direction = Some(Direction::LeftToRight);
            return Some(1);
        }
        "top to bottom direction" => {
            common.direction = Some(Direction::TopToBottom);
            return Some(1);
        }
        _ => {}
    }
    if lower.starts_with("set separator") || lower.starts_with("set namespaceseparator") {
        return Some(1);
    }
    if let Some(value) = after_keyword(line, "scale").and_then(plain_scale) {
        common.scale = Some(value);
        return Some(1);
    }

    if let Some(rest) = after_keyword(line, "legend") {
        if let Some(align) = legend_align(rest) {
            let (text, used) = block_body(lines, "legend")?;
            common.legend = Some(text);
            common.legend_align = align;
            return Some(used);
        }
    }
    for (keyword, slot) in [
        ("title", &mut common.title),
        ("header", &mut common.header),
        ("footer", &mut common.footer),
    ] {
        if let Some(rest) = after_keyword(line, keyword) {
            if rest.is_empty() {
                let (text, used) = block_body(lines, keyword)?;
                *slot = Some(text);
                return Some(used);
            }
            *slot = Some(rest.to_string());
            return Some(1);
        }
    }
    // Выравнивание: `left header текст`, `center footer текст`.
    for prefix in ["left ", "right ", "center "] {
        if let Some(tail) = lower.strip_prefix(prefix) {
            let offset = prefix.len();
            if tail.starts_with("header") || tail.starts_with("footer") {
                return take_command(&[&line[offset..]], common);
            }
        }
    }
    for (keyword, slot) in [
        ("caption", &mut common.caption),
        ("mainframe", &mut common.mainframe),
    ] {
        if let Some(rest) = after_keyword(line, keyword).filter(|r| !r.is_empty()) {
            *slot = Some(rest.to_string());
            return Some(1);
        }
    }
    None
}

/// Тело многострочной команды и число занятых строк вместе с заголовком и концом.
fn block_body(lines: &[&str], keyword: &str) -> Option<(String, usize)> {
    let end = lines.iter().position(|l| is_block_end(l, keyword))?;
    let body: Vec<&str> = lines[1..end].iter().map(|l| l.trim()).collect();
    Some((body.join("\n"), end + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(source: &str) -> (String, DiagramMetadata) {
        extract(source)
    }

    #[test]
    fn test_commands_are_removed_and_line_numbers_kept() {
        let source = "@startuml\ntitle T\nheader H\nA -> B\nfooter F\ncaption C\n@enduml\n";
        let (text, common) = run(source);
        assert_eq!(text.lines().count(), source.lines().count());
        assert_eq!(text.lines().nth(3), Some("A -> B"));
        assert_eq!(common.title.as_deref(), Some("T"));
        assert_eq!(common.header.as_deref(), Some("H"));
        assert_eq!(common.footer.as_deref(), Some("F"));
        assert_eq!(common.caption.as_deref(), Some("C"));
    }

    #[test]
    fn test_multiline_blocks() {
        let (text, common) =
            run("title\nстрока1\nстрока2\nend title\nlegend right\nл1\nendlegend\nA -> B\n");
        assert_eq!(common.title.as_deref(), Some("строка1\nстрока2"));
        assert_eq!(common.legend.as_deref(), Some("л1"));
        assert_eq!(common.legend_align, LegendAlign::Right);
        assert!(text.contains("A -> B"));
        assert!(!text.contains("строка1"));
    }

    /// Ключевые слова внутри тела класса и заметки — это текст.
    #[test]
    fn test_keywords_inside_blocks_stay() {
        let (text, common) =
            run("class A {\ntitle : String\n}\nnote left of A\ntitle заметки\nend note\n");
        assert!(common.title.is_none());
        assert!(text.contains("title : String"));
        assert!(text.contains("title заметки"));
    }

    #[test]
    fn test_participant_named_like_keyword() {
        let (text, common) = run("title -> B : x\n");
        assert!(common.title.is_none());
        assert!(text.contains("title -> B"));
    }

    #[test]
    fn test_yaml_key_is_not_a_title() {
        let (text, common) = run("@startyaml\ntitle: x\n@endyaml\n");
        assert!(common.title.is_none());
        assert!(text.contains("title: x"));
    }

    #[test]
    fn test_direction_and_scale() {
        let (text, common) = run("left to right direction\nscale 2\nscale 200 width\n");
        assert_eq!(common.direction, Some(Direction::LeftToRight));
        assert_eq!(common.scale, Some(2.0));
        assert!(text.contains("scale 200 width"));
    }

    #[test]
    fn test_sprite_blocks_are_collected() {
        let (text, common) = run("sprite $s1 [4x4/16] {\nFFFF\nF00F\nF00F\nFFFF\n}\nA -> B\n");
        assert_eq!(text.lines().count(), 7);
        assert!(text.contains("A -> B"));
        assert!(!text.contains("FFFF"));
        assert_eq!(common.sprites.len(), 1, "{:?}", common.sprites);
        assert_eq!(common.sprites[0].name, "s1");
        assert_eq!(common.sprites[0].rows.len(), 4);
    }

    #[test]
    fn test_unclosed_block_is_not_swallowed() {
        let (text, common) = run("title\nA -> B\n");
        assert!(common.title.is_none());
        assert!(text.contains("A -> B"));
    }
}
