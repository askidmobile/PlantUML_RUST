//! Парсер Gantt Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML gantt diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::gantt::{
    ClosedDay, GanttDate, GanttDiagram, GanttMilestone, GanttScale, GanttSeparator, GanttTask,
    Holiday, MilestoneTime, TaskDuration, TaskStart, Weekday,
};

use crate::{ParseError, Result};

#[derive(Parser)]
#[grammar = "grammars/gantt.pest"]
pub struct GanttParser;

/// Парсит gantt diagram из исходного кода
pub fn parse_gantt(source: &str) -> Result<GanttDiagram> {
    let pairs = GanttParser::parse(Rule::diagram, source).map_err(|e| ParseError::SyntaxError {
        line: e.line().to_string().parse().unwrap_or(0),
        message: e.to_string(),
    })?;

    let mut diagram = GanttDiagram::new();
    let mut last_task_id: Option<String> = None;

    for pair in pairs {
        if pair.as_rule() == Rule::diagram {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::body {
                    parse_body(inner, &mut diagram, &mut last_task_id);
                }
            }
        }
    }

    Ok(diagram)
}

/// Парсит тело диаграммы
fn parse_body(
    pair: pest::iterators::Pair<Rule>,
    diagram: &mut GanttDiagram,
    last_task_id: &mut Option<String>,
) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::title_stmt => {
                diagram.metadata.title = extract_title(inner);
            }
            Rule::project_start => {
                diagram.project_start = parse_project_start(inner);
            }
            Rule::print_scale => {
                if let Some(scale) = parse_print_scale(inner) {
                    diagram.scale = scale;
                }
            }
            Rule::closed_stmt => {
                if let Some(closed) = parse_closed_stmt(inner) {
                    diagram.closed_days.extend(closed);
                }
            }
            Rule::holiday_stmt => {
                if let Some(holiday) = parse_holiday(inner) {
                    diagram.holidays.push(holiday);
                }
            }
            Rule::task_def => {
                if let Some(task) = parse_task(inner, last_task_id) {
                    *last_task_id = task.id.clone().or_else(|| Some(task.name.clone()));
                    diagram.tasks.push(task);
                }
            }
            Rule::then_stmt => {
                if let Some(task) = parse_then_task(inner, last_task_id) {
                    *last_task_id = task.id.clone().or_else(|| Some(task.name.clone()));
                    diagram.tasks.push(task);
                }
            }
            Rule::milestone_def => {
                if let Some(milestone) = parse_milestone(inner) {
                    diagram.milestones.push(milestone);
                }
            }
            Rule::separator => {
                diagram.separators.push(parse_separator(inner));
            }
            _ => {}
        }
    }
}

/// Парсит project starts
fn parse_project_start(pair: pest::iterators::Pair<Rule>) -> Option<GanttDate> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::date_value {
            return GanttDate::parse(inner.as_str());
        }
    }
    None
}

/// Парсит printscale
fn parse_print_scale(pair: pest::iterators::Pair<Rule>) -> Option<GanttScale> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::scale_value {
            return GanttScale::parse(inner.as_str());
        }
    }
    None
}

/// Парсит closed statement
fn parse_closed_stmt(pair: pest::iterators::Pair<Rule>) -> Option<Vec<ClosedDay>> {
    let mut days = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::weekday => {
                if let Some(day) = Weekday::parse(inner.as_str()) {
                    days.push(ClosedDay { day });
                }
            }
            Rule::weekday_list => {
                for weekday in inner.into_inner() {
                    if weekday.as_rule() == Rule::weekday {
                        if let Some(day) = Weekday::parse(weekday.as_str()) {
                            days.push(ClosedDay { day });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if days.is_empty() {
        None
    } else {
        Some(days)
    }
}

/// Парсит holiday
fn parse_holiday(pair: pest::iterators::Pair<Rule>) -> Option<Holiday> {
    let mut date: Option<GanttDate> = None;
    let mut name: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::date_value => {
                date = GanttDate::parse(inner.as_str());
            }
            Rule::holiday_name => {
                name = Some(inner.as_str().trim().to_string());
            }
            _ => {}
        }
    }

    date.map(|d| Holiday { date: d, name })
}

/// Парсит задачу
fn parse_task(
    pair: pest::iterators::Pair<Rule>,
    _last_task_id: &Option<String>,
) -> Option<GanttTask> {
    let mut name = String::new();
    let mut task = GanttTask::new("");

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::task_name => {
                name = extract_task_name(inner);
            }
            Rule::task_modifiers => {
                parse_task_modifiers(inner, &mut task);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    task.name = name;
    Some(task)
}

/// Парсит then task
fn parse_then_task(
    pair: pest::iterators::Pair<Rule>,
    last_task_id: &Option<String>,
) -> Option<GanttTask> {
    let mut name = String::new();
    let mut task = GanttTask::new("");

    // then задача начинается после предыдущей
    if let Some(ref prev_id) = last_task_id {
        task.start = TaskStart::After(prev_id.clone());
    }

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::task_name => {
                name = extract_task_name(inner);
            }
            Rule::task_modifiers => {
                parse_task_modifiers(inner, &mut task);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    task.name = name;
    Some(task)
}

/// Парсит модификаторы задачи
fn parse_task_modifiers(pair: pest::iterators::Pair<Rule>, task: &mut GanttTask) {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::task_modifier {
            for modifier in inner.into_inner() {
                match modifier.as_rule() {
                    Rule::alias_modifier => {
                        task.id = extract_task_ref(modifier);
                    }
                    Rule::lasts_modifier => {
                        task.duration = parse_lasts_modifier(modifier);
                    }
                    Rule::starts_modifier => {
                        task.start = parse_starts_modifier(modifier);
                    }
                    Rule::ends_modifier => {
                        task.duration = parse_ends_modifier(modifier);
                    }
                    Rule::complete_modifier => {
                        task.complete = parse_complete_modifier(modifier);
                    }
                    Rule::on_modifier => {
                        task.resource = parse_on_modifier(modifier);
                    }
                    Rule::requires_modifier => {
                        if let Some(ref_id) = extract_task_ref(modifier) {
                            task.links.push(ref_id);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Парсит lasts modifier
fn parse_lasts_modifier(pair: pest::iterators::Pair<Rule>) -> TaskDuration {
    let mut amount = 1u32;
    let mut is_weeks = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::number => {
                amount = inner.as_str().parse().unwrap_or(1);
            }
            Rule::duration_unit => {
                is_weeks = inner.as_str().starts_with("week");
            }
            _ => {}
        }
    }

    if is_weeks {
        TaskDuration::Weeks(amount)
    } else {
        TaskDuration::Days(amount)
    }
}

/// Парсит starts modifier
fn parse_starts_modifier(pair: pest::iterators::Pair<Rule>) -> TaskStart {
    let text = pair.as_str();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::date_value => {
                if let Some(date) = GanttDate::parse(inner.as_str()) {
                    return TaskStart::AtDate(date);
                }
            }
            Rule::task_ref => {
                let task_id = extract_task_ref_inner(inner);
                if text.contains("'s end") {
                    return TaskStart::AtEnd(task_id);
                } else if text.contains("after") {
                    return TaskStart::After(task_id);
                } else if text.contains("with") {
                    return TaskStart::With(task_id);
                } else {
                    return TaskStart::After(task_id);
                }
            }
            _ => {}
        }
    }

    TaskStart::AfterPrevious
}

/// Парсит ends modifier
fn parse_ends_modifier(pair: pest::iterators::Pair<Rule>) -> TaskDuration {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::date_value => {
                if let Some(date) = GanttDate::parse(inner.as_str()) {
                    return TaskDuration::Until(date);
                }
            }
            Rule::task_ref => {
                return TaskDuration::EndsAt(extract_task_ref_inner(inner));
            }
            _ => {}
        }
    }

    TaskDuration::Days(1)
}

/// Парсит complete modifier
fn parse_complete_modifier(pair: pest::iterators::Pair<Rule>) -> Option<u8> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::number {
            return inner.as_str().parse().ok().map(|v: u8| v.min(100));
        }
    }
    None
}

/// Парсит on modifier (resource)
fn parse_on_modifier(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::resource_name {
            return Some(inner.as_str().trim().to_string());
        }
    }
    None
}

/// Парсит milestone
fn parse_milestone(pair: pest::iterators::Pair<Rule>) -> Option<GanttMilestone> {
    let mut name = String::new();
    let mut happens = MilestoneTime::AfterPrevious;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::task_name => {
                name = extract_task_name(inner);
            }
            Rule::milestone_time => {
                happens = parse_milestone_time(inner);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(GanttMilestone {
        name,
        happens,
        color: None,
    })
}

/// Парсит milestone time
fn parse_milestone_time(pair: pest::iterators::Pair<Rule>) -> MilestoneTime {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::date_value => {
                if let Some(date) = GanttDate::parse(inner.as_str()) {
                    return MilestoneTime::AtDate(date);
                }
            }
            Rule::task_ref => {
                return MilestoneTime::After(extract_task_ref_inner(inner));
            }
            _ => {}
        }
    }

    MilestoneTime::AfterPrevious
}

/// Парсит separator
fn parse_separator(pair: pest::iterators::Pair<Rule>) -> GanttSeparator {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::separator_label {
            return GanttSeparator::with_label(inner.as_str().trim());
        }
    }
    GanttSeparator::new()
}

// === Вспомогательные функции ===

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

fn extract_task_name(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::task_name_inner {
            return inner.as_str().trim().to_string();
        }
    }
    String::new()
}

fn extract_task_ref(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::task_ref {
            return Some(extract_task_ref_inner(inner));
        }
    }
    None
}

fn extract_task_ref_inner(pair: pest::iterators::Pair<Rule>) -> String {
    let fallback = pair
        .as_str()
        .trim_matches(|c| c == '[' || c == ']')
        .to_string();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::task_ref_inner {
            return inner.as_str().trim().to_string();
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_gantt() {
        let source = r#"
@startgantt
[Task 1] lasts 5 days
[Task 2] lasts 3 days
@endgantt
"#;

        let diagram = parse_gantt(source).unwrap();
        assert_eq!(diagram.tasks.len(), 2);
        assert_eq!(diagram.tasks[0].name, "Task 1");
        assert_eq!(diagram.tasks[1].name, "Task 2");
    }

    #[test]
    fn test_parse_with_project_start() {
        let source = r#"
@startgantt
project starts 2024-01-15
[Design] lasts 10 days
@endgantt
"#;

        let diagram = parse_gantt(source).unwrap();
        assert!(diagram.project_start.is_some());
        let start = diagram.project_start.unwrap();
        assert_eq!(start.year, 2024);
        assert_eq!(start.month, 1);
        assert_eq!(start.day, 15);
    }

    #[test]
    fn test_parse_task_with_alias() {
        let source = r#"
@startgantt
[Design Phase] as [T1] lasts 5 days
[Implementation] starts after [T1] lasts 10 days
@endgantt
"#;

        let diagram = parse_gantt(source).unwrap();
        assert_eq!(diagram.tasks.len(), 2);
        assert_eq!(diagram.tasks[0].id, Some("T1".to_string()));
        assert!(matches!(diagram.tasks[1].start, TaskStart::After(ref id) if id == "T1"));
    }

    #[test]
    fn test_parse_then_statement() {
        let source = r#"
@startgantt
[Task 1] lasts 3 days
then [Task 2] lasts 5 days
then [Task 3] lasts 2 days
@endgantt
"#;

        let diagram = parse_gantt(source).unwrap();
        assert_eq!(diagram.tasks.len(), 3);
        // Task 2 должна начинаться после Task 1
        assert!(matches!(diagram.tasks[1].start, TaskStart::After(_)));
    }

    #[test]
    fn test_parse_closed_days() {
        let source = r#"
@startgantt
saturday are closed
sunday are closed
[Task] lasts 5 days
@endgantt
"#;

        let diagram = parse_gantt(source).unwrap();
        assert_eq!(diagram.closed_days.len(), 2);
    }

    #[test]
    fn test_parse_complete_percentage() {
        let source = r#"
@startgantt
[Task 1] lasts 5 days is 50% completed
@endgantt
"#;

        let diagram = parse_gantt(source).unwrap();
        assert_eq!(diagram.tasks[0].complete, Some(50));
    }

    #[test]
    fn test_parse_separator() {
        let source = r#"
@startgantt
[Task 1] lasts 3 days
-- Phase 2 --
[Task 2] lasts 5 days
@endgantt
"#;

        let diagram = parse_gantt(source).unwrap();
        assert_eq!(diagram.separators.len(), 1);
        assert_eq!(diagram.separators[0].label, Some("Phase 2".to_string()));
    }
}
