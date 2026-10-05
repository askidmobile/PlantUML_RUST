//! Парсер Class Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML class diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::class::{
    ClassDiagram, Classifier, ClassifierType, Member, Package, Relationship, RelationshipType,
    Visibility,
};
use plantuml_ast::common::{Color, LineStyle, Note, NotePosition, Stereotype};

use crate::error::syntax_error_from_pest;
use crate::Result;

#[derive(Parser)]
#[grammar = "grammars/class.pest"]
pub struct ClassParser;

/// Парсит class diagram из исходного кода
pub fn parse_class(source: &str) -> Result<ClassDiagram> {
    let pairs = ClassParser::parse(Rule::diagram, source)
        .map_err(|e| syntax_error_from_pest(&e, source))?;

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
            if let Some(result) = parse_class_decl_with_inheritance(pair, ClassifierType::Interface)
            {
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
            if let Some(result) =
                parse_class_decl_with_inheritance(pair, ClassifierType::AbstractClass)
            {
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
        Rule::direction_stmt => {
            // `left to right direction`. Грамматика строку принимала, но
            // ни один парсер её не обрабатывал — направление терялось.
            if let Some(direction) = parse_direction_text(pair.as_str()) {
                diagram.metadata.direction = Some(direction);
            }
        }
        Rule::sprite_stmt => {
            // Спрайт: `sprite $имя [ШxВ/цветов] { ...hex... }`.
            // Грамматика его принимала, но парсер отбрасывал — содержимое
            // библиотек иконок (logos, office, tupadr3) терялось.
            if let Some(sprite) = parse_sprite(pair) {
                diagram.sprites.push(sprite);
            }
        }
        Rule::note_stmt => {
            // Заметки class-диаграмм. Грамматика их знала и раньше
            // принимала, но парсер не обрабатывал — заметка молча терялась,
            // и в выводе не было ни текста, ни рамки.
            if let Some(note) = parse_note(pair) {
                diagram.notes.push(note);
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
    let mut alias: Option<String> = None;
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
            // `class "Длинное имя" as short`. Грамматика алиас принимала,
            // но парсер его не читал — алиас молча терялся, и связи по
            // нему не находились.
            Rule::alias_part => {
                alias = inner
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::identifier || p.as_rule() == Rule::quoted_string)
                    .map(|p| p.as_str().trim().trim_matches('"').to_string());
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
            id: plantuml_ast::common::Identifier { name, alias },
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
/// Разбирает заметку class-диаграммы.
///
/// Поддерживаются те же три формы, что и в component-грамматике:
/// `note right of A : текст`, `note "текст" as N` и многострочная
/// `note right of A ... end note`.
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
                            let target = n.as_str().trim().trim_matches('"').to_string();
                            if !target.is_empty() {
                                anchors.push(target);
                            }
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
                    match n.as_rule() {
                        Rule::quoted_string => {
                            text = n.as_str().trim().trim_matches('"').to_string();
                        }
                        // Идентификатор — имя заметки (`note "текст" as N`).
                        // Оно попадает в anchors: так заметку можно привязать
                        // к элементу по имени.
                        Rule::identifier if !text.is_empty() => {
                            anchors.push(n.as_str().to_string());
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

/// Разбирает позицию заметки.
fn parse_note_position(s: &str) -> NotePosition {
    match s.to_lowercase().as_str() {
        "left" => NotePosition::Left,
        "right" => NotePosition::Right,
        "top" => NotePosition::Top,
        "bottom" => NotePosition::Bottom,
        _ => NotePosition::Right,
    }
}

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

/// Разбирает направление раскладки из `direction_stmt`.
///
/// Возвращает `None`, если строка не распознана.
pub fn parse_direction_text(text: &str) -> Option<plantuml_ast::common::Direction> {
    use plantuml_ast::common::Direction;

    // Порядок слов ВАЖЕН: `top to bottom` и `bottom to top` содержат одни
    // и те же слова, различает их только последовательность.
    let lower = text.to_lowercase();
    let top = lower.find("top");
    let bottom = lower.find("bottom");
    let left = lower.find("left");
    let right = lower.find("right");

    match (top, bottom, left, right) {
        (Some(t), Some(b), _, _) if t < b => Some(Direction::TopToBottom),
        (Some(t), Some(b), _, _) if b < t => Some(Direction::BottomToTop),
        (_, _, Some(l), Some(r)) if l < r => Some(Direction::LeftToRight),
        (_, _, Some(l), Some(r)) if r < l => Some(Direction::RightToLeft),
        _ => None,
    }
}

/// Разбирает `sprite_stmt` в `Sprite`.
///
/// Формат заголовка: `[ШxВ/цветов]`, тело — строки шестнадцатеричных
/// цифр, по одной на пиксель. Палитра задаётся отдельным ключевым словом
/// (`sprite $имя [4x4/16] { ... }` использует стандартную палитру
/// PlantUML), поэтому здесь храним только индексы.
fn parse_sprite(pair: pest::iterators::Pair<Rule>) -> Option<plantuml_ast::class::Sprite> {
    let mut name = String::new();
    let mut width = 0usize;
    let mut height = 0usize;
    let mut rows: Vec<String> = Vec::new();
    let mut body = String::new();
    let mut compressed = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::sprite_size => {
                // `ШxВ/цветов`, где суффикс цвета может оканчиваться на
                // `z` — это признак СЖАТОГО тела (формат `16z`).
                let size = inner.as_str();
                let without_comment = size.split_whitespace().next().unwrap_or(size);
                let mut parts = without_comment.split('/');
                let dims = parts.next().unwrap_or(without_comment);
                let colours = parts.next().unwrap_or("");
                compressed = colours.ends_with('z');

                let mut dims = dims.split(['x', 'X']);
                width = dims.next().and_then(|v| v.trim().parse().ok()).unwrap_or(0);
                height = dims.next().and_then(|v| v.trim().parse().ok()).unwrap_or(0);
            }
            Rule::sprite_body => {
                body = inner.as_str().to_string();
            }
            _ => {}
        }
    }

    if rows.is_empty() && compressed && !body.is_empty() && width > 0 && height > 0 {
        // Сжатое тело: base64 алфавитом PlantUML плюс raw deflate.
        // Формат разобран на k8s-sprites-unlabeled-25pct.iuml.
        match plantuml_stdlib::inflate::decode_compressed_sprite(&body, width, height) {
            Ok(decoded) => rows = decoded,
            Err(_) => return None,
        }
    }

    if rows.is_empty() && !body.is_empty() {
        // Несжатое тело: строки шестнадцатеричных цифр.
        for line in body.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                rows.push(trimmed.to_string());
            }
        }
    }

    if name.is_empty() || rows.is_empty() {
        return None;
    }

    // Если размер не указан или не согласуется с телом — берём из данных.
    if width == 0 || height == 0 {
        width = rows.iter().map(String::len).max().unwrap_or(0);
        height = rows.len();
    }

    Some(plantuml_ast::class::Sprite {
        name,
        rows,
        width,
        height,
        palette: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Сжатый спрайт (формат `16z`) разбирается и распаковывается.
    ///
    /// Регрессия: тело сжатого спрайта — не hex-цифры, а base64 плюс
    /// raw deflate, поэтому прежний разбор отбрасывал такие спрайты
    /// молча. Именно в этом формате лежат kubernetes, aws и azure.
    #[test]
    fn test_compressed_sprite_parses() {
        // Спрайт 2x2 из четырёх пикселей значения 1, 2, 3, 4.
        // Тело: raw deflate `636462660100`, затем base64 алфавитом
        // PlantUML — получается `OsHYPW40`.
        let source = "@startuml\n\
            sprite $tiny [2x2/16z] {\n\
            OsHYPW40\n\
            }\n\
            class A\n\
            @enduml";

        let diagram = parse_class(source).expect("диаграмма должна разбираться");
        // Если распаковка не поддержана, спрайт отбрасывается — тогда
        // список пуст, и тест это покажет.
        assert_eq!(diagram.sprites.len(), 1, "сжатый спрайт потерян");
        let sprite = &diagram.sprites[0];
        assert_eq!(sprite.width, 2);
        assert_eq!(sprite.height, 2);
        assert_eq!(sprite.rows.len(), 2, "строк пикселей должно быть две");
    }

    /// Определение спрайта доходит до AST.
    ///
    /// Регрессия: грамматика `sprite_stmt` существовала, но парсер его
    /// не обрабатывал — спрайт молча терялся, и содержимое библиотек
    /// иконок (logos, office, tupadr3) пропадало после разбора.
    #[test]
    fn test_sprite_reaches_ast() {
        let source = "@startuml\nsprite $s [4x4/4] {\n0123\n1230\n2301\n3012\n}\nclass A\n@enduml";
        let diagram = parse_class(source).expect("диаграмма должна разбираться");
        assert_eq!(diagram.sprites.len(), 1, "спрайт потерян");
        let sprite = &diagram.sprites[0];
        assert_eq!(sprite.name, "s");
        assert_eq!(sprite.width, 4);
        assert_eq!(sprite.height, 4);
        assert_eq!(sprite.rows.len(), 4);
        assert_eq!(sprite.rows[0], "0123");
    }

    /// `left to right direction` и `top to bottom direction` сохраняются.
    ///
    /// Регрессия: грамматика принимала эту строку, но НИ ОДИН парсер её не
    /// обрабатывал — направление молча терялось, а тип `Direction` в AST
    /// не использовался вовсе.
    ///
    /// Проверены только две формы: `bottom to top` и `right to left`
    /// PlantUML отвергает (HTTP 400), и наш парсер тоже.
    #[test]
    fn test_direction_directive() {
        let diagram = parse_class("@startuml\nleft to right direction\nclass A\n@enduml")
            .expect("диаграмма должна разбираться");
        assert_eq!(
            diagram.metadata.direction,
            Some(plantuml_ast::common::Direction::LeftToRight)
        );

        let diagram = parse_class("@startuml\ntop to bottom direction\nclass A\n@enduml")
            .expect("диаграмма должна разбираться");
        assert_eq!(
            diagram.metadata.direction,
            Some(plantuml_ast::common::Direction::TopToBottom),
            "порядок слов должен учитываться"
        );

        // Без директивы направления нет
        let diagram = parse_class("@startuml\nclass A\n@enduml").expect("разбирается");
        assert_eq!(diagram.metadata.direction, None);
    }

    /// Алиас объявления: `class "Длинное имя" as short`.
    ///
    /// Регрессия: правила алиаса в class-грамматике не было вовсе, поэтому
    /// такая запись не разбиралась. После добавления правила выяснилось,
    /// что парсер его не читает — алиас молча терялся, и связи по нему
    /// не находились.
    #[test]
    fn test_class_alias() {
        let diagram =
            parse_class("@startuml\nclass \"Длинное имя\" as short\n@enduml").expect("разбирается");
        assert_eq!(diagram.classifiers.len(), 1);
        assert_eq!(diagram.classifiers[0].id.name, "Длинное имя");
        assert_eq!(diagram.classifiers[0].id.alias.as_deref(), Some("short"));

        // Связь по алиасу
        let source =
            "@startuml\nclass \"Первый\" as a1\nclass \"Второй\" as a2\na1 --> a2\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        assert_eq!(diagram.relationships.len(), 1, "связь по алиасу потеряна");

        // Обычная форма без алиаса
        let diagram = parse_class("@startuml\nclass A\n@enduml").expect("разбирается");
        assert_eq!(diagram.classifiers[0].id.alias, None);
    }

    /// Заметки class-диаграмм разбираются.
    ///
    /// Регрессия: грамматика принимала `note`, но парсер его не
    /// обрабатывал, поэтому заметка молча терялась — ни текста, ни рамки
    /// в выводе не было.
    #[test]
    fn test_parse_class_notes() {
        let cases = [
            (
                "@startuml\nclass A\nnote right of A : пояснение\n@enduml",
                "пояснение",
            ),
            (
                "@startuml\nclass A\nnote left of A : слева\n@enduml",
                "слева",
            ),
            (
                "@startuml\nclass A\nnote right of A\n  строка\nend note\n@enduml",
                "строка",
            ),
        ];

        for (source, expected_text) in cases {
            let diagram = parse_class(source).expect("заметка должна разбираться");
            assert_eq!(diagram.notes.len(), 1, "заметка потеряна: {source}");
            assert_eq!(diagram.notes[0].text, expected_text);
        }
    }

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
