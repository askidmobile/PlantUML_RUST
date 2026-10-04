//! Парсер Timing Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML timing diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::timing::{
    ParticipantType, StateChange, TimeConstraint, TimeValue, TimingDiagram, TimingParticipant,
};

use crate::{ParseError, Result};

#[derive(Parser)]
#[grammar = "grammars/timing.pest"]
pub struct TimingParser;

/// Парсит timing diagram из исходного кода
pub fn parse_timing(source: &str) -> Result<TimingDiagram> {
    let pairs =
        TimingParser::parse(Rule::diagram, source).map_err(|e| ParseError::SyntaxError {
            line: e.line().to_string().parse().unwrap_or(0),
            message: e.to_string(),
        })?;

    let mut diagram = TimingDiagram::new();
    let mut current_time: Option<TimeValue> = None;

    for pair in pairs {
        if pair.as_rule() == Rule::diagram {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::body => {
                        parse_body(inner, &mut diagram, &mut current_time);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(diagram)
}

/// Парсит тело диаграммы
fn parse_body(
    pair: pest::iterators::Pair<Rule>,
    diagram: &mut TimingDiagram,
    current_time: &mut Option<TimeValue>,
) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::title_stmt => {
                diagram.metadata.title = extract_title(inner);
            }
            Rule::participant_decl => {
                if let Some(p) = parse_participant(inner) {
                    diagram.participants.push(p);
                }
            }
            Rule::time_marker => {
                *current_time = parse_time_marker(inner);
            }
            Rule::state_change => {
                if let Some(change) = parse_state_change(inner, current_time) {
                    diagram.state_changes.push(change);
                }
            }
            Rule::state_transition => {
                if let Some(changes) = parse_state_transition(inner, current_time) {
                    diagram.state_changes.extend(changes);
                }
            }
            Rule::constraint_stmt => {
                if let Some(constraint) = parse_constraint(inner) {
                    diagram.constraints.push(constraint);
                }
            }
            _ => {}
        }
    }
}

/// Парсит объявление участника
fn parse_participant(pair: pest::iterators::Pair<Rule>) -> Option<TimingParticipant> {
    let mut name = String::new();
    let mut alias: Option<String> = None;
    let mut participant_type = ParticipantType::Robust;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::participant_type => {
                participant_type = match inner.as_str().to_lowercase().as_str() {
                    "robust" => ParticipantType::Robust,
                    "concise" => ParticipantType::Concise,
                    "clock" => ParticipantType::Clock,
                    "binary" => ParticipantType::Binary,
                    _ => ParticipantType::Robust,
                };
            }
            Rule::participant_name => {
                name = extract_name(inner);
            }
            Rule::simple_identifier => {
                // Это alias после "as"
                alias = Some(inner.as_str().to_string());
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(TimingParticipant {
        name,
        alias,
        participant_type,
        states: Vec::new(),
        color: None,
    })
}

/// Парсит маркер времени (@0, @100, @+50)
fn parse_time_marker(pair: pest::iterators::Pair<Rule>) -> Option<TimeValue> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::time_value {
            return parse_time_value(inner);
        }
    }
    None
}

/// Парсит значение времени
fn parse_time_value(pair: pest::iterators::Pair<Rule>) -> Option<TimeValue> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::absolute_time => {
                if let Ok(v) = inner.as_str().parse::<f64>() {
                    return Some(TimeValue::Absolute(v));
                }
            }
            Rule::relative_time => {
                // Формат: +50
                let text = inner.as_str();
                if let Ok(v) = text.trim_start_matches('+').parse::<f64>() {
                    return Some(TimeValue::Relative(v));
                }
            }
            Rule::named_time => {
                return Some(TimeValue::Named(inner.as_str().to_string()));
            }
            _ => {}
        }
    }
    None
}

/// Парсит изменение состояния (Entity is State)
fn parse_state_change(
    pair: pest::iterators::Pair<Rule>,
    current_time: &Option<TimeValue>,
) -> Option<StateChange> {
    let mut participant = String::new();
    let mut state = String::new();
    let mut label: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::entity_ref => {
                participant = inner.as_str().to_string();
            }
            Rule::state_value => {
                state = extract_state_value(inner);
            }
            Rule::label_text => {
                let text = inner.as_str().trim().to_string();
                if !text.is_empty() {
                    label = Some(text);
                }
            }
            _ => {}
        }
    }

    if participant.is_empty() || state.is_empty() {
        return None;
    }

    let time = current_time.clone().unwrap_or(TimeValue::Absolute(0.0));

    Some(StateChange {
        participant,
        time,
        state,
        label,
    })
}

/// Парсит переход состояния (Entity: Old -> New)
fn parse_state_transition(
    pair: pest::iterators::Pair<Rule>,
    current_time: &Option<TimeValue>,
) -> Option<Vec<StateChange>> {
    let mut participant = String::new();
    let mut states: Vec<String> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::entity_ref => {
                participant = inner.as_str().to_string();
            }
            Rule::state_value => {
                states.push(extract_state_value(inner));
            }
            _ => {}
        }
    }

    if participant.is_empty() || states.len() < 2 {
        return None;
    }

    let time = current_time.clone().unwrap_or(TimeValue::Absolute(0.0));

    // Создаём одно изменение для конечного состояния
    Some(vec![StateChange {
        participant,
        time,
        state: states.pop().unwrap_or_default(),
        label: None,
    }])
}

/// Парсит ограничение времени
fn parse_constraint(pair: pest::iterators::Pair<Rule>) -> Option<TimeConstraint> {
    let mut label: Option<String> = None;

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::constraint_content {
            label = Some(inner.as_str().trim().to_string());
        }
    }

    // Ограничение без явных времён - просто метка
    if let Some(lbl) = label {
        return Some(TimeConstraint {
            from_time: TimeValue::Absolute(0.0),
            to_time: TimeValue::Absolute(0.0),
            label: Some(lbl),
        });
    }

    None
}

/// Извлекает title из title_stmt
fn extract_title(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::rest_of_line {
            let text = inner.as_str().trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

/// Извлекает имя из различных правил
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

/// Извлекает значение состояния
fn extract_state_value(pair: pest::iterators::Pair<Rule>) -> String {
    // Сохраняем текст до into_inner()
    let text = pair.as_str().trim();
    let fallback = if text.starts_with('{') && text.ends_with('}') {
        text[1..text.len() - 1].to_string()
    } else {
        text.to_string()
    };

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::quoted_string => {
                return extract_quoted_string(inner);
            }
            Rule::simple_state => {
                return inner.as_str().to_string();
            }
            _ => {}
        }
    }
    fallback
}

/// Извлекает строку в кавычках
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
    fn test_parse_simple_timing() {
        let source = r#"
@startuml
robust "Web Browser" as WB
concise "Server" as S

@0
WB is Idle
S is Idle

@100
WB is Running
S is Processing
@enduml
"#;

        let diagram = parse_timing(source).unwrap();
        assert_eq!(diagram.participants.len(), 2);
        assert_eq!(diagram.participants[0].name, "Web Browser");
        assert_eq!(diagram.participants[0].alias, Some("WB".to_string()));
        assert_eq!(
            diagram.participants[0].participant_type,
            ParticipantType::Robust
        );
        assert_eq!(diagram.participants[1].name, "Server");
        assert_eq!(
            diagram.participants[1].participant_type,
            ParticipantType::Concise
        );

        assert_eq!(diagram.state_changes.len(), 4);
    }

    #[test]
    fn test_parse_clock() {
        let source = r#"
@startuml
clock clk
binary "Data" as D

@0
clk is high
D is 0

@50
clk is low
D is 1
@enduml
"#;

        let diagram = parse_timing(source).unwrap();
        assert_eq!(diagram.participants.len(), 2);
        assert_eq!(
            diagram.participants[0].participant_type,
            ParticipantType::Clock
        );
        assert_eq!(
            diagram.participants[1].participant_type,
            ParticipantType::Binary
        );
    }

    #[test]
    fn test_parse_with_title() {
        let source = r#"
@startuml
title Timing Diagram Example

robust Signal as S

@0
S is Low
@enduml
"#;

        let diagram = parse_timing(source).unwrap();
        assert_eq!(
            diagram.metadata.title,
            Some("Timing Diagram Example".to_string())
        );
    }

    #[test]
    fn test_parse_relative_time() {
        let source = r#"
@startuml
concise Entity as E

@0
E is A

@+50
E is B

@+100
E is C
@enduml
"#;

        let diagram = parse_timing(source).unwrap();
        assert_eq!(diagram.state_changes.len(), 3);
    }
}
