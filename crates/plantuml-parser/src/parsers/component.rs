//! Парсер Component Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML component diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::common::{Color, Note, NotePosition, Stereotype};
use plantuml_ast::component::{
    Component, ComponentDiagram, ComponentPackage, ComponentType, Connection, PackageType,
};

use crate::error::syntax_error_from_pest;
use crate::Result;

#[derive(Parser)]
#[grammar = "grammars/component.pest"]
pub struct ComponentParser;

/// Парсит component diagram из исходного кода
pub fn parse_component(source: &str) -> Result<ComponentDiagram> {
    let pairs = ComponentParser::parse(Rule::diagram, source)
        .map_err(|e| syntax_error_from_pest(&e, source))?;

    let mut diagram = ComponentDiagram::new();

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

/// Разбирает `archimate #Layer "Name" as alias`.
///
/// Слой Archimate определяет и цвет заливки, и стереотип элемента.
fn parse_archimate_def(pair: pest::iterators::Pair<Rule>) -> Option<Component> {
    use plantuml_ast::common::Color;

    let mut name = String::new();
    let mut alias = None;
    let mut layer = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::archimate_layer => {
                // Убираем ведущий '#'
                layer = Some(inner.as_str().trim_start_matches('#').to_string());
            }
            Rule::quoted_string => {
                name = inner.as_str().trim_matches('"').to_string();
            }
            Rule::identifier => {
                alias = Some(inner.as_str().to_string());
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    // Цвета слоёв Archimate по документации PlantUML
    let (color, stereotype) = match layer.as_deref() {
        Some("Business") => ("#FFFFCC", "business"),
        Some("Application") => ("#CCFFFF", "application"),
        Some("Technology") => ("#CCFFCC", "technology"),
        Some("Motivation") => ("#CCCCFF", "motivation"),
        Some("Strategy") => ("#FFCCFF", "strategy"),
        Some("Physical") => ("#FFEECC", "physical"),
        Some("Implementation") => ("#FFCCCC", "implementation"),
        _ => ("#FFFFFF", "archimate"),
    };

    let mut component = Component::new(name);
    component.alias = alias;
    component.color = Some(Color::from_hex(color));
    component.stereotype = Some(plantuml_ast::common::Stereotype::new(stereotype));
    Some(component)
}

/// Парсит тело диаграммы
fn parse_body(pair: pest::iterators::Pair<Rule>, diagram: &mut ComponentDiagram) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::component_def => {
                if let Some(comp) = parse_component_def(inner) {
                    diagram.components.push(comp);
                }
            }
            Rule::archimate_def => {
                // Archimate-элемент: слой (#Business, #Application, ...)
                // задаёт цвет и стереотип. Ранее правило разбиралось
                // грамматикой, но здесь не обрабатывалось, поэтому
                // диаграмма получалась пустой.
                if let Some(comp) = parse_archimate_def(inner) {
                    diagram.components.push(comp);
                }
            }
            Rule::interface_def => {
                if let Some(iface) = parse_interface_def(inner) {
                    diagram.components.push(iface);
                }
            }
            Rule::package_def | Rule::nested_package_def => {
                if let Some(pkg) = parse_package_def(inner) {
                    diagram.packages.push(pkg);
                }
            }
            Rule::connection => {
                if let Some(conn) = parse_connection(inner) {
                    diagram.connections.push(conn);
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

/// Парсит определение компонента (простой, без тела)
fn parse_component_def(pair: pest::iterators::Pair<Rule>) -> Option<Component> {
    let mut name = String::new();
    let mut alias: Option<String> = None;
    let mut component_type = ComponentType::Component;
    let mut stereotype: Option<Stereotype> = None;
    let mut color: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::component_keyword | Rule::container_keyword | Rule::non_container_keyword => {
                component_type = parse_component_type(inner.as_str());
            }
            Rule::component_name => {
                name = extract_name(inner);
            }
            Rule::bracket_component => {
                name = extract_bracket_content(inner);
            }
            Rule::alias_part => {
                alias = extract_alias(inner);
            }
            Rule::stereotype_part => {
                stereotype = extract_stereotype(inner);
            }
            Rule::color_part => {
                color = extract_color(inner);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Component {
        name,
        alias,
        component_type,
        stereotype,
        color,
        children: Vec::new(),
        interfaces: Vec::new(),
        ports: Vec::new(),
    })
}

/// Парсит определение интерфейса
fn parse_interface_def(pair: pest::iterators::Pair<Rule>) -> Option<Component> {
    let mut name = String::new();
    let mut alias: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::interface_provided | Rule::interface_required | Rule::interface_simple => {
                for iface_inner in inner.into_inner() {
                    match iface_inner.as_rule() {
                        Rule::interface_name => {
                            name = extract_name(iface_inner);
                        }
                        Rule::alias_part => {
                            alias = extract_alias(iface_inner);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Component {
        name,
        alias,
        component_type: ComponentType::Interface,
        stereotype: None,
        color: None,
        children: Vec::new(),
        interfaces: Vec::new(),
        ports: Vec::new(),
    })
}

/// Парсит пакет
fn parse_package_def(pair: pest::iterators::Pair<Rule>) -> Option<ComponentPackage> {
    let mut name = String::new();
    let mut package_type = PackageType::Package;
    let mut stereotype: Option<Stereotype> = None;
    let mut color: Option<Color> = None;
    let mut components = Vec::new();
    let mut packages = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::package_keyword | Rule::nested_package_keyword => {
                package_type = parse_package_type(inner.as_str());
            }
            Rule::package_name => {
                name = extract_name(inner);
            }
            Rule::stereotype_part => {
                stereotype = extract_stereotype(inner);
            }
            Rule::color_part => {
                color = extract_color(inner);
            }
            Rule::package_body => {
                // Парсим содержимое пакета
                parse_package_body(inner, &mut components, &mut packages);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(ComponentPackage {
        name,
        package_type,
        stereotype,
        color,
        components,
        packages,
    })
}

/// Парсит содержимое пакета
fn parse_package_body(
    pair: pest::iterators::Pair<Rule>,
    components: &mut Vec<Component>,
    packages: &mut Vec<ComponentPackage>,
) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::component_def => {
                if let Some(comp) = parse_component_def(inner) {
                    components.push(comp);
                }
            }
            Rule::interface_def => {
                if let Some(iface) = parse_interface_def(inner) {
                    components.push(iface);
                }
            }
            Rule::package_def | Rule::nested_package_def => {
                if let Some(pkg) = parse_package_def(inner) {
                    packages.push(pkg);
                }
            }
            _ => {}
        }
    }
}

/// Парсит связь
fn parse_connection(pair: pest::iterators::Pair<Rule>) -> Option<Connection> {
    let mut from = String::new();
    let mut to = String::new();
    let mut label: Option<String> = None;
    let mut dashed = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::connection_from => {
                from = extract_connection_endpoint(inner);
            }
            Rule::connection_to => {
                to = extract_connection_endpoint(inner);
            }
            Rule::arrow => {
                // Проверяем тип стрелки
                for arrow_inner in inner.into_inner() {
                    if arrow_inner.as_rule() == Rule::arrow_dashed {
                        dashed = true;
                    }
                }
            }
            Rule::connection_label => {
                label = extract_label(inner);
            }
            _ => {}
        }
    }

    if from.is_empty() || to.is_empty() {
        return None;
    }

    Some(Connection {
        from,
        to,
        connection_type: plantuml_ast::component::ConnectionType::Simple,
        label,
        color: None,
        dashed,
    })
}

/// Извлекает endpoint связи
fn extract_connection_endpoint(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::bracket_component => {
                return extract_bracket_content(inner);
            }
            Rule::interface_ref => {
                // Извлекаем имя интерфейса
                for iface_inner in inner.into_inner() {
                    if iface_inner.as_rule() == Rule::simple_identifier {
                        return iface_inner.as_str().to_string();
                    }
                }
            }
            Rule::simple_identifier => {
                return inner.as_str().to_string();
            }
            // Имя в кавычках: `"app.jar" --> pg`. Грамматика эту форму
            // принимала, но парсер её не знал — endpoint получался пустым,
            // и связь отбрасывалась целиком из-за проверки на пустоту.
            Rule::quoted_string => {
                return inner.as_str().trim().trim_matches('"').to_string();
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
            Rule::note_on_component | Rule::note_multiline => {
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

fn parse_component_type(s: &str) -> ComponentType {
    ComponentType::parse(s).unwrap_or(ComponentType::Component)
}

fn parse_package_type(s: &str) -> PackageType {
    match s.to_lowercase().as_str() {
        "package" => PackageType::Package,
        "node" => PackageType::Node,
        "folder" => PackageType::Folder,
        "frame" => PackageType::Frame,
        "cloud" => PackageType::Cloud,
        "rectangle" => PackageType::Rectangle,
        _ => PackageType::Package,
    }
}

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

fn extract_bracket_content(pair: pest::iterators::Pair<Rule>) -> String {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::bracket_content {
            return inner.as_str().trim().to_string();
        }
    }
    String::new()
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

fn extract_color(pair: pest::iterators::Pair<Rule>) -> Option<Color> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::color {
            let text = inner.as_str();
            if let Some(hex) = text.strip_prefix('#') {
                return Some(Color::from_hex(hex));
            }
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
            Rule::bracket_component => return extract_bracket_content(inner),
            Rule::simple_identifier => return inner.as_str().to_string(),
            _ => {}
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Связь с именем в кавычках не теряется.
    ///
    /// Регрессия: `extract_connection_endpoint` не знал `quoted_string`,
    /// поэтому endpoint получался пустым, и связь отбрасывалась целиком
    /// из-за проверки `from.is_empty() || to.is_empty()`.
    #[test]
    fn test_connection_with_quoted_endpoints() {
        let source = "@startuml\nnode \"Сервер\" {\n  artifact \"app.jar\"\n}\ndatabase pg as pg\n\"app.jar\" --> pg : JDBC\n@enduml";
        let diagram = parse_component(source).expect("диаграмма должна разбираться");

        assert_eq!(diagram.connections.len(), 1, "связь потеряна");
        assert_eq!(diagram.connections[0].from, "app.jar");
        assert_eq!(diagram.connections[0].to, "pg");
        assert_eq!(diagram.connections[0].label.as_deref(), Some("JDBC"));
    }

    #[test]
    fn test_parse_simple_component() {
        let source = r#"
@startuml
component API
database MySQL
cloud AWS
@enduml
"#;

        let diagram = parse_component(source).unwrap();
        assert_eq!(diagram.components.len(), 3);

        assert_eq!(diagram.components[0].name, "API");
        assert_eq!(
            diagram.components[0].component_type,
            ComponentType::Component
        );

        assert_eq!(diagram.components[1].name, "MySQL");
        assert_eq!(
            diagram.components[1].component_type,
            ComponentType::Database
        );

        assert_eq!(diagram.components[2].name, "AWS");
        assert_eq!(diagram.components[2].component_type, ComponentType::Cloud);
    }

    #[test]
    fn test_parse_bracket_component() {
        let source = r#"
@startuml
[User Service]
[Order Service]
[User Service] --> [Order Service]
@enduml
"#;

        let diagram = parse_component(source).unwrap();
        assert_eq!(diagram.components.len(), 2);
        assert_eq!(diagram.connections.len(), 1);

        assert_eq!(diagram.connections[0].from, "User Service");
        assert_eq!(diagram.connections[0].to, "Order Service");
    }

    #[test]
    fn test_parse_connection_with_label() {
        let source = r#"
@startuml
component API
database DB
API --> DB : uses
API ..> DB : optional
@enduml
"#;

        let diagram = parse_component(source).unwrap();
        assert_eq!(diagram.connections.len(), 2);

        assert_eq!(diagram.connections[0].label, Some("uses".to_string()));
        assert!(!diagram.connections[0].dashed);

        assert_eq!(diagram.connections[1].label, Some("optional".to_string()));
        assert!(diagram.connections[1].dashed);
    }

    #[test]
    fn test_parse_package() {
        let source = r#"
@startuml
package "Backend" {
    component API
    component Worker
}
@enduml
"#;

        let diagram = parse_component(source).unwrap();
        assert_eq!(diagram.packages.len(), 1);
        assert_eq!(diagram.packages[0].name, "Backend");
        assert_eq!(diagram.packages[0].components.len(), 2);
    }

    #[test]
    fn test_parse_interface() {
        let source = r#"
@startuml
interface HTTP
() REST
component API
API --> HTTP
@enduml
"#;

        let diagram = parse_component(source).unwrap();
        // interface и () создают компоненты типа Interface
        let interfaces: Vec<_> = diagram
            .components
            .iter()
            .filter(|c| c.component_type == ComponentType::Interface)
            .collect();
        assert!(!interfaces.is_empty());
    }

    #[test]
    fn test_parse_component_with_alias() {
        let source = r#"
@startuml
component "User Management Service" as UMS
database "PostgreSQL Database" as DB
UMS --> DB
@enduml
"#;

        let diagram = parse_component(source).unwrap();
        assert_eq!(diagram.components.len(), 2);

        assert_eq!(diagram.components[0].name, "User Management Service");
        assert_eq!(diagram.components[0].alias, Some("UMS".to_string()));

        assert_eq!(diagram.components[1].name, "PostgreSQL Database");
        assert_eq!(diagram.components[1].alias, Some("DB".to_string()));
    }

    #[test]
    fn test_parse_nested_node() {
        let source = r#"
@startuml
node "Outer" {
    node "Inner" {
        [Component]
    }
}
@enduml
"#;

        let diagram = parse_component(source).unwrap();
        assert_eq!(diagram.packages.len(), 1, "Should have 1 outer package");
        assert_eq!(diagram.packages[0].name, "Outer");
        assert_eq!(
            diagram.packages[0].packages.len(),
            1,
            "Should have 1 nested package"
        );
        assert_eq!(diagram.packages[0].packages[0].name, "Inner");
        assert_eq!(
            diagram.packages[0].packages[0].components.len(),
            1,
            "Should have 1 component"
        );
    }
}
