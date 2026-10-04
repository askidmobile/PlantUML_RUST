//! Парсер Activity Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML activity diagrams (новый синтаксис).

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::activity::{
    Action, ActionStyle, ActivityDiagram, ActivityElement, Condition, ElseIfBranch, Fork, JoinType,
    RepeatLoop, Swimlane, WhileLoop,
};
use plantuml_ast::common::{Color, Note, NotePosition};

use crate::{ParseError, Result};

#[derive(Parser)]
#[grammar = "grammars/activity.pest"]
pub struct ActivityParser;

/// Парсит activity diagram из исходного кода
pub fn parse_activity(source: &str) -> Result<ActivityDiagram> {
    let pairs =
        ActivityParser::parse(Rule::diagram, source).map_err(|e| ParseError::SyntaxError {
            line: e.line().to_string().parse().unwrap_or(0),
            message: e.to_string(),
        })?;

    let mut diagram = ActivityDiagram::new();

    for pair in pairs {
        if pair.as_rule() == Rule::diagram {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::body {
                    diagram.elements = parse_body(inner);
                }
            }
        }
    }

    Ok(diagram)
}

/// Парсит тело диаграммы (последовательность элементов)
fn parse_body(pair: pest::iterators::Pair<Rule>) -> Vec<ActivityElement> {
    let mut elements = Vec::new();

    for inner in pair.into_inner() {
        if let Some(element) = parse_statement(inner) {
            elements.push(element);
        }
    }

    elements
}

/// Парсит отдельный statement
fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Option<ActivityElement> {
    match pair.as_rule() {
        Rule::start_node => Some(ActivityElement::Start),
        Rule::stop_node => Some(ActivityElement::Stop),
        Rule::end_node => Some(ActivityElement::End),
        Rule::detach_stmt => Some(ActivityElement::Detach),
        Rule::kill_stmt => Some(ActivityElement::Kill),
        Rule::action | Rule::action_simple | Rule::action_colored | Rule::action_multiline => {
            parse_action(pair).map(ActivityElement::Action)
        }
        Rule::if_stmt => parse_if_stmt(pair).map(ActivityElement::Condition),
        Rule::while_stmt => parse_while_stmt(pair).map(ActivityElement::While),
        Rule::repeat_stmt => parse_repeat_stmt(pair).map(ActivityElement::Repeat),
        Rule::fork_stmt => parse_fork_stmt(pair).map(ActivityElement::Fork),
        Rule::swimlane_stmt => parse_swimlane(pair).map(ActivityElement::SwimlaneChange),
        Rule::connector_stmt => parse_connector(pair).map(ActivityElement::Connector),
        Rule::note_stmt | Rule::note_inline | Rule::note_multiline => {
            parse_note(pair).map(ActivityElement::Note)
        }
        Rule::break_stmt => Some(ActivityElement::Detach), // break как detach
        _ => None,
    }
}

/// Парсит действие (action)
fn parse_action(pair: pest::iterators::Pair<Rule>) -> Option<Action> {
    let mut label = String::new();
    let mut background_color: Option<Color> = None;
    let mut style = ActionStyle::Normal;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::action_text | Rule::action_multiline_text => {
                label = inner.as_str().trim().to_string();
            }
            Rule::color => {
                background_color = parse_color(inner);
            }
            Rule::action_end => {
                style = match inner.as_str() {
                    ">" => ActionStyle::SendSignal,
                    "<" => ActionStyle::ReceiveSignal,
                    "}" => ActionStyle::Condition,
                    _ => ActionStyle::Normal,
                };
            }
            Rule::action_simple | Rule::action_colored | Rule::action_multiline => {
                // Рекурсивно обрабатываем вложенные action
                return parse_action(inner);
            }
            _ => {}
        }
    }

    if label.is_empty() {
        return None;
    }

    Some(Action {
        label,
        background_color,
        style,
        arrow_label: None,
    })
}

/// Парсит условие if/elseif/else
fn parse_if_stmt(pair: pest::iterators::Pair<Rule>) -> Option<Condition> {
    let mut condition_text = String::new();
    let mut then_branch = Vec::new();
    let mut then_label: Option<String> = None;
    let mut elseif_branches = Vec::new();
    let mut else_branch: Option<Vec<ActivityElement>> = None;
    let mut else_label: Option<String> = None;

    // Состояние парсинга: 0 = then, 1+ = elseif/else
    let mut in_else = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::condition_text => {
                if condition_text.is_empty() {
                    condition_text = inner.as_str().trim().to_string();
                }
            }
            Rule::branch_label => {
                let label_str = extract_label(inner);
                if !in_else && then_label.is_none() {
                    then_label = Some(label_str);
                } else if in_else && else_label.is_none() {
                    else_label = Some(label_str);
                }
            }
            Rule::body => {
                let elements = parse_body(inner);
                if !in_else && elseif_branches.is_empty() && else_branch.is_none() {
                    then_branch = elements;
                } else if in_else && else_branch.is_none() {
                    else_branch = Some(elements);
                }
            }
            Rule::elseif_clause => {
                if let Some(branch) = parse_elseif_clause(inner) {
                    elseif_branches.push(branch);
                }
            }
            Rule::else_clause => {
                in_else = true;
                for clause_inner in inner.into_inner() {
                    match clause_inner.as_rule() {
                        Rule::branch_label => {
                            else_label = Some(extract_label(clause_inner));
                        }
                        Rule::body => {
                            else_branch = Some(parse_body(clause_inner));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Some(Condition {
        condition: condition_text,
        then_branch,
        then_label,
        elseif_branches,
        else_branch,
        else_label,
    })
}

/// Парсит elseif clause
fn parse_elseif_clause(pair: pest::iterators::Pair<Rule>) -> Option<ElseIfBranch> {
    let mut condition = String::new();
    let mut elements = Vec::new();
    let mut label: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::condition_text => {
                condition = inner.as_str().trim().to_string();
            }
            Rule::branch_label => {
                label = Some(extract_label(inner));
            }
            Rule::body => {
                elements = parse_body(inner);
            }
            _ => {}
        }
    }

    Some(ElseIfBranch {
        condition,
        elements,
        label,
    })
}

/// Парсит цикл while
fn parse_while_stmt(pair: pest::iterators::Pair<Rule>) -> Option<WhileLoop> {
    let mut condition = String::new();
    let mut body = Vec::new();
    let mut end_label: Option<String> = None;
    let mut backward_label: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::condition_text => {
                condition = inner.as_str().trim().to_string();
            }
            Rule::body => {
                body = parse_body(inner);
            }
            Rule::is_clause => {
                // is (label) после while
                for is_inner in inner.into_inner() {
                    if is_inner.as_rule() == Rule::label_text {
                        end_label = Some(is_inner.as_str().trim().to_string());
                    }
                }
            }
            Rule::backward_label => {
                backward_label = Some(extract_label(inner));
            }
            _ => {}
        }
    }

    Some(WhileLoop {
        condition,
        body,
        end_label,
        backward_label,
    })
}

/// Парсит цикл repeat
fn parse_repeat_stmt(pair: pest::iterators::Pair<Rule>) -> Option<RepeatLoop> {
    let mut body = Vec::new();
    let mut condition = String::new();
    let mut backward_label: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::body => {
                body = parse_body(inner);
            }
            Rule::condition_text => {
                condition = inner.as_str().trim().to_string();
            }
            Rule::backward_label | Rule::backward_stmt => {
                // Обрабатываем backward
                for back_inner in inner.into_inner() {
                    if back_inner.as_rule() == Rule::action_text {
                        backward_label = Some(back_inner.as_str().trim().to_string());
                    }
                }
            }
            _ => {}
        }
    }

    Some(RepeatLoop {
        body,
        condition,
        backward_label,
    })
}

/// Парсит fork/join
fn parse_fork_stmt(pair: pest::iterators::Pair<Rule>) -> Option<Fork> {
    let mut branches: Vec<Vec<ActivityElement>> = Vec::new();
    let mut current_branch: Vec<ActivityElement> = Vec::new();
    let mut join_type = JoinType::And;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::body => {
                current_branch = parse_body(inner);
            }
            Rule::fork_again_clause => {
                // Сохраняем предыдущую ветку и начинаем новую
                if !current_branch.is_empty() {
                    branches.push(std::mem::take(&mut current_branch));
                }
                for fork_inner in inner.into_inner() {
                    if fork_inner.as_rule() == Rule::body {
                        current_branch = parse_body(fork_inner);
                    }
                }
            }
            Rule::end_fork => {
                // Проверяем тип слияния
                let text = inner.as_str().to_lowercase();
                if text.contains("merge") {
                    join_type = JoinType::Or;
                }
            }
            _ => {}
        }
    }

    // Добавляем последнюю ветку
    if !current_branch.is_empty() {
        branches.push(current_branch);
    }

    if branches.is_empty() {
        return None;
    }

    Some(Fork {
        branches,
        join_type,
    })
}

/// Парсит swimlane
/// Поддерживает синтаксис: |name| или |#color|name|
fn parse_swimlane(pair: pest::iterators::Pair<Rule>) -> Option<Swimlane> {
    let mut name = String::new();
    let mut color: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::swimlane_name => {
                name = inner.as_str().trim().to_string();
            }
            Rule::swimlane_color => {
                // swimlane_color содержит color внутри
                for color_inner in inner.into_inner() {
                    if color_inner.as_rule() == Rule::color {
                        color = parse_color(color_inner);
                    }
                }
            }
            Rule::color => {
                color = parse_color(inner);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Swimlane { name, color })
}

/// Парсит коннектор
fn parse_connector(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::connector_name {
            return Some(inner.as_str().trim().to_string());
        }
    }
    None
}

/// Парсит заметку
fn parse_note(pair: pest::iterators::Pair<Rule>) -> Option<Note> {
    let mut position = NotePosition::Right;
    let mut text = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::note_position => {
                position = match inner.as_str() {
                    "left" => NotePosition::Left,
                    "right" => NotePosition::Right,
                    _ => NotePosition::Right,
                };
            }
            Rule::note_text | Rule::note_body => {
                text = inner.as_str().trim().to_string();
            }
            Rule::note_inline | Rule::note_multiline => {
                // Рекурсивно обрабатываем
                return parse_note(inner);
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
        anchors: vec![],
        background_color: None,
    })
}

/// Извлекает текст метки из branch_label или backward_label
fn extract_label(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::label_text {
            return inner.as_str().trim().to_string();
        }
    }
    String::new()
}

/// Парсит цвет
/// Поддерживает: #RRGGBB, #RGB, #NamedColor
fn parse_color(pair: pest::iterators::Pair<Rule>) -> Option<Color> {
    // Сначала попробуем распарсить весь текст как цвет
    let text = pair.as_str();

    // Если начинается с #, используем Color::parse который понимает и hex и named
    if text.starts_with('#') {
        return Some(Color::parse(text));
    }

    // Попробуем найти hex_color или named_color внутри
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::hex_color => {
                let hex = inner.as_str();
                return Some(Color::from_hex(hex));
            }
            Rule::named_color => {
                let name = inner.as_str();
                return Some(Color::named(name));
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_activity() {
        let source = r#"
@startuml
start
:Hello world;
:This is a test;
stop
@enduml
"#;

        let diagram = parse_activity(source).unwrap();
        assert!(!diagram.elements.is_empty());

        // Проверяем структуру
        assert!(matches!(diagram.elements[0], ActivityElement::Start));
        assert!(matches!(diagram.elements[1], ActivityElement::Action(_)));
        assert!(matches!(diagram.elements[2], ActivityElement::Action(_)));
        assert!(matches!(diagram.elements[3], ActivityElement::Stop));
    }

    #[test]
    fn test_parse_if_else() {
        let source = r#"
@startuml
start
if (condition?) then (yes)
  :action 1;
else (no)
  :action 2;
endif
stop
@enduml
"#;

        let diagram = parse_activity(source).unwrap();

        // Находим условие
        let condition = diagram
            .elements
            .iter()
            .find(|e| matches!(e, ActivityElement::Condition(_)));
        assert!(condition.is_some());

        if let Some(ActivityElement::Condition(cond)) = condition {
            assert_eq!(cond.condition, "condition?");
            assert!(cond.then_label.is_some());
            assert!(cond.else_branch.is_some());
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let source = r#"
@startuml
start
while (condition?) is (yes)
  :action;
endwhile (no)
stop
@enduml
"#;

        let diagram = parse_activity(source).unwrap();

        let while_loop = diagram
            .elements
            .iter()
            .find(|e| matches!(e, ActivityElement::While(_)));
        assert!(while_loop.is_some());

        if let Some(ActivityElement::While(w)) = while_loop {
            assert_eq!(w.condition, "condition?");
            assert!(!w.body.is_empty());
        }
    }

    #[test]
    fn test_parse_fork() {
        let source = r#"
@startuml
start
fork
  :action 1;
fork again
  :action 2;
end fork
stop
@enduml
"#;

        let diagram = parse_activity(source).unwrap();

        let fork = diagram
            .elements
            .iter()
            .find(|e| matches!(e, ActivityElement::Fork(_)));
        assert!(fork.is_some());

        if let Some(ActivityElement::Fork(f)) = fork {
            assert_eq!(f.branches.len(), 2);
            assert_eq!(f.join_type, JoinType::And);
        }
    }

    #[test]
    fn test_parse_swimlane() {
        let source = r#"
@startuml
|Swimlane1|
start
:action;
|Swimlane2|
:another action;
stop
@enduml
"#;

        let diagram = parse_activity(source).unwrap();

        let swimlanes: Vec<_> = diagram
            .elements
            .iter()
            .filter_map(|e| {
                if let ActivityElement::SwimlaneChange(s) = e {
                    Some(s)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(swimlanes.len(), 2);
        assert_eq!(swimlanes[0].name, "Swimlane1");
        assert_eq!(swimlanes[1].name, "Swimlane2");
    }

    #[test]
    fn test_parse_swimlane_with_color() {
        let source = r#"
@startuml
|Swimlane1|
start
:action;
|#AntiqueWhite|Swimlane2|
:another action;
|#FF0000|Swimlane3|
:third action;
stop
@enduml
"#;

        let diagram = parse_activity(source).unwrap();

        let swimlanes: Vec<_> = diagram
            .elements
            .iter()
            .filter_map(|e| {
                if let ActivityElement::SwimlaneChange(s) = e {
                    Some(s)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(swimlanes.len(), 3);
        assert_eq!(swimlanes[0].name, "Swimlane1");
        assert!(swimlanes[0].color.is_none());

        assert_eq!(swimlanes[1].name, "Swimlane2");
        assert!(swimlanes[1].color.is_some(), "Expected color for Swimlane2");
        // Проверяем, что цвет правильно распарсен как именованный
        assert_eq!(
            swimlanes[1].color.as_ref().unwrap().to_css(),
            "AntiqueWhite"
        );

        assert_eq!(swimlanes[2].name, "Swimlane3");
        assert!(swimlanes[2].color.is_some(), "Expected color for Swimlane3");
        // Проверяем, что hex цвет правильно распарсен
        assert_eq!(swimlanes[2].color.as_ref().unwrap().to_css(), "#FF0000");
    }

    #[test]
    fn test_parse_repeat() {
        let source = r#"
@startuml
start
repeat
  :action;
repeat while (condition?)
stop
@enduml
"#;

        let diagram = parse_activity(source).unwrap();

        let repeat = diagram
            .elements
            .iter()
            .find(|e| matches!(e, ActivityElement::Repeat(_)));
        assert!(repeat.is_some());
    }
}
