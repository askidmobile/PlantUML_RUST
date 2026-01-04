//! Парсер Class Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML class diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::class::{
    ClassDiagram, Classifier, ClassifierType, Member, Package, Relationship, RelationshipType,
    Visibility,
};
use plantuml_ast::common::{Color, LineStyle, Stereotype};

use crate::{ParseError, Result};

#[derive(Parser)]
#[grammar = "grammars/class.pest"]
pub struct ClassParser;

/// Парсит class diagram из исходного кода
pub fn parse_class(source: &str) -> Result<ClassDiagram> {
    let pairs = ClassParser::parse(Rule::diagram, source).map_err(|e| ParseError::SyntaxError {
        line: e.line().to_string().parse().unwrap_or(0),
        message: e.to_string(),
    })?;

    let mut diagram = ClassDiagram::new();
    let mut package_stack: Vec<Package> = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::diagram {
            for inner in pair.into_inner() {
                process_rule(inner, &mut diagram, &mut package_stack);
            }
        }
    }

    Ok(diagram)
}

/// Обрабатывает правило грамматики
fn process_rule(
    pair: pest::iterators::Pair<Rule>,
    diagram: &mut ClassDiagram,
    package_stack: &mut Vec<Package>,
) {
    match pair.as_rule() {
        Rule::class_decl => {
            if let Some(result) = parse_class_decl_with_inheritance(pair, ClassifierType::Class) {
                let class_name = result.classifier.id.name.clone();
                add_classifier(result.classifier, diagram, package_stack);
                // Создаём relationship для extends
                if let Some(parent) = result.extends {
                    diagram.add_relationship(Relationship {
                        from: class_name.clone(),
                        to: parent,
                        relationship_type: RelationshipType::Inheritance,
                        label: None,
                        from_cardinality: None,
                        to_cardinality: None,
                        line_style: plantuml_ast::common::LineStyle::Solid,
                        direction: None,
                    });
                }
                // Создаём relationship для implements
                for iface in result.implements {
                    diagram.add_relationship(Relationship {
                        from: class_name.clone(),
                        to: iface,
                        relationship_type: RelationshipType::Realization,
                        label: None,
                        from_cardinality: None,
                        to_cardinality: None,
                        line_style: plantuml_ast::common::LineStyle::Dashed,
                        direction: None,
                    });
                }
            }
        }
        Rule::interface_decl => {
            if let Some(result) = parse_class_decl_with_inheritance(pair, ClassifierType::Interface) {
                let class_name = result.classifier.id.name.clone();
                add_classifier(result.classifier, diagram, package_stack);
                // Интерфейсы тоже могут наследовать от других интерфейсов
                if let Some(parent) = result.extends {
                    diagram.add_relationship(Relationship {
                        from: class_name.clone(),
                        to: parent,
                        relationship_type: RelationshipType::Inheritance,
                        label: None,
                        from_cardinality: None,
                        to_cardinality: None,
                        line_style: plantuml_ast::common::LineStyle::Solid,
                        direction: None,
                    });
                }
            }
        }
        Rule::abstract_decl => {
            if let Some(result) = parse_class_decl_with_inheritance(pair, ClassifierType::AbstractClass) {
                let class_name = result.classifier.id.name.clone();
                add_classifier(result.classifier, diagram, package_stack);
                if let Some(parent) = result.extends {
                    diagram.add_relationship(Relationship {
                        from: class_name.clone(),
                        to: parent,
                        relationship_type: RelationshipType::Inheritance,
                        label: None,
                        from_cardinality: None,
                        to_cardinality: None,
                        line_style: plantuml_ast::common::LineStyle::Solid,
                        direction: None,
                    });
                }
                for iface in result.implements {
                    diagram.add_relationship(Relationship {
                        from: class_name.clone(),
                        to: iface,
                        relationship_type: RelationshipType::Realization,
                        label: None,
                        from_cardinality: None,
                        to_cardinality: None,
                        line_style: plantuml_ast::common::LineStyle::Dashed,
                        direction: None,
                    });
                }
            }
        }
        Rule::enum_decl => {
            if let Some(classifier) = parse_class_decl(pair, ClassifierType::Enum) {
                add_classifier(classifier, diagram, package_stack);
            }
        }
        Rule::annotation_decl => {
            if let Some(classifier) = parse_class_decl(pair, ClassifierType::Annotation) {
                add_classifier(classifier, diagram, package_stack);
            }
        }
        Rule::relationship => {
            if let Some(rel) = parse_relationship(pair) {
                diagram.add_relationship(rel);
            }
        }
        Rule::package_start => {
            let pkg = parse_package_start(pair);
            package_stack.push(pkg);
        }
        Rule::package_end => {
            if let Some(pkg) = package_stack.pop() {
                if package_stack.is_empty() {
                    diagram.packages.push(pkg);
                } else {
                    package_stack.last_mut().unwrap().packages.push(pkg);
                }
            }
        }
        Rule::title_stmt => {
            if let Some(title) = parse_title(pair) {
                diagram.metadata.title = Some(title);
            }
        }
        _ => {}
    }
}

/// Добавляет классификатор в диаграмму или текущий пакет
fn add_classifier(
    classifier: Classifier,
    diagram: &mut ClassDiagram,
    package_stack: &mut [Package],
) {
    if package_stack.is_empty() {
        diagram.add_class(classifier);
    } else {
        package_stack
            .last_mut()
            .unwrap()
            .classifiers
            .push(classifier);
    }
}

/// Результат парсинга объявления класса
struct ClassDeclResult {
    classifier: Classifier,
    extends: Option<String>,
    implements: Vec<String>,
}

/// Парсит объявление класса/интерфейса/enum
fn parse_class_decl(
    pair: pest::iterators::Pair<Rule>,
    default_type: ClassifierType,
) -> Option<Classifier> {
    parse_class_decl_with_inheritance(pair, default_type).map(|r| r.classifier)
}

/// Парсит объявление класса с информацией о наследовании
fn parse_class_decl_with_inheritance(
    pair: pest::iterators::Pair<Rule>,
    default_type: ClassifierType,
) -> Option<ClassDeclResult> {
    let mut name = String::new();
    let mut classifier_type = default_type;
    let mut stereotype: Option<Stereotype> = None;
    let mut color: Option<Color> = None;
    let mut generics: Option<String> = None;
    let mut fields: Vec<Member> = Vec::new();
    let mut methods: Vec<Member> = Vec::new();
    let mut extends: Option<String> = None;
    let mut implements: Vec<String> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::class_keyword => {
                let kw = inner.as_str().to_lowercase();
                if kw.contains("abstract") {
                    classifier_type = ClassifierType::AbstractClass;
                }
            }
            Rule::class_name | Rule::qualified_name => {
                name = extract_name(inner);
            }
            Rule::stereotype => {
                let s = inner.as_str();
                let content = s.trim_start_matches("<<").trim_end_matches(">>");
                stereotype = Some(Stereotype::new(content));
            }
            Rule::color => {
                color = Some(Color::from_hex(inner.as_str()));
            }
            Rule::generic_params => {
                generics = Some(inner.as_str().to_string());
            }
            Rule::extends_clause => {
                // extends_clause = { "extends" ~ ws+ ~ class_name }
                for ext_inner in inner.into_inner() {
                    if ext_inner.as_rule() == Rule::class_name {
                        extends = Some(extract_name(ext_inner));
                    }
                }
            }
            Rule::implements_clause => {
                // implements_clause = { "implements" ~ ws+ ~ class_name ~ ("," ~ ws* ~ class_name)* }
                for impl_inner in inner.into_inner() {
                    if impl_inner.as_rule() == Rule::class_name {
                        implements.push(extract_name(impl_inner));
                    }
                }
            }
            Rule::class_body | Rule::enum_body => {
                parse_class_body(inner, &mut fields, &mut methods);
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(ClassDeclResult {
        classifier: Classifier {
            id: plantuml_ast::common::Identifier::new(name),
            classifier_type,
            fields,
            methods,
            stereotype,
            background_color: color,
            border_color: None,
            generics,
        },
        extends,
        implements,
    })
}

/// Парсит тело класса
fn parse_class_body(
    pair: pest::iterators::Pair<Rule>,
    fields: &mut Vec<Member>,
    methods: &mut Vec<Member>,
) {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::member => {
                for member_inner in inner.into_inner() {
                    match member_inner.as_rule() {
                        Rule::field => {
                            if let Some(field) = parse_field(member_inner) {
                                fields.push(field);
                            }
                        }
                        Rule::method => {
                            if let Some(method) = parse_method(member_inner) {
                                methods.push(method);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Rule::field => {
                if let Some(field) = parse_field(inner) {
                    fields.push(field);
                }
            }
            Rule::method => {
                if let Some(method) = parse_method(inner) {
                    methods.push(method);
                }
            }
            Rule::enum_value => {
                // Для enum значения добавляем как поля
                let value = inner.as_str().trim().to_string();
                if !value.is_empty() {
                    fields.push(Member {
                        name: value,
                        member_type: None,
                        visibility: Visibility::Public,
                        is_static: false,
                        is_abstract: false,
                        parameters: Vec::new(),
                    });
                }
            }
            _ => {}
        }
    }
}

/// Парсит поле класса
fn parse_field(pair: pest::iterators::Pair<Rule>) -> Option<Member> {
    let mut name = String::new();
    let mut field_type: Option<String> = None;
    let mut visibility = Visibility::Private;
    let mut is_static = false;
    let mut is_abstract = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::visibility => {
                visibility = parse_visibility(inner.as_str());
            }
            Rule::modifier => {
                let mod_str = inner.as_str().to_lowercase();
                if mod_str.contains("static") {
                    is_static = true;
                }
                if mod_str.contains("abstract") {
                    is_abstract = true;
                }
            }
            Rule::field_name => {
                name = inner.as_str().to_string();
            }
            Rule::field_type => {
                field_type = Some(inner.as_str().trim().to_string());
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Member {
        name,
        member_type: field_type,
        visibility,
        is_static,
        is_abstract,
        parameters: Vec::new(),
    })
}

/// Парсит метод класса
fn parse_method(pair: pest::iterators::Pair<Rule>) -> Option<Member> {
    let mut name = String::new();
    let mut return_type: Option<String> = None;
    let mut visibility = Visibility::Public;
    let mut is_static = false;
    let mut is_abstract = false;
    let mut parameters: Vec<plantuml_ast::class::Parameter> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::visibility => {
                visibility = parse_visibility(inner.as_str());
            }
            Rule::modifier => {
                let mod_str = inner.as_str().to_lowercase();
                if mod_str.contains("static") {
                    is_static = true;
                }
                if mod_str.contains("abstract") {
                    is_abstract = true;
                }
            }
            Rule::method_name => {
                name = inner.as_str().to_string();
            }
            Rule::method_params => {
                parameters = parse_method_params(inner);
            }
            Rule::return_type => {
                return_type = Some(inner.as_str().trim().to_string());
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Member {
        name,
        member_type: return_type,
        visibility,
        is_static,
        is_abstract,
        parameters,
    })
}

/// Парсит параметры метода
fn parse_method_params(pair: pest::iterators::Pair<Rule>) -> Vec<plantuml_ast::class::Parameter> {
    let mut params = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::method_param {
            let mut param_name = String::new();
            let mut param_type = String::new();

            for param_inner in inner.into_inner() {
                match param_inner.as_rule() {
                    Rule::param_name => {
                        param_name = param_inner.as_str().to_string();
                    }
                    Rule::param_type => {
                        param_type = param_inner.as_str().trim().to_string();
                    }
                    _ => {}
                }
            }

            if !param_name.is_empty() {
                params.push(plantuml_ast::class::Parameter {
                    name: param_name,
                    param_type,
                });
            }
        }
    }

    params
}

/// Парсит видимость
fn parse_visibility(s: &str) -> Visibility {
    match s.trim() {
        "+" => Visibility::Public,
        "-" => Visibility::Private,
        "#" => Visibility::Protected,
        "~" => Visibility::Package,
        "{method}" => Visibility::Public,
        "{field}" => Visibility::Private,
        _ => Visibility::Private,
    }
}

/// Парсит отношение
fn parse_relationship(pair: pest::iterators::Pair<Rule>) -> Option<Relationship> {
    let mut from = String::new();
    let mut to = String::new();
    let mut label: Option<String> = None;
    let mut rel_type = RelationshipType::Association;
    let mut line_style = LineStyle::Solid;
    let mut from_cardinality: Option<String> = None;
    let mut to_cardinality: Option<String> = None;
    let mut seen_arrow = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::class_ref | Rule::qualified_name => {
                let name = extract_name(inner);
                if from.is_empty() {
                    from = name;
                } else {
                    to = name;
                }
            }
            Rule::cardinality => {
                let card = extract_cardinality(inner);
                if !seen_arrow {
                    // Кардинальность до стрелки - это from_cardinality
                    from_cardinality = Some(card);
                } else {
                    // Кардинальность после стрелки - это to_cardinality
                    to_cardinality = Some(card);
                }
            }
            Rule::relationship_arrow => {
                seen_arrow = true;
                let (rtype, lstyle) = parse_arrow(inner);
                rel_type = rtype;
                line_style = lstyle;
            }
            Rule::relationship_label => {
                let text = inner.as_str().trim();
                if !text.is_empty() {
                    label = Some(text.to_string());
                }
            }
            _ => {}
        }
    }

    if from.is_empty() || to.is_empty() {
        return None;
    }

    Some(Relationship {
        from,
        to,
        relationship_type: rel_type,
        label,
        from_cardinality,
        to_cardinality,
        line_style,
        direction: None,
    })
}

/// Извлекает значение кардинальности из кавычек
fn extract_cardinality(pair: pest::iterators::Pair<Rule>) -> String {
    let fallback = pair.as_str().trim_matches('"').to_string();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::cardinality_value {
            return inner.as_str().to_string();
        }
    }
    fallback
}

/// Парсит стрелку отношения
fn parse_arrow(pair: pest::iterators::Pair<Rule>) -> (RelationshipType, LineStyle) {
    let mut left_side = "";
    let mut line = "";
    let mut right_side = "";

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::arrow_left_side => {
                left_side = inner.as_str();
            }
            Rule::arrow_line => {
                line = inner.as_str();
            }
            Rule::arrow_right_side => {
                right_side = inner.as_str();
            }
            _ => {}
        }
    }

    let line_style = if line.contains("..") || line == "." {
        LineStyle::Dashed
    } else {
        LineStyle::Solid
    };

    // Определяем тип отношения по комбинации left + right
    let rel_type = match (left_side, right_side) {
        ("<|", _) | (_, "|>") => {
            if line_style == LineStyle::Dashed {
                RelationshipType::Realization
            } else {
                RelationshipType::Inheritance
            }
        }
        ("*", _) | (_, "*") => RelationshipType::Composition,
        ("o", _) | (_, "o") => RelationshipType::Aggregation,
        ("<", _) | (_, ">") => {
            if line_style == LineStyle::Dashed {
                RelationshipType::Dependency
            } else {
                RelationshipType::Association
            }
        }
        _ => RelationshipType::Link,
    };

    (rel_type, line_style)
}

/// Парсит начало пакета
fn parse_package_start(pair: pest::iterators::Pair<Rule>) -> Package {
    let mut name = String::new();
    let mut stereotype: Option<Stereotype> = None;
    let mut color: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::package_name | Rule::identifier => {
                name = extract_name(inner);
            }
            Rule::package_style => {
                let s = inner.as_str();
                let content = s.trim_start_matches("<<").trim_end_matches(">>");
                stereotype = Some(Stereotype::new(content));
            }
            Rule::color => {
                color = Some(Color::from_hex(inner.as_str()));
            }
            _ => {}
        }
    }

    Package {
        name,
        stereotype,
        classifiers: Vec::new(),
        packages: Vec::new(),
        background_color: color,
    }
}

/// Парсит заголовок
fn parse_title(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::rest_of_line {
            let text = inner.as_str().trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

/// Извлекает имя из quoted_string или identifier
fn extract_name(pair: pest::iterators::Pair<Rule>) -> String {
    let fallback = pair.as_str().trim_matches('"').to_string();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::quoted_string | Rule::inner_string => {
                return inner.as_str().trim_matches('"').to_string();
            }
            Rule::identifier | Rule::qualified_name => {
                return inner.as_str().to_string();
            }
            _ => {}
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_class() {
        let source = r#"@startuml
class User {
    -id: Long
    -name: String
    +getId(): Long
    +setName(name: String): void
}
@enduml"#;

        let result = parse_class(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.classifiers.len(), 1);

        let user = &diagram.classifiers[0];
        assert_eq!(user.id.name, "User");
        assert_eq!(user.classifier_type, ClassifierType::Class);
        assert_eq!(user.fields.len(), 2);
        assert_eq!(user.methods.len(), 2);
    }

    #[test]
    fn test_parse_interface() {
        let source = r#"@startuml
interface Runnable {
    +run(): void
}
@enduml"#;

        let result = parse_class(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.classifiers.len(), 1);
        assert_eq!(
            diagram.classifiers[0].classifier_type,
            ClassifierType::Interface
        );
    }

    #[test]
    fn test_parse_enum() {
        let source = r#"@startuml
enum Status {
    PENDING
    ACTIVE
    CLOSED
}
@enduml"#;

        let result = parse_class(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.classifiers.len(), 1);
        assert_eq!(diagram.classifiers[0].classifier_type, ClassifierType::Enum);
        assert_eq!(diagram.classifiers[0].fields.len(), 3);
    }

    #[test]
    fn test_parse_inheritance() {
        let source = r#"@startuml
class Animal
class Dog
Dog --|> Animal
@enduml"#;

        let result = parse_class(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.classifiers.len(), 2);
        assert_eq!(diagram.relationships.len(), 1);

        let rel = &diagram.relationships[0];
        assert_eq!(rel.from, "Dog");
        assert_eq!(rel.to, "Animal");
        assert_eq!(rel.relationship_type, RelationshipType::Inheritance);
    }

    #[test]
    fn test_parse_realization() {
        let source = r#"@startuml
interface Flyable
class Bird
Bird ..|> Flyable
@enduml"#;

        let result = parse_class(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.relationships.len(), 1);
        assert_eq!(
            diagram.relationships[0].relationship_type,
            RelationshipType::Realization
        );
    }

    #[test]
    fn test_parse_composition_aggregation() {
        let source = r#"@startuml
class Car
class Engine
class Wheel

Car *-- Engine : contains
Car o-- Wheel : has
@enduml"#;

        let result = parse_class(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.relationships.len(), 2);

        assert_eq!(
            diagram.relationships[0].relationship_type,
            RelationshipType::Composition
        );
        assert_eq!(
            diagram.relationships[1].relationship_type,
            RelationshipType::Aggregation
        );
    }

    #[test]
    fn test_parse_package() {
        let source = r#"@startuml
package "com.example" {
    class User
    class Order
}
@enduml"#;

        let result = parse_class(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.packages.len(), 1);
        assert_eq!(diagram.packages[0].name, "com.example");
        assert_eq!(diagram.packages[0].classifiers.len(), 2);
    }

    #[test]
    fn test_parse_cardinality() {
        let source = r#"@startuml
class User
class Order
User "1" -- "*" Order : places
@enduml"#;

        let result = parse_class(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.classifiers.len(), 2);
        assert_eq!(diagram.relationships.len(), 1);

        let rel = &diagram.relationships[0];
        assert_eq!(rel.from, "User");
        assert_eq!(rel.to, "Order");
        assert_eq!(rel.from_cardinality, Some("1".to_string()));
        assert_eq!(rel.to_cardinality, Some("*".to_string()));
        assert_eq!(rel.label, Some("places".to_string()));
    }

    #[test]
    fn test_parse_cardinality_complex() {
        let source = r#"@startuml
class Customer
class Order
Customer "1" --o "0..*" Order
@enduml"#;

        let result = parse_class(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        let rel = &diagram.relationships[0];
        assert_eq!(rel.from_cardinality, Some("1".to_string()));
        assert_eq!(rel.to_cardinality, Some("0..*".to_string()));
        assert_eq!(rel.relationship_type, RelationshipType::Aggregation);
    }
}
