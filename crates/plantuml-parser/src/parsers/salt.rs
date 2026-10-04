//! Парсер Salt (Wireframe) диаграмм
//!
//! Парсит Salt диаграммы PlantUML для создания UI wireframes.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::salt::{
    BorderStyle, Container, SaltDiagram, SaltWidget, ScrollbarType, SeparatorType, TreeNode,
};

use crate::error::ParseError;

#[derive(Parser)]
#[grammar = "grammars/salt.pest"]
struct SaltParser;

/// Парсит Salt диаграмму
pub fn parse_salt(source: &str) -> crate::Result<SaltDiagram> {
    // Извлекаем содержимое salt блока
    let salt_content = extract_salt_content(source)?;

    let pairs = SaltParser::parse(Rule::salt_diagram, &salt_content)
        .map_err(|e| ParseError::GrammarError(format!("Ошибка парсинга Salt: {}", e)))?;

    let mut diagram = SaltDiagram::new();

    for pair in pairs {
        if pair.as_rule() == Rule::salt_diagram {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::container {
                    diagram.root = parse_container(inner)?;
                }
            }
        }
    }

    Ok(diagram)
}

/// Извлекает содержимое salt блока из исходного кода
fn extract_salt_content(source: &str) -> crate::Result<String> {
    let source = source.trim();

    // Проверяем наличие @startsalt / @endsalt
    if source.starts_with("@startsalt") {
        let end_idx = source.rfind("@endsalt").unwrap_or(source.len());
        let start_idx = source.find('\n').map(|i| i + 1).unwrap_or(10);
        return Ok(source[start_idx..end_idx].trim().to_string());
    }

    // Или @startuml с salt внутри
    if source.starts_with("@startuml") {
        let end_idx = source.rfind("@enduml").unwrap_or(source.len());
        let start_idx = source.find('\n').map(|i| i + 1).unwrap_or(9);
        let content = &source[start_idx..end_idx];

        // Убираем ключевое слово salt если есть
        let content = content.trim();
        if let Some(rest) = content.strip_prefix("salt") {
            return Ok(rest.trim().to_string());
        }
        return Ok(content.to_string());
    }

    Ok(source.to_string())
}

/// Парсит контейнер
fn parse_container(pair: pest::iterators::Pair<Rule>) -> crate::Result<SaltWidget> {
    let mut container = Container::new();
    let mut is_tree = false;
    let mut is_tabs = false;
    let mut is_menu = false;
    let mut is_scroll = false;
    let mut scroll_type = ScrollbarType::Both;
    let mut group_title: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::border_style => {
                let style_str = inner.as_str();
                if let Some(first_char) = style_str.chars().next() {
                    match first_char {
                        '#' => container.border_style = BorderStyle::All,
                        '!' => container.border_style = BorderStyle::Vertical,
                        '-' => container.border_style = BorderStyle::Horizontal,
                        '+' => container.border_style = BorderStyle::External,
                        '^' => {
                            // Group box
                            if let Some(title_pair) = inner.into_inner().next() {
                                if title_pair.as_rule() == Rule::quoted_title {
                                    if let Some(content) = title_pair.into_inner().next() {
                                        group_title = Some(content.as_str().to_string());
                                    }
                                }
                            }
                        }
                        'S' => {
                            is_scroll = true;
                            let rest = &style_str[1..];
                            if rest.contains('I') {
                                scroll_type = ScrollbarType::Vertical;
                            } else if rest.contains('-') {
                                scroll_type = ScrollbarType::Horizontal;
                            }
                        }
                        'T' => is_tree = true,
                        '/' => is_tabs = true,
                        '*' => is_menu = true,
                        _ => {}
                    }
                }
            }
            Rule::container_content => {
                for row_pair in inner.into_inner() {
                    if row_pair.as_rule() == Rule::row {
                        let row = parse_row(row_pair)?;
                        if !row.is_empty() {
                            container.add_row(row);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Обрабатываем специальные типы контейнеров
    if is_tree {
        return Ok(parse_tree_content(&container));
    }

    if is_tabs {
        return Ok(parse_tabs_content(&container));
    }

    if is_menu {
        return Ok(parse_menu_content(&container));
    }

    let widget = SaltWidget::Container(container);

    if is_scroll {
        return Ok(SaltWidget::ScrollArea {
            content: Box::new(widget),
            scrollbar: scroll_type,
        });
    }

    if let Some(title) = group_title {
        return Ok(SaltWidget::GroupBox {
            title,
            content: Box::new(widget),
        });
    }

    Ok(widget)
}

/// Парсит строку
fn parse_row(pair: pest::iterators::Pair<Rule>) -> crate::Result<Vec<SaltWidget>> {
    let mut widgets = Vec::new();

    for cell_pair in pair.into_inner() {
        if cell_pair.as_rule() == Rule::cell {
            for widget_pair in cell_pair.into_inner() {
                // widget теперь silent (_), так что мы получаем напрямую типы виджетов
                if let Some(w) = parse_widget_inner(widget_pair)? {
                    widgets.push(w);
                }
            }
        }
    }

    Ok(widgets)
}

/// Парсит внутренний виджет
fn parse_widget_inner(pair: pest::iterators::Pair<Rule>) -> crate::Result<Option<SaltWidget>> {
    match pair.as_rule() {
        Rule::container => {
            return Ok(Some(parse_container(pair)?));
        }
        Rule::button => {
            if let Some(content) = pair.into_inner().next() {
                return Ok(Some(SaltWidget::Button(
                    content.as_str().trim().to_string(),
                )));
            }
        }
        Rule::textfield => {
            if let Some(content) = pair.into_inner().next() {
                return Ok(Some(SaltWidget::TextField(content.as_str().to_string())));
            }
        }
        Rule::checkbox_checked => {
            return Ok(Some(SaltWidget::Checkbox {
                label: String::new(),
                checked: true,
            }));
        }
        Rule::checkbox_unchecked => {
            return Ok(Some(SaltWidget::Checkbox {
                label: String::new(),
                checked: false,
            }));
        }
        Rule::radio_checked => {
            return Ok(Some(SaltWidget::Radio {
                label: String::new(),
                checked: true,
            }));
        }
        Rule::radio_unchecked => {
            return Ok(Some(SaltWidget::Radio {
                label: String::new(),
                checked: false,
            }));
        }
        Rule::droplist => {
            let items: Vec<String> = pair
                .into_inner()
                .filter(|p| p.as_rule() == Rule::droplist_item)
                .map(|p| p.as_str().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let open = items.len() > 1;
            return Ok(Some(SaltWidget::Droplist { items, open }));
        }
        Rule::separator => {
            for sep in pair.into_inner() {
                let sep_type = match sep.as_rule() {
                    Rule::double_line => SeparatorType::Double,
                    Rule::dotted_line => SeparatorType::Dotted,
                    Rule::wavy_line => SeparatorType::Wavy,
                    Rule::single_line => SeparatorType::Single,
                    _ => continue,
                };
                return Ok(Some(SaltWidget::Separator(sep_type)));
            }
        }
        Rule::plain_text => {
            let text = pair.as_str().trim();
            if !text.is_empty() {
                return Ok(Some(SaltWidget::Text(text.to_string())));
            }
        }
        Rule::tree_item => {
            let text = pair.as_str();
            let level = text.chars().take_while(|c| *c == '+').count();
            let content = text.trim_start_matches('+').trim();
            return Ok(Some(SaltWidget::Tree(TreeNode::new(content, level))));
        }
        _ => {}
    }
    Ok(None)
}

/// Парсит содержимое дерева
fn parse_tree_content(container: &Container) -> SaltWidget {
    let mut root_nodes: Vec<TreeNode> = Vec::new();
    let mut stack: Vec<(usize, TreeNode)> = Vec::new();

    for row in &container.rows {
        for widget in row {
            if let SaltWidget::Tree(node) = widget {
                let level = node.level;
                let new_node = node.clone();

                // Закрываем узлы с большим или равным уровнем
                while let Some((parent_level, _)) = stack.last() {
                    if *parent_level >= level {
                        let (_, finished) = stack.pop().unwrap();
                        if let Some((_, parent)) = stack.last_mut() {
                            parent.children.push(finished);
                        } else {
                            root_nodes.push(finished);
                        }
                    } else {
                        break;
                    }
                }

                stack.push((level, new_node));
            } else if let SaltWidget::Text(text) = widget {
                // Текст в дереве тоже может быть узлом
                let level = 1;
                let new_node = TreeNode::new(text.clone(), level);
                stack.push((level, new_node));
            }
        }
    }

    // Закрываем все оставшиеся узлы
    while let Some((_, finished)) = stack.pop() {
        if let Some((_, parent)) = stack.last_mut() {
            parent.children.push(finished);
        } else {
            root_nodes.push(finished);
        }
    }

    // Возвращаем корневой узел или пустой
    if root_nodes.len() == 1 {
        SaltWidget::Tree(root_nodes.remove(0))
    } else if !root_nodes.is_empty() {
        // Создаём виртуальный корень
        let mut root = TreeNode::new("", 0);
        root.children = root_nodes;
        SaltWidget::Tree(root)
    } else {
        SaltWidget::Tree(TreeNode::new("", 0))
    }
}

/// Парсит содержимое вкладок
fn parse_tabs_content(container: &Container) -> SaltWidget {
    let mut items = Vec::new();

    if let Some(first_row) = container.rows.first() {
        for widget in first_row {
            if let SaltWidget::Text(text) = widget {
                items.push(text.clone());
            }
        }
    }

    SaltWidget::Tabs { items, selected: 0 }
}

/// Парсит содержимое меню
fn parse_menu_content(container: &Container) -> SaltWidget {
    use plantuml_ast::salt::MenuItem;

    let mut items = Vec::new();

    if let Some(first_row) = container.rows.first() {
        for widget in first_row {
            if let SaltWidget::Text(text) = widget {
                if text == "-" {
                    items.push(MenuItem::separator());
                } else {
                    items.push(MenuItem::new(text.clone()));
                }
            }
        }
    }

    SaltWidget::Menu { items }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_salt() {
        let source = r#"@startsalt
{
  Login | "MyName"
}
@endsalt"#;

        let diagram = parse_salt(source).unwrap();
        if let SaltWidget::Container(container) = &diagram.root {
            assert!(!container.rows.is_empty());
        } else {
            panic!("Expected Container");
        }
    }

    #[test]
    fn test_parse_with_border() {
        let source = r#"@startsalt
{+
  Login | "MyName"
}
@endsalt"#;

        let diagram = parse_salt(source).unwrap();
        if let SaltWidget::Container(container) = &diagram.root {
            assert_eq!(container.border_style, BorderStyle::External);
        } else {
            panic!("Expected Container");
        }
    }

    #[test]
    fn test_parse_checkbox_radio() {
        let source = r#"@startsalt
{
  [X]
  []
  (X)
  ()
}
@endsalt"#;

        let diagram = parse_salt(source).unwrap();
        if let SaltWidget::Container(container) = &diagram.root {
            assert!(!container.rows.is_empty());
        } else {
            panic!("Expected Container");
        }
    }

    #[test]
    fn test_parse_droplist() {
        let source = r#"@startsalt
{
  ^Dropdown^
}
@endsalt"#;

        let diagram = parse_salt(source).unwrap();
        // Просто проверяем что парсится без ошибок
        assert!(matches!(diagram.root, SaltWidget::Container(_)));
    }
}
