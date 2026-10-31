//! Парсер Class Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML class diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::class::{
    ClassDiagram, Classifier, ClassifierType, Member, NoteLink, NoteTarget, Package, Parameter,
    Relationship, RelationshipType, Visibility,
};
use plantuml_ast::common::{Color, Direction, LineStyle, Note, NotePosition, Stereotype};

use crate::error::syntax_error_from_pest;
use crate::Result;

#[derive(Parser)]
#[grammar = "grammars/class.pest"]
pub struct ClassParser;

/// Открытый блок `{ ... }`: пакет (рисуется рамкой) или `together` (только группирует).
enum Block {
    Package(Package),
    Together,
}

/// Команда `hide`/`show`/`remove`/`restore`: применяется после разбора всей схемы,
/// потому что может стоять раньше объявления элемента.
struct VisCommand {
    keyword: String,
    filter: String,
}

/// Состояние разбора между строками.
#[derive(Default)]
struct State {
    stack: Vec<Block>,
    /// Последний объявленный элемент: на нём стоит `note left: текст` без `of`.
    last_element: Option<String>,
    /// Последняя связь: к ней привязывается `note on link`.
    last_link: Option<(String, String)>,
    commands: Vec<VisCommand>,
}

impl State {
    /// Ближайший открытый пакет: в него попадают объявленные элементы.
    fn package_mut(&mut self) -> Option<&mut Package> {
        self.stack.iter_mut().rev().find_map(|block| match block {
            Block::Package(package) => Some(package),
            Block::Together => None,
        })
    }

    /// Закрывает самый внутренний блок и кладёт готовый пакет в родителя или в схему.
    fn close_block(&mut self, diagram: &mut ClassDiagram) {
        if let Some(Block::Package(package)) = self.stack.pop() {
            match self.package_mut() {
                Some(parent) => parent.packages.push(package),
                None => diagram.packages.push(package),
            }
        }
    }
}

/// Парсит class diagram из исходного кода
pub fn parse_class(source: &str) -> Result<ClassDiagram> {
    let pairs = ClassParser::parse(Rule::diagram, source)
        .map_err(|e| syntax_error_from_pest(&e, source))?;

    let mut diagram = ClassDiagram::new();
    let mut state = State::default();

    for pair in pairs {
        if pair.as_rule() == Rule::diagram {
            for inner in pair.into_inner() {
                process_rule(inner, &mut diagram, &mut state);
            }
        }
    }

    // Незакрытый блок не должен терять своё содержимое.
    while !state.stack.is_empty() {
        state.close_block(&mut diagram);
    }

    resolve_aliases(&mut diagram);
    convert_note_relationships(&mut diagram);
    apply_commands(&mut diagram, &state.commands);
    add_implicit_classifiers(&mut diagram);

    Ok(diagram)
}

/// Обходит все классификаторы схемы, в том числе вложенные в пакеты.
fn for_each_classifier(diagram: &mut ClassDiagram, f: &mut dyn FnMut(&mut Classifier)) {
    fn walk(packages: &mut [Package], f: &mut dyn FnMut(&mut Classifier)) {
        for package in packages {
            package.classifiers.iter_mut().for_each(&mut *f);
            walk(&mut package.packages, f);
        }
    }
    diagram.classifiers.iter_mut().for_each(&mut *f);
    walk(&mut diagram.packages, f);
}

/// Заменяет псевдонимы элементов их именами в связях и привязках заметок.
///
/// Объявление `class "Длинное имя" as short` позволяет ссылаться на элемент
/// по `short`. Без замены такая связь заводила второй, пустой элемент.
fn resolve_aliases(diagram: &mut ClassDiagram) {
    let mut map = std::collections::HashMap::new();
    for_each_classifier(diagram, &mut |c| {
        if let Some(alias) = &c.id.alias {
            map.insert(alias.clone(), c.id.name.clone());
        }
    });
    if map.is_empty() {
        return;
    }
    let resolve = |name: &mut String| {
        if let Some(full) = map.get(name.as_str()) {
            *name = full.clone();
        }
    };
    for rel in &mut diagram.relationships {
        resolve(&mut rel.from);
        resolve(&mut rel.to);
    }
    for note in &mut diagram.notes {
        note.anchors.iter_mut().for_each(resolve);
    }
    for link in &mut diagram.note_links {
        match &mut link.target {
            NoteTarget::Element(name) => resolve(name),
            NoteTarget::Link { from, to } => {
                resolve(from);
                resolve(to);
            }
        }
    }
}

/// Связь с плавающей заметкой (`A .. N`) становится линией заметки.
///
/// Имя заметки не класс: без этого шага `N` попал бы в схему как пустой класс.
fn convert_note_relationships(diagram: &mut ClassDiagram) {
    if diagram.note_aliases.is_empty() {
        return;
    }
    let note_of = |name: &str| {
        diagram
            .note_aliases
            .iter()
            .find(|(alias, _)| alias == name)
            .map(|(_, index)| *index)
    };
    let mut links = Vec::new();
    let mut kept = Vec::new();
    for rel in std::mem::take(&mut diagram.relationships) {
        match (note_of(&rel.from), note_of(&rel.to)) {
            (Some(note), None) => links.push(NoteLink {
                note,
                target: NoteTarget::Element(rel.to),
                line_style: rel.line_style,
            }),
            (None, Some(note)) => links.push(NoteLink {
                note,
                target: NoteTarget::Element(rel.from),
                line_style: rel.line_style,
            }),
            (Some(_), Some(_)) => {}
            (None, None) => kept.push(rel),
        }
    }
    diagram.relationships = kept;
    diagram.note_links.extend(links);
}

/// Применяет `hide`/`show`/`remove`/`restore` по порядку их записи.
///
/// Скрытие полей и методов очищает списки: так элемент рисуется без тела, как
/// в PlantUML. `remove` убирает элемент вместе со связями. Значок (`circle`)
/// отдельно не хранится, поэтому его скрытие ни на что не влияет.
fn apply_commands(diagram: &mut ClassDiagram, commands: &[VisCommand]) {
    if commands.is_empty() {
        return;
    }
    struct Flags {
        removed: bool,
        fields: bool,
        methods: bool,
        stereotype: bool,
    }

    let linked: std::collections::HashSet<String> = diagram
        .relationships
        .iter()
        .flat_map(|r| [r.from.clone(), r.to.clone()])
        .chain(diagram.note_links.iter().flat_map(|l| match &l.target {
            NoteTarget::Element(name) => vec![name.clone()],
            NoteTarget::Link { from, to } => vec![from.clone(), to.clone()],
        }))
        .collect();

    const KINDS: &[(&str, ClassifierType)] = &[
        ("class", ClassifierType::Class),
        ("interface", ClassifierType::Interface),
        ("enum", ClassifierType::Enum),
        ("abstract", ClassifierType::AbstractClass),
        ("annotation", ClassifierType::Annotation),
        ("entity", ClassifierType::Entity),
        ("struct", ClassifierType::Struct),
        ("protocol", ClassifierType::Protocol),
        ("exception", ClassifierType::Exception),
        ("metaclass", ClassifierType::Metaclass),
    ];

    let mut doomed: Vec<String> = Vec::new();

    for_each_classifier(diagram, &mut |c| {
        let mut flags = Flags {
            removed: false,
            fields: false,
            methods: false,
            stereotype: false,
        };
        for command in commands {
            let hiding = matches!(command.keyword.as_str(), "hide" | "remove");
            let mut feature: Option<&str> = None;
            let mut selectors: Vec<&str> = Vec::new();
            let mut empty_only = false;
            for word in command.filter.split_whitespace() {
                match word.to_lowercase().as_str() {
                    "empty" => empty_only = true,
                    "members" | "attributes" | "fields" | "methods" | "circle" | "stereotype"
                    | "stereotypes"
                        if feature.is_none() =>
                    {
                        feature = Some(word)
                    }
                    _ => selectors.push(word),
                }
            }
            let matches_element = selectors.is_empty()
                || selectors.iter().any(|sel| {
                    if *sel == "@unlinked" {
                        !linked.contains(&c.id.name)
                    } else if let Some(inner) = sel.strip_prefix("<<") {
                        let inner = inner.trim_end_matches(">>").trim();
                        c.stereotype
                            .as_ref()
                            .is_some_and(|s| s.names.iter().any(|n| n == inner))
                    } else if let Some((_, kind)) = KINDS
                        .iter()
                        .find(|(word, _)| word.eq_ignore_ascii_case(sel))
                    {
                        c.classifier_type == *kind
                    } else {
                        c.id.name == *sel || c.id.alias.as_deref() == Some(*sel)
                    }
                });
            if !matches_element {
                continue;
            }
            match feature {
                // `hide empty members` скрывает только пустые секции, а они и так не несут текста.
                _ if empty_only => {}
                None => flags.removed = hiding,
                Some(f) => match f.to_lowercase().as_str() {
                    "members" => {
                        flags.fields = hiding;
                        flags.methods = hiding;
                    }
                    "fields" | "attributes" => flags.fields = hiding,
                    "methods" => flags.methods = hiding,
                    "stereotype" | "stereotypes" => flags.stereotype = hiding,
                    _ => {}
                },
            }
        }
        if flags.fields {
            c.fields.clear();
        }
        if flags.methods {
            c.methods.clear();
        }
        if flags.stereotype {
            c.stereotype = None;
        }
        if flags.removed {
            doomed.push(c.id.name.clone());
        }
    });

    if doomed.is_empty() {
        return;
    }
    fn prune(packages: &mut [Package], doomed: &[String]) {
        for package in packages {
            package.classifiers.retain(|c| !doomed.contains(&c.id.name));
            prune(&mut package.packages, doomed);
        }
    }
    diagram.classifiers.retain(|c| !doomed.contains(&c.id.name));
    prune(&mut diagram.packages, &doomed);
    diagram
        .relationships
        .retain(|r| !doomed.contains(&r.from) && !doomed.contains(&r.to));
}

/// Создаёт элементы, которые упомянуты только в связях.
///
/// В PlantUML `Animal <|-- Dog` сам объявляет оба класса. Раньше такая схема
/// разбиралась успешно, но связи вели в никуда и картинка выходила пустой.
fn add_implicit_classifiers(diagram: &mut ClassDiagram) {
    fn collect(package: &Package, known: &mut std::collections::HashSet<String>) {
        known.extend(package.classifiers.iter().map(|c| c.id.name.clone()));
        for nested in &package.packages {
            collect(nested, known);
        }
    }

    let mut known: std::collections::HashSet<String> = diagram
        .classifiers
        .iter()
        .map(|c| c.id.name.clone())
        .collect();
    for package in &diagram.packages {
        collect(package, &mut known);
    }
    let names: Vec<String> = diagram
        .relationships
        .iter()
        .flat_map(|r| [r.from.clone(), r.to.clone()])
        .collect();
    for name in names {
        if known.insert(name.clone()) {
            diagram.classifiers.push(Classifier::new(name));
        }
    }
}

/// Кладёт объявленный элемент в ближайший пакет и добавляет связи `extends`/`implements`.
fn register_declaration(result: ClassDeclResult, diagram: &mut ClassDiagram, state: &mut State) {
    let class_name = result.classifier.id.name.clone();
    state.last_element = Some(class_name.clone());
    match state.package_mut() {
        Some(package) => package.classifiers.push(result.classifier),
        None => diagram.add_class(result.classifier),
    }
    if let Some(parent) = result.extends {
        diagram.add_relationship(Relationship {
            from: class_name.clone(),
            to: parent,
            relationship_type: RelationshipType::Inheritance,
            label: None,
            from_cardinality: None,
            to_cardinality: None,
            line_style: LineStyle::Solid,
            direction: None,
            arrow: plantuml_ast::common::ArrowShape::default(),
            declared_by_keyword: true,
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
            line_style: LineStyle::Dashed,
            direction: None,
            arrow: plantuml_ast::common::ArrowShape::default(),
            declared_by_keyword: true,
        });
    }
}

/// Обрабатывает правило грамматики
fn process_rule(pair: pest::iterators::Pair<Rule>, diagram: &mut ClassDiagram, state: &mut State) {
    let declared_type = match pair.as_rule() {
        Rule::class_decl | Rule::kind_decl | Rule::plain_kind_decl | Rule::shortform_decl => {
            Some(ClassifierType::Class)
        }
        Rule::interface_decl => Some(ClassifierType::Interface),
        Rule::abstract_decl => Some(ClassifierType::AbstractClass),
        Rule::enum_decl => Some(ClassifierType::Enum),
        Rule::annotation_decl => Some(ClassifierType::Annotation),
        _ => None,
    };
    if let Some(default_type) = declared_type {
        if let Some(result) = parse_class_decl_with_inheritance(pair, default_type) {
            register_declaration(result, diagram, state);
        }
        return;
    }

    match pair.as_rule() {
        Rule::relationship => {
            if let Some(rel) = parse_relationship(pair) {
                state.last_link = Some((rel.from.clone(), rel.to.clone()));
                diagram.add_relationship(rel);
            }
        }
        Rule::class_description => parse_class_description(pair, diagram, state),
        Rule::vis_stmt => {
            let mut keyword = String::new();
            let mut filter = String::new();
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::vis_kw => keyword = inner.as_str().trim().to_lowercase(),
                    Rule::vis_filter => filter = inner.as_str().trim().to_string(),
                    _ => {}
                }
            }
            state.commands.push(VisCommand { keyword, filter });
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
        Rule::note_stmt => parse_note_statement(pair, diagram, state),
        Rule::package_start => {
            let pkg = parse_package_start(pair);
            state.stack.push(Block::Package(pkg));
        }
        Rule::together_start => state.stack.push(Block::Together),
        Rule::block_end => state.close_block(diagram),
        Rule::title_stmt => {
            if let Some(title) = parse_title(pair) {
                diagram.metadata.title = Some(title);
            }
        }
        _ => {}
    }
}

/// `A : текст` добавляет строку в тело элемента (и создаёт элемент, если его нет).
fn parse_class_description(
    pair: pest::iterators::Pair<Rule>,
    diagram: &mut ClassDiagram,
    state: &mut State,
) {
    let mut target = String::new();
    let mut text = String::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier | Rule::quoted_string => target = extract_name(inner),
            Rule::description_text => text = inner.as_str().trim().to_string(),
            _ => {}
        }
    }
    if target.is_empty() || text.is_empty() {
        return;
    }

    let mut members: (Vec<Member>, Vec<Member>) = (Vec::new(), Vec::new());
    if let Ok(mut pairs) = ClassParser::parse(Rule::member, &text) {
        if let Some(member) = pairs.next() {
            parse_member(member, &mut members.0, &mut members.1);
        }
    }
    if members.0.is_empty() && members.1.is_empty() {
        members.0.push(plain_member(text));
    }

    let mut apply = |c: &mut Classifier| {
        if c.id.name == target || c.id.alias.as_deref() == Some(target.as_str()) {
            c.fields.append(&mut members.0);
            c.methods.append(&mut members.1);
        }
    };
    for_each_classifier(diagram, &mut apply);
    for block in state.stack.iter_mut() {
        if let Block::Package(package) = block {
            package.classifiers.iter_mut().for_each(&mut apply);
        }
    }

    // Остаток значит, что элемента с таким именем не нашлось.
    if !(members.0.is_empty() && members.1.is_empty()) {
        // Элемента не было: `A : поле` объявляет его.
        let mut classifier = Classifier::new(target);
        classifier.fields = std::mem::take(&mut members.0);
        classifier.methods = std::mem::take(&mut members.1);
        register_declaration(
            ClassDeclResult {
                classifier,
                extends: None,
                implements: Vec::new(),
            },
            diagram,
            state,
        );
    }
}

/// Строка тела без разбора: рисуется как есть, чтобы текст не терялся.
fn plain_member(text: String) -> Member {
    Member {
        name: text,
        member_type: None,
        visibility: Visibility::Public,
        is_static: false,
        is_abstract: false,
        parameters: Vec::new(),
        separator: None,
        is_method: false,
    }
}

/// Результат парсинга объявления класса
struct ClassDeclResult {
    classifier: Classifier,
    extends: Option<String>,
    implements: Vec<String>,
}

/// Тип элемента по ключевому слову (`struct`, `()`, `<>` и т. д.).
fn kind_type(keyword: &str) -> Option<ClassifierType> {
    match keyword.trim() {
        "()" => Some(ClassifierType::Circle),
        "<>" => Some(ClassifierType::Diamond),
        other => ClassifierType::parse(other),
    }
}

/// Разбирает краску (`#red`, `#red-green`, `##[bold]blue`, `#line:green;text:red`).
fn apply_paint(token: &str, background: &mut Option<Color>, border: &mut Option<Color>) {
    let border_only = token.starts_with("##");
    let body = token.trim_start_matches('#');
    let color_of = |value: &str| {
        // Градиент `red-green` рисуем первым цветом.
        let value = value.trim_start_matches(['[', ']']);
        let value = value.split(['-', '|', '/', '\\']).next().unwrap_or("");
        (!value.is_empty()).then(|| Color::parse(value))
    };
    if body.contains(':') {
        for part in body.split(';') {
            if let Some(value) = part.strip_prefix("line:") {
                *border = color_of(value).or_else(|| border.take());
            } else if let Some(value) = part.strip_prefix("back:") {
                *background = color_of(value).or_else(|| background.take());
            }
        }
        return;
    }
    // Стиль линии в скобках (`##[bold]blue`) цвета не меняет.
    let body = match body.strip_prefix('[') {
        Some(rest) => rest.split_once(']').map_or("", |(_, color)| color),
        None => body,
    };
    match color_of(body) {
        Some(color) if border_only => *border = Some(color),
        Some(color) => *background = Some(color),
        None => {}
    }
}

/// Разбирает `<<имя>>` и `<<(X,#цвет) имя>>`.
fn parse_stereotype(raw: &str) -> Stereotype {
    let content = raw
        .trim()
        .trim_start_matches("<<")
        .trim_end_matches(">>")
        .trim();
    if let Some(rest) = content.strip_prefix('(') {
        if let Some((spot, name)) = rest.split_once(')') {
            let mut parts = spot.splitn(2, ',');
            let letter = parts.next().and_then(|l| l.trim().chars().next());
            let color = parts
                .next()
                .map(str::trim)
                .filter(|c| !c.is_empty())
                .map(Color::parse);
            if letter.is_some() {
                let mut stereotype = Stereotype::new(name.trim());
                if name.trim().is_empty() {
                    stereotype.names.clear();
                }
                stereotype.spot = letter;
                stereotype.background_color = color;
                return stereotype;
            }
        }
    }
    Stereotype::new(content)
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
    let mut border: Option<Color> = None;
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
            Rule::kind_keyword | Rule::plain_kind_keyword | Rule::shortform_kw => {
                if let Some(kind) = kind_type(inner.as_str()) {
                    classifier_type = kind;
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
                stereotype = Some(parse_stereotype(inner.as_str()));
            }
            Rule::class_paint => apply_paint(inner.as_str(), &mut color, &mut border),
            Rule::generic_params => {
                let raw = inner.as_str().trim();
                let raw = raw.strip_prefix('<').unwrap_or(raw);
                let raw = raw.strip_suffix('>').unwrap_or(raw);
                generics = Some(raw.trim().to_string());
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
            border_color: border,
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
            Rule::member => parse_member(inner, fields, methods),
            Rule::enum_value => {
                // Для enum значения добавляем как поля
                let value = inner.as_str().trim().to_string();
                if !value.is_empty() {
                    fields.push(plain_member(value));
                }
            }
            _ => {}
        }
    }
}

/// Одна строка тела: поле, метод, разделитель или произвольный текст.
fn parse_member(
    pair: pest::iterators::Pair<Rule>,
    fields: &mut Vec<Member>,
    methods: &mut Vec<Member>,
) {
    for member_inner in pair.into_inner() {
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
            Rule::separator => {
                let line = member_inner.as_str().chars().next().unwrap_or('-');
                let title = member_inner
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::separator_text)
                    .map(|p| p.as_str().trim().to_string())
                    .unwrap_or_default();
                let mut separator = plain_member(title);
                separator.separator = Some(line);
                // Разделитель после методов относится к секции методов.
                if methods.is_empty() {
                    fields.push(separator);
                } else {
                    methods.push(separator);
                }
            }
            Rule::free_line => {
                let text = member_inner.as_str().trim().to_string();
                if !text.is_empty() {
                    let target = if methods.is_empty() {
                        &mut *fields
                    } else {
                        &mut *methods
                    };
                    target.push(plain_member(text));
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
        separator: None,
        is_method: false,
    })
}

/// Парсит метод класса
fn parse_method(pair: pest::iterators::Pair<Rule>) -> Option<Member> {
    let mut name = String::new();
    let mut prefix = String::new();
    let mut return_type: Option<String> = None;
    let mut visibility = Visibility::Public;
    let mut is_static = false;
    let mut is_abstract = false;
    let mut parameters: Vec<Parameter> = Vec::new();

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
            Rule::method_prefix => {
                prefix = inner.as_str().to_string();
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
    // Тип перед именем (`void start()`) входит в подпись целиком.
    if !prefix.is_empty() {
        name = format!("{prefix} {name}");
    }

    Some(Member {
        name,
        member_type: return_type,
        visibility,
        is_static,
        is_abstract,
        parameters,
        separator: None,
        is_method: true,
    })
}

/// Парсит параметры метода
fn parse_method_params(pair: pest::iterators::Pair<Rule>) -> Vec<Parameter> {
    let mut params = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() != Rule::method_param {
            continue;
        }
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
                // Запись вроде `int timeout` хранится как есть.
                Rule::param_text => {
                    param_name = param_inner.as_str().trim().to_string();
                }
                _ => {}
            }
        }

        if !param_name.is_empty() {
            params.push(Parameter {
                name: param_name,
                param_type,
            });
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

/// Текст заметки: строки без отступов, пустые края отброшены.
fn note_text_of(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Элемент, к которому привязана заметка: `A::counter` и `A.start()` относятся к `A`.
fn note_anchor_of(target: &str) -> String {
    let target = target.trim().trim_matches('"');
    if let Some((owner, _)) = target.split_once("::") {
        return owner.to_string();
    }
    if target.ends_with(')') {
        if let Some((owner, _)) = target.rsplit_once('.') {
            return owner.to_string();
        }
    }
    target.to_string()
}

/// Разбирает заметку class-диаграммы и кладёт её в схему.
///
/// Формы: `note right of A : текст`, `note right of A ... end note`,
/// `note "текст" as N`, `note as N ... end note`, `note left: текст`
/// (на последнем элементе) и `note on link: текст` (на последней связи).
fn parse_note_statement(
    pair: pest::iterators::Pair<Rule>,
    diagram: &mut ClassDiagram,
    state: &mut State,
) {
    let mut position = NotePosition::Right;
    let mut text = String::new();
    let mut anchors: Vec<String> = Vec::new();
    let mut alias: Option<String> = None;
    let mut background: Option<Color> = None;
    let mut border: Option<Color> = None;
    let mut on_link = false;
    let mut unanchored = false;

    for inner in pair.into_inner() {
        let kind = inner.as_rule();
        match kind {
            Rule::note_on_element
            | Rule::note_multiline
            | Rule::note_floating
            | Rule::note_block_alias
            | Rule::note_unanchored
            | Rule::note_on_link => {
                on_link = kind == Rule::note_on_link;
                unanchored = kind == Rule::note_unanchored;
                let mut has_target = false;
                for n in inner.into_inner() {
                    match n.as_rule() {
                        Rule::note_position => position = parse_note_position(n.as_str()),
                        Rule::note_target => {
                            let target = note_anchor_of(n.as_str());
                            if !target.is_empty() {
                                anchors.push(target);
                                has_target = true;
                            }
                        }
                        Rule::note_text | Rule::note_body => text = note_text_of(n.as_str()),
                        Rule::quoted_string => {
                            text = n.as_str().trim().trim_matches('"').to_string();
                        }
                        Rule::identifier => alias = Some(n.as_str().to_string()),
                        Rule::class_paint | Rule::note_paint => {
                            apply_paint(n.as_str(), &mut background, &mut border)
                        }
                        _ => {}
                    }
                }
                // `note left` без `of` стоит на последнем объявленном элементе.
                if kind == Rule::note_multiline && !has_target {
                    unanchored = true;
                }
            }
            _ => {}
        }
    }

    if text.is_empty() {
        return;
    }
    if unanchored {
        if let Some(last) = &state.last_element {
            anchors.push(last.clone());
        }
    }

    let index = diagram.notes.len();
    diagram.notes.push(Note {
        text,
        position,
        anchors,
        background_color: background,
    });
    if let Some(alias) = alias {
        diagram.note_aliases.push((alias, index));
    }
    if on_link {
        if let Some((from, to)) = state.last_link.clone() {
            diagram.note_links.push(NoteLink {
                note: index,
                target: NoteTarget::Link { from, to },
                line_style: LineStyle::Dashed,
            });
        }
    }
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
    let mut direction: Option<Direction> = None;
    let mut arrow = plantuml_ast::common::ArrowShape::default();
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
                arrow = plantuml_ast::common::ArrowShape::parse(inner.as_str());
                let (rtype, lstyle, dir) = parse_arrow(inner);
                rel_type = rtype;
                line_style = lstyle;
                direction = dir;
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
        direction,
        arrow,
        declared_by_keyword: false,
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
fn parse_arrow(
    pair: pest::iterators::Pair<Rule>,
) -> (RelationshipType, LineStyle, Option<Direction>) {
    let mut left_side = "";
    let mut line = "";
    let mut right_side = "";
    let mut style = "";
    let mut direction = None;

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
            Rule::arrow_style => style = inner.as_str(),
            // Направление — куда идёт стрелка: `-up->` ведёт вверх.
            Rule::arrow_dir => {
                direction = match inner.as_str() {
                    "up" | "u" => Some(Direction::BottomToTop),
                    "down" | "d" => Some(Direction::TopToBottom),
                    "left" | "l" => Some(Direction::RightToLeft),
                    "right" | "r" => Some(Direction::LeftToRight),
                    _ => None,
                };
            }
            _ => {}
        }
    }

    let line_style = if line.contains("..")
        || line == "."
        || style.contains("dashed")
        || style.contains("dotted")
    {
        LineStyle::Dashed
    } else if style.contains("bold") {
        LineStyle::Bold
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

    (rel_type, line_style, direction)
}

/// Парсит начало пакета
fn parse_package_start(pair: pest::iterators::Pair<Rule>) -> Package {
    let mut name = String::new();
    let mut stereotype: Option<Stereotype> = None;
    let mut color: Option<Color> = None;
    let mut border: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::package_name | Rule::identifier => {
                name = extract_name(inner);
            }
            Rule::package_style => {
                stereotype = Some(parse_stereotype(inner.as_str()));
            }
            Rule::class_paint => apply_paint(inner.as_str(), &mut color, &mut border),
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

/// Разбирает объявление спрайта из текста: спрайты доступны во всех типах диаграмм.
pub fn parse_sprite_text(text: &str) -> Option<plantuml_ast::class::Sprite> {
    // Конец тела грамматика узнаёт по следующей строке, поэтому даём ей такую строку.
    let source = format!("{}\n@", text.trim());
    let pair = ClassParser::parse(Rule::sprite_stmt, &source)
        .ok()?
        .next()?;
    parse_sprite(pair)
}

/// Разбирает `sprite_stmt` в `Sprite`.
///
/// Формат заголовка: `[ШxВ/цветов]`, тело — строки шестнадцатеричных
/// цифр, по одной на пиксель. Палитра задаётся отдельным ключевым словом
/// (`sprite $имя [4x4/16] { ... }` использует стандартную палитру
/// PlantUML), поэтому здесь храним только индексы.
fn parse_sprite(pair: pest::iterators::Pair<Rule>) -> Option<plantuml_ast::class::Sprite> {
    for inner in pair.clone().into_inner() {
        match inner.as_rule() {
            Rule::sprite_svg => return parse_svg_sprite(inner),
            Rule::sprite_raster => return parse_raster_sprite(inner),
            _ => {}
        }
    }
    None
}

/// Разбирает векторный спрайт `sprite имя <svg ...> ... </svg>`.
///
/// Размер берётся из `viewBox` открывающего тега: он задаёт систему
/// координат, в которой нарисовано тело. `width`/`height` тега при этом
/// могут быть в миллиметрах (так делает библиотека Archimate), поэтому
/// для геометрии используется именно `viewBox`.
fn parse_svg_sprite(pair: pest::iterators::Pair<Rule>) -> Option<plantuml_ast::class::Sprite> {
    let mut name = String::new();
    let mut attrs = String::new();
    let mut body = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::sprite_name => name = inner.as_str().trim_start_matches('$').to_string(),
            Rule::svg_attrs => attrs = inner.as_str().trim().to_string(),
            Rule::svg_body => body = inner.as_str().to_string(),
            _ => {}
        }
    }

    if name.is_empty() || body.trim().is_empty() {
        return None;
    }

    // `viewBox="minX minY ширина высота"`. Без него взять размер неоткуда,
    // а PlantUML в таком случае использует width/height тега.
    let (width, height) = parse_svg_extent(&attrs);

    Some(plantuml_ast::class::Sprite {
        name,
        rows: Vec::new(),
        width,
        height,
        palette: Vec::new(),
        svg: Some(plantuml_ast::class::SvgSprite { attrs, body }),
    })
}

/// Извлекает размер спрайта из атрибутов тега `<svg>`.
///
/// Сначала пробует `viewBox` (он задаёт систему координат тела), затем
/// `width`/`height` с отбрасыванием единиц измерения.
fn parse_svg_extent(attrs: &str) -> (usize, usize) {
    if let Some(value) = attribute_value(attrs, "viewBox") {
        let parts: Vec<f64> = value
            .split([' ', ','])
            .filter_map(|v| v.trim().parse().ok())
            .collect();
        if parts.len() == 4 && parts[2] > 0.0 && parts[3] > 0.0 {
            return (parts[2].round() as usize, parts[3].round() as usize);
        }
    }

    let parse_length = |raw: &str| -> Option<usize> {
        let digits: String = raw
            .trim()
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        digits.parse::<f64>().ok().map(|v| v.round() as usize)
    };

    let width = attribute_value(attrs, "width")
        .and_then(|v| parse_length(&v))
        .unwrap_or(0);
    let height = attribute_value(attrs, "height")
        .and_then(|v| parse_length(&v))
        .unwrap_or(0);

    (width, height)
}

/// Возвращает значение атрибута из строки вида `key="value"`.
fn attribute_value(attrs: &str, key: &str) -> Option<String> {
    // Ищем `key="` без вложенных кавычек в форматной строке:
    // так надёжнее и читается проще.
    let needle = String::from(key) + "=\"";
    let start = attrs.find(&needle)? + needle.len();
    let rest = &attrs[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Разбирает растровый спрайт `sprite $имя [ШxВ/цветов] { ... }`.
fn parse_raster_sprite(pair: pest::iterators::Pair<Rule>) -> Option<plantuml_ast::class::Sprite> {
    let mut name = String::new();
    let mut width = 0usize;
    let mut height = 0usize;
    let mut rows: Vec<String> = Vec::new();
    let mut body = String::new();
    let mut compressed = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::sprite_name => name = inner.as_str().trim_start_matches('$').to_string(),
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
        svg: None,
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

    /// Все виды классификаторов принимаются и различаются по типу.
    #[test]
    fn test_classifier_kinds() {
        let source = "@startuml\nstruct S\nprotocol P\nexception E\nmetaclass M\nstereotype T\nentity N\ncircle C\ndiamond D\n() Short\n<> Rhomb\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        let kinds: Vec<ClassifierType> = diagram
            .classifiers
            .iter()
            .map(|c| c.classifier_type)
            .collect();
        assert_eq!(
            kinds,
            vec![
                ClassifierType::Struct,
                ClassifierType::Protocol,
                ClassifierType::Exception,
                ClassifierType::Metaclass,
                ClassifierType::Stereotype,
                ClassifierType::Entity,
                ClassifierType::Circle,
                ClassifierType::Diamond,
                ClassifierType::Circle,
                ClassifierType::Diamond,
            ]
        );
    }

    /// Стереотип со значком `<<(X,#ff0000) имя>>` и обобщённые параметры.
    #[test]
    fn test_stereotype_spot_and_generics() {
        let source = "@startuml\nclass A <<s1>>\nclass B <<(X,#ff0000) s2>>\nclass Foo<T extends Element>\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        let a = diagram.classifiers[0].stereotype.as_ref().expect("есть");
        assert_eq!(a.names, vec!["s1".to_string()]);
        let b = diagram.classifiers[1].stereotype.as_ref().expect("есть");
        assert_eq!(b.names, vec!["s2".to_string()]);
        assert_eq!(b.spot, Some('X'));
        assert!(b.background_color.is_some());
        assert_eq!(
            diagram.classifiers[2].generics.as_deref(),
            Some("T extends Element")
        );
    }

    /// Направление стрелки внутри линии: `-up->`, `-u->`, `-left->`.
    #[test]
    fn test_arrow_direction() {
        use plantuml_ast::common::Direction;
        let source = "@startuml\nA -up-> B\nA -d-> B\nA -left-> B\nA -r-> B\nA --> B\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        let dirs: Vec<Option<Direction>> =
            diagram.relationships.iter().map(|r| r.direction).collect();
        assert_eq!(
            dirs,
            vec![
                Some(Direction::BottomToTop),
                Some(Direction::TopToBottom),
                Some(Direction::RightToLeft),
                Some(Direction::LeftToRight),
                None
            ]
        );
    }

    /// Пакеты, `namespace`, `together`: закрывающая скобка у всех одна.
    #[test]
    fn test_blocks_share_closing_brace() {
        let source = "@startuml\npackage \"П1\" {\nclass A\ntogether {\nclass T1\nclass T2\n}\n}\nnamespace N1 {\nclass B\n}\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        assert_eq!(diagram.packages.len(), 2);
        assert_eq!(diagram.packages[0].name, "П1");
        assert_eq!(diagram.packages[0].classifiers.len(), 3);
        assert_eq!(diagram.packages[1].name, "N1");
        assert_eq!(diagram.packages[1].classifiers.len(), 1);
    }

    /// Заметки: плавающая со связью, на последнем элементе, на связи, на поле.
    #[test]
    fn test_note_forms() {
        let source = "@startuml\nclass A\nnote \"n2\" as N\nA .. N\nnote left: на последнем\nnote right of A::counter\n  на поле\nend note\nA --> B\nnote on link #red: на связи\nnote as M\n  строка 1\n  строка 2\nend note\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        assert_eq!(diagram.notes.len(), 5);
        // `N` не стал классом, связь превратилась в линию заметки
        assert!(diagram.classifiers.iter().all(|c| c.id.name != "N"));
        assert!(diagram
            .note_links
            .iter()
            .any(|l| l.note == 0 && l.target == NoteTarget::Element("A".into())));
        assert_eq!(diagram.notes[1].anchors, vec!["A".to_string()]);
        assert_eq!(diagram.notes[2].anchors, vec!["A".to_string()]);
        assert!(diagram.note_links.iter().any(|l| l.note == 3
            && l.target
                == NoteTarget::Link {
                    from: "A".into(),
                    to: "B".into()
                }));
        assert_eq!(diagram.notes[4].text, "строка 1\nстрока 2");
        assert!(diagram.note_aliases.iter().any(|(a, _)| a == "M"));
    }

    /// Разделители тела с подписью и пустые строки с пробелами не теряют текст.
    #[test]
    fn test_body_separators_and_blank_lines() {
        let source = "@startuml\nclass U {\n    .. Getters ..\n    + getName()\n    \n    == Setters ==\n    - int age\n    +void start(int timeout)\n}\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        let c = &diagram.classifiers[0];
        let texts: Vec<String> = c
            .fields
            .iter()
            .chain(c.methods.iter())
            .map(|m| m.display_text())
            .collect();
        for want in [
            "Getters",
            "getName()",
            "Setters",
            "int age",
            "void start(int timeout)",
        ] {
            assert!(texts.iter().any(|t| t == want), "нет {want}: {texts:?}");
        }
    }

    /// `hide`, `remove` с фильтрами применяются после разбора.
    #[test]
    fn test_hide_and_remove_filters() {
        let source = "@startuml\nclass A {\n+x : int\n+f()\n}\nclass B <<s>> {\n+y : int\n}\nclass C\nclass D\nA --> B\nhide A members\nhide <<s>> stereotype\nremove C\nremove @unlinked\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        let names: Vec<&str> = diagram
            .classifiers
            .iter()
            .map(|c| c.id.name.as_str())
            .collect();
        assert_eq!(names, vec!["A", "B"], "C и D убраны");
        assert!(diagram.classifiers[0].fields.is_empty());
        assert!(diagram.classifiers[0].methods.is_empty());
        assert_eq!(diagram.classifiers[1].fields.len(), 1);
        assert!(diagram.classifiers[1].stereotype.is_none());
    }

    /// Цвета элемента: фон, градиент и линия.
    #[test]
    fn test_paint_tokens() {
        let source = "@startuml\nclass Foo #red-green ##[bold]blue\nclass Bar #line:green;line.dashed;text:red\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        assert_eq!(
            diagram.classifiers[0].background_color,
            Some(Color::parse("red"))
        );
        assert_eq!(
            diagram.classifiers[0].border_color,
            Some(Color::parse("blue"))
        );
        assert_eq!(
            diagram.classifiers[1].border_color,
            Some(Color::parse("green"))
        );
    }

    /// Связь по псевдониму ведёт к настоящему элементу, а не к пустому.
    #[test]
    fn test_alias_in_relationship_resolves() {
        let source = "@startuml\nclass \"Abstract Class\" as AC\nclass B\nAC --> B\n@enduml";
        let diagram = parse_class(source).expect("разбирается");
        assert_eq!(diagram.classifiers.len(), 2);
        assert_eq!(diagram.relationships[0].from, "Abstract Class");
    }
}
