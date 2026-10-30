//! Парсер Use Case Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML use case diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::common::{Direction, Note, NotePosition, Stereotype};
use plantuml_ast::usecase::{
    UseCase, UseCaseActor, UseCaseDiagram, UseCasePackage, UseCaseRelationType, UseCaseRelationship,
};

use crate::error::syntax_error_from_pest;
use crate::Result;

#[derive(Parser)]
#[grammar = "grammars/usecase.pest"]
pub struct UseCaseParser;

/// Парсит use case diagram из исходного кода
pub fn parse_usecase(source: &str) -> Result<UseCaseDiagram> {
    let pairs = UseCaseParser::parse(Rule::diagram, source)
        .map_err(|e| syntax_error_from_pest(&e, source))?;

    let mut diagram = UseCaseDiagram::new();
    let mut mentioned = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::diagram {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::body {
                    parse_body(inner, &mut diagram, &mut mentioned);
                }
            }
        }
    }

    declare_implicit_endpoints(&mut diagram, mentioned);
    Ok(diagram)
}

/// Создаёт элементы, которые упомянуты только в связях.
///
/// PlantUML рисует `User --> (Start)` без объявлений: имя в `(..)` — вариант
/// использования, остальное — актёр. Без этого связь и обе подписи молча
/// пропадали, потому что раскладка ищет концы связи среди элементов.
fn declare_implicit_endpoints(diagram: &mut UseCaseDiagram, mentioned: Vec<(String, bool)>) {
    let mut known: std::collections::HashSet<String> = std::collections::HashSet::new();
    for a in &diagram.actors {
        known.insert(a.name.clone());
        known.extend(a.alias.clone());
    }
    for uc in diagram
        .use_cases
        .iter()
        .chain(diagram.packages.iter().flat_map(|p| p.use_cases.iter()))
    {
        known.insert(uc.name.clone());
        known.extend(uc.alias.clone());
    }
    known.extend(diagram.packages.iter().map(|p| p.name.clone()));
    for (name, is_use_case) in mentioned {
        if known.insert(name.clone()) {
            if is_use_case {
                diagram.use_cases.push(UseCase::new(name));
            } else {
                diagram.actors.push(UseCaseActor::new(name));
            }
        }
    }
}

/// Парсит тело диаграммы
fn parse_body(
    pair: pest::iterators::Pair<Rule>,
    diagram: &mut UseCaseDiagram,
    mentioned: &mut Vec<(String, bool)>,
) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::actor_def => {
                if let Some(actor) = parse_actor_def(inner) {
                    diagram.actors.push(actor);
                }
            }
            Rule::usecase_def => {
                if let Some(uc) = parse_usecase_def(inner) {
                    diagram.use_cases.push(uc);
                }
            }
            Rule::package_def | Rule::rectangle_def => {
                if let Some(pkg) = parse_package_def(inner, diagram, mentioned) {
                    diagram.packages.push(pkg);
                }
            }
            Rule::relationship => {
                if let Some((rel, ends)) = parse_relationship(inner) {
                    mentioned.extend(ends);
                    diagram.relationships.push(rel);
                }
            }
            Rule::note_stmt => {
                if let Some(note) = parse_note(inner) {
                    diagram.notes.push(note);
                }
            }
            Rule::left_to_right => {
                diagram.direction = Direction::LeftToRight;
            }
            Rule::top_to_bottom => {
                diagram.direction = Direction::TopToBottom;
            }
            _ => {}
        }
    }
}

/// Парсит определение актёра
fn parse_actor_def(pair: pest::iterators::Pair<Rule>) -> Option<UseCaseActor> {
    let mut name = String::new();
    let mut alias: Option<String> = None;
    let mut stereotype: Option<Stereotype> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::actor_name => {
                name = extract_name(inner);
            }
            Rule::colon_actor => {
                // :Actor: синтаксис
                for colon_inner in inner.into_inner() {
                    match colon_inner.as_rule() {
                        Rule::actor_inner_name => {
                            name = colon_inner.as_str().trim().to_string();
                        }
                        Rule::alias_part => {
                            alias = extract_alias(colon_inner);
                        }
                        Rule::stereotype_part => {
                            stereotype = extract_stereotype(colon_inner);
                        }
                        _ => {}
                    }
                }
            }
            Rule::alias_part => {
                alias = extract_alias(inner);
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

    Some(UseCaseActor {
        name,
        alias,
        stereotype,
        color: None,
    })
}

/// Парсит определение варианта использования
fn parse_usecase_def(pair: pest::iterators::Pair<Rule>) -> Option<UseCase> {
    let mut name = String::new();
    let mut alias: Option<String> = None;
    let mut stereotype: Option<Stereotype> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::usecase_name => {
                name = extract_name(inner);
            }
            Rule::paren_usecase => {
                // (Use Case) синтаксис
                for paren_inner in inner.into_inner() {
                    match paren_inner.as_rule() {
                        Rule::usecase_inner_name => {
                            name = paren_inner.as_str().trim().to_string();
                        }
                        Rule::alias_part => {
                            alias = extract_alias(paren_inner);
                        }
                        Rule::stereotype_part => {
                            stereotype = extract_stereotype(paren_inner);
                        }
                        _ => {}
                    }
                }
            }
            Rule::alias_part => {
                alias = extract_alias(inner);
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

    Some(UseCase {
        name,
        alias,
        stereotype,
        color: None,
    })
}

/// Парсит пакет/прямоугольник
fn parse_package_def(
    pair: pest::iterators::Pair<Rule>,
    diagram: &mut UseCaseDiagram,
    mentioned: &mut Vec<(String, bool)>,
) -> Option<UseCasePackage> {
    let mut name = String::new();
    let mut use_cases = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::package_name => {
                name = extract_name(inner);
            }
            Rule::body => {
                // Парсим вложенные элементы
                //
                // Акторы, связи и заметки тела принадлежат всей диаграмме,
                // вложенные пакеты вливаются в этот: раскладка рисует одну
                // рамку, а варианты ищет по именам среди всех.
                let mut sub_diagram = UseCaseDiagram::new();
                parse_body(inner, &mut sub_diagram, mentioned);
                use_cases = sub_diagram.use_cases;
                for nested in sub_diagram.packages {
                    use_cases.extend(nested.use_cases);
                }
                diagram.actors.extend(sub_diagram.actors);
                diagram.relationships.extend(sub_diagram.relationships);
                diagram.notes.extend(sub_diagram.notes);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(UseCasePackage {
        name,
        use_cases,
        color: None,
    })
}

/// Парсит связь
fn parse_relationship(
    pair: pest::iterators::Pair<Rule>,
) -> Option<(UseCaseRelationship, Vec<(String, bool)>)> {
    let mut from = String::new();
    let mut to = String::new();
    let mut relation_type = UseCaseRelationType::Association;
    let mut label: Option<String> = None;
    let mut is_dashed = false;
    let mut ends = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::relationship_from => {
                from = extract_relationship_endpoint(inner.clone());
                ends.push((from.clone(), is_paren_endpoint(&inner)));
            }
            Rule::relationship_to => {
                to = extract_relationship_endpoint(inner.clone());
                ends.push((to.clone(), is_paren_endpoint(&inner)));
            }
            Rule::arrow => {
                // Определяем тип связи по стрелке
                for arrow_inner in inner.into_inner() {
                    match arrow_inner.as_rule() {
                        Rule::arrow_generalization => {
                            relation_type = UseCaseRelationType::Generalization;
                        }
                        Rule::arrow_dashed => {
                            is_dashed = true;
                        }
                        _ => {}
                    }
                }
            }
            Rule::relationship_label => {
                label = extract_label(inner);
            }
            _ => {}
        }
    }

    // Определяем тип по метке если стрелка пунктирная
    if is_dashed {
        if let Some(ref lbl) = label {
            let lbl_lower = lbl.to_lowercase();
            if lbl_lower.contains("include") {
                relation_type = UseCaseRelationType::Include;
            } else if lbl_lower.contains("extend") {
                relation_type = UseCaseRelationType::Extend;
            }
        }
    }

    if from.is_empty() || to.is_empty() {
        return None;
    }

    Some((
        UseCaseRelationship {
            from,
            to,
            relation_type,
            label,
        },
        ends,
    ))
}

/// Конец связи записан как `(Вариант)`.
fn is_paren_endpoint(pair: &pest::iterators::Pair<Rule>) -> bool {
    pair.clone()
        .into_inner()
        .any(|p| p.as_rule() == Rule::paren_usecase_ref)
}

/// Извлекает endpoint связи
fn extract_relationship_endpoint(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::colon_actor_ref => {
                for colon_inner in inner.into_inner() {
                    if colon_inner.as_rule() == Rule::actor_inner_name {
                        return colon_inner.as_str().trim().to_string();
                    }
                }
            }
            Rule::paren_usecase_ref => {
                for paren_inner in inner.into_inner() {
                    if paren_inner.as_rule() == Rule::usecase_inner_name {
                        return paren_inner.as_str().trim().to_string();
                    }
                }
            }
            Rule::simple_identifier => {
                return inner.as_str().to_string();
            }
            _ => {}
        }
    }
    String::new()
}

/// Парсит заметку
fn parse_note(pair: pest::iterators::Pair<Rule>) -> Option<Note> {
    let mut position = NotePosition::Right;
    let mut text = String::new();
    let mut anchors = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::note_on_element | Rule::note_multiline => {
                for n in inner.into_inner() {
                    match n.as_rule() {
                        Rule::note_position => {
                            position = parse_note_position(n.as_str());
                        }
                        Rule::note_target => {
                            anchors.push(extract_note_target(n));
                        }
                        Rule::note_text | Rule::note_body => {
                            text = n.as_str().trim().to_string();
                        }
                        _ => {}
                    }
                }
            }
            Rule::note_floating => {
                for n in inner.into_inner() {
                    if n.as_rule() == Rule::quoted_string {
                        text = extract_quoted_string(n);
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
            // `actor :Имя:` и `usecase (Имя)`: имя лежит внутри скобок.
            Rule::actor_inner_name | Rule::usecase_inner_name => {
                return inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    text
}

fn extract_alias(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::simple_identifier {
            return Some(inner.as_str().to_string());
        }
    }
    None
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

fn extract_note_target(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::colon_actor_ref => {
                for colon_inner in inner.into_inner() {
                    if colon_inner.as_rule() == Rule::actor_inner_name {
                        return colon_inner.as_str().trim().to_string();
                    }
                }
            }
            Rule::paren_usecase_ref => {
                for paren_inner in inner.into_inner() {
                    if paren_inner.as_rule() == Rule::usecase_inner_name {
                        return paren_inner.as_str().trim().to_string();
                    }
                }
            }
            Rule::simple_identifier => return inner.as_str().to_string(),
            _ => {}
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Элементы, упомянутые только в связях, создаются: `(..)` — вариант, остальное — актёр.
    #[test]
    fn test_implicit_endpoints_and_arrows() {
        let source = "@startuml\nUser -> (Start)\n:Admin: ---> (Use)\n(U1) .> (U2) : <<include>>\nUser <|-- Admin\n@enduml";
        let d = parse_usecase(source).expect("разбирается");
        assert_eq!(d.relationships.len(), 4);
        let actors: Vec<_> = d.actors.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(actors, ["User", "Admin"]);
        assert_eq!(d.use_cases.len(), 4);
        assert_eq!(
            d.relationships[2].relation_type,
            UseCaseRelationType::Include
        );
        assert_eq!(
            d.relationships[3].relation_type,
            UseCaseRelationType::Generalization
        );
    }

    /// Формы объявлений: `(Имя) as (UC)`, `actor :Имя: as A`, хвостовой `/`.
    #[test]
    fn test_declaration_forms() {
        let source = "@startuml\n(Another) as (UC2)\nusecase (Last) as UC4\nactor :Админ: as A\n(Business)/\nactor/ BA\n@enduml";
        let d = parse_usecase(source).expect("разбирается");
        assert_eq!(d.use_cases[0].alias.as_deref(), Some("UC2"));
        assert_eq!(d.use_cases[1].name, "Last");
        assert_eq!(d.actors[0].name, "Админ");
        assert_eq!(d.actors[0].alias.as_deref(), Some("A"));
        assert_eq!(d.use_cases[2].name, "Business");
        assert_eq!(d.actors[1].name, "BA");
    }

    /// Связи и актёры внутри `rectangle` не теряются.
    #[test]
    fn test_body_of_rectangle_keeps_relationships() {
        let source = "@startuml\nrectangle S {\n usecase \"A\" as UC1\n usecase \"B\" as UC2\n UC1 .> UC2 : <<include>>\n}\n@enduml";
        let d = parse_usecase(source).expect("разбирается");
        assert_eq!(d.relationships.len(), 1);
        assert_eq!(d.packages[0].use_cases.len(), 2);
        assert!(d.use_cases.is_empty() && d.actors.is_empty());
    }

    /// `rectangle` без тела — обычный элемент, а не контейнер.
    ///
    /// Регрессия: правило требовало `{`, поэтому `rectangle R` не
    /// разбирался вовсе, хотя в PlantUML это допустимая форма.
    #[test]
    fn test_rectangle_without_body() {
        let source = "@startuml\nactor A\nrectangle R\nA --> R\n@enduml";
        let diagram = parse_usecase(source).expect("диаграмма должна разбираться");
        // Форма без тела разбирается и не теряется: элемент присутствует
        // как контейнер без содержимого.
        assert_eq!(diagram.packages.len(), 1, "rectangle без тела потерян");
        assert_eq!(diagram.packages[0].name, "R");
        assert!(diagram.packages[0].use_cases.is_empty());

        // Форма с телом продолжает работать
        let source = "@startuml\nrectangle \"Система\" {\n  usecase Вход\n}\n@enduml";
        let diagram = parse_usecase(source).expect("диаграмма должна разбираться");
        assert_eq!(diagram.packages.len(), 1);
        assert_eq!(diagram.packages[0].use_cases.len(), 1);
    }

    #[test]
    fn test_parse_simple_actors() {
        let source = r#"
@startuml
actor User
actor Admin
@enduml
"#;

        let diagram = parse_usecase(source).unwrap();
        assert_eq!(diagram.actors.len(), 2);
        assert_eq!(diagram.actors[0].name, "User");
        assert_eq!(diagram.actors[1].name, "Admin");
    }

    #[test]
    fn test_parse_colon_actor() {
        let source = r#"
@startuml
:User:
:System Admin:
@enduml
"#;

        let diagram = parse_usecase(source).unwrap();
        assert_eq!(diagram.actors.len(), 2);
        assert_eq!(diagram.actors[0].name, "User");
        assert_eq!(diagram.actors[1].name, "System Admin");
    }

    #[test]
    fn test_parse_use_cases() {
        let source = r#"
@startuml
usecase Login
usecase "View Dashboard" as UC2
(Register)
(Create Order)
@enduml
"#;

        let diagram = parse_usecase(source).unwrap();
        assert_eq!(diagram.use_cases.len(), 4);
        assert_eq!(diagram.use_cases[0].name, "Login");
        assert_eq!(diagram.use_cases[1].name, "View Dashboard");
        assert_eq!(diagram.use_cases[1].alias, Some("UC2".to_string()));
    }

    #[test]
    fn test_parse_relationships() {
        let source = r#"
@startuml
actor User
usecase Login
usecase Authenticate

User --> Login
Login ..> Authenticate : <<include>>
@enduml
"#;

        let diagram = parse_usecase(source).unwrap();
        assert_eq!(diagram.relationships.len(), 2);

        assert_eq!(diagram.relationships[0].from, "User");
        assert_eq!(diagram.relationships[0].to, "Login");
        assert_eq!(
            diagram.relationships[0].relation_type,
            UseCaseRelationType::Association
        );
    }

    #[test]
    fn test_parse_package() {
        let source = r#"
@startuml
rectangle "E-Commerce System" {
    usecase Login
    usecase "Browse Products"
    usecase Checkout
}
@enduml
"#;

        let diagram = parse_usecase(source).unwrap();
        assert_eq!(diagram.packages.len(), 1);
        assert_eq!(diagram.packages[0].name, "E-Commerce System");
        assert_eq!(diagram.packages[0].use_cases.len(), 3);
    }

    #[test]
    fn test_parse_with_paren_syntax() {
        let source = r#"
@startuml
:User: --> (Login)
(Login) --> (Dashboard)
@enduml
"#;

        let diagram = parse_usecase(source).unwrap();
        assert_eq!(diagram.relationships.len(), 2);
        assert_eq!(diagram.relationships[0].from, "User");
        assert_eq!(diagram.relationships[0].to, "Login");
    }
}
