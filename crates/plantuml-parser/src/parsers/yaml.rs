//! Парсер YAML диаграмм
//!
//! Парсит YAML данные и преобразует их в AST.
//! Использует те же структуры что и JSON для совместимости layout.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::json::JsonNode;
use plantuml_ast::yaml::YamlDiagram;

use crate::error::ParseError;

#[derive(Parser)]
#[grammar = "grammars/yaml.pest"]
struct YamlParser;

/// Строка YAML без отступа: (отступ, текст).
type YamlLine = (usize, String);

/// Парсит YAML диаграмму
///
/// Структуру задаёт отступ, а не скобки, поэтому блоки разбираются вручную:
/// контекстно-свободная грамматика pest не различает вложенность по пробелам
/// и сплющивала `key:\n  child: v` в соседние ключи. Pest остаётся для
/// однострочных значений (`{a: 1}`, `[1, 2]`).
pub fn parse_yaml(source: &str) -> crate::Result<YamlDiagram> {
    let mut diagram = YamlDiagram::new();
    let mut lines: Vec<YamlLine> = Vec::new();

    for raw in source.lines() {
        let text = raw.trim_end();
        let body = text.trim_start();
        if body.is_empty()
            || body.starts_with("@start")
            || body.starts_with("@end")
            || body == "---"
        {
            continue;
        }
        if let Some(rest) = body.strip_prefix("title ") {
            diagram.metadata.title = Some(rest.trim().to_string());
        } else if let Some(rest) = body.strip_prefix("#highlight ") {
            diagram.highlights.push(rest.trim().to_string());
        } else if !body.starts_with('#') {
            let indent = text
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .map(|c| if c == '\t' { 2 } else { 1 })
                .sum();
            lines.push((indent, body.to_string()));
        }
    }

    if !lines.is_empty() {
        let mut pos = 0;
        let indent = lines[0].0;
        let root = parse_block(&mut lines, &mut pos, indent, None)?;
        if pos < lines.len() {
            return Err(ParseError::GrammarError(format!(
                "failed to parse YAML: unexpected indentation in \"{}\"",
                lines[pos].1
            )));
        }
        diagram.root = Some(root);
    }

    Ok(diagram)
}

/// Блок на отступе `indent`: список, словарь или одиночный скаляр.
fn parse_block(
    lines: &mut [YamlLine],
    pos: &mut usize,
    indent: usize,
    key: Option<String>,
) -> crate::Result<JsonNode> {
    let first = lines[*pos].1.clone();
    if is_sequence_item(&first) {
        let mut items = Vec::new();
        while *pos < lines.len() && lines[*pos].0 == indent && is_sequence_item(&lines[*pos].1) {
            let line = lines[*pos].1.clone();
            let rest = line[1..].trim_start();
            if rest.is_empty() {
                *pos += 1;
                match lines.get(*pos) {
                    Some((deeper, _)) if *deeper > indent => {
                        let deeper = *deeper;
                        items.push(parse_block(lines, pos, deeper, None)?);
                    }
                    _ => items.push(JsonNode::null(None)),
                }
            } else {
                // `- key: v` или `- - a`: остаток строки сам становится началом
                // блока на своей колонке, следующие строки на ней же его продолжают.
                let column = indent + (line.len() - rest.len());
                lines[*pos] = (column, rest.to_string());
                if is_sequence_item(rest) || split_key(rest).is_some() {
                    items.push(parse_block(lines, pos, column, None)?);
                } else {
                    *pos += 1;
                    items.push(parse_inline(None, rest, lines, pos, indent)?);
                }
            }
        }
        return Ok(JsonNode::array(key, items));
    }

    if split_key(&first).is_none() {
        *pos += 1;
        return parse_inline(key, &first, lines, pos, indent);
    }

    let mut children = Vec::new();
    while *pos < lines.len() && lines[*pos].0 == indent {
        let line = lines[*pos].1.clone();
        let Some((entry_key, rest)) = split_key(&line) else {
            break;
        };
        *pos += 1;
        let entry_key = unquote_key(entry_key);
        if rest.is_empty() {
            match lines.get(*pos) {
                // Список допускается на том же отступе, что и ключ.
                Some((next, text))
                    if *next > indent || (*next == indent && is_sequence_item(text)) =>
                {
                    let next = *next;
                    children.push(parse_block(lines, pos, next, Some(entry_key))?);
                }
                _ => children.push(JsonNode::null(Some(entry_key))),
            }
        } else {
            children.push(parse_inline(Some(entry_key), rest, lines, pos, indent)?);
        }
    }
    Ok(JsonNode::object(key, children))
}

fn is_sequence_item(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

/// Делит `key: value` по первому двоеточию вне кавычек.
fn split_key(text: &str) -> Option<(&str, &str)> {
    let mut quote: Option<char> = None;
    for (i, c) in text.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (None, '"' | '\'') if i == 0 => quote = Some(c),
            (None, ':') => {
                let after = &text[i + 1..];
                if after.is_empty() || after.starts_with(' ') || after.starts_with('\t') {
                    return Some((text[..i].trim(), after.trim()));
                }
            }
            _ => {}
        }
    }
    None
}

fn unquote_key(key: &str) -> String {
    if key.len() >= 2 && (key.starts_with('"') || key.starts_with('\'')) {
        parse_yaml_string(key)
    } else {
        key.to_string()
    }
}

/// Значение в одной строке: поток `{..}`/`[..]`, блочный скаляр или обычный скаляр.
fn parse_inline(
    key: Option<String>,
    text: &str,
    lines: &mut [YamlLine],
    pos: &mut usize,
    parent_indent: usize,
) -> crate::Result<JsonNode> {
    // Блочный скаляр `|` / `>`: все более глубокие строки — одна строка текста.
    if matches!(text.chars().next(), Some('|' | '>')) {
        let joiner = if text.starts_with('|') { "\n" } else { " " };
        let mut parts = Vec::new();
        while let Some((deeper, line)) = lines.get(*pos) {
            if *deeper <= parent_indent {
                break;
            }
            parts.push(line.clone());
            *pos += 1;
        }
        return Ok(JsonNode::string(key, parts.join(joiner)));
    }

    if text.starts_with('{') || text.starts_with('[') {
        let parsed = YamlParser::parse(Rule::yaml_inline_value, text)
            .ok()
            .and_then(|mut p| p.next())
            .filter(|p| p.as_str().len() == text.len());
        if let Some(pair) = parsed {
            return parse_yaml_value(key, pair);
        }
    }

    Ok(scalar_node(key, text))
}

/// Скаляр: строка в кавычках, число, логическое значение, null или текст.
fn scalar_node(key: Option<String>, text: &str) -> JsonNode {
    let text = text.trim();
    if text.starts_with('"') || text.starts_with('\'') {
        return JsonNode::string(key, parse_yaml_string(text));
    }
    // Комментарий в конце строки отделён пробелом.
    let text = text.split(" #").next().unwrap_or(text).trim();
    match text {
        "null" | "~" => JsonNode::null(key),
        "true" | "yes" | "on" => JsonNode::boolean(key, true),
        "false" | "no" | "off" => JsonNode::boolean(key, false),
        _ => match text.parse::<f64>() {
            Ok(n)
                if text
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, '-' | '.')) =>
            {
                JsonNode::number(key, n)
            }
            _ => JsonNode::string(key, text.to_string()),
        },
    }
}

/// Парсит YAML значение
fn parse_yaml_value(
    key: Option<String>,
    pair: pest::iterators::Pair<Rule>,
) -> crate::Result<JsonNode> {
    match pair.as_rule() {
        Rule::yaml_value => {
            // Пара-обёртка может оказаться пустой — прежний `unwrap()`
            // в этом случае ронял процесс целиком.
            let inner = pair.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("yaml_value without a nested value".to_string())
            })?;
            parse_yaml_value(key, inner)
        }
        Rule::yaml_mapping => {
            let mut children = Vec::new();
            for entry in pair.into_inner() {
                if entry.as_rule() == Rule::yaml_mapping_entry {
                    let node = parse_yaml_mapping_entry(entry)?;
                    children.push(node);
                }
            }
            Ok(JsonNode::object(key, children))
        }
        Rule::yaml_sequence => {
            let mut items = Vec::new();
            for item in pair.into_inner() {
                if item.as_rule() == Rule::yaml_sequence_item {
                    if let Some(value) = item.into_inner().next() {
                        let node = parse_yaml_value(None, value)?;
                        items.push(node);
                    }
                }
            }
            Ok(JsonNode::array(key, items))
        }
        Rule::yaml_inline_mapping => {
            let mut children = Vec::new();
            for entry in pair.into_inner() {
                if entry.as_rule() == Rule::yaml_inline_entry {
                    let node = parse_yaml_inline_entry(entry)?;
                    children.push(node);
                }
            }
            Ok(JsonNode::object(key, children))
        }
        Rule::yaml_inline_sequence => {
            let mut items = Vec::new();
            for item in pair.into_inner() {
                let node = parse_yaml_value(None, item)?;
                items.push(node);
            }
            Ok(JsonNode::array(key, items))
        }
        Rule::yaml_scalar => {
            let inner = pair.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("yaml_scalar without a nested value".to_string())
            })?;
            parse_yaml_scalar(key, inner)
        }
        Rule::yaml_inline_value => {
            let inner = pair.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("yaml_inline_value without a nested value".to_string())
            })?;
            parse_yaml_value(key, inner)
        }
        _ => Err(ParseError::GrammarError(format!(
            "unexpected YAML rule: {:?}",
            pair.as_rule()
        ))),
    }
}

/// Парсит entry mapping (key: value)
fn parse_yaml_mapping_entry(pair: pest::iterators::Pair<Rule>) -> crate::Result<JsonNode> {
    let mut inner = pair.into_inner();

    let key_pair = inner.next().ok_or_else(|| {
        ParseError::GrammarError("expected a key in the YAML mapping".to_string())
    })?;
    let key = key_pair.as_str().to_string();

    // Значение может отсутствовать (будет null)
    if let Some(value_pair) = inner.next() {
        parse_yaml_value(Some(key), value_pair)
    } else {
        Ok(JsonNode::null(Some(key)))
    }
}

/// Парсит inline entry (key: value в {})
fn parse_yaml_inline_entry(pair: pest::iterators::Pair<Rule>) -> crate::Result<JsonNode> {
    let mut inner = pair.into_inner();

    let key_pair = inner.next().ok_or_else(|| {
        ParseError::GrammarError("expected a key in the YAML inline entry".to_string())
    })?;
    let key = key_pair.as_str().to_string();

    let value_pair = inner.next().ok_or_else(|| {
        ParseError::GrammarError("expected a value in the YAML inline entry".to_string())
    })?;

    parse_yaml_value(Some(key), value_pair)
}

/// Парсит YAML скаляр
fn parse_yaml_scalar(
    key: Option<String>,
    pair: pest::iterators::Pair<Rule>,
) -> crate::Result<JsonNode> {
    match pair.as_rule() {
        Rule::yaml_string => {
            let s = parse_yaml_string(pair.as_str());
            Ok(JsonNode::string(key, s))
        }
        Rule::yaml_unquoted_string => {
            let s = pair.as_str().trim().to_string();
            Ok(JsonNode::string(key, s))
        }
        Rule::yaml_number => {
            let n: f64 = pair.as_str().parse().unwrap_or(0.0);
            Ok(JsonNode::number(key, n))
        }
        Rule::yaml_boolean => {
            let s = pair.as_str().to_lowercase();
            let b = matches!(s.as_str(), "true" | "yes" | "on");
            Ok(JsonNode::boolean(key, b))
        }
        Rule::yaml_null => Ok(JsonNode::null(key)),
        _ => Err(ParseError::GrammarError(format!(
            "unexpected YAML scalar: {:?}",
            pair.as_rule()
        ))),
    }
}

/// Парсит YAML строку, убирая кавычки
fn parse_yaml_string(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') {
        // Двойные кавычки - обрабатываем escape
        let inner = &s[1..s.len() - 1];
        let mut result = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }
        result
    } else if s.starts_with('\'') && s.ends_with('\'') {
        // Одинарные кавычки - только '' -> '
        let inner = &s[1..s.len() - 1];
        inner.replace("''", "'")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::json::JsonValue;

    #[test]
    fn test_parse_simple_yaml() {
        let source = r#"@startyaml
name: John
age: 30
@endyaml"#;

        let diagram = parse_yaml(source).unwrap();
        assert!(diagram.root.is_some());

        let root = diagram.root.unwrap();
        if let JsonValue::Object(children) = &root.value {
            assert_eq!(children.len(), 2);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_yaml_nested_and_lists() {
        let source = "@startyaml\nk1: v1\nk2:\n  - a1\n  - a2\nobj:\n  child: 1\n  deep:\n    - x: 1\n      y: 2\n    - - z\nsame:\n- s\n@endyaml";
        let diagram = parse_yaml(source).expect("разбирается");
        let root = diagram.root.expect("есть корень");
        let JsonValue::Object(top) = &root.value else {
            panic!("ожидался словарь")
        };
        assert_eq!(top.len(), 4);
        assert!(matches!(&top[1].value, JsonValue::Array(a) if a.len() == 2));
        let JsonValue::Object(obj) = &top[2].value else {
            panic!("ожидался вложенный словарь")
        };
        assert_eq!(obj.len(), 2);
        let JsonValue::Array(deep) = &obj[1].value else {
            panic!("ожидался список")
        };
        assert!(matches!(&deep[0].value, JsonValue::Object(o) if o.len() == 2));
        assert!(matches!(&deep[1].value, JsonValue::Array(_)));
        assert!(matches!(&top[3].value, JsonValue::Array(a) if a.len() == 1));
    }

    #[test]
    fn test_parse_yaml_inline() {
        let source = r#"@startyaml
person: {name: John, age: 30}
@endyaml"#;

        let diagram = parse_yaml(source).unwrap();
        assert!(diagram.root.is_some());
    }

    #[test]
    fn test_parse_yaml_with_title() {
        let source = r#"@startyaml
title Configuration
host: localhost
@endyaml"#;

        let diagram = parse_yaml(source).unwrap();
        assert_eq!(diagram.metadata.title, Some("Configuration".to_string()));
    }
}
