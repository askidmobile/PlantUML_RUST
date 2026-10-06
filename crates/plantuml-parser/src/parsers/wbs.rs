//! Парсер для WBS (Work Breakdown Structure) диаграмм
//!
//! Поддерживает стили:
//! - Asterisk: * ** ***
//! - OrgMode: + - (для правой/левой стороны)

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::wbs::{WbsDiagram, WbsNode, WbsNodeStyle};

use crate::error::ParseError;

#[derive(Parser)]
#[grammar = "grammars/wbs.pest"]
struct WbsParser;

/// Парсит WBS диаграмму из исходного кода
pub fn parse_wbs(source: &str) -> crate::Result<WbsDiagram> {
    let pairs =
        WbsParser::parse(Rule::wbs, source).map_err(|e| ParseError::GrammarError(e.to_string()))?;

    let mut diagram = WbsDiagram::new();
    let mut node_stack: Vec<WbsNode> = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::wbs {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::title_stmt => {
                        let title = extract_text_value(inner);
                        diagram.metadata.title = Some(title);
                    }
                    Rule::caption_stmt => {
                        let caption = extract_text_value(inner);
                        diagram.metadata.caption = Some(caption);
                    }
                    Rule::node_line => {
                        if let Some(node) = parse_node_line(inner) {
                            add_node_to_stack(&mut node_stack, node);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Строим дерево из стека узлов
    diagram.root = build_tree_from_stack(node_stack);

    Ok(diagram)
}

/// Парсит строку с узлом
fn parse_node_line(pair: pest::iterators::Pair<Rule>) -> Option<WbsNode> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::asterisk_node => return Some(parse_asterisk_node(inner)),
            Rule::orgmode_node => return Some(parse_orgmode_node(inner)),
            _ => {}
        }
    }
    None
}

/// Парсит узел в стиле asterisk
fn parse_asterisk_node(pair: pest::iterators::Pair<Rule>) -> WbsNode {
    let mut level = 1;
    let mut text = String::new();
    let mut style = WbsNodeStyle::Default;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::asterisk_level => {
                level = inner.as_str().len();
            }
            Rule::node_style => {
                style = parse_node_style(inner.as_str());
            }
            Rule::node_text => {
                text = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    WbsNode::new(level, text).with_style(style)
}

/// Парсит узел в стиле OrgMode
fn parse_orgmode_node(pair: pest::iterators::Pair<Rule>) -> WbsNode {
    let mut level = 1;
    let mut text = String::new();
    let mut style = WbsNodeStyle::Default;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::orgmode_level => {
                level = inner.as_str().len();
            }
            Rule::node_style => {
                style = parse_node_style(inner.as_str());
            }
            Rule::node_text => {
                text = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    WbsNode::new(level, text).with_style(style)
}

/// Парсит стиль узла
fn parse_node_style(s: &str) -> WbsNodeStyle {
    match s {
        "_" => WbsNodeStyle::Box,
        "-" => WbsNodeStyle::NoBorder,
        ";" => WbsNodeStyle::Strikethrough,
        _ => WbsNodeStyle::Default,
    }
}

/// Добавляет узел в стек
fn add_node_to_stack(stack: &mut Vec<WbsNode>, node: WbsNode) {
    if stack.is_empty() {
        stack.push(node);
        return;
    }

    let node_level = node.level;

    // Сворачиваем стек до узла с меньшим уровнем
    while stack.len() > 1 {
        let last_level = stack.last().map(|n| n.level).unwrap_or(0);
        if last_level >= node_level {
            // `let ... else` вместо `unwrap()`: стек защищён проверкой
            // `len() > 1`, но полагаться на это — значит зависеть от
            // инварианта в другом месте функции.
            let Some(child) = stack.pop() else {
                break;
            };
            if let Some(parent) = stack.last_mut() {
                parent.children.push(child);
            }
        } else {
            break;
        }
    }

    stack.push(node);
}

/// Строит дерево из стека узлов
fn build_tree_from_stack(mut stack: Vec<WbsNode>) -> Option<WbsNode> {
    if stack.is_empty() {
        return None;
    }

    while stack.len() > 1 {
        let Some(child) = stack.pop() else {
            break;
        };
        if let Some(parent) = stack.last_mut() {
            parent.children.push(child);
        }
    }

    stack.pop()
}

/// Извлекает текстовое значение
fn extract_text_value(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::text_to_eol {
            return inner.as_str().trim().to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_wbs() {
        let source = r#"@startwbs
* Project
** Phase 1
** Phase 2
@endwbs"#;

        let diagram = parse_wbs(source).unwrap();
        assert!(diagram.root.is_some());
        let root = diagram.root.unwrap();
        assert_eq!(root.text, "Project");
        assert_eq!(root.children.len(), 2);
    }

    #[test]
    fn test_parse_deep_wbs() {
        let source = r#"@startwbs
* Project
** Phase 1
*** Task 1.1
*** Task 1.2
** Phase 2
@endwbs"#;

        let diagram = parse_wbs(source).unwrap();
        assert_eq!(diagram.max_depth(), 3);
    }

    #[test]
    fn test_parse_with_title() {
        let source = r#"@startwbs
title My WBS
* Project
** Phase
@endwbs"#;

        let diagram = parse_wbs(source).unwrap();
        assert_eq!(diagram.metadata.title, Some("My WBS".to_string()));
    }

    #[test]
    fn test_parse_orgmode_style() {
        let source = r#"@startwbs
+ Project
++ Phase 1
++ Phase 2
@endwbs"#;

        let diagram = parse_wbs(source).unwrap();
        assert!(diagram.root.is_some());
    }
}
