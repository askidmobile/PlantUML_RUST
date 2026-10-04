//! Парсер State Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML state diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::common::{Note, NotePosition};
use plantuml_ast::state::{State, StateDiagram, StateType, Transition};

use crate::error::syntax_error_from_pest;
use crate::Result;

#[derive(Parser)]
#[grammar = "grammars/state.pest"]
pub struct StateParser;

/// Парсит state diagram из исходного кода
pub fn parse_state(source: &str) -> Result<StateDiagram> {
    let pairs = StateParser::parse(Rule::diagram, source)
        .map_err(|e| syntax_error_from_pest(&e, source))?;

    let mut diagram = StateDiagram::new();

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
fn parse_body(pair: pest::iterators::Pair<Rule>, diagram: &mut StateDiagram) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::state_def => {
                if let Some(state) = parse_state_def(inner) {
                    diagram.add_state(state);
                }
            }
            Rule::transition => {
                if let Some(trans) = parse_transition(inner) {
                    diagram.add_transition(trans);
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

/// Парсит определение состояния
fn parse_state_def(pair: pest::iterators::Pair<Rule>) -> Option<State> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::state_composite => return parse_state_composite(inner),
            Rule::state_simple => return parse_state_simple(inner),
            Rule::state_alias => return parse_state_alias(inner),
            Rule::state_choice => return parse_state_special(inner, StateType::Choice),
            Rule::state_fork_join => {
                let text = inner.as_str().to_lowercase();
                if text.contains("fork") {
                    return parse_state_special(inner, StateType::Fork);
                } else {
                    return parse_state_special(inner, StateType::Join);
                }
            }
            Rule::state_entry_exit => {
                let text = inner.as_str().to_lowercase();
                if text.contains("entry") {
                    return parse_state_special(inner, StateType::EntryPoint);
                } else {
                    return parse_state_special(inner, StateType::ExitPoint);
                }
            }
            _ => {}
        }
    }
    None
}

/// Парсит составное состояние с вложенными
fn parse_state_composite(pair: pest::iterators::Pair<Rule>) -> Option<State> {
    let mut name = String::new();
    let mut alias: Option<String> = None;
    let mut substates = Vec::new();
    let mut internal_transitions = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::state_name_part => {
                name = extract_name(inner);
            }
            Rule::state_alias_part => {
                alias = extract_alias(inner);
            }
            Rule::body => {
                // Парсим вложенное тело
                let mut sub_diagram = StateDiagram::new();
                parse_body(inner, &mut sub_diagram);
                substates = sub_diagram.states;
                internal_transitions = sub_diagram.transitions;
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(State {
        name,
        alias,
        description: None,
        stereotype: None,
        state_type: StateType::Composite,
        substates,
        internal_transitions,
        regions: Vec::new(),
        color: None,
        entry_action: None,
        exit_action: None,
        do_action: None,
    })
}

/// Парсит простое состояние
fn parse_state_simple(pair: pest::iterators::Pair<Rule>) -> Option<State> {
    let mut name = String::new();
    let mut alias: Option<String> = None;
    let mut description: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::state_name_part => {
                name = extract_name(inner);
            }
            Rule::state_alias_part => {
                alias = extract_alias(inner);
            }
            Rule::state_description_part => {
                description = extract_description(inner);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(State {
        name,
        alias,
        description,
        stereotype: None,
        state_type: StateType::Simple,
        substates: Vec::new(),
        internal_transitions: Vec::new(),
        regions: Vec::new(),
        color: None,
        entry_action: None,
        exit_action: None,
        do_action: None,
    })
}

/// Парсит алиас состояния (Name : description)
fn parse_state_alias(pair: pest::iterators::Pair<Rule>) -> Option<State> {
    let mut parts: Vec<&str> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::simple_identifier => {
                parts.push(inner.as_str());
            }
            Rule::state_description_text => {
                parts.push(inner.as_str());
            }
            _ => {}
        }
    }

    if parts.is_empty() {
        return None;
    }

    let name = parts[0].to_string();
    let description = parts.get(1).map(|s| s.trim().to_string());

    Some(State {
        name,
        alias: None,
        description,
        stereotype: None,
        state_type: StateType::Simple,
        substates: Vec::new(),
        internal_transitions: Vec::new(),
        regions: Vec::new(),
        color: None,
        entry_action: None,
        exit_action: None,
        do_action: None,
    })
}

/// Парсит специальное состояние (choice, fork, join, etc.)
fn parse_state_special(pair: pest::iterators::Pair<Rule>, state_type: StateType) -> Option<State> {
    let mut name = String::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::state_name_part {
            name = extract_name(inner);
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(State {
        name,
        alias: None,
        description: None,
        stereotype: None,
        state_type,
        substates: Vec::new(),
        internal_transitions: Vec::new(),
        regions: Vec::new(),
        color: None,
        entry_action: None,
        exit_action: None,
        do_action: None,
    })
}

/// Парсит переход
fn parse_transition(pair: pest::iterators::Pair<Rule>) -> Option<Transition> {
    let mut from = String::new();
    let mut to = String::new();
    let mut event: Option<String> = None;
    let mut guard: Option<String> = None;
    let mut action: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::transition_from => {
                from = extract_transition_endpoint(inner);
            }
            Rule::transition_to => {
                to = extract_transition_endpoint(inner);
            }
            Rule::transition_label => {
                let (e, g, a) = extract_transition_label(inner);
                event = e;
                guard = g;
                action = a;
            }
            _ => {}
        }
    }

    if from.is_empty() || to.is_empty() {
        return None;
    }

    Some(Transition {
        from,
        to,
        event,
        guard,
        action,
        color: None,
    })
}

/// Извлекает endpoint перехода
fn extract_transition_endpoint(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::initial_final_state => return "[*]".to_string(),
            Rule::history_state => return inner.as_str().to_string(),
            Rule::simple_identifier => return inner.as_str().to_string(),
            _ => {}
        }
    }
    String::new()
}

/// Извлекает компоненты метки перехода
fn extract_transition_label(
    pair: pest::iterators::Pair<Rule>,
) -> (Option<String>, Option<String>, Option<String>) {
    let mut event: Option<String> = None;
    let mut guard: Option<String> = None;
    let mut action: Option<String> = None;

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::transition_label_content {
            for part in inner.into_inner() {
                match part.as_rule() {
                    Rule::event_part => {
                        let e = part.as_str().trim().to_string();
                        if !e.is_empty() {
                            event = Some(e);
                        }
                    }
                    Rule::guard_part => {
                        for g in part.into_inner() {
                            if g.as_rule() == Rule::guard_content {
                                let gc = g.as_str().trim().to_string();
                                if !gc.is_empty() {
                                    guard = Some(gc);
                                }
                            }
                        }
                    }
                    Rule::action_part => {
                        for a in part.into_inner() {
                            if a.as_rule() == Rule::action_content {
                                let ac = a.as_str().trim().to_string();
                                if !ac.is_empty() {
                                    action = Some(ac);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    (event, guard, action)
}

/// Парсит заметку
fn parse_note(pair: pest::iterators::Pair<Rule>) -> Option<Note> {
    let mut position = NotePosition::Right;
    let mut text = String::new();
    let mut anchors = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::note_on_link => {
                // note on link
                for n in inner.into_inner() {
                    if n.as_rule() == Rule::note_text {
                        text = n.as_str().trim().to_string();
                    }
                }
            }
            Rule::note_on_state => {
                for n in inner.into_inner() {
                    match n.as_rule() {
                        Rule::note_position => {
                            position = match n.as_str() {
                                "left" => NotePosition::Left,
                                "right" => NotePosition::Right,
                                "top" => NotePosition::Top,
                                "bottom" => NotePosition::Bottom,
                                _ => NotePosition::Right,
                            };
                        }
                        Rule::simple_identifier => {
                            anchors.push(n.as_str().to_string());
                        }
                        Rule::note_text => {
                            text = n.as_str().trim().to_string();
                        }
                        _ => {}
                    }
                }
            }
            Rule::note_multiline => {
                for n in inner.into_inner() {
                    match n.as_rule() {
                        Rule::note_position => {
                            position = match n.as_str() {
                                "left" => NotePosition::Left,
                                "right" => NotePosition::Right,
                                "top" => NotePosition::Top,
                                "bottom" => NotePosition::Bottom,
                                _ => NotePosition::Right,
                            };
                        }
                        Rule::simple_identifier => {
                            anchors.push(n.as_str().to_string());
                        }
                        Rule::note_body => {
                            text = n.as_str().trim().to_string();
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

/// Извлекает имя из state_name_part
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

/// Извлекает алиас
fn extract_alias(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::simple_identifier {
            return Some(inner.as_str().to_string());
        }
    }
    None
}

/// Извлекает описание
fn extract_description(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::state_description_text {
            let desc = inner.as_str().trim().to_string();
            if !desc.is_empty() {
                return Some(desc);
            }
        }
    }
    None
}

/// Извлекает строку из кавычек
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
    fn test_parse_simple_transition() {
        let source = r#"
@startuml
[*] --> State1
State1 --> State2
State2 --> [*]
@enduml
"#;

        let diagram = parse_state(source).unwrap();
        assert_eq!(diagram.transitions.len(), 3);

        assert_eq!(diagram.transitions[0].from, "[*]");
        assert_eq!(diagram.transitions[0].to, "State1");

        assert_eq!(diagram.transitions[1].from, "State1");
        assert_eq!(diagram.transitions[1].to, "State2");

        assert_eq!(diagram.transitions[2].from, "State2");
        assert_eq!(diagram.transitions[2].to, "[*]");
    }

    #[test]
    fn test_parse_transition_with_label() {
        let source = r#"
@startuml
[*] --> Active : start
Active --> Inactive : timeout [counter > 0] / reset()
Inactive --> [*]
@enduml
"#;

        let diagram = parse_state(source).unwrap();
        assert_eq!(diagram.transitions.len(), 3);

        // Первый переход
        assert_eq!(diagram.transitions[0].event, Some("start".to_string()));

        // Второй переход с guard и action
        assert_eq!(diagram.transitions[1].from, "Active");
        assert_eq!(diagram.transitions[1].to, "Inactive");
        assert!(diagram.transitions[1].event.is_some());
        assert!(diagram.transitions[1].guard.is_some());
        assert!(diagram.transitions[1].action.is_some());
    }

    #[test]
    fn test_parse_state_definition() {
        let source = r#"
@startuml
state "Long State Name" as LSN
state Processing
[*] --> Processing
@enduml
"#;

        let diagram = parse_state(source).unwrap();
        assert_eq!(diagram.states.len(), 2);

        assert_eq!(diagram.states[0].name, "Long State Name");
        assert_eq!(diagram.states[0].alias, Some("LSN".to_string()));

        assert_eq!(diagram.states[1].name, "Processing");
    }

    #[test]
    fn test_parse_composite_state() {
        let source = r#"
@startuml
state Composite {
    [*] --> Inner1
    Inner1 --> Inner2
    Inner2 --> [*]
}
@enduml
"#;

        let diagram = parse_state(source).unwrap();
        assert_eq!(diagram.states.len(), 1);

        let composite = &diagram.states[0];
        assert_eq!(composite.name, "Composite");
        assert_eq!(composite.state_type, StateType::Composite);
        assert_eq!(composite.internal_transitions.len(), 3);
    }

    #[test]
    fn test_parse_choice_state() {
        let source = r#"
@startuml
state choice1 <<choice>>
[*] --> choice1
choice1 --> State1 : [condition1]
choice1 --> State2 : [condition2]
@enduml
"#;

        let diagram = parse_state(source).unwrap();

        let choice = diagram.states.iter().find(|s| s.name == "choice1");
        assert!(choice.is_some());
        assert_eq!(choice.unwrap().state_type, StateType::Choice);
    }

    #[test]
    fn test_parse_fork_join() {
        let source = r#"
@startuml
state fork1 <<fork>>
state join1 <<join>>
[*] --> fork1
fork1 --> State1
fork1 --> State2
State1 --> join1
State2 --> join1
join1 --> [*]
@enduml
"#;

        let diagram = parse_state(source).unwrap();

        let fork = diagram.states.iter().find(|s| s.name == "fork1");
        assert!(fork.is_some());
        assert_eq!(fork.unwrap().state_type, StateType::Fork);

        let join = diagram.states.iter().find(|s| s.name == "join1");
        assert!(join.is_some());
        assert_eq!(join.unwrap().state_type, StateType::Join);
    }
}
