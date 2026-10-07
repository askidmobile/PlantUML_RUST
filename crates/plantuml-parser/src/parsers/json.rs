//! Парсер JSON диаграмм
//!
//! Парсит JSON данные и преобразует их в AST для визуализации.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::json::{JsonDiagram, JsonNode};

use crate::error::ParseError;

#[derive(Parser)]
#[grammar = "grammars/json.pest"]
struct JsonParser;

/// Парсит JSON диаграмму
pub fn parse_json(source: &str) -> crate::Result<JsonDiagram> {
    let pairs = JsonParser::parse(Rule::json_diagram, source)
        .map_err(|e| ParseError::GrammarError(format!("Ошибка парсинга JSON: {}", e)))?;

    let mut diagram = JsonDiagram::new();

    for pair in pairs {
        if pair.as_rule() == Rule::json_diagram {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::title_directive => {
                        if let Some(title_pair) = inner.into_inner().next() {
                            diagram.metadata.title = Some(title_pair.as_str().to_string());
                        }
                    }
                    Rule::highlight_directive => {
                        if let Some(path_pair) = inner.into_inner().next() {
                            diagram.highlights.push(path_pair.as_str().to_string());
                        }
                    }
                    Rule::json_content => {
                        if let Some(value_pair) = inner.into_inner().next() {
                            diagram.root = Some(parse_json_value(None, value_pair)?);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(diagram)
}

/// Парсит JSON значение
fn parse_json_value(
    key: Option<String>,
    pair: pest::iterators::Pair<Rule>,
) -> crate::Result<JsonNode> {
    match pair.as_rule() {
        Rule::json_value => {
            // Пара-обёртка `json_value` может оказаться пустой, если
            // грамматика совпала на пустом значении. Прежний `unwrap()`
            // в этом случае ронял процесс целиком.
            let inner = pair.into_inner().next().ok_or_else(|| {
                ParseError::GrammarError("json_value без вложенного значения".to_string())
            })?;
            parse_json_value(key, inner)
        }
        Rule::json_object => {
            let mut children = Vec::new();
            for member in pair.into_inner() {
                if member.as_rule() == Rule::json_member {
                    let node = parse_json_member(member)?;
                    children.push(node);
                }
            }
            Ok(JsonNode::object(key, children))
        }
        Rule::json_array => {
            let mut items = Vec::new();
            for item in pair.into_inner() {
                if item.as_rule() == Rule::json_value {
                    let node = parse_json_value(None, item)?;
                    items.push(node);
                }
            }
            Ok(JsonNode::array(key, items))
        }
        Rule::json_string => {
            let s = parse_json_string(pair.as_str());
            Ok(JsonNode::string(key, s))
        }
        Rule::json_number => {
            // Литерал сохраняем: PlantUML печатает число так, как оно
            // записано (`150.00` остаётся `150.00`).
            let text = pair.as_str().to_string();
            let n: f64 = text.parse().unwrap_or(0.0);
            Ok(JsonNode::number_literal(key, n, text))
        }
        Rule::json_boolean => {
            let b = pair.as_str() == "true";
            Ok(JsonNode::boolean(key, b))
        }
        Rule::json_null => Ok(JsonNode::null(key)),
        _ => Err(ParseError::GrammarError(format!(
            "Неожиданное правило: {:?}",
            pair.as_rule()
        ))),
    }
}

/// Парсит член объекта (key: value)
fn parse_json_member(pair: pest::iterators::Pair<Rule>) -> crate::Result<JsonNode> {
    let mut inner = pair.into_inner();

    let key_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Ожидался ключ в JSON объекте".to_string()))?;
    let key = parse_json_string(key_pair.as_str());

    let value_pair = inner
        .next()
        .ok_or_else(|| ParseError::GrammarError("Ожидалось значение в JSON объекте".to_string()))?;

    parse_json_value(Some(key), value_pair)
}

/// Парсит JSON строку, убирая кавычки и обрабатывая escape-последовательности
fn parse_json_string(s: &str) -> String {
    // Убираем кавычки
    let s = s.trim_matches('"');

    // Обрабатываем escape-последовательности
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('b') => result.push('\x08'),
                Some('f') => result.push('\x0c'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('u') => {
                    // Unicode escape \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::json::JsonValue;

    #[test]
    fn test_parse_simple_json() {
        let source = r#"@startjson
{
  "name": "John",
  "age": 30
}
@endjson"#;

        let diagram = parse_json(source).unwrap();
        assert!(diagram.root.is_some());

        let root = diagram.root.unwrap();
        if let JsonValue::Object(children) = &root.value {
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].key, Some("name".to_string()));
            assert_eq!(children[1].key, Some("age".to_string()));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_nested_json() {
        let source = r#"@startjson
{
  "person": {
    "name": "Alice",
    "address": {
      "city": "NYC"
    }
  }
}
@endjson"#;

        let diagram = parse_json(source).unwrap();
        assert_eq!(diagram.max_depth(), 4);
    }

    #[test]
    fn test_parse_json_array() {
        let source = r#"@startjson
{
  "numbers": [1, 2, 3],
  "names": ["Alice", "Bob"]
}
@endjson"#;

        let diagram = parse_json(source).unwrap();
        assert!(diagram.root.is_some());
    }

    #[test]
    fn test_parse_json_with_title() {
        let source = r#"@startjson
title User Data
{
  "id": 1
}
@endjson"#;

        let diagram = parse_json(source).unwrap();
        assert_eq!(diagram.metadata.title, Some("User Data".to_string()));
    }

    #[test]
    fn test_parse_json_primitives() {
        let source = r#"@startjson
{
  "string": "hello",
  "number": 42.5,
  "bool_true": true,
  "bool_false": false,
  "nothing": null
}
@endjson"#;

        let diagram = parse_json(source).unwrap();
        let root = diagram.root.unwrap();

        if let JsonValue::Object(children) = &root.value {
            assert_eq!(children.len(), 5);
        } else {
            panic!("Expected object");
        }
    }
}
