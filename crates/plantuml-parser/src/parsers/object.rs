//! Парсер Object Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML object diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::common::{Note, NotePosition, Stereotype};
use plantuml_ast::object::{Object, ObjectDiagram, ObjectField, ObjectLink, ObjectLinkType};

use crate::{ParseError, Result};

#[derive(Parser)]
#[grammar = "grammars/object.pest"]
pub struct ObjectParser;

/// Парсит object diagram из исходного кода
pub fn parse_object(source: &str) -> Result<ObjectDiagram> {
    let pairs =
        ObjectParser::parse(Rule::diagram, source).map_err(|e| ParseError::SyntaxError {
            line: e.line().to_string().parse().unwrap_or(0),
            message: e.to_string(),
        })?;

    let mut diagram = ObjectDiagram::new();

    for pair in pairs {
        if pair.as_rule() == Rule::diagram {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::body {
                    parse_body(inner, &mut diagram);
                }
            }
        }
    }

    Ok(diagram)
}

/// Парсит тело диаграммы
fn parse_body(pair: pest::iterators::Pair<Rule>, diagram: &mut ObjectDiagram) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::object_def => {
                if let Some(obj) = parse_object_def(inner) {
                    diagram.objects.push(obj);
                }
            }
            Rule::map_def => {
                if let Some(obj) = parse_map_def(inner) {
                    diagram.objects.push(obj);
                }
            }
            Rule::link => {
                if let Some(link) = parse_link(inner) {
                    diagram.links.push(link);
                }
            }
            Rule::note_stmt => {
                if let Some(note) = parse_note(inner) {
                    diagram.notes.push(note);
                }
            }
            _ => {}
        }
    }
}

/// Парсит определение объекта
fn parse_object_def(pair: pest::iterators::Pair<Rule>) -> Option<Object> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::object_with_body => return parse_object_with_body(inner),
            Rule::object_simple => return parse_object_simple(inner),
            _ => {}
        }
    }
    None
}

/// Парсит объект с телом
fn parse_object_with_body(pair: pest::iterators::Pair<Rule>) -> Option<Object> {
    let mut name = String::new();
    let mut class_name: Option<String> = None;
    let mut stereotype: Option<Stereotype> = None;
    let mut fields = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::object_name => {
                name = extract_name(inner);
            }
            Rule::class_part => {
                class_name = extract_class_name(inner);
            }
            Rule::stereotype_part => {
                stereotype = extract_stereotype(inner);
            }
            Rule::object_body => {
                fields = parse_object_body(inner);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Object {
        name,
        class_name,
        fields,
        stereotype,
        background_color: None,
    })
}

/// Парсит простой объект (без тела)
fn parse_object_simple(pair: pest::iterators::Pair<Rule>) -> Option<Object> {
    let mut name = String::new();
    let mut class_name: Option<String> = None;
    let mut stereotype: Option<Stereotype> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::object_name => {
                name = extract_name(inner);
            }
            Rule::class_part => {
                class_name = extract_class_name(inner);
            }
            Rule::stereotype_part => {
                stereotype = extract_stereotype(inner);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Object {
        name,
        class_name,
        fields: Vec::new(),
        stereotype,
        background_color: None,
    })
}

/// Парсит тело объекта (поля)
fn parse_object_body(pair: pest::iterators::Pair<Rule>) -> Vec<ObjectField> {
    let mut fields = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::field_def {
            if let Some(field) = parse_field_def(inner) {
                fields.push(field);
            }
        }
    }

    fields
}

/// Парсит определение поля
fn parse_field_def(pair: pest::iterators::Pair<Rule>) -> Option<ObjectField> {
    let mut name = String::new();
    let mut value = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::field_name => {
                name = inner.as_str().to_string();
            }
            Rule::field_value => {
                value = extract_field_value(inner);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(ObjectField::new(name, value))
}

/// Парсит map (ассоциативный массив) как объект
fn parse_map_def(pair: pest::iterators::Pair<Rule>) -> Option<Object> {
    let mut name = String::new();
    let mut fields = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::object_name => {
                name = extract_name(inner);
            }
            Rule::map_body => {
                fields = parse_map_body(inner);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Object {
        name,
        class_name: Some("Map".to_string()),
        fields,
        stereotype: None,
        background_color: None,
    })
}

/// Парсит тело map
fn parse_map_body(pair: pest::iterators::Pair<Rule>) -> Vec<ObjectField> {
    let mut fields = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::map_entry {
            if let Some(field) = parse_map_entry(inner) {
                fields.push(field);
            }
        }
    }

    fields
}

/// Парсит запись map (key => value)
fn parse_map_entry(pair: pest::iterators::Pair<Rule>) -> Option<ObjectField> {
    let mut key = String::new();
    let mut value = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::map_key => {
                key = extract_name(inner);
            }
            Rule::map_value => {
                value = extract_name(inner);
            }
            _ => {}
        }
    }

    if key.is_empty() {
        return None;
    }

    Some(ObjectField::new(key, value))
}

/// Парсит связь между объектами
fn parse_link(pair: pest::iterators::Pair<Rule>) -> Option<ObjectLink> {
    let mut from = String::new();
    let mut to = String::new();
    let mut label: Option<String> = None;
    let mut link_type = ObjectLinkType::Association;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::link_from => {
                from = extract_name(inner);
            }
            Rule::link_to => {
                to = extract_name(inner);
            }
            Rule::arrow => {
                link_type = parse_arrow_type(inner);
            }
            Rule::link_label => {
                label = extract_label(inner);
            }
            _ => {}
        }
    }

    if from.is_empty() || to.is_empty() {
        return None;
    }

    Some(ObjectLink {
        from,
        to,
        label,
        link_type,
    })
}

/// Парсит тип стрелки
fn parse_arrow_type(pair: pest::iterators::Pair<Rule>) -> ObjectLinkType {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::arrow_composition => return ObjectLinkType::Composition,
            Rule::arrow_aggregation => return ObjectLinkType::Aggregation,
            Rule::arrow_dependency => return ObjectLinkType::Dependency,
            Rule::arrow_association => return ObjectLinkType::Association,
            Rule::arrow_link => return ObjectLinkType::Link,
            _ => {}
        }
    }
    ObjectLinkType::Association
}

/// Парсит заметку
fn parse_note(pair: pest::iterators::Pair<Rule>) -> Option<Note> {
    let mut text = String::new();
    let mut position = NotePosition::Right;
    let mut anchors = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::note_on_object | Rule::note_floating | Rule::note_multiline => {
                for note_inner in inner.into_inner() {
                    match note_inner.as_rule() {
                        Rule::note_position => {
                            position = parse_note_position(note_inner.as_str());
                        }
                        Rule::note_target => {
                            anchors.push(extract_name(note_inner));
                        }
                        Rule::note_text | Rule::note_body => {
                            text = note_inner.as_str().trim().to_string();
                        }
                        Rule::quoted_string => {
                            text = extract_quoted_string(note_inner);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if text.is_empty() {
        return None;
    }

    Some(Note {
        text,
        position,
        anchors,
        background_color: None,
    })
}

// === Вспомогательные функции ===

fn parse_note_position(s: &str) -> NotePosition {
    match s.to_lowercase().as_str() {
        "left" => NotePosition::Left,
        "right" => NotePosition::Right,
        "top" => NotePosition::Top,
        "bottom" => NotePosition::Bottom,
        _ => NotePosition::Right,
    }
}

fn extract_name(pair: pest::iterators::Pair<Rule>) -> String {
    let text = pair.as_str().trim().to_string();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::quoted_string => {
                return extract_quoted_string(inner);
            }
            Rule::simple_identifier => {
                return inner.as_str().to_string();
            }
            _ => {}
        }
    }

    text
}

fn extract_class_name(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::class_name {
            return Some(inner.as_str().to_string());
        }
    }
    None
}

fn extract_field_value(pair: pest::iterators::Pair<Rule>) -> String {
    let text = pair.as_str().trim().to_string();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::quoted_string => {
                return extract_quoted_string(inner);
            }
            Rule::unquoted_value => {
                return inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }
    text
}

fn extract_stereotype(pair: pest::iterators::Pair<Rule>) -> Option<Stereotype> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::stereotype_name {
            return Some(Stereotype::new(inner.as_str().trim()));
        }
    }
    None
}

fn extract_label(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::label_text {
            let text = inner.as_str().trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

fn extract_quoted_string(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::inner_string {
            return inner.as_str().to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let source = r#"
@startuml
object user1
object user2
@enduml
"#;

        let diagram = parse_object(source).unwrap();
        assert_eq!(diagram.objects.len(), 2);
        assert_eq!(diagram.objects[0].name, "user1");
        assert_eq!(diagram.objects[1].name, "user2");
    }

    #[test]
    fn test_parse_object_with_fields() {
        let source = r#"
@startuml
object user1 {
    name = "John"
    age = 30
}
@enduml
"#;

        let diagram = parse_object(source).unwrap();
        assert_eq!(diagram.objects.len(), 1);
        assert_eq!(diagram.objects[0].name, "user1");
        assert_eq!(diagram.objects[0].fields.len(), 2);
        assert_eq!(diagram.objects[0].fields[0].name, "name");
        assert_eq!(diagram.objects[0].fields[0].value, "John");
        assert_eq!(diagram.objects[0].fields[1].name, "age");
        assert_eq!(diagram.objects[0].fields[1].value, "30");
    }

    #[test]
    fn test_parse_object_with_class() {
        let source = r#"
@startuml
object user1 : User {
    name = "John"
}
@enduml
"#;

        let diagram = parse_object(source).unwrap();
        assert_eq!(diagram.objects.len(), 1);
        assert_eq!(diagram.objects[0].name, "user1");
        assert_eq!(diagram.objects[0].class_name, Some("User".to_string()));
    }

    #[test]
    fn test_parse_object_links() {
        let source = r#"
@startuml
object user1
object user2
user1 --> user2 : friend
user1 -- user2
@enduml
"#;

        let diagram = parse_object(source).unwrap();
        assert_eq!(diagram.links.len(), 2);
        assert_eq!(diagram.links[0].from, "user1");
        assert_eq!(diagram.links[0].to, "user2");
        assert_eq!(diagram.links[0].label, Some("friend".to_string()));
        assert_eq!(diagram.links[0].link_type, ObjectLinkType::Association);
        assert_eq!(diagram.links[1].link_type, ObjectLinkType::Link);
    }

    #[test]
    fn test_parse_map() {
        let source = r#"
@startuml
map config {
    host => localhost
    port => 8080
}
@enduml
"#;

        let diagram = parse_object(source).unwrap();
        assert_eq!(diagram.objects.len(), 1);
        assert_eq!(diagram.objects[0].name, "config");
        assert_eq!(diagram.objects[0].class_name, Some("Map".to_string()));
        assert_eq!(diagram.objects[0].fields.len(), 2);
    }
}
