//! Парсер ER диаграмм
//!
//! Парсит Entity-Relationship диаграммы PlantUML.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::er::{Attribute, Cardinality, Entity, ErDiagram, ErRelationship};

use crate::error::ParseError;

#[derive(Parser)]
#[grammar = "grammars/er.pest"]
struct ErParser;

/// Парсит ER диаграмму
pub fn parse_er(source: &str) -> crate::Result<ErDiagram> {
    let pairs = ErParser::parse(Rule::er_diagram, source)
        .map_err(|e| ParseError::GrammarError(format!("Ошибка парсинга ER: {}", e)))?;

    let mut diagram = ErDiagram::new();

    for pair in pairs {
        if pair.as_rule() == Rule::er_diagram {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::title_directive => {
                        if let Some(title) = inner.into_inner().next() {
                            diagram.metadata.title = Some(title.as_str().to_string());
                        }
                    }
                    Rule::entity_definition => {
                        let entity = parse_entity(inner)?;
                        diagram.add_entity(entity);
                    }
                    Rule::er_relationship => {
                        let rel = parse_relationship(inner)?;
                        diagram.add_relationship(rel);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(diagram)
}

/// Парсит определение сущности
fn parse_entity(pair: pest::iterators::Pair<Rule>) -> crate::Result<Entity> {
    let mut name = String::new();
    let mut entity = Entity::new("");

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::entity_name => {
                // Имя может быть в кавычках: `entity "User" as user`.
                // Кавычки — часть синтаксиса, в имени они не нужны.
                name = inner.as_str().trim_matches('"').to_string();
            }
            Rule::entity_alias => {
                // Используем alias
                entity.id.alias = Some(inner.as_str().to_string());
            }
            Rule::stereotype => {
                if let Some(content) = inner.into_inner().next() {
                    entity.stereotype = Some(plantuml_ast::common::Stereotype::new(
                        content.as_str().to_string(),
                    ));
                }
            }
            Rule::color_spec => {
                if let Some(color) = inner.into_inner().next() {
                    entity.background_color = Some(plantuml_ast::common::Color::from_hex(format!(
                        "#{}",
                        color.as_str()
                    )));
                }
            }
            Rule::entity_member => {
                let attr = parse_attribute(inner)?;
                entity.add_attribute(attr);
            }
            _ => {}
        }
    }

    entity.id.name = name;
    Ok(entity)
}

/// Парсит атрибут сущности
fn parse_attribute(pair: pest::iterators::Pair<Rule>) -> crate::Result<Attribute> {
    let mut attr = Attribute::new("");
    let mut is_required = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::required_marker => {
                is_required = true;
            }
            Rule::attribute_name => {
                attr.name = inner.as_str().to_string();
            }
            Rule::attribute_type => {
                attr.data_type = Some(inner.as_str().trim().to_string());
            }
            Rule::stereotype => {
                if let Some(content) = inner.into_inner().next() {
                    let st = content.as_str().to_uppercase();
                    attr.stereotype = Some(st.clone());
                    if st == "PK" {
                        attr.is_primary_key = true;
                        is_required = true;
                    } else if st == "FK" {
                        attr.is_foreign_key = true;
                    } else if st == "UK" {
                        attr.is_unique = true;
                    }
                }
            }
            _ => {}
        }
    }

    attr.is_required = is_required;
    Ok(attr)
}

/// Парсит связь между сущностями
fn parse_relationship(pair: pest::iterators::Pair<Rule>) -> crate::Result<ErRelationship> {
    let mut from = String::new();
    let mut to = String::new();
    let mut from_card = Cardinality::One;
    let mut to_card = Cardinality::Many;
    let mut label = None;
    let mut is_first_entity = true;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::entity_ref => {
                if is_first_entity {
                    from = inner.as_str().to_string();
                    is_first_entity = false;
                } else {
                    to = inner.as_str().to_string();
                }
            }
            Rule::left_cardinality => {
                from_card = parse_cardinality(inner.as_str());
            }
            Rule::right_cardinality => {
                to_card = parse_cardinality(inner.as_str());
            }
            Rule::relation_label => {
                label = Some(inner.as_str().trim().to_string());
            }
            _ => {}
        }
    }

    let mut rel = ErRelationship::new(from, to);
    rel.from_cardinality = from_card;
    rel.to_cardinality = to_card;
    rel.label = label;

    Ok(rel)
}

/// Парсит символ кардинальности
fn parse_cardinality(s: &str) -> Cardinality {
    match s {
        "||" | "|" => Cardinality::One,
        "|o" | "o|" => Cardinality::ZeroOrOne,
        "}|" | "|{" => Cardinality::OneOrMany,
        "}o" | "o{" => Cardinality::ZeroOrMany,
        "}" | "{" => Cardinality::Many,
        _ => Cardinality::One,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Сущность без тела — `entity ORDER` — самая обычная форма PlantUML.
    ///
    /// Регрессия: грамматика требовала `{`, поэтому такой ввод не разбирался,
    /// и ER-диаграмма не строилась вовсе.
    #[test]
    fn test_parse_entity_without_body() {
        let source = r#"@startuml
entity USER
entity ORDER
USER ||--o{ ORDER
@enduml"#;

        let diagram = parse_er(source).unwrap();
        assert_eq!(diagram.entities.len(), 2, "обе сущности должны разобраться");
        assert_eq!(diagram.entities[0].id.name, "USER");
        assert_eq!(diagram.entities[1].id.name, "ORDER");
        assert_eq!(diagram.relationships.len(), 1);
    }

    /// Имя сущности в кавычках с алиасом: `entity "User" as user`.
    #[test]
    fn test_parse_quoted_entity_name() {
        let source = r#"@startuml
entity "User" as user {
  * id : number
  --
  name : string
}
entity "Order" as order
user ||--o{ order
@enduml"#;

        let diagram = parse_er(source).unwrap();
        let user = diagram
            .entities
            .iter()
            .find(|e| e.id.alias.as_deref() == Some("user"))
            .expect("сущность user не найдена");

        // Кавычки — часть синтаксиса, в имени их быть не должно
        assert_eq!(user.id.name, "User", "кавычки не сняты с имени");
        assert_eq!(diagram.entities.len(), 2);
    }

    /// Смешанная форма: одна сущность с телом, другая без.
    #[test]
    fn test_parse_mixed_entity_forms() {
        let source = r#"@startuml
entity USER {
  * id : int
}
entity ORDER
USER ||--o{ ORDER : places
@enduml"#;

        let diagram = parse_er(source).unwrap();
        assert_eq!(diagram.entities.len(), 2);
        assert_eq!(diagram.entities[0].attributes.len(), 1);
        assert!(diagram.entities[1].attributes.is_empty());
    }

    #[test]
    fn test_parse_simple_er() {
        let source = r#"@startuml
entity User {
  * id : int <<PK>>
  --
  name : varchar
  email : varchar
}
@enduml"#;

        let diagram = parse_er(source).unwrap();
        assert_eq!(diagram.entities.len(), 1);

        let user = &diagram.entities[0];
        assert_eq!(user.id.name, "User");
        assert_eq!(user.attributes.len(), 3);
        assert!(user.attributes[0].is_primary_key);
    }

    #[test]
    fn test_parse_er_relationship() {
        let source = r#"@startuml
entity User {
  * id : int <<PK>>
}
entity Order {
  * id : int <<PK>>
}
User ||--o{ Order : places
@enduml"#;

        let diagram = parse_er(source).unwrap();
        assert_eq!(diagram.entities.len(), 2);
        assert_eq!(diagram.relationships.len(), 1);

        let rel = &diagram.relationships[0];
        assert_eq!(rel.from, "User");
        assert_eq!(rel.to, "Order");
        assert_eq!(rel.label, Some("places".to_string()));
    }

    #[test]
    fn test_parse_cardinality() {
        assert!(matches!(parse_cardinality("||"), Cardinality::One));
        assert!(matches!(parse_cardinality("|o"), Cardinality::ZeroOrOne));
        assert!(matches!(parse_cardinality("}|"), Cardinality::OneOrMany));
        assert!(matches!(parse_cardinality("}o"), Cardinality::ZeroOrMany));
    }
}
