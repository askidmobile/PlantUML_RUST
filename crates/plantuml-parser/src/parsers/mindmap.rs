//! Парсер для MindMap диаграмм
//!
//! Поддерживает три стиля синтаксиса:
//! - Asterisk: * ** ***
//! - OrgMode: + - (для правой/левой стороны)
//! - Markdown: # ## ###

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::common::Color;
use plantuml_ast::mindmap::{MindMapDiagram, MindMapNode, NodeDirection, NodeStyle};

use crate::error::ParseError;

#[derive(Parser)]
#[grammar = "grammars/mindmap.pest"]
struct MindMapParser;

/// Парсит MindMap диаграмму из исходного кода
pub fn parse_mindmap(source: &str) -> crate::Result<MindMapDiagram> {
    let pairs = MindMapParser::parse(Rule::mindmap, source)
        .map_err(|e| ParseError::GrammarError(e.to_string()))?;

    let mut diagram = MindMapDiagram::new();
    let mut node_stack: Vec<MindMapNode> = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::mindmap {
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
fn parse_node_line(pair: pest::iterators::Pair<Rule>) -> Option<MindMapNode> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::asterisk_node => return Some(parse_asterisk_node(inner)),
            Rule::orgmode_node => return Some(parse_orgmode_node(inner)),
            Rule::markdown_node => return Some(parse_markdown_node(inner)),
            _ => {}
        }
    }
    None
}

/// Парсит узел в стиле asterisk (* ** ***)
fn parse_asterisk_node(pair: pest::iterators::Pair<Rule>) -> MindMapNode {
    let mut level = 1;
    let mut text = String::new();
    let mut style = NodeStyle::Default;
    let mut color: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::asterisk_level => {
                level = inner.as_str().len();
            }
            Rule::node_style => {
                style = parse_node_style(inner.as_str());
            }
            Rule::node_color => {
                color = parse_node_color(inner);
            }
            Rule::node_text => {
                text = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    let mut node = MindMapNode::new(level, text);
    node.style = style;
    node.color = color;
    node.direction = NodeDirection::Right; // По умолчанию справа
    node
}

/// Парсит узел в стиле OrgMode (+ -)
fn parse_orgmode_node(pair: pest::iterators::Pair<Rule>) -> MindMapNode {
    let mut level = 1;
    let mut text = String::new();
    let mut style = NodeStyle::Default;
    let mut color: Option<Color> = None;
    let mut direction = NodeDirection::Right;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::orgmode_level => {
                let s = inner.as_str();
                level = s.len();
                // Определяем направление по первому символу
                if s.starts_with('-') {
                    direction = NodeDirection::Left;
                }
            }
            Rule::node_style => {
                style = parse_node_style(inner.as_str());
            }
            Rule::node_color => {
                color = parse_node_color(inner);
            }
            Rule::node_text => {
                text = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    let mut node = MindMapNode::new(level, text);
    node.style = style;
    node.color = color;
    node.direction = direction;
    node
}

/// Парсит узел в стиле Markdown (# ## ###)
fn parse_markdown_node(pair: pest::iterators::Pair<Rule>) -> MindMapNode {
    let mut level = 1;
    let mut text = String::new();
    let mut style = NodeStyle::Default;
    let mut color: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::markdown_level => {
                level = inner.as_str().len();
            }
            Rule::node_style => {
                style = parse_node_style(inner.as_str());
            }
            Rule::node_color => {
                color = parse_node_color(inner);
            }
            Rule::node_text => {
                text = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    let mut node = MindMapNode::new(level, text);
    node.style = style;
    node.color = color;
    node.direction = NodeDirection::Right;
    node
}

/// Парсит стиль узла
fn parse_node_style(s: &str) -> NodeStyle {
    match s {
        "_" => NodeStyle::Box,
        "-" => NodeStyle::NoBorder,
        ";" => NodeStyle::Strikethrough,
        _ => NodeStyle::Default,
    }
}

/// Парсит цвет узла
fn parse_node_color(pair: pest::iterators::Pair<Rule>) -> Option<Color> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::color_value {
            return Some(Color::from_hex(inner.as_str()));
        }
    }
    None
}

/// Добавляет узел в стек с учётом уровня вложенности
fn add_node_to_stack(stack: &mut Vec<MindMapNode>, node: MindMapNode) {
    // Если стек пуст, просто добавляем
    if stack.is_empty() {
        stack.push(node);
        return;
    }

    // Ищем родителя для нового узла
    let node_level = node.level;

    // Сворачиваем стек до узла с меньшим уровнем
    while stack.len() > 1 {
        let last_level = stack.last().map(|n| n.level).unwrap_or(0);
        if last_level >= node_level {
            // Последний узел в стеке - на том же или более глубоком уровне
            // Нужно "закрыть" его и добавить к родителю
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
fn build_tree_from_stack(mut stack: Vec<MindMapNode>) -> Option<MindMapNode> {
    if stack.is_empty() {
        return None;
    }

    // Сворачиваем весь стек в одно дерево
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

/// Извлекает текстовое значение из пары
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
    fn test_parse_simple_mindmap() {
        let source = r#"@startmindmap
* Root
** Branch 1
** Branch 2
@endmindmap"#;

        let diagram = parse_mindmap(source).unwrap();
        assert!(diagram.root.is_some());
        let root = diagram.root.unwrap();
        assert_eq!(root.text, "Root");
        assert_eq!(root.children.len(), 2);
    }

    #[test]
    fn test_parse_deep_hierarchy() {
        let source = r#"@startmindmap
* Root
** Level 2
*** Level 3
**** Level 4
@endmindmap"#;

        let diagram = parse_mindmap(source).unwrap();
        assert_eq!(diagram.max_depth(), 4);
    }

    #[test]
    fn test_parse_with_title() {
        let source = r#"@startmindmap
title My Mind Map
* Root
** Branch
@endmindmap"#;

        let diagram = parse_mindmap(source).unwrap();
        assert_eq!(diagram.metadata.title, Some("My Mind Map".to_string()));
    }

    #[test]
    fn test_parse_orgmode_style() {
        let source = r#"@startmindmap
+ Root
++ Right Branch
-- Left Branch
@endmindmap"#;

        let diagram = parse_mindmap(source).unwrap();
        assert!(diagram.root.is_some());
    }

    #[test]
    fn test_parse_with_colors() {
        let source = r#"@startmindmap
*[#FF0000] Red Root
**[#00FF00] Green Branch
@endmindmap"#;

        let diagram = parse_mindmap(source).unwrap();
        let root = diagram.root.unwrap();
        assert!(root.color.is_some());
    }
}
