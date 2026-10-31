//! Парсер Sequence Diagrams
//!
//! Использует pest грамматику для парсинга PlantUML sequence diagrams.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::common::{Color, LineStyle, Note, NotePosition, NoteShape, Stereotype};
use plantuml_ast::sequence::{
    Activation, ActivationType, ArrowType, AutonumberCommand, AutonumberStart, BoundarySide, Delay,
    Divider, Duration, Fragment, FragmentSection, FragmentType, Message, Participant,
    ParticipantBox, ParticipantType, Reference, Return, SequenceDiagram, SequenceElement,
};

use crate::error::syntax_error_from_pest;
use crate::Result;

#[derive(Parser)]
#[grammar = "grammars/sequence.pest"]
pub struct SequenceParser;

/// Состояние стека фрагментов: (тип, условие фрагмента, текущее условие секции,
/// секции, элементы ВНЕШНЕЙ секции, накопленные до входа в этот фрагмент).
///
/// Последнее поле обязательно: буфер элементов один на всю функцию, и вход во
/// вложенный фрагмент его очищал. Из-за этого сообщения, стоявшие во внешнем
/// фрагменте ПЕРЕД вложенным, пропадали из AST — в `alt` с тремя сообщениями и
/// вложенным `loop` оставался только `loop`.
type FragmentStackEntry = (
    FragmentType,
    Option<String>,
    Option<String>,
    Vec<FragmentSection>,
    Vec<SequenceElement>,
);

/// Оформление открытого фрагмента: название группы, рамка, фон и фон текущей
/// ветки. Стоит рядом со стеком, а не в нём, чтобы не раздувать кортеж.
#[derive(Default, Clone)]
struct FragmentStyle {
    title: Option<String>,
    color: Option<Color>,
    background: Option<Color>,
    section_background: Option<Color>,
}

/// Состояние открытого box: (title, color, participants)
type BoxState = (Option<String>, Option<Color>, Vec<String>);

/// Всё изменяемое состояние разбора, которое не лежит в самой диаграмме.
struct ParseState {
    fragment_stack: Vec<FragmentStackEntry>,
    styles: Vec<FragmentStyle>,
    current_section_elements: Vec<SequenceElement>,
    /// Стек открытых box: вложенные box лежат друг над другом
    boxes: Vec<BoxState>,
}

/// Парсит sequence diagram из исходного кода
pub fn parse_sequence(source: &str) -> Result<SequenceDiagram> {
    let pairs = SequenceParser::parse(Rule::diagram, source)
        .map_err(|e| syntax_error_from_pest(&e, source))?;

    let mut diagram = SequenceDiagram::new();
    let mut state = ParseState {
        fragment_stack: Vec::new(),
        styles: Vec::new(),
        current_section_elements: Vec::new(),
        boxes: Vec::new(),
    };

    for pair in pairs {
        if pair.as_rule() == Rule::diagram {
            for inner in pair.into_inner() {
                process_rule(inner, &mut diagram, &mut state);
            }
        }
    }

    Ok(diagram)
}

/// Обрабатывает правило грамматики
fn process_rule(
    pair: pest::iterators::Pair<Rule>,
    diagram: &mut SequenceDiagram,
    state: &mut ParseState,
) {
    let ParseState {
        fragment_stack,
        styles,
        current_section_elements,
        boxes,
    } = state;
    match pair.as_rule() {
        Rule::box_start => {
            let (title, color) = parse_box_start(pair);
            boxes.push((title, color, Vec::new()));
        }
        Rule::box_end => {
            if let Some((title, color, participants)) = boxes.pop() {
                diagram.add_box(ParticipantBox {
                    title,
                    color,
                    participants,
                    depth: boxes.len(),
                });
            }
        }
        Rule::colon_actor_decl => {
            // Короткая форма актёра `:Alice:`. Грамматика её принимала, но
            // парсер не обрабатывал — участник не создавался, и на диаграмме
            // его не было.
            let name = pair
                .into_inner()
                .find(|inner| inner.as_rule() == Rule::colon_actor_name)
                .map(|inner| inner.as_str().trim().to_string());

            if let Some(name) = name {
                let participant = Participant::actor(name);
                let boxed_name = participant
                    .id
                    .alias
                    .clone()
                    .unwrap_or_else(|| participant.id.name.clone());
                for (_, _, participants) in boxes.iter_mut() {
                    participants.push(boxed_name.clone());
                }
                diagram.add_participant(participant);
            }
        }
        Rule::participant_decl => {
            if let Some(participant) = parse_participant(pair) {
                // Участник принадлежит каждому открытому box: внешний
                // охватывает и вложенные.
                let name = participant
                    .id
                    .alias
                    .clone()
                    .unwrap_or_else(|| participant.id.name.clone());
                for (_, _, participants) in boxes.iter_mut() {
                    participants.push(name.clone());
                }
                diagram.add_participant(participant);
            }
        }
        Rule::message => {
            let parallel = pair
                .clone()
                .into_inner()
                .any(|inner| inner.as_rule() == Rule::parallel_prefix);
            if let Some(message) = parse_message(pair) {
                if parallel {
                    push_element(
                        diagram,
                        fragment_stack,
                        current_section_elements,
                        SequenceElement::Parallel,
                    );
                }
                let element = SequenceElement::Message(message);
                push_element(diagram, fragment_stack, current_section_elements, element);
            }
        }
        Rule::duration_stmt => {
            let mut anchors = Vec::new();
            let mut label = String::new();
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::anchor_tag => {
                        if let Some(name) = inner
                            .into_inner()
                            .find(|part| part.as_rule() == Rule::anchor_name)
                        {
                            anchors.push(name.as_str().to_string());
                        }
                    }
                    Rule::message_text => label = inner.as_str().trim().to_string(),
                    _ => {}
                }
            }
            if let [from, to] = anchors.as_slice() {
                let element = SequenceElement::Duration(Duration {
                    from: from.clone(),
                    to: to.clone(),
                    label,
                });
                push_element(diagram, fragment_stack, current_section_elements, element);
            }
        }
        Rule::fragment_start => {
            let (frag_type, condition, color, background) = parse_fragment_start(pair);
            // Условие из fragment_start становится условием первой секции.
            //
            // Элементы внешней секции ЗАБИРАЕМ в запись стека, а не очищаем
            // буфер: иначе всё, что внешний фрагмент накопил до вложенного,
            // терялось безвозвратно.
            let saved = std::mem::take(current_section_elements);
            fragment_stack.push((frag_type, condition.clone(), condition, vec![], saved));
            styles.push(FragmentStyle {
                title: None,
                color,
                background,
                section_background: None,
            });
        }
        Rule::fragment_else => {
            let (new_condition, new_background) = parse_fragment_else(pair);
            if let Some((_, _, ref mut current_condition, ref mut sections, _)) =
                fragment_stack.last_mut()
            {
                let previous_background = styles
                    .last_mut()
                    .and_then(|style| style.section_background.take());
                // Сохраняем текущую секцию с её условием
                sections.push(FragmentSection {
                    condition: current_condition.take(),
                    elements: std::mem::take(current_section_elements),
                    background: previous_background,
                });
                // Устанавливаем условие для следующей секции
                *current_condition = new_condition;
                if let Some(style) = styles.last_mut() {
                    style.section_background = new_background;
                }
            }
        }
        Rule::fragment_end | Rule::group_end => {
            if let Some((frag_type, condition, current_condition, mut sections, saved)) =
                fragment_stack.pop()
            {
                let style = styles.pop().unwrap_or_default();
                // Добавляем последнюю секцию с её условием
                sections.push(FragmentSection {
                    condition: current_condition,
                    elements: std::mem::take(current_section_elements),
                    background: style.section_background,
                });

                let fragment = Fragment {
                    fragment_type: frag_type,
                    condition,
                    sections,
                    title: style.title,
                    color: style.color,
                    background: style.background,
                };

                // Возвращаем буферу элементы внешней секции и дописываем в
                // них готовый фрагмент.
                *current_section_elements = saved;
                let element = SequenceElement::Fragment(fragment);
                push_element(diagram, fragment_stack, current_section_elements, element);
            }
        }
        Rule::group_start => {
            // `group Название [подпись]`: название стоит на ярлыке вместо
            // слова `group`, подпись в скобках — условие, как у `alt`.
            let (title, label) = parse_group_label(pair);
            let saved = std::mem::take(current_section_elements);
            fragment_stack.push((FragmentType::Group, label.clone(), label, vec![], saved));
            styles.push(FragmentStyle {
                title,
                ..FragmentStyle::default()
            });
        }
        Rule::note_stmt => {
            let parallel = pair
                .clone()
                .into_inner()
                .any(|inner| inner.as_rule() == Rule::parallel_prefix);
            if let Some(note) = parse_note(pair) {
                if parallel {
                    push_element(
                        diagram,
                        fragment_stack,
                        current_section_elements,
                        SequenceElement::Parallel,
                    );
                }
                let element = SequenceElement::Note(note);
                if fragment_stack.is_empty() {
                    diagram.add_element(element);
                } else {
                    current_section_elements.push(element);
                }
            }
        }
        Rule::divider => {
            let text = parse_divider_text(pair);
            let element = SequenceElement::Divider(Divider {
                text: text.unwrap_or_default(),
            });
            if fragment_stack.is_empty() {
                diagram.add_element(element);
            } else {
                current_section_elements.push(element);
            }
        }
        Rule::delay => {
            let text = parse_delay_text(pair);
            let element = SequenceElement::Delay(Delay { text });
            if fragment_stack.is_empty() {
                diagram.add_element(element);
            } else {
                current_section_elements.push(element);
            }
        }
        Rule::title_stmt => {
            if let Some(title) = parse_title(pair) {
                diagram.metadata.title = Some(title);
            }
        }
        Rule::header_stmt => {
            if let Some(header) = parse_title(pair) {
                diagram.metadata.header = Some(header);
            }
        }
        Rule::footer_stmt => {
            if let Some(footer) = parse_title(pair) {
                diagram.metadata.footer = Some(footer);
            }
        }
        Rule::direction_stmt => {
            if let Some(direction) = parse_direction_text(pair.as_str()) {
                diagram.metadata.direction = Some(direction);
            }
        }
        Rule::legend_stmt => {
            // Легенда: текст между `legend` и `endlegend`. Раньше правило
            // грамматики было, а парсер его не обрабатывал — легенда
            // молча терялась.
            diagram.metadata.legend = Some(parse_legend(pair));
        }
        Rule::mainframe_stmt => {
            diagram.metadata.mainframe = parse_optional_text(pair);
        }
        Rule::newpage_stmt => {
            // PlantUML рисует только первую страницу, но текст автора терять
            // нельзя: следующие страницы идут в той же диаграмме, а разрыв
            // отмечен разделителем с названием страницы.
            let title = parse_optional_text(pair);
            diagram.metadata.newpage = title.clone();
            let element = SequenceElement::Divider(Divider {
                text: title.unwrap_or_default(),
            });
            push_element(diagram, fragment_stack, current_section_elements, element);
        }
        Rule::hide_stmt => {
            // hide footbox — нижние блоки участников не рисуются
            diagram.hide_footbox = true;
        }
        Rule::activate_stmt => {
            if let Some((participant_id, color)) = parse_activate(pair) {
                let element = SequenceElement::Activation(Activation {
                    participant: participant_id,
                    activation_type: ActivationType::Activate,
                    color,
                });
                if fragment_stack.is_empty() {
                    diagram.add_element(element);
                } else {
                    current_section_elements.push(element);
                }
            }
        }
        Rule::deactivate_stmt => {
            if let Some(participant_id) = parse_deactivate(pair) {
                let element = SequenceElement::Activation(Activation {
                    participant: participant_id,
                    activation_type: ActivationType::Deactivate,
                    color: None,
                });
                if fragment_stack.is_empty() {
                    diagram.add_element(element);
                } else {
                    current_section_elements.push(element);
                }
            }
        }
        Rule::destroy_stmt => {
            if let Some(participant_id) = parse_destroy(pair) {
                let element = SequenceElement::Activation(Activation {
                    participant: participant_id,
                    activation_type: ActivationType::Destroy,
                    color: None,
                });
                if fragment_stack.is_empty() {
                    diagram.add_element(element);
                } else {
                    current_section_elements.push(element);
                }
            }
        }
        Rule::autonumber => {
            if let Some(cmd) = parse_autonumber(pair) {
                let element = SequenceElement::Autonumber(cmd);
                if fragment_stack.is_empty() {
                    diagram.add_element(element);
                } else {
                    current_section_elements.push(element);
                }
            }
        }
        Rule::return_stmt => {
            let ret = parse_return(pair);
            let element = SequenceElement::Return(ret);
            if fragment_stack.is_empty() {
                diagram.add_element(element);
            } else {
                current_section_elements.push(element);
            }
        }
        Rule::ref_stmt => {
            if let Some(reference) = parse_ref_stmt(pair) {
                let element = SequenceElement::Reference(reference);
                if fragment_stack.is_empty() {
                    diagram.add_element(element);
                } else {
                    current_section_elements.push(element);
                }
            }
        }
        _ => {}
    }
}

/// Кладёт элемент в текущую секцию открытого фрагмента или в корень диаграммы.
fn push_element(
    diagram: &mut SequenceDiagram,
    fragment_stack: &[FragmentStackEntry],
    current_section_elements: &mut Vec<SequenceElement>,
    element: SequenceElement,
) {
    if fragment_stack.is_empty() {
        diagram.add_element(element);
    } else {
        current_section_elements.push(element);
    }
}

/// Парсит объявление участника
fn parse_participant(pair: pest::iterators::Pair<Rule>) -> Option<Participant> {
    let mut participant_type = ParticipantType::Participant;
    let mut name = String::new();
    let mut alias: Option<String> = None;
    let mut stereotype: Option<Stereotype> = None;
    let mut color: Option<Color> = None;
    let mut order: Option<i32> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::participant_type => {
                participant_type = match inner.as_str().to_lowercase().as_str() {
                    "participant" => ParticipantType::Participant,
                    "actor" => ParticipantType::Actor,
                    "boundary" => ParticipantType::Boundary,
                    "control" => ParticipantType::Control,
                    "entity" => ParticipantType::Entity,
                    "database" => ParticipantType::Database,
                    "collections" => ParticipantType::Collections,
                    "queue" => ParticipantType::Queue,
                    _ => ParticipantType::Participant,
                };
            }
            Rule::participant_name => {
                name = extract_name(inner);
            }
            Rule::identifier | Rule::simple_identifier => {
                // Это alias после "as"
                if name.is_empty() {
                    name = inner.as_str().to_string();
                } else {
                    alias = Some(inner.as_str().to_string());
                }
            }
            Rule::stereotype => {
                let s = inner.as_str();
                let content = s.trim_start_matches("<<").trim_end_matches(">>").trim();
                stereotype = Some(parse_stereotype(content));
            }
            Rule::color => {
                let s = inner.as_str();
                color = Some(Color::parse(s));
            }
            Rule::number => {
                if let Ok(n) = inner.as_str().parse() {
                    order = Some(n);
                }
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }

    let mut participant = Participant::new(name, participant_type);
    if let Some(a) = alias {
        participant.id.alias = Some(a);
    }
    participant.stereotype = stereotype;
    participant.color = color;
    participant.order = order;

    Some(participant)
}

/// Разбирает содержимое `<< ... >>`: необязательный кружок `(C,#ADD1B2)` и
/// имя. Цвет кружка кладётся в `background_color`; буква кружка в модели не
/// хранится.
fn parse_stereotype(content: &str) -> Stereotype {
    let (spot_color, name) = match content
        .strip_prefix('(')
        .and_then(|rest| rest.split_once(')'))
    {
        Some((spot, rest)) => (
            spot.split_once(',')
                .map(|(_, color)| Color::parse(color.trim())),
            rest.trim(),
        ),
        None => (None, content),
    };
    let mut stereotype = if name.is_empty() {
        Stereotype::multiple(Vec::new())
    } else {
        Stereotype::new(name)
    };
    stereotype.background_color = spot_color;
    stereotype
}

/// Извлекает имя из quoted_string или identifier
fn extract_name(pair: pest::iterators::Pair<Rule>) -> String {
    let fallback = pair.as_str().trim_matches('"').to_string();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::quoted_string => {
                let s = inner.as_str();
                return s.trim_matches('"').to_string();
            }
            Rule::identifier => {
                return inner.as_str().to_string();
            }
            Rule::inner_string => {
                return inner.as_str().to_string();
            }
            _ => {}
        }
    }
    fallback
}

/// Парсит сообщение
fn parse_message(pair: pest::iterators::Pair<Rule>) -> Option<Message> {
    // Концы сообщения в порядке записи: имя участника либо край диаграммы.
    let mut ends: Vec<(String, Option<BoundarySide>)> = Vec::new();
    let mut label = String::new();
    let mut arrow = ArrowSpec::default();
    let mut anchor: Option<String> = None;
    let mut activate = false;
    let mut deactivate = false;
    let mut create = false;
    let mut destroy = false;
    let mut activation_color: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::participant_ref => ends.push((inner.as_str().to_string(), None)),
            Rule::boundary_left_mark => ends.push((String::new(), Some(BoundarySide::Left))),
            Rule::boundary_right_mark => ends.push((String::new(), Some(BoundarySide::Right))),
            Rule::anchor_tag => {
                anchor = inner
                    .into_inner()
                    .find(|part| part.as_rule() == Rule::anchor_name)
                    .map(|part| part.as_str().to_string());
            }
            Rule::arrow => arrow = parse_arrow(inner.as_str()),
            Rule::target_activation => {
                let (act, deact, crt, dst, color) = parse_target_activation(inner);
                activate = act;
                deactivate = deact;
                create = crt;
                destroy = dst;
                activation_color = color;
            }
            Rule::message_text => {
                label = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    let mut ends = ends.into_iter();
    let (first, first_boundary) = ends.next()?;
    let (second, second_boundary) = ends.next()?;

    // Стрелка налево (`<-`, `<--`): `Alice <-- Bob` означает стрелку от Bob
    // к Alice, поэтому концы меняются вместе со своими краями диаграммы.
    let ((from, from_boundary), (to, to_boundary)) = if arrow.points_left {
        ((second, second_boundary), (first, first_boundary))
    } else {
        ((first, first_boundary), (second, second_boundary))
    };

    let mut message = Message::new(from, to, label);
    message.line_style = arrow.line_style;
    message.arrow_type = arrow.arrow_type;
    message.start_arrow = arrow.start_arrow;
    message.color = arrow.color;
    message.bold = arrow.bold;
    message.shift = arrow.shift;
    message.anchor = anchor;
    message.from_boundary = from_boundary;
    message.to_boundary = to_boundary;
    message.activate = activate;
    message.deactivate = deactivate;
    message.create = create;
    message.destroy = destroy;
    message.activation_color = activation_color;

    Some(message)
}

/// Парсит target_activation (++, --, **, !!, --++ и т.д.)
/// Возвращает: (activate, deactivate, create, destroy, color)
fn parse_target_activation(
    pair: pest::iterators::Pair<Rule>,
) -> (bool, bool, bool, bool, Option<Color>) {
    let mut activate = false;
    let mut deactivate = false;
    let mut create = false;
    let mut destroy = false;
    let mut color: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::source_deactivation => {
                // -- перед действием означает деактивацию source
                deactivate = true;
            }
            Rule::target_action => {
                let action = inner.as_str();
                match action {
                    "++" => activate = true,
                    "**" => create = true,
                    "!!" => destroy = true,
                    _ => {}
                }
            }
            Rule::color => {
                color = Some(Color::parse(inner.as_str()));
            }
            _ => {}
        }
    }

    (activate, deactivate, create, destroy, color)
}

/// Разобранная стрелка сообщения.
#[derive(Default)]
struct ArrowSpec {
    line_style: LineStyle,
    arrow_type: ArrowType,
    /// Наконечник у источника (двусторонние стрелки)
    start_arrow: Option<ArrowType>,
    /// Стрелка смотрит на левый из записанных участников
    points_left: bool,
    color: Option<Color>,
    bold: bool,
    shift: Option<u32>,
}

/// Наконечник из текста стрелки: `>`, `>>`, `\`, `/`, с крестом или кружком.
fn head_type(head: &str, deco: Option<char>) -> ArrowType {
    match deco {
        Some('x') => ArrowType::Cross,
        Some('o') => ArrowType::Circle,
        _ => match head {
            ">>" | "<<" => ArrowType::Thin,
            "\\" | "\\\\" => ArrowType::HalfTop,
            "/" | "//" => ArrowType::HalfBottom,
            _ => ArrowType::Normal,
        },
    }
}

/// Разбирает текст стрелки по частям: смещение `(10)`, стиль в скобках
/// `[#red,bold]`, тело из тире и точек, наконечники по краям.
fn parse_arrow(text: &str) -> ArrowSpec {
    let mut spec = ArrowSpec::default();
    let mut rest = text.to_string();

    // Смещение конца: `->(10)`
    if let (Some(open), true) = (rest.rfind('('), rest.ends_with(')')) {
        spec.shift = rest[open + 1..rest.len() - 1].parse().ok();
        rest.truncate(open);
    }

    // Стиль в квадратных скобках: цвет, `bold`, `dashed`, `dotted`
    let mut dashed_by_style = false;
    if let (Some(open), Some(close)) = (rest.find('['), rest.find(']')) {
        if close > open {
            for part in rest[open + 1..close].split(',') {
                let part = part.trim();
                match part {
                    "bold" => spec.bold = true,
                    "dashed" | "dotted" => dashed_by_style = true,
                    _ if part.starts_with('#') => spec.color = Some(Color::parse(part)),
                    _ => {}
                }
            }
            rest.replace_range(open..=close, "");
        }
    }

    let is_body = |c: char| c == '-' || c == '.';
    let body_start = rest.find(is_body).unwrap_or(0);
    let body_end = rest.rfind(is_body).map_or(rest.len(), |i| i + 1);
    let body = rest.get(body_start..body_end).unwrap_or("");
    let left = rest.get(..body_start).unwrap_or("");
    let right = rest.get(body_end..).unwrap_or("");

    spec.line_style = if dashed_by_style || body.len() >= 2 || body.contains('.') {
        LineStyle::Dashed
    } else {
        LineStyle::Solid
    };

    // Левая часть: необязательные `o`/`x`, затем наконечник
    let left_deco = left.chars().next().filter(|c| matches!(c, 'o' | 'x'));
    let left_head = left.trim_start_matches(['o', 'x']);
    let start = if left_head.is_empty() && left_deco.is_none() {
        None
    } else {
        Some(head_type(left_head, left_deco))
    };

    // Правая часть: наконечник и необязательные `x`/`o`
    let right_deco = right.chars().last().filter(|c| matches!(c, 'o' | 'x'));
    let right_head = right.trim_end_matches(['o', 'x']);
    let end = if right_head.is_empty() && right_deco.is_none() {
        None
    } else {
        Some(head_type(right_head, right_deco))
    };

    match (start, end) {
        (Some(start), None) => {
            spec.points_left = true;
            spec.arrow_type = start;
        }
        (Some(start), Some(end)) => {
            spec.arrow_type = end;
            spec.start_arrow = Some(start);
        }
        (None, Some(end)) => spec.arrow_type = end,
        (None, None) => {}
    }

    spec
}

/// Парсит начало фрагмента: тип, условие, цвет рамки и фона
fn parse_fragment_start(
    pair: pest::iterators::Pair<Rule>,
) -> (FragmentType, Option<String>, Option<Color>, Option<Color>) {
    let mut frag_type = FragmentType::Group;
    let mut condition: Option<String> = None;
    let mut colors: Vec<Color> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::fragment_type => {
                frag_type = match inner.as_str().to_lowercase().as_str() {
                    "alt" => FragmentType::Alt,
                    "opt" => FragmentType::Opt,
                    "loop" => FragmentType::Loop,
                    "par" => FragmentType::Par,
                    "break" => FragmentType::Break,
                    "critical" => FragmentType::Critical,
                    "group" => FragmentType::Group,
                    _ => FragmentType::Group,
                };
            }
            Rule::color => colors.push(Color::parse(inner.as_str())),
            Rule::fragment_condition => {
                condition = Some(inner.as_str().trim().to_string());
            }
            _ => {}
        }
    }

    // Первый цвет — рамка, второй — фон
    let mut colors = colors.into_iter();
    let border = colors.next();
    let background = colors.next();
    (frag_type, condition, border, background)
}

/// Парсит else фрагмента: условие и цвет фона ветки
fn parse_fragment_else(pair: pest::iterators::Pair<Rule>) -> (Option<String>, Option<Color>) {
    let mut condition = None;
    let mut background = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::fragment_condition => condition = Some(inner.as_str().trim().to_string()),
            Rule::color => background = Some(Color::parse(inner.as_str())),
            _ => {}
        }
    }
    (condition, background)
}

/// Парсит заметку
fn parse_note(pair: pest::iterators::Pair<Rule>) -> Option<Note> {
    let mut position = NotePosition::Right;
    let mut anchors: Vec<String> = Vec::new();
    let mut text = String::new();
    let mut shape = NoteShape::Folded;

    fn parse_note_inner(
        pair: pest::iterators::Pair<Rule>,
        position: &mut NotePosition,
        anchors: &mut Vec<String>,
        text: &mut String,
    ) {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::note_over => {
                    *position = NotePosition::Over;
                    parse_note_inner(inner, position, anchors, text);
                }
                Rule::note_left_right | Rule::note_multi => {
                    parse_note_inner(inner, position, anchors, text);
                }
                // hnote/rnote бывают только `over`, форма рамки — их отличие
                Rule::hnote | Rule::rnote => {
                    *position = NotePosition::Over;
                    parse_note_inner(inner, position, anchors, text);
                }
                Rule::note_position => {
                    *position = match inner.as_str().to_lowercase().as_str() {
                        "left" => NotePosition::Left,
                        "right" => NotePosition::Right,
                        "across" => NotePosition::Over, // across тоже считается Over
                        _ => NotePosition::Right,
                    };
                }
                Rule::identifier_list => {
                    for id in inner.into_inner() {
                        if id.as_rule() == Rule::identifier
                            || id.as_rule() == Rule::simple_identifier
                        {
                            anchors.push(id.as_str().to_string());
                        }
                    }
                }
                Rule::identifier | Rule::simple_identifier => {
                    anchors.push(inner.as_str().to_string());
                }
                Rule::note_text | Rule::note_body => {
                    let t = inner.as_str().trim();
                    // Убираем начальное двоеточие если есть
                    *text = t.trim_start_matches(':').trim().to_string();
                }
                _ => {}
            }
        }
    }

    for inner in pair.clone().into_inner() {
        match inner.as_rule() {
            Rule::hnote => shape = NoteShape::Hexagon,
            Rule::rnote => shape = NoteShape::Box,
            _ => {}
        }
    }
    parse_note_inner(pair, &mut position, &mut anchors, &mut text);

    Some(Note {
        position,
        anchors,
        text,
        background_color: None,
        shape,
    })
}

/// Парсит текст разделителя
fn parse_divider_text(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::divider_text {
            let text = inner.as_str().trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

/// Парсит текст задержки
fn parse_delay_text(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::delay_text {
            let text = inner.as_str().trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

/// Парсит заголовок
/// Извлекает текст легенды между `legend` и `endlegend`.
fn parse_legend(pair: pest::iterators::Pair<Rule>) -> String {
    let raw = pair.as_str();
    let start = raw.find("legend").map(|i| i + "legend".len()).unwrap_or(0);
    let end = raw.rfind("endlegend").unwrap_or(raw.len());
    let body = raw.get(start..end).unwrap_or("");

    // Первая строка — модификатор (left/right/center), её пропускаем
    let mut lines = body.lines();
    let first = lines.next().unwrap_or("").trim();
    let skip_first = matches!(first, "left" | "right" | "center" | "top" | "bottom");

    let rest: Vec<&str> = if skip_first {
        lines.collect()
    } else {
        std::iter::once(first).chain(lines).collect()
    };

    rest.join("\n").trim().to_string()
}

/// Извлекает необязательный текст из оператора вида `mainframe Заголовок`.
///
/// Для `newpage Раздел 2` возвращает `Some("Раздел 2")`, для `mainframe`
/// без заголовка — `None`.
fn parse_optional_text(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    let raw = pair.as_str();
    let keyword = raw.split_whitespace().next().unwrap_or("");
    let rest = raw.get(keyword.len()..).unwrap_or("").trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

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

/// Парсит activate
fn parse_activate(pair: pest::iterators::Pair<Rule>) -> Option<(String, Option<Color>)> {
    let mut participant: Option<String> = None;
    let mut color: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier | Rule::simple_identifier => {
                participant = Some(inner.as_str().to_string());
            }
            Rule::color => {
                color = Some(Color::parse(inner.as_str()));
            }
            _ => {}
        }
    }

    participant.map(|p| (p, color))
}

/// Парсит deactivate
fn parse_deactivate(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::identifier || inner.as_rule() == Rule::simple_identifier {
            return Some(inner.as_str().to_string());
        }
    }
    None
}

/// Парсит destroy
fn parse_destroy(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::participant_name => {
                return Some(extract_name(inner));
            }
            Rule::identifier | Rule::simple_identifier => {
                return Some(inner.as_str().to_string());
            }
            _ => {}
        }
    }
    None
}

/// Парсит начало box
fn parse_box_start(pair: pest::iterators::Pair<Rule>) -> (Option<String>, Option<Color>) {
    let mut title: Option<String> = None;
    let mut color: Option<Color> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::box_title => {
                // Может быть quoted_string или simple_identifier
                let text = inner.as_str();
                if text.starts_with('"') {
                    title = Some(text.trim_matches('"').to_string());
                } else {
                    title = Some(text.to_string());
                }
            }
            Rule::quoted_string => {
                title = Some(inner.as_str().trim_matches('"').to_string());
            }
            Rule::color => {
                color = Some(Color::parse(inner.as_str()));
            }
            _ => {}
        }
    }

    (title, color)
}

/// Парсит команду autonumber
/// Поддерживаемые варианты:
/// - autonumber                    - включить с 1
/// - autonumber 15                 - начать с 15
/// - autonumber 40 10              - начать с 40, шаг 10
/// - autonumber "[00]"             - формат без чисел
/// - autonumber 10 "[00]"          - начало + формат
/// - autonumber 10 10 "[00]"       - начало + шаг + формат
/// - autonumber stop               - остановить
/// - autonumber resume             - продолжить
/// - autonumber resume 50          - продолжить с 50
/// - autonumber resume "[00]"      - продолжить с форматом
/// - autonumber inc A              - инкремент уровня
fn parse_autonumber(pair: pest::iterators::Pair<Rule>) -> Option<AutonumberCommand> {
    let text = pair.as_str();

    // Проверяем специальные команды
    if text.contains("stop") {
        return Some(AutonumberCommand::Stop);
    }

    if text.contains("resume") {
        // Парсим параметры после resume
        let params = parse_autonumber_params_from_inner(pair);
        return Some(AutonumberCommand::Resume(params));
    }

    if text.contains(" inc ") {
        // autonumber inc <level>
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::identifier || inner.as_rule() == Rule::simple_identifier {
                return Some(AutonumberCommand::Inc(inner.as_str().to_string()));
            }
        }
        return None;
    }

    // Обычный autonumber [start] [step] [format]
    let params = parse_autonumber_params_from_inner(pair);
    Some(AutonumberCommand::Start(params.unwrap_or_default()))
}

/// Парсит параметры autonumber из inner rules
fn parse_autonumber_params_from_inner(
    pair: pest::iterators::Pair<Rule>,
) -> Option<AutonumberStart> {
    let mut start: Option<u32> = None;
    let mut step: Option<u32> = None;
    let mut format: Option<String> = None;
    let mut levels: Option<Vec<u32>> = None;
    let mut has_params = false;

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::autonumber_params {
            has_params = true;
            // Парсим вложенные элементы
            for param in inner.into_inner() {
                match param.as_rule() {
                    Rule::multilevel_number => {
                        // Многоуровневая нумерация: "1.1" -> [1, 1]
                        let parsed: Vec<u32> = param
                            .as_str()
                            .split('.')
                            .filter_map(|s| s.parse().ok())
                            .collect();
                        if !parsed.is_empty() {
                            levels = Some(parsed);
                        }
                    }
                    Rule::number => {
                        if let Ok(n) = param.as_str().parse() {
                            if start.is_none() {
                                start = Some(n);
                            } else {
                                step = Some(n);
                            }
                        }
                    }
                    Rule::quoted_string => {
                        let s = param.as_str();
                        format = Some(s.trim_matches('"').to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    if has_params || start.is_some() || format.is_some() || levels.is_some() {
        let mut params = AutonumberStart::new(start, step, format);
        params.levels = levels;
        Some(params)
    } else {
        None
    }
}

/// Разбирает метку группы `Название [подпись]`: название стоит на ярлыке,
/// подпись в скобках — условие. Возвращает (название, подпись).
fn parse_group_label(pair: pest::iterators::Pair<Rule>) -> (Option<String>, Option<String>) {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::group_label {
            let text = inner.as_str().trim();
            if text.is_empty() {
                return (None, None);
            }
            return match text.find('[') {
                Some(open) if text.ends_with(']') => {
                    let title = text[..open].trim();
                    let note = text[open + 1..text.len() - 1].trim();
                    (
                        (!title.is_empty()).then(|| title.to_string()),
                        (!note.is_empty()).then(|| note.to_string()),
                    )
                }
                _ => (Some(text.to_string()), None),
            };
        }
    }
    (None, None)
}

/// Парсит return statement
fn parse_return(pair: pest::iterators::Pair<Rule>) -> Return {
    let mut label: Option<String> = None;

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::message_text {
            let text = inner.as_str().trim();
            if !text.is_empty() {
                label = Some(text.to_string());
            }
        }
    }

    Return { label }
}

/// Парсит ref statement (ссылку на другую диаграмму)
/// Поддерживает:
/// - `ref over Alice, Bob: See other diagram`
/// - многострочную форму `ref over Alice, Bob` … `end ref`
fn parse_ref_stmt(pair: pest::iterators::Pair<Rule>) -> Option<Reference> {
    let mut participants: Vec<String> = Vec::new();
    let mut text = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier_list => {
                for id in inner.into_inner() {
                    match id.as_rule() {
                        Rule::identifier | Rule::simple_identifier => {
                            participants.push(id.as_str().to_string());
                        }
                        _ => {}
                    }
                }
            }
            Rule::ref_text | Rule::ref_body => {
                text = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    Some(Reference { text, participants })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Короткая форма актёра `:Alice:`.
    ///
    /// Регрессия: грамматика её не знала вовсе, поэтому такая диаграмма не
    /// разбиралась. После добавления правила выяснилось, что парсер его не
    /// обрабатывает — участник не создавался.
    #[test]
    fn test_short_actor_form() {
        let diagram = parse_sequence("@startuml\n:Alice:\n:Bob:\n@enduml")
            .expect("диаграмма должна разбираться");
        assert_eq!(diagram.participants.len(), 2, "участники не созданы");
        assert_eq!(diagram.participants[0].id.name, "Alice");
        assert_eq!(
            diagram.participants[0].participant_type,
            plantuml_ast::sequence::ParticipantType::Actor
        );

        // Сообщение между короткими актёрами
        let diagram = parse_sequence("@startuml\n:Alice: -> :Bob: : привет\n@enduml")
            .expect("диаграмма должна разбираться");
        assert!(!diagram.elements.is_empty());
    }

    /// Задержка: `...` и `delay: текст` — обе формы проверены на
    /// plantuml.com.
    ///
    /// Регрессия: ранее была добавлена форма `delay текст` БЕЗ двоеточия.
    /// Проверка на сервере показала, что такого синтаксиса нет (HTTP 400);
    /// правильная форма — с двоеточием.
    #[test]
    fn test_delay_forms() {
        // Допустимые формы
        for source in [
            "@startuml\nA -> B\ndelay: 10 минут\nB -> A\n@enduml",
            "@startuml\nA -> B\n... текст\nB -> A\n@enduml",
            "@startuml\nA -> B\n... 5 минут ...\nB -> A\n@enduml",
        ] {
            assert!(
                parse_sequence(source).is_ok(),
                "форма должна разбираться: {source}"
            );
        }

        // Формы, которых в PlantUML нет
        for source in [
            "@startuml\nA -> B\ndelay 10 минут\n@enduml",
            "@startuml\nA -> B\ndelay 10\n@enduml",
        ] {
            assert!(
                parse_sequence(source).is_err(),
                "формы без двоеточия в PlantUML нет: {source}"
            );
        }
    }

    /// `legend`, `mainframe` и `newpage` попадают в метаданные.
    ///
    /// Регрессия: правила грамматики были добавлены, но парсер их не
    /// обрабатывал — то есть конструкции принимались и молча терялись.
    #[test]
    fn test_legend_mainframe_newpage() {
        let source = "@startuml\nlegend left\nМоя легенда\nendlegend\nmainframe Рамка\nA -> B\nnewpage Раздел 2\nB -> A\n@enduml";
        let diagram = parse_sequence(source).expect("диаграмма должна разбираться");

        assert_eq!(diagram.metadata.legend.as_deref(), Some("Моя легенда"));
        assert_eq!(diagram.metadata.mainframe.as_deref(), Some("Рамка"));
        assert_eq!(diagram.metadata.newpage.as_deref(), Some("Раздел 2"));
    }

    #[test]
    fn test_parse_basic_sequence() {
        let source = r#"@startuml
Alice -> Bob: Hello
Bob --> Alice: Hi
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.elements.len(), 2);
    }

    #[test]
    fn test_parse_participants() {
        let source = r#"@startuml
participant Alice
actor Bob
database DB
Alice -> Bob: message
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.participants.len(), 3);
        assert_eq!(diagram.participants[0].id.name, "Alice");
        assert_eq!(
            diagram.participants[0].participant_type,
            ParticipantType::Participant
        );
        assert_eq!(
            diagram.participants[1].participant_type,
            ParticipantType::Actor
        );
        assert_eq!(
            diagram.participants[2].participant_type,
            ParticipantType::Database
        );
    }

    #[test]
    fn test_parse_participant_with_alias() {
        let source = r#"@startuml
participant "Сервис Обработки" as Processor
Processor -> Processor: Инициализация
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        // Должен быть только ОДИН участник
        assert_eq!(
            diagram.participants.len(),
            1,
            "Expected 1 participant, got {}: {:?}",
            diagram.participants.len(),
            diagram
                .participants
                .iter()
                .map(|p| (&p.id.name, &p.id.alias))
                .collect::<Vec<_>>()
        );

        // Проверяем имя и алиас
        assert_eq!(diagram.participants[0].id.name, "Сервис Обработки");
        assert_eq!(
            diagram.participants[0].id.alias,
            Some("Processor".to_string())
        );

        // Проверяем сообщение - использует alias
        if let SequenceElement::Message(msg) = &diagram.elements[0] {
            assert_eq!(msg.from, "Processor");
            assert_eq!(msg.to, "Processor");
        } else {
            panic!("Expected Message");
        }
    }

    #[test]
    fn test_parse_messages() {
        let source = r#"@startuml
Alice -> Bob: Hello
Bob --> Alice: Hi
Alice ->> Bob: Async
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.elements.len(), 3);

        if let SequenceElement::Message(msg) = &diagram.elements[0] {
            assert_eq!(msg.from, "Alice");
            assert_eq!(msg.to, "Bob");
            assert_eq!(msg.label, "Hello");
            assert_eq!(msg.line_style, LineStyle::Solid);
        } else {
            panic!("Expected Message");
        }

        if let SequenceElement::Message(msg) = &diagram.elements[1] {
            assert_eq!(msg.line_style, LineStyle::Dashed);
        } else {
            panic!("Expected Message");
        }
    }

    #[test]
    fn test_parse_fragment() {
        let source = r#"@startuml
Alice -> Bob: Request

alt Success
    Bob --> Alice: OK
else Failure
    Bob --> Alice: Error
end
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        // Should have 1 message + 1 fragment
        assert!(!diagram.elements.is_empty());
    }

    #[test]
    fn test_parse_activate_deactivate() {
        let source = r#"@startuml
Alice -> Bob: Request
activate Bob
Bob --> Alice: Response
deactivate Bob
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        // 2 messages + 1 activate + 1 deactivate = 4 elements
        assert_eq!(diagram.elements.len(), 4);

        // Проверяем типы элементов
        match &diagram.elements[1] {
            SequenceElement::Activation(act) => {
                assert_eq!(act.participant, "Bob");
                assert_eq!(act.activation_type, ActivationType::Activate);
            }
            _ => panic!("Expected Activation after first message"),
        }

        match &diagram.elements[3] {
            SequenceElement::Activation(act) => {
                assert_eq!(act.participant, "Bob");
                assert_eq!(act.activation_type, ActivationType::Deactivate);
            }
            _ => panic!("Expected Deactivation after second message"),
        }
    }

    #[test]
    fn test_parse_activate_with_color() {
        let source = r#"@startuml
Alice -> Bob: Request
activate Bob #FFBBBB
Bob --> Alice: Response
deactivate Bob
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[1] {
            SequenceElement::Activation(act) => {
                assert_eq!(act.participant, "Bob");
                assert!(act.color.is_some(), "Expected color on activation");
            }
            _ => panic!("Expected Activation"),
        }
    }

    #[test]
    fn test_parse_destroy() {
        let source = r#"@startuml
Alice -> Bob: Request
destroy Bob
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[1] {
            SequenceElement::Activation(act) => {
                assert_eq!(act.participant, "Bob");
                assert_eq!(act.activation_type, ActivationType::Destroy);
            }
            _ => panic!("Expected Destroy activation"),
        }
    }

    #[test]
    fn test_parse_note_right() {
        let source = r#"@startuml
Alice -> Bob: Hello
note right: This is a note
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.elements.len(), 2);

        match &diagram.elements[1] {
            SequenceElement::Note(note) => {
                assert_eq!(note.position, NotePosition::Right);
                assert!(note.text.contains("This is a note"));
            }
            _ => panic!("Expected Note"),
        }
    }

    #[test]
    fn test_parse_note_over() {
        let source = r#"@startuml
Alice -> Bob: Hello
note over Alice, Bob: Shared note
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[1] {
            SequenceElement::Note(note) => {
                assert_eq!(note.position, NotePosition::Over);
                assert!(note.anchors.contains(&"Alice".to_string()));
                assert!(note.anchors.contains(&"Bob".to_string()));
            }
            _ => panic!("Expected Note"),
        }
    }

    #[test]
    fn test_parse_divider() {
        let source = r#"@startuml
Alice -> Bob: Hello
== Section 1 ==
Bob --> Alice: Hi
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.elements.len(), 3);

        match &diagram.elements[1] {
            SequenceElement::Divider(div) => {
                assert_eq!(div.text, "Section 1");
            }
            _ => panic!("Expected Divider"),
        }
    }

    #[test]
    fn test_parse_delay() {
        let source = r#"@startuml
Alice -> Bob: Hello
...5 minutes later...
Bob --> Alice: Hi
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[1] {
            SequenceElement::Delay(delay) => {
                assert!(delay.text.as_ref().unwrap().contains("minutes"));
            }
            _ => panic!("Expected Delay"),
        }
    }

    #[test]
    fn test_parse_simple_box() {
        let source = r#"@startuml
box
participant Alice
end box
Alice -> Alice: Test
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.boxes.len(), 1, "Expected 1 box");
        assert_eq!(
            diagram.boxes[0].participants.len(),
            1,
            "Expected 1 participant in box"
        );
    }

    #[test]
    fn test_parse_box_with_title() {
        let source = r#"@startuml
box "Frontend"
participant Alice
end box
Alice -> Alice: Test
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.boxes.len(), 1, "Expected 1 box");
        assert_eq!(diagram.boxes[0].title, Some("Frontend".to_string()));
    }

    #[test]
    fn test_parse_box_with_color() {
        let source = r#"@startuml
box "Frontend" #LightBlue
participant Alice
end box
Alice -> Alice: Test
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.boxes.len(), 1, "Expected 1 box");
        assert!(diagram.boxes[0].color.is_some(), "Expected color in box");
    }

    #[test]
    fn test_parse_box_with_cyrillic_title() {
        let source = r#"@startuml
box "Фронтенд" #LightBlue
participant "React App" as React
end box
React -> React: test
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.boxes.len(), 1, "Expected 1 box");
        assert_eq!(diagram.boxes[0].title, Some("Фронтенд".to_string()));
    }

    #[test]
    fn test_parse_full_box_example() {
        // Точный код со скриншота
        let source = r#"@startuml
box "Фронтенд" #LightBlue
    participant "React App" as React
    participant "Redux Store" as Redux
end box

box "Бэкенд" #LightGreen
    participant "API Gateway" as API
    participant "Auth Service" as Auth
    participant "User Service" as User
end box

React -> Redux: dispatch(login)
Redux -> API: POST /auth/login
API -> Auth: validateCredentials
Auth -> User: getUserById
User --> Auth: user data
Auth --> API: JWT token
API --> Redux: { token, user }
Redux --> React: state updated
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.boxes.len(), 2, "Expected 2 boxes");
        assert_eq!(diagram.boxes[0].title, Some("Фронтенд".to_string()));
        assert_eq!(diagram.boxes[1].title, Some("Бэкенд".to_string()));
        assert_eq!(diagram.participants.len(), 5, "Expected 5 participants");
    }

    // ============== Тесты для синтаксиса активации через стрелку ==============

    #[test]
    fn test_parse_shortcut_activation() {
        // Синтаксис: Alice -> Bob++ : hello (активация Bob)
        let source = r#"@startuml
Alice -> Bob++: hello
Bob --> Alice: response
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.elements.len(), 2);

        match &diagram.elements[0] {
            SequenceElement::Message(msg) => {
                assert_eq!(msg.from, "Alice");
                assert_eq!(msg.to, "Bob");
                assert!(msg.activate, "Expected activate=true for Bob++");
                assert!(!msg.deactivate, "Expected deactivate=false");
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_parse_shortcut_activation_with_color() {
        // Синтаксис: Alice -> Bob++ #FFBBBB : hello
        let source = r#"@startuml
Alice -> Bob++ #FFBBBB: hello
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Message(msg) => {
                assert!(msg.activate, "Expected activate=true");
                assert!(
                    msg.activation_color.is_some(),
                    "Expected color for activation"
                );
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_parse_shortcut_deactivate_activate() {
        // Синтаксис: Bob -->-- Alice : done (деактивация Bob)
        // или: alice -> bob --++ : hello (деактивация alice, активация bob)
        let source = r#"@startuml
Alice -> Bob--++: hello
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Message(msg) => {
                assert!(msg.activate, "Expected activate=true for Bob");
                assert!(msg.deactivate, "Expected deactivate=true for Alice");
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_parse_shortcut_create() {
        // Синтаксис: Alice -> Bob** : create (создание Bob)
        let source = r#"@startuml
Alice -> Bob**: create new
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Message(msg) => {
                assert!(msg.create, "Expected create=true for Bob");
                assert!(!msg.activate, "Expected activate=false");
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_parse_shortcut_destroy() {
        // Синтаксис: Alice -> Bob!! : destroy (уничтожение Bob)
        let source = r#"@startuml
Alice -> Bob!!: destroy
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Message(msg) => {
                assert!(msg.destroy, "Expected destroy=true for Bob");
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_parse_complex_activation_example() {
        // Пример из скриншота пользователя
        let source = r#"@startuml
participant "Потребитель API" as apiConsumer
participant "IGA.SSO" as iam

apiConsumer->iam++: Атентификация - Client Credentials flow
iam-->apiConsumer: Технический токен
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();
        assert_eq!(diagram.participants.len(), 2);
        assert_eq!(diagram.elements.len(), 2);

        // Первое сообщение должно активировать iam
        match &diagram.elements[0] {
            SequenceElement::Message(msg) => {
                assert_eq!(msg.from, "apiConsumer");
                assert_eq!(msg.to, "iam");
                assert!(msg.activate, "Expected iam to be activated via ++");
            }
            _ => panic!("Expected Message"),
        }
    }

    // ============== Тесты для autonumber ==============

    #[test]
    fn test_parse_autonumber_basic() {
        let source = r#"@startuml
autonumber
Alice -> Bob: first
Alice -> Bob: second
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Autonumber(cmd) => match cmd {
                AutonumberCommand::Start(params) => {
                    assert!(params.start.is_none() || params.start == Some(1));
                }
                _ => panic!("Expected AutonumberCommand::Start"),
            },
            _ => panic!("Expected Autonumber element"),
        }
    }

    /// Многострочная заметка `note over A` ... `end note`.
    ///
    /// Регрессия: правило многострочной заметки стояло в грамматике после
    /// однострочного и перехватывалось им, поэтому такая форма давала ошибку
    /// парсинга.
    #[test]
    fn test_parse_multiline_note_over() {
        let source = "@startuml\nA -> B\nnote over A\n строка1\n строка2\nend note\n@enduml";
        let diagram = parse_sequence(source).expect("разбор многострочной заметки");

        let note = diagram
            .elements
            .iter()
            .find_map(|e| match e {
                SequenceElement::Note(n) => Some(n),
                _ => None,
            })
            .expect("заметка не найдена");
        assert!(
            note.text.contains("строка1"),
            "текст заметки: {:?}",
            note.text
        );
        assert!(
            note.text.contains("строка2"),
            "текст заметки: {:?}",
            note.text
        );
    }

    /// Многострочная заметка с привязкой `note left of A`.
    #[test]
    fn test_parse_multiline_note_left_of() {
        let source = "@startuml\nA -> B\nnote left of A\n текст\nend note\n@enduml";
        let diagram = parse_sequence(source).expect("разбор заметки слева");
        assert!(diagram
            .elements
            .iter()
            .any(|e| matches!(e, SequenceElement::Note(_))));
    }

    /// `group Название ... end` разбирается как фрагмент.
    ///
    /// Регрессия: правило было в грамматике, но парсер его не обрабатывал,
    /// и группа молча исчезала из вывода.
    #[test]
    fn test_parse_group() {
        let source = "@startuml\ngroup Группа\nA -> B\nend\n@enduml";
        let diagram = parse_sequence(source).expect("разбор группы");

        let fragment = diagram
            .elements
            .iter()
            .find_map(|e| match e {
                SequenceElement::Fragment(f) => Some(f),
                _ => None,
            })
            .expect("группа не найдена");
        assert_eq!(fragment.fragment_type, FragmentType::Group);
        assert_eq!(fragment.title.as_deref(), Some("Группа"));
    }

    /// `group Название [подпись]`: название на ярлыке, подпись — условие.
    #[test]
    fn test_parse_group_title_and_note() {
        let source = "@startuml
group Заголовок [подпись]
A -> B
end
@enduml";
        let diagram = parse_sequence(source).expect("разбор группы");
        let Some(SequenceElement::Fragment(fragment)) = diagram.elements.first() else {
            panic!("группа не найдена");
        };
        assert_eq!(fragment.title.as_deref(), Some("Заголовок"));
        assert_eq!(fragment.condition.as_deref(), Some("подпись"));
    }

    fn first_message(source: &str) -> Message {
        let diagram = parse_sequence(source).unwrap_or_else(|e| panic!("{source}\n{e}"));
        diagram
            .elements
            .iter()
            .find_map(|e| match e {
                SequenceElement::Message(m) => Some(m.clone()),
                _ => None,
            })
            .expect("сообщение не найдено")
    }

    /// Все формы стрелок из справочника: наконечники, цвет, смещение.
    #[test]
    fn test_parse_arrow_forms() {
        let wrap = |arrow: &str| format!("@startuml\nA {arrow} B : t\n@enduml");
        for (arrow, head, dashed, left) in [
            ("->", ArrowType::Normal, false, false),
            ("-->", ArrowType::Normal, true, false),
            ("->>", ArrowType::Thin, false, false),
            ("->x", ArrowType::Cross, false, false),
            ("->o", ArrowType::Circle, false, false),
            ("-\\", ArrowType::HalfTop, false, false),
            ("-/", ArrowType::HalfBottom, false, false),
            ("//--", ArrowType::HalfBottom, true, true),
            ("<-", ArrowType::Normal, false, true),
        ] {
            let message = first_message(&wrap(arrow));
            assert_eq!(message.arrow_type, head, "{arrow}");
            assert_eq!(message.line_style == LineStyle::Dashed, dashed, "{arrow}");
            // Стрелка налево меняет концы местами
            assert_eq!(message.from == "B", left, "{arrow}");
        }

        let both = first_message(&wrap("<->"));
        assert_eq!(both.start_arrow, Some(ArrowType::Normal));
        assert_eq!((both.from.as_str(), both.to.as_str()), ("A", "B"));

        let colored = first_message(&wrap("-[#red]>"));
        assert!(colored.color.is_some());
        assert_eq!(colored.line_style, LineStyle::Solid);

        let shifted = first_message(&wrap("->(10)"));
        assert_eq!(shifted.shift, Some(10));
    }

    /// Сообщения за краем диаграммы: `[->`, `->]`, `?->`.
    #[test]
    fn test_parse_boundary_messages() {
        let incoming = first_message("@startuml\n[-> A : x\n@enduml");
        assert_eq!(incoming.from_boundary, Some(BoundarySide::Left));
        assert_eq!(incoming.to, "A");

        let outgoing = first_message("@startuml\nA ->] : x\n@enduml");
        assert_eq!(outgoing.to_boundary, Some(BoundarySide::Right));
        assert_eq!(outgoing.from, "A");

        let short = first_message("@startuml\n?-> A : x\n@enduml");
        assert_eq!(short.from_boundary, Some(BoundarySide::Left));
    }

    /// Фрагмент с цветами и пустым `else`.
    #[test]
    fn test_parse_fragment_colors() {
        let source =
            "@startuml\nalt#Gold #LightBlue успех\nA -> B\nelse #Pink ошибка\nA -> B\nend\n@enduml";
        let diagram = parse_sequence(source).expect("разбор");
        let Some(SequenceElement::Fragment(fragment)) = diagram.elements.first() else {
            panic!("фрагмент не найден");
        };
        assert!(fragment.color.is_some() && fragment.background.is_some());
        assert_eq!(fragment.condition.as_deref(), Some("успех"));
        assert_eq!(fragment.sections[1].condition.as_deref(), Some("ошибка"));
        assert!(fragment.sections[1].background.is_some());
    }

    /// Якоря, длительность, параллельные `&` и `/`, вложенные box.
    #[test]
    fn test_parse_anchors_parallel_and_nested_box() {
        let source = "@startuml\n{start} A -> B : a\n& B -> C : b\n{end} C -> A : c\n{start} <-> {end} : t\nnote over A : n1\n/ note over B : n2\n@enduml";
        let diagram = parse_sequence(source).expect("разбор");
        let kinds: Vec<&str> = diagram
            .elements
            .iter()
            .map(|e| match e {
                SequenceElement::Message(_) => "m",
                SequenceElement::Parallel => "p",
                SequenceElement::Duration(_) => "d",
                SequenceElement::Note(_) => "n",
                _ => "?",
            })
            .collect();
        assert_eq!(kinds, ["m", "p", "m", "m", "d", "n", "p", "n"]);

        let source = "@startuml\nbox \"Outer\"\nparticipant A\nbox \"Inner\"\nparticipant B\nend box\nend box\n@enduml";
        let diagram = parse_sequence(source).expect("разбор");
        let outer = diagram
            .boxes
            .iter()
            .find(|b| b.depth == 0)
            .expect("внешний");
        let inner = diagram
            .boxes
            .iter()
            .find(|b| b.depth == 1)
            .expect("вложенный");
        assert_eq!(outer.participants, ["A", "B"]);
        assert_eq!(inner.participants, ["B"]);
    }

    /// `end group` и группа без метки.
    #[test]
    fn test_parse_group_forms() {
        for source in [
            "@startuml\ngroup Группа\nA -> B\nend group\n@enduml",
            "@startuml\ngroup\nA -> B\nend\n@enduml",
        ] {
            let diagram =
                parse_sequence(source).unwrap_or_else(|e| panic!("не разобрано: {source}\n{e}"));
            assert!(
                diagram
                    .elements
                    .iter()
                    .any(|e| matches!(e, SequenceElement::Fragment(_))),
                "группа не найдена в: {source}"
            );
        }
    }

    /// `header` и `footer` разбираются в метаданные.
    ///
    /// Регрессия: эти директивы не были описаны в грамматике, поэтому
    /// диаграмма с ними не парсилась вовсе.
    #[test]
    fn test_parse_header_and_footer() {
        let source = "@startuml\nheader Верхний\nfooter Нижний\nA -> B\n@enduml";
        let diagram = parse_sequence(source).expect("разбор должен пройти");

        assert_eq!(diagram.metadata.header.as_deref(), Some("Верхний"));
        assert_eq!(diagram.metadata.footer.as_deref(), Some("Нижний"));
    }

    /// `hide footbox` выставляет флаг в AST.
    #[test]
    fn test_parse_hide_footbox() {
        let source = "@startuml\nhide footbox\nA -> B\n@enduml";
        let diagram = parse_sequence(source).expect("разбор должен пройти");

        assert!(diagram.hide_footbox, "флаг hide_footbox не выставлен");

        // Без директивы флаг снят
        let plain = parse_sequence("@startuml\nA -> B\n@enduml").unwrap();
        assert!(!plain.hide_footbox);
    }

    /// `title` попадает в метаданные и выживает в layout.
    #[test]
    fn test_parse_title_reaches_metadata() {
        let source = "@startuml\ntitle Моя диаграмма\nA -> B\n@enduml";
        let diagram = parse_sequence(source).expect("разбор должен пройти");
        assert_eq!(diagram.metadata.title.as_deref(), Some("Моя диаграмма"));
    }

    #[test]
    fn test_parse_autonumber_with_start() {
        let source = r#"@startuml
autonumber 10
Alice -> Bob: message
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Autonumber(AutonumberCommand::Start(params)) => {
                assert_eq!(params.start, Some(10));
            }
            _ => panic!("Expected Autonumber with start=10"),
        }
    }

    #[test]
    fn test_parse_autonumber_multilevel() {
        // autonumber 1.1 — многоуровневая нумерация
        let source = r#"@startuml
autonumber 1.1
Alice -> Bob: message
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Autonumber(AutonumberCommand::Start(params)) => {
                assert_eq!(params.levels, Some(vec![1, 1]));
            }
            _ => panic!("Expected Autonumber with levels=[1,1]"),
        }
    }

    #[test]
    fn test_parse_autonumber_multilevel_three() {
        // autonumber 1.2.3 — три уровня
        let source = r#"@startuml
autonumber 1.2.3
Alice -> Bob: message
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Autonumber(AutonumberCommand::Start(params)) => {
                assert_eq!(params.levels, Some(vec![1, 2, 3]));
            }
            _ => panic!("Expected Autonumber with levels=[1,2,3]"),
        }
    }

    #[test]
    fn test_parse_autonumber_with_start_and_step() {
        let source = r#"@startuml
autonumber 10 5
Alice -> Bob: message
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Autonumber(AutonumberCommand::Start(params)) => {
                assert_eq!(params.start, Some(10));
                assert_eq!(params.step, Some(5));
            }
            _ => panic!("Expected Autonumber with start=10, step=5"),
        }
    }

    #[test]
    fn test_parse_autonumber_with_format() {
        let source = r#"@startuml
autonumber "[00]"
Alice -> Bob: message
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Autonumber(AutonumberCommand::Start(params)) => {
                assert_eq!(params.format, Some("[00]".to_string()));
            }
            _ => panic!("Expected Autonumber with format"),
        }
    }

    #[test]
    fn test_parse_autonumber_with_all_params() {
        let source = r#"@startuml
autonumber 10 5 "[000]"
Alice -> Bob: message
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[0] {
            SequenceElement::Autonumber(AutonumberCommand::Start(params)) => {
                assert_eq!(params.start, Some(10));
                assert_eq!(params.step, Some(5));
                assert_eq!(params.format, Some("[000]".to_string()));
            }
            _ => panic!("Expected Autonumber with all params"),
        }
    }

    #[test]
    fn test_parse_autonumber_stop() {
        let source = r#"@startuml
autonumber
Alice -> Bob: first
autonumber stop
Alice -> Bob: second
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        // Ищем autonumber stop
        let mut found_stop = false;
        for element in &diagram.elements {
            if let SequenceElement::Autonumber(AutonumberCommand::Stop) = element {
                found_stop = true;
                break;
            }
        }
        assert!(found_stop, "Expected autonumber stop command");
    }

    #[test]
    fn test_parse_autonumber_resume() {
        let source = r#"@startuml
autonumber
Alice -> Bob: first
autonumber stop
Alice -> Bob: second
autonumber resume
Alice -> Bob: third
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        // Ищем autonumber resume
        let mut found_resume = false;
        for element in &diagram.elements {
            if let SequenceElement::Autonumber(AutonumberCommand::Resume(_)) = element {
                found_resume = true;
                break;
            }
        }
        assert!(found_resume, "Expected autonumber resume command");
    }

    // ============== Тесты для return ==============

    #[test]
    fn test_parse_return_with_label() {
        let source = r#"@startuml
Alice -> Bob++: request
return response
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[1] {
            SequenceElement::Return(ret) => {
                assert_eq!(ret.label, Some("response".to_string()));
            }
            _ => panic!("Expected Return element"),
        }
    }

    #[test]
    fn test_parse_return_without_label() {
        let source = r#"@startuml
Alice -> Bob++: request
return
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        match &diagram.elements[1] {
            SequenceElement::Return(ret) => {
                assert!(ret.label.is_none());
            }
            _ => panic!("Expected Return element"),
        }
    }

    #[test]
    fn test_parse_full_example_with_autonumber_and_return() {
        // Полный пример из задачи пользователя
        let source = r#"@startuml
autonumber "[00]"

participant "Потребитель API" as apiConsumer
participant "IGA.SSO" as iam
participant "Integration Platform" as ip

apiConsumer->iam++: Атентификация
autonumber stop
return Технический токен
autonumber resume

apiConsumer->ip++: Запрос через API
autonumber stop
return Ответ
autonumber resume
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        // Проверяем наличие всех типов элементов
        let mut has_autonumber_start = false;
        let mut has_autonumber_stop = false;
        let mut has_autonumber_resume = false;
        let mut has_return = false;
        let mut return_count = 0;

        for element in &diagram.elements {
            match element {
                SequenceElement::Autonumber(AutonumberCommand::Start(params)) => {
                    has_autonumber_start = true;
                    assert_eq!(params.format, Some("[00]".to_string()));
                }
                SequenceElement::Autonumber(AutonumberCommand::Stop) => {
                    has_autonumber_stop = true;
                }
                SequenceElement::Autonumber(AutonumberCommand::Resume(_)) => {
                    has_autonumber_resume = true;
                }
                SequenceElement::Return(_) => {
                    has_return = true;
                    return_count += 1;
                }
                _ => {}
            }
        }

        assert!(has_autonumber_start, "Expected autonumber start");
        assert!(has_autonumber_stop, "Expected autonumber stop");
        assert!(has_autonumber_resume, "Expected autonumber resume");
        assert!(has_return, "Expected return statements");
        assert_eq!(return_count, 2, "Expected 2 return statements");
    }

    // ============== Тесты для ref ==============

    #[test]
    fn test_parse_ref_over_single_line() {
        let source = r#"@startuml
Alice -> Bob: Request
ref over Bob, Carol: See Other Diagram
Bob -> Carol: Process
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        // Должен быть: Message, Reference, Message
        let mut found_ref = false;
        for element in &diagram.elements {
            if let SequenceElement::Reference(ref_elem) = element {
                found_ref = true;
                assert_eq!(ref_elem.text, "See Other Diagram");
                assert!(ref_elem.participants.contains(&"Bob".to_string()));
                assert!(ref_elem.participants.contains(&"Carol".to_string()));
            }
        }
        assert!(found_ref, "Expected Reference element");
    }

    #[test]
    fn test_parse_ref_over_one_participant() {
        let source = r#"@startuml
Alice -> Bob: Hello
ref over Bob: Internal processing
Bob --> Alice: Done
@enduml"#;

        let result = parse_sequence(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let diagram = result.unwrap();

        let mut found_ref = false;
        for element in &diagram.elements {
            if let SequenceElement::Reference(ref_elem) = element {
                found_ref = true;
                assert_eq!(ref_elem.participants.len(), 1);
                assert_eq!(ref_elem.participants[0], "Bob");
            }
        }
        assert!(found_ref, "Expected Reference element");
    }
}
