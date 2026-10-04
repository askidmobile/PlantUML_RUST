//! # plantuml-parser
//!
//! Лексер и парсер для PlantUML синтаксиса.
//!
//! Использует двухфазный подход:
//! 1. Лексер (logos) — быстрая токенизация
//! 2. Парсер (pest) — PEG грамматики для структуры

pub mod error;
pub mod parsers;

pub use error::ParseError;
pub use parsers::{
    parse_activity, parse_class, parse_component, parse_er, parse_gantt, parse_json, parse_mindmap,
    parse_network, parse_object, parse_salt, parse_sequence, parse_state, parse_timing,
    parse_usecase, parse_wbs, parse_yaml,
};
pub use plantuml_ast::Diagram;

/// Результат парсинга
pub type Result<T> = std::result::Result<T, ParseError>;

/// Парсит PlantUML исходный код и возвращает AST диаграммы.
///
/// # Аргументы
/// * `source` - исходный код PlantUML
///
/// # Возвращает
/// * `Ok(Diagram)` - распарсенная диаграмма
/// * `Err(ParseError)` - ошибка парсинга
///
/// # Пример
///
/// ```rust
/// use plantuml_parser::parse;
///
/// let source = "@startuml\nAlice -> Bob: Hello\n@enduml";
/// let diagram = parse(source);
/// assert!(diagram.is_ok());
/// ```
pub fn parse(source: &str) -> Result<Diagram> {
    // Определяем тип диаграммы
    let diagram_type = detect_diagram_type(source)?;

    match diagram_type {
        DiagramKind::Sequence => parse_sequence_diagram(source),
        DiagramKind::Class => parse_class_diagram(source),
        DiagramKind::Activity => parse_activity_diagram(source),
        DiagramKind::State => parse_state_diagram(source),
        DiagramKind::Component => parse_component_diagram(source),
        DiagramKind::Deployment => parse_deployment_diagram(source),
        DiagramKind::UseCase => parse_usecase_diagram(source),
        DiagramKind::Object => parse_object_diagram(source),
        DiagramKind::Timing => parse_timing_diagram(source),
        DiagramKind::Gantt => parse_gantt_diagram(source),
        DiagramKind::MindMap => parse_mindmap_diagram(source),
        DiagramKind::Wbs => parse_wbs_diagram(source),
        DiagramKind::Json => parse_json_diagram(source),
        DiagramKind::Yaml => parse_yaml_diagram(source),
        DiagramKind::Er => parse_er_diagram(source),
        DiagramKind::Network => parse_network_diagram(source),
        DiagramKind::Salt => parse_salt_diagram(source),
        DiagramKind::Archimate => parse_archimate_diagram(source),
        DiagramKind::Unknown => Err(ParseError::UnknownDiagramType),
    }
}

/// Тип диаграммы
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramKind {
    Sequence,
    Class,
    Activity,
    State,
    Component,
    Deployment,
    UseCase,
    Object,
    Timing,
    Gantt,
    MindMap,
    Wbs,
    Json,
    Yaml,
    Er,
    Network,
    Salt,
    Archimate,
    Unknown,
}

/// Проверяет наличие паттерна [Component] в тексте
/// Ищет [слово] (одно слово в квадратных скобках) как признак component diagram
fn has_component_bracket_pattern(source: &str) -> bool {
    // Паттерн [Component] — одно или несколько слов в квадратных скобках
    // Но НЕ массивы типа users[] или data[0]
    let mut i = 0;
    let bytes = source.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'[' {
            // Найдена открывающая скобка
            let start = i + 1;
            i += 1;
            // Ищем закрывающую
            while i < bytes.len() && bytes[i] != b']' {
                i += 1;
            }
            if i < bytes.len() {
                let content = &source[start..i];
                // Проверяем что внутри есть хоть что-то и это не пустая или цифра
                let trimmed = content.trim();
                if !trimmed.is_empty()
                    && !trimmed.chars().all(|c| c.is_ascii_digit())
                    && trimmed
                        .chars()
                        .next()
                        .map(|c| c.is_alphabetic())
                        .unwrap_or(false)
                {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// Проверяет наличие ключевого слова "state" как определение состояния
/// (в начале строки или после пробелов), а не как часть сообщения
fn has_state_keyword(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        // "state " в начале строки — это определение состояния
        if trimmed.starts_with("state ") {
            return true;
        }
    }
    false
}

/// Проверяет наличие паттерна :Actor: --> (UseCase) — характерно для use case диаграмм
fn has_colon_actor_usecase_pattern(source: &str) -> bool {
    // Ищем :Name: --> (Name) или :Name: --> Name
    for line in source.lines() {
        let trimmed = line.trim();
        // Проверяем начинается ли строка с :Name: и содержит --> и (
        if trimmed.starts_with(':') && trimmed.contains(':') {
            // Находим вторую : чтобы убедиться что это :Name:
            if let Some(second_colon) = trimmed[1..].find(':') {
                let after_actor = &trimmed[second_colon + 2..];
                // Проверяем что после :Name: идёт --> и (
                if (after_actor.contains("-->") || after_actor.contains("->"))
                    && after_actor.contains('(')
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Проверяет наличие паттерна (UseCase) в скобках — характерно для use case диаграмм
/// Ищем паттерны типа (Login), (Some Use Case) со стрелками
fn has_paren_usecase_pattern(source: &str) -> bool {
    // Проверяем наличие (Name) --> или --> (Name) паттернов
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '(' {
            // Ищем закрывающую скобку
            let mut j = i + 1;
            while j < len && chars[j] != ')' && chars[j] != '\n' {
                j += 1;
            }
            if j < len && chars[j] == ')' {
                // Нашли (something), проверяем есть ли рядом стрелки
                // Проверяем что перед ( или после ) есть -->
                let before = if i > 4 {
                    source.get(i.saturating_sub(5)..i).unwrap_or("")
                } else {
                    ""
                };
                let after = if j + 5 < len {
                    source.get(j + 1..j + 5).unwrap_or("")
                } else {
                    source.get(j + 1..).unwrap_or("")
                };

                if before.contains("-->")
                    || before.contains("->")
                    || after.contains("-->")
                    || after.contains("->")
                {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// Готовит исходник к анализу типа диаграммы: оставляет только структуру.
///
/// # Зачем
///
/// Определение типа по подстрокам во всём тексте давало ложные срабатывания
/// на содержимом подписей: диаграмма `Alice -> Bob: class loaded`
/// распознавалась как Class и рендерилась пустой (участники исчезали),
/// а `render()` при этом возвращал `Ok`. Это молчаливая потеря данных.
///
/// Скелет убирает всё, что не является структурой PlantUML:
/// - строчные (`'`) и блочные (`/' … '/`) комментарии;
/// - тексты подписей: в строке `A -> B: текст` остаётся только `A -> B`;
/// - содержимое многострочных блоков (`note … end note`, `legend … endlegend`
///   и блочные формы `title`/`header`/`footer`/`caption`).
///
/// Кавычки не удаляются: имена в кавычках (`participant "Имя" as L`)
/// структурны, а ключевые слова идут до них.
fn structural_skeleton(source: &str) -> String {
    /// Маркер закрытия многострочного текстового блока, если строка его открывает.
    fn block_end(line_lower: &str) -> Option<&'static str> {
        // Заметки: однострочная форма содержит ':' и маркера не имеет
        if (line_lower == "note"
            || line_lower.starts_with("note ")
            || line_lower.starts_with("hnote ")
            || line_lower.starts_with("rnote "))
            && !line_lower.contains(':')
        {
            return Some("end note");
        }
        if line_lower == "legend" || line_lower.starts_with("legend ") {
            return Some("endlegend");
        }
        // Блочные формы заголовков: ключевое слово без текста на той же строке
        for (open, close) in [
            ("title", "end title"),
            ("header", "end header"),
            ("footer", "end footer"),
            ("caption", "end caption"),
        ] {
            if line_lower == open {
                return Some(close);
            }
        }
        None
    }

    /// Маркеры связей, после которых `:` открывает подпись, а не структуру.
    const ARROWS: &[&str] = &[
        "-->", "<--", "->>", "<<-", "->", "<-", "--", "..", "..>", "<..", "-[", "o--", "*--",
        "|--", "||--",
    ];

    let mut out = String::new();
    let mut closing: Option<&str> = None;
    let mut in_comment = false;

    for raw in source.lines() {
        let line = raw.trim();

        // Блочный комментарий
        if in_comment {
            if line.contains("'/") {
                in_comment = false;
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("/'") {
            if !rest.contains("'/") {
                in_comment = true;
            }
            continue;
        }

        // Строчный комментарий
        if line.starts_with('\'') {
            continue;
        }

        // Внутри многострочного текстового блока структуры нет
        if let Some(end) = closing {
            if line.eq_ignore_ascii_case(end) {
                closing = None;
            }
            continue;
        }

        let lower = line.to_lowercase();

        // Директивы сохраняем: по ним распознаются @startgantt/@startjson,
        // archimate, nwdiag и прочие теги
        if lower.starts_with('!') || lower.starts_with('@') {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if let Some(end) = block_end(&lower) {
            closing = Some(end);
            continue;
        }

        // Отсекаем подпись связи: всё после ':' — текст, а не структура
        if let Some(colon) = line.find(':') {
            let head = &line[..colon];
            if ARROWS.iter().any(|a| head.contains(a)) {
                out.push_str(head.trim_end());
                out.push('\n');
                continue;
            }
        }

        out.push_str(line);
        out.push('\n');
    }

    out
}

/// Определяет тип диаграммы по содержимому
pub fn detect_diagram_type(source: &str) -> Result<DiagramKind> {
    // Анализируем структурный скелет, а не сырой текст: иначе ключевые слова
    // внутри подписей сообщений приводят к ложной детекции.
    let skeleton = structural_skeleton(source);
    let source_lower = skeleton.to_lowercase();

    // Salt Diagram — проверяем по @startsalt или salt keyword
    if source_lower.contains("@startsalt")
        || (source_lower.contains("salt") && source_lower.contains("{"))
    {
        return Ok(DiagramKind::Salt);
    }

    // Archimate Diagram — проверяем по archimate keyword
    if source_lower.contains("archimate ") || source_lower.contains("!include <archimate") {
        return Ok(DiagramKind::Archimate);
    }

    // Network Diagram — проверяем по nwdiag
    if source_lower.contains("nwdiag {") || source_lower.contains("nwdiag{") {
        return Ok(DiagramKind::Network);
    }

    // Gantt Diagram — проверяем по специальному тегу @startgantt
    if source_lower.contains("@startgantt") {
        return Ok(DiagramKind::Gantt);
    }

    // MindMap Diagram — проверяем по специальному тегу @startmindmap
    if source_lower.contains("@startmindmap") {
        return Ok(DiagramKind::MindMap);
    }

    // WBS Diagram — проверяем по специальному тегу @startwbs
    if source_lower.contains("@startwbs") {
        return Ok(DiagramKind::Wbs);
    }

    // JSON Diagram — проверяем по специальному тегу @startjson
    if source_lower.contains("@startjson") {
        return Ok(DiagramKind::Json);
    }

    // YAML Diagram — проверяем по специальному тегу @startyaml
    if source_lower.contains("@startyaml") {
        return Ok(DiagramKind::Yaml);
    }

    // Timing Diagram — проверяем первой, robust/concise уникальны
    if source_lower.contains("robust ")
        || source_lower.contains("concise ")
        || source_lower.contains("clock ")
        || source_lower.contains("binary ")
    {
        return Ok(DiagramKind::Timing);
    }

    // State Diagram — проверяем первой, так как [*] может смешаться с другими
    // ВАЖНО: "state " должен быть в начале строки или после пробелов, чтобы не путать с
    // сообщениями типа "state updated" в sequence диаграммах
    if source_lower.contains("[*] -->")
        || source_lower.contains("--> [*]")
        || has_state_keyword(&source_lower)
    {
        return Ok(DiagramKind::State);
    }

    // Object Diagram — проверяем ДО Class
    // object keyword уникален для Object Diagram
    if source_lower.contains("object ") || source_lower.contains("map ") {
        return Ok(DiagramKind::Object);
    }

    // ER Diagram.
    //
    // `entity` используется и в sequence-диаграммах (как тип участника),
    // поэтому одного ключевого слова недостаточно. Различители:
    //  - тело сущности: `entity User { ... }` — в sequence его не бывает;
    //  - ER-связи с кардинальностью: `||--o{`, `}|--`, `|{`, `}o`, `o{`;
    //  - маркеры ключей: `<<pk>>`, `<<fk>>`.
    //
    // Прежняя логика требовала одновременно `entity`, `{` и ER-маркер, из-за
    // чего `entity ORDER` (объявление без тела) не распознавалось вовсе.
    if source_lower.contains("<<pk>>")
        || source_lower.contains("<<fk>>")
        || source_lower.contains("||--")
        || source_lower.contains("}|--")
        || source_lower.contains("}o")
        || source_lower.contains("o{")
        || source_lower.contains("|{")
    {
        return Ok(DiagramKind::Er);
    }
    // `entity X {` — тело сущности, в sequence такого синтаксиса нет
    if source_lower.contains("entity ") && source_lower.contains('{') {
        return Ok(DiagramKind::Er);
    }

    // Class Diagram
    if source_lower.contains("class ")
        || source_lower.contains("interface ")
        || source_lower.contains("abstract class")
        || source_lower.contains("<|--")
        || source_lower.contains("..|>")
    {
        return Ok(DiagramKind::Class);
    }

    // Activity Diagram
    if source_lower.contains(":start")
        || source_lower.contains("start\n")
        || source_lower.contains("if (")
        || source_lower.contains("while (")
    {
        return Ok(DiagramKind::Activity);
    }

    // UseCase Diagram — проверяем РАНЬШЕ Sequence!
    // actor + usecase или actor + (Name) паттерн — это UseCase
    // actor + rectangle — также UseCase
    // :Actor: --> (UseCase) паттерн — также UseCase
    if source_lower.contains("usecase ")
        || source_lower.contains("(usecase)")
        || (source_lower.contains("actor ") && source_lower.contains("rectangle "))
        || (source_lower.contains("actor ") && has_paren_usecase_pattern(&source_lower))
        || has_colon_actor_usecase_pattern(&source_lower)
    {
        return Ok(DiagramKind::UseCase);
    }

    // Sequence Diagram — проверяем РАНЬШЕ Component!
    // participant и actor (без rectangle/usecase) — явные признаки sequence
    // database/collections/queue вместе с participant — это тоже sequence
    // box — группировка участников в sequence диаграммах
    if source_lower.contains("participant ")
        || source_lower.contains("participant\"")
        || (source_lower.contains("actor ")
            && !source_lower.contains("rectangle ")
            && !source_lower.contains("usecase ")
            && !has_paren_usecase_pattern(&source_lower))
        || source_lower.contains("boundary ")
        || source_lower.contains("control ")
        || source_lower.contains("collections ")
        || source_lower.contains("queue ")
        || (source_lower.contains("box ") && source_lower.contains("end box"))
    {
        return Ok(DiagramKind::Sequence);
    }

    // database в сочетании с sequence-паттернами (-> или -->) — это sequence
    if source_lower.contains("database ")
        && (source_lower.contains(" -> ") || source_lower.contains(" --> "))
        && !source_lower.contains("component ")
        && !source_lower.contains("package ")
    {
        return Ok(DiagramKind::Sequence);
    }

    // Deployment Diagram — проверяем ДО Component
    // Специфичные для deployment: device, agent, node с вложенными элементами
    if source_lower.contains("device ")
        || source_lower.contains("agent ")
        || (source_lower.contains("node ") && source_lower.contains("{"))
    {
        return Ok(DiagramKind::Deployment);
    }

    // Component Diagram
    // Проверяем характерные паттерны: [Component], component keyword, package без class
    // ВАЖНО: [Component] паттерн должен быть более точным — только [word] без других символов
    if source_lower.contains("component ")
        || source_lower.contains("cloud ")
        || source_lower.contains("storage ")
        || source_lower.contains("artifact ")
        || source_lower.contains("interface ")
        || (source_lower.contains("package ") && !source_lower.contains("class "))
    {
        return Ok(DiagramKind::Component);
    }

    // Паттерн [Component] --> [Other] — но только если нет sequence-признаков
    if has_component_bracket_pattern(&source_lower)
        && !source_lower.contains(" -> ")  // sequence arrows с пробелами
        && !source_lower.contains(" --> ")
    {
        return Ok(DiagramKind::Component);
    }

    // database/node без явных sequence-паттернов — скорее всего component
    if (source_lower.contains("database ") || source_lower.contains("node "))
        && !source_lower.contains(" -> ")
        && !source_lower.contains(" --> ")
    {
        return Ok(DiagramKind::Component);
    }

    // Sequence Diagram — остальные случаи со стрелками
    if source_lower.contains("-->") || source_lower.contains("->>") || source_lower.contains("->") {
        return Ok(DiagramKind::Sequence);
    }

    Ok(DiagramKind::Unknown)
}

// Парсер для sequence diagrams использует pest грамматику

fn parse_sequence_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_sequence(source)?;
    Ok(Diagram::Sequence(diagram))
}

fn parse_class_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_class(source)?;
    Ok(Diagram::Class(diagram))
}

fn parse_activity_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_activity(source)?;
    Ok(Diagram::Activity(diagram))
}

fn parse_state_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_state(source)?;
    Ok(Diagram::State(diagram))
}

fn parse_component_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_component(source)?;
    Ok(Diagram::Component(diagram))
}

fn parse_deployment_diagram(source: &str) -> Result<Diagram> {
    // Deployment использует ту же грамматику что и Component
    let diagram = parsers::parse_component(source)?;
    Ok(Diagram::Deployment(diagram))
}

fn parse_usecase_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_usecase(source)?;
    Ok(Diagram::UseCase(diagram))
}

fn parse_object_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_object(source)?;
    Ok(Diagram::Object(diagram))
}

fn parse_timing_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_timing(source)?;
    Ok(Diagram::Timing(diagram))
}

fn parse_gantt_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_gantt(source)?;
    Ok(Diagram::Gantt(diagram))
}

fn parse_mindmap_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_mindmap(source)?;
    Ok(Diagram::MindMap(diagram))
}

fn parse_wbs_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_wbs(source)?;
    Ok(Diagram::Wbs(diagram))
}

fn parse_json_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_json(source)?;
    Ok(Diagram::Json(diagram))
}

fn parse_yaml_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_yaml(source)?;
    Ok(Diagram::Yaml(diagram))
}

fn parse_er_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_er(source)?;
    Ok(Diagram::Er(diagram))
}

fn parse_network_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_network(source)?;
    Ok(Diagram::Network(diagram))
}

fn parse_salt_diagram(source: &str) -> Result<Diagram> {
    let diagram = parsers::parse_salt(source)?;
    Ok(Diagram::Salt(diagram))
}

fn parse_archimate_diagram(source: &str) -> Result<Diagram> {
    // Archimate использует тот же синтаксис что и Component
    let diagram = parsers::parse_component(source)?;
    Ok(Diagram::Archimate(diagram))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Позиция ошибки в `SyntaxError` должна быть настоящей, а не нулём.
    ///
    /// Регрессия: раньше все парсеры делали
    /// `line: e.line().to_string().parse().unwrap_or(0)`, но `pest::Error::line()`
    /// возвращает ТЕКСТ строки, а не номер, поэтому поле всегда было нулевым.
    #[test]
    fn test_syntax_error_reports_real_line() {
        let cases = [
            // (исходник, ожидаемая строка)
            (
                "@startuml
participant A
participant B
garbage line here
@enduml
",
                4,
            ),
            (
                "@startuml
garbage here
participant A
@enduml",
                2,
            ),
        ];

        for (source, expected) in cases {
            match parse(source) {
                Err(ParseError::SyntaxError { line, .. }) => {
                    assert_eq!(
                        line,
                        expected,
                        "неверная строка ошибки для:\n{source}\nсообщение: {}",
                        parse(source).unwrap_err()
                    );
                }
                Err(other) => panic!("ожидалась SyntaxError, получено: {other}"),
                Ok(_) => panic!("ожидалась ошибка, но разбор прошёл:\n{source}"),
            }
        }
    }

    #[test]
    fn test_detect_sequence() {
        let source = "@startuml\nAlice -> Bob: Hello\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Sequence);
    }

    /// Ключевые слова внутри подписей сообщений не должны менять тип.
    ///
    /// Регрессия: раньше `Alice -> Bob: class loaded` распознавалась как
    /// Class, рендерилась пустой, и `render()` возвращал `Ok` — молчаливая
    /// потеря данных.
    #[test]
    fn test_message_text_does_not_affect_detection() {
        let triggers = [
            "class loaded",
            "object created",
            "map lookup",
            "usecase done",
            "component deployed",
            "if (ready)",
            "state updated",
            "entity saved",
            "node down",
            "actor left",
            "salt {",
            "participant added",
            "abstract class parsed",
        ];
        for text in triggers {
            let source = format!("@startuml\nAlice -> Bob: {text}\n@enduml");
            assert_eq!(
                detect_diagram_type(&source).unwrap(),
                DiagramKind::Sequence,
                "подпись «{text}» ломает детекцию: должна быть Sequence"
            );
        }
    }

    /// Тексты внутри многострочных блоков тоже не влияют на детекцию.
    #[test]
    fn test_multiline_text_blocks_do_not_affect_detection() {
        // Блочная заметка
        let note = "@startuml\nA -> B\nnote over A\n  class Fake\n  state Fake2\nend note\n@enduml";
        assert_eq!(detect_diagram_type(note).unwrap(), DiagramKind::Sequence);

        // Легенда
        let legend = "@startuml\nA -> B\nlegend\n  class NotAClass\nendlegend\n@enduml";
        assert_eq!(detect_diagram_type(legend).unwrap(), DiagramKind::Sequence);

        // Комментарии
        let comments = "@startuml\n' class Commented\n/' state AlsoCommented '/\nA -> B\n@enduml";
        assert_eq!(
            detect_diagram_type(comments).unwrap(),
            DiagramKind::Sequence
        );
    }

    /// Скелет сохраняет структуру: реальные объявления не теряются.
    #[test]
    fn test_skeleton_keeps_structure() {
        let class = "@startuml\nclass User {\n  + name: String\n}\n@enduml";
        assert_eq!(detect_diagram_type(class).unwrap(), DiagramKind::Class);

        let gantt = "@startgantt\n[Task] lasts 5 days\n@endgantt";
        assert_eq!(detect_diagram_type(gantt).unwrap(), DiagramKind::Gantt);

        // Директива @startjson должна распознаваться несмотря на содержимое
        let json = "@startjson\n{\"class\": \"not a class\"}\n@endjson";
        assert_eq!(detect_diagram_type(json).unwrap(), DiagramKind::Json);
    }

    #[test]
    fn test_detect_class() {
        let source = "@startuml\nclass User\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Class);
    }

    #[test]
    fn test_detect_activity() {
        let source = "@startuml\nstart\n:Action;\nstop\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Activity);
    }

    #[test]
    fn test_detect_state() {
        let source = "@startuml\n[*] --> Active\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::State);
    }

    #[test]
    fn test_detect_deployment() {
        let source = "@startuml\nnode \"Web Server\" {\n    [Apache]\n}\n@enduml";
        assert_eq!(
            detect_diagram_type(source).unwrap(),
            DiagramKind::Deployment
        );

        let source2 = "@startuml\ndevice Mobile\n@enduml";
        assert_eq!(
            detect_diagram_type(source2).unwrap(),
            DiagramKind::Deployment
        );
    }

    #[test]
    fn test_detect_component() {
        let source = "@startuml\ncomponent API\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Component);
    }

    #[test]
    fn test_detect_object() {
        let source = "@startuml\nobject user1\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Object);

        let source2 = "@startuml\nmap config {\n  host => localhost\n}\n@enduml";
        assert_eq!(detect_diagram_type(source2).unwrap(), DiagramKind::Object);
    }

    #[test]
    fn test_detect_timing() {
        let source = "@startuml\nrobust \"Web Browser\" as WB\nconcise \"Server\" as S\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Timing);

        let source2 = "@startuml\nclock clk\nbinary data\n@enduml";
        assert_eq!(detect_diagram_type(source2).unwrap(), DiagramKind::Timing);
    }

    #[test]
    fn test_detect_gantt() {
        let source = "@startgantt\n[Task 1] lasts 5 days\n@endgantt";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Gantt);
    }

    #[test]
    fn test_detect_mindmap() {
        let source = "@startmindmap\n* Root\n** Branch\n@endmindmap";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::MindMap);
    }

    #[test]
    fn test_detect_wbs() {
        let source = "@startwbs\n* Project\n** Phase\n@endwbs";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Wbs);
    }

    #[test]
    fn test_detect_json() {
        let source = "@startjson\n{\"key\": \"value\"}\n@endjson";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Json);
    }

    #[test]
    fn test_detect_yaml() {
        let source = "@startyaml\nkey: value\n@endyaml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Yaml);
    }

    #[test]
    fn test_detect_er() {
        let source = "@startuml\nentity User {\n  id : int\n}\nentity Order {\n  id : int\n}\nUser ||--o{ Order\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Er);
    }

    #[test]
    fn test_detect_network() {
        let source = "@startuml\nnwdiag {\n  network dmz {\n    web01\n  }\n}\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Network);
    }

    #[test]
    fn test_detect_salt() {
        let source = "@startsalt\n{\n  [Button]\n}\n@endsalt";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Salt);
    }

    #[test]
    fn test_detect_archimate() {
        let source = "@startuml\narchimate #Business \"Actor\" as actor\n@enduml";
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Archimate);
    }

    #[test]
    fn test_parse_returns_diagram() {
        let source = "@startuml\nAlice -> Bob\n@enduml";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_sequence_with_box() {
        let source = r#"@startuml
box "Frontend" #LightBlue
    participant "React App" as React
end box
React -> React: test
@enduml"#;
        assert_eq!(detect_diagram_type(source).unwrap(), DiagramKind::Sequence);
    }
}
