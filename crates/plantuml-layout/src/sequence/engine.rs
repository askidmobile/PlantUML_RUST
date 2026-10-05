//! Sequence Diagram Layout Engine
//!
//! Реализация алгоритма размещения элементов sequence diagram.

use plantuml_ast::common::{LineStyle, Note, NotePosition};

/// Ширина петли self-message в пикселях.
///
/// По эталону PlantUML. Используется и при построении геометрии, и при
/// проверке переполнения текста — раньше эти два места использовали разные
/// значения (42.0 и 40.0).
const SELF_MESSAGE_LOOP_WIDTH: f64 = 42.0;

/// Высота петли self-message в пикселях.
const SELF_MESSAGE_LOOP_HEIGHT: f64 = 13.0;

/// Отступ от петли self-message до текста сообщения.
const SELF_MESSAGE_TEXT_GAP: f64 = 5.0;

/// Кегль текста примечания (эталон `sequence_notes`: `font-size="13"`).
const NOTE_FONT_SIZE: f64 = 13.0;

/// Отступ текста примечания от левого края рамки.
///
/// Измерено по эталону `sequence_notes`: рамка на x = 110.987,
/// текст на x = 116.987.
const NOTE_TEXT_INSET: f64 = 6.0;

/// Размер среза загнутого угла примечания.
///
/// Измерено по эталону `sequence_notes`: правый край на 204.987,
/// срез начинается на 194.987, то есть 10.
const NOTE_FOLD: f64 = 10.0;

/// Отступ примечания от стрелки сообщения, после которого оно стоит.
///
/// Измерено по эталону `sequence_notes`: стрелка 70.43, примечание 83.43.
const NOTE_TOP_GAP: f64 = 13.0;

/// Зазор от низа примечания до следующего сообщения.
///
/// Измерено по эталону: низ примечания 108.43, следующая стрелка 134.695.
const NOTE_BOTTOM_GAP: f64 = 26.265;

const NOTE_BACKGROUND: &str = "#FEFFDD";

/// Отступ заметки от линии жизни (измерено по эталону sequence_notes).
const NOTE_ANCHOR_GAP: f64 = 4.5;

/// Отступ от активации до нижнего блока участника.
///
/// В эталоне PlantUML активация заканчивается на 10px выше footer
/// (активация до 200.633 при footer на 210.633).
const ACTIVATION_FOOTER_GAP: f64 = 10.0;

/// Отступ от последнего сообщения до нижнего блока участника (footer).
///
/// Измерено по эталону PlantUML: последнее сообщение на y=157.828,
/// footer на y=175.828, разница ровно 18px.
const FOOTER_GAP: f64 = 18.0;

/// Сдвиг верхнего ряда участников вниз при наличии актёра.
///
/// Измерено по эталону PlantUML: в sequence_participants (с actor)
/// прямоугольники стоят на y=55, в sequence_simple (без actor) — на y=10.
/// PlantUML освобождает место для стик-фигуры над прямоугольником.
const ACTOR_HEAD_OFFSET: f64 = 45.0;

/// Насколько фигура участника выступает НАД полосой подписи.
///
/// В PlantUML `participant` рисуется прямоугольником, а остальные типы —
/// фигурами, которые стоят выше подписи. Числа сняты с эталона
/// `sequence_participants`: полоса подписи занимает y 55..85.297, а фигуры
/// поднимаются до указанных значений.
fn figure_extra_above(participant_type: ParticipantType) -> f64 {
    match participant_type {
        // Голова стик-фигуры на y=10.5
        ParticipantType::Actor => 44.5,
        // Кружок со скобкой и кружок с подчёркиванием: верх на y=42
        ParticipantType::Boundary | ParticipantType::Entity => 13.0,
        // Кружок со стрелкой: стрелка выше кружка, верх на y=37
        ParticipantType::Control => 18.0,
        // Цилиндр: верх на y=24
        ParticipantType::Database => 31.0,
        // Прямоугольник занимает полосу целиком
        _ => 0.0,
    }
}

/// Насколько фигура участника выступает ПОД нижней полосой.
///
/// Нижние блоки PlantUML рисует зеркально: подпись сверху, фигура снизу.
/// Числа сняты с эталона `sequence_participants` (полоса 249.961..280.258).
fn figure_extra_below(participant_type: ParticipantType) -> f64 {
    match participant_type {
        ParticipantType::Actor => 44.5,
        ParticipantType::Boundary | ParticipantType::Control => 14.0,
        ParticipantType::Entity => 16.0,
        ParticipantType::Database => 32.0,
        _ => 0.0,
    }
}

/// Цвет подложки легенды в PlantUML.
const SEQUENCE_LEGEND_BACKGROUND: &str = "#DDDDDD";

/// Внутренний отступ легенды.
const SEQUENCE_LEGEND_PADDING: f64 = 10.0;

/// Отступ рамки `mainframe` от содержимого.
const SEQUENCE_FRAME_PADDING: f64 = 10.0;

/// Зазор между диаграммой и легендой.
const SEQUENCE_LEGEND_GAP: f64 = 20.0;
use plantuml_ast::sequence::{
    Activation, ActivationType, AutonumberCommand, Delay, Divider, Fragment, FragmentType, Message,
    ParticipantType, Reference, Return, SequenceDiagram, SequenceElement,
};
use plantuml_model::{Point, Rect};

use super::config::SequenceLayoutConfig;
use super::metrics::{DiagramMetrics, ParticipantMetrics};
use crate::{EdgeType, ElementType, FragmentSection, LayoutConfig, LayoutElement, LayoutResult};

/// Насколько рамка фрагмента поднимается над текущей позицией потока.
const FRAGMENT_TOP_OFFSET: f64 = 17.0;

/// Насколько низ фрагмента поднимается над текущей позицией потока.
const FRAGMENT_BOTTOM_OFFSET: f64 = 21.14;

/// Продвижение потока после низа фрагмента.
///
/// Эталон `sequence_fragments`: последнее сообщение на 178.63, низ рамки
/// на 186.63, нижний блок участника — на 210.63. Формула
/// `footer_y = current_y + FOOTER_GAP - message_spacing` даёт отсюда
/// 35.13. Прежние 15 подбирались, когда рамка была на 19 выше нужного.
const FRAGMENT_FOOTER_ADVANCE: f64 = 34.13;

/// Отступ от верха фрагмента до первого сообщения.
///
/// Эталон `sequence_fragments`: 48.26 от верха рамки минус 17, на которые
/// рамка поднята над потоком.
const FRAGMENT_HEADER_GAP: f64 = 31.26;

/// Кегль условий фрагмента: по нему считается ширина рамки.
///
/// Эталон `sequence_fragments` рисует условия кеглем 11.
const FRAGMENT_CONDITION_FONT_SIZE: f64 = 11.0;

/// Добавка к ширине условия при расчёте рамки фрагмента.
///
/// Проверено на сервере: 66.96 + 95.44 = 162.40 и 194.89 + 95.44 = 290.33.
const FRAGMENT_CONDITION_EXTRA: f64 = 95.44;

/// Насколько рамка фрагмента уже левого края участника.
///
/// Эталон `sequence_fragments`: рамка начинается на 16.955 при левом крае
/// блока участника 10, то есть на 6.955 правее (прежние −10 давали 0).
const FRAGMENT_FRAME_LEFT_INSET: f64 = 16.955;

/// Layout engine для sequence diagrams
pub struct SequenceLayoutEngine {
    config: SequenceLayoutConfig,
}

impl SequenceLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: SequenceLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: SequenceLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &SequenceDiagram) -> LayoutResult {
        let mut metrics = DiagramMetrics::new();
        let mut elements = Vec::new();

        // 1. Размещаем участников
        self.layout_participants(diagram, &mut metrics, &mut elements);

        // 1.5. Добавляем box группировки (фоновые прямоугольники)
        // Должны быть добавлены в начало, чтобы рендерились под участниками
        let box_elements = self.layout_boxes(diagram, &metrics);

        // 2. Начальная позиция Y после блоков участников
        // Используем Y позицию из header_bounds первого участника + высота участника + отступ
        let first_participant_y = metrics
            .participants
            .values()
            .next()
            .map(|p| p.header_bounds.y)
            .unwrap_or(self.config.margin);
        metrics.current_y = first_participant_y + self.config.participant_height + 30.0;

        // 3. Обрабатываем элементы диаграммы
        for element in &diagram.elements {
            self.layout_element(element, &mut metrics, &mut elements);
        }

        // 4. Завершаем все незакрытые активации.
        //
        // Конец активации — на 10px выше нижнего блока участника, как в
        // эталоне PlantUML (активация до 200.633 при footer на 210.633).
        // Раньше передавался metrics.current_y, который уже сдвинут на
        // FOOTER_GAP, поэтому активация вылезала за footer на ~11px.
        let activation_end_y = self.activation_footer_y(&metrics);
        metrics.finalize_activations(activation_end_y);

        // 5. Добавляем lifelines
        self.add_lifelines(&metrics, &mut elements);

        // 6. Добавляем прямоугольники активаций
        self.add_activations(&metrics, &mut elements);

        // 7. Добавляем нижние блоки участников (footers) - как в PlantUML.
        //    Директива `hide footbox` отключает их отрисовку.
        if !diagram.hide_footbox {
            self.add_participant_footers(&metrics, &mut elements);
        }

        // 8. Вычисляем финальную высоту диаграммы (footer_y + footer_height + margin)
        let footer_y = metrics.current_y + FOOTER_GAP - self.config.message_spacing;
        let total_height = footer_y + self.config.participant_height + self.config.margin;

        // 9. Обновляем высоту box элементов
        let box_elements: Vec<LayoutElement> = box_elements
            .into_iter()
            .map(|mut el| {
                // Box должен занимать всю высоту от верха до низа диаграммы
                el.bounds.height = total_height - el.bounds.y - 5.0;
                el
            })
            .collect();

        // 9.5. Заголовок, подписи и легенда — как в PlantUML.
        //      Раньше `title` разбирался парсером, но в вывод не попадал:
        //      метаданные не читались layout-движком.
        let mut header_elements = Vec::new();

        // `header` рисуется над диаграммой
        if let Some(header) = &diagram.metadata.header {
            header_elements.push(LayoutElement {
                id: "header".to_string(),
                bounds: Rect::new(self.config.margin, 0.0, 0.0, self.config.line_height),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Text {
                    text: header.clone(),
                    font_size: self.config.font_size,
                },
            });
        }

        // `title` — по центру над диаграммой
        if let Some(title) = &diagram.metadata.title {
            header_elements.push(LayoutElement {
                id: "title".to_string(),
                bounds: Rect::new(
                    self.config.margin,
                    if diagram.metadata.header.is_some() {
                        self.config.line_height
                    } else {
                        0.0
                    },
                    0.0,
                    self.config.line_height,
                ),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Text {
                    text: title.clone(),
                    font_size: self.config.font_size,
                },
            });
        }

        // `footer` и `caption` — под диаграммой
        let mut trailing_elements = Vec::new();
        if let Some(caption) = &diagram.metadata.caption {
            trailing_elements.push(LayoutElement {
                id: "caption".to_string(),
                bounds: Rect::new(
                    self.config.margin,
                    total_height,
                    0.0,
                    self.config.line_height,
                ),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Text {
                    text: caption.clone(),
                    font_size: self.config.font_size,
                },
            });
        }
        if let Some(footer) = &diagram.metadata.footer {
            trailing_elements.push(LayoutElement {
                id: "footer_text".to_string(),
                bounds: Rect::new(
                    self.config.margin,
                    total_height
                        + if diagram.metadata.caption.is_some() {
                            self.config.line_height
                        } else {
                            0.0
                        },
                    0.0,
                    self.config.line_height,
                ),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Text {
                    text: footer.clone(),
                    font_size: self.config.font_size,
                },
            });
        }

        // `mainframe Заголовок` — рамка вокруг всей диаграммы с подписью
        // в левом верхнем углу. Раньше поле не читалось вовсе.
        if let Some(frame) = &diagram.metadata.mainframe {
            trailing_elements.push(LayoutElement {
                id: "mainframe".to_string(),
                bounds: Rect::new(
                    self.config.margin,
                    -self.config.line_height - SEQUENCE_FRAME_PADDING,
                    metrics.max_x + SEQUENCE_FRAME_PADDING,
                    total_height + self.config.line_height + SEQUENCE_FRAME_PADDING * 2.0,
                ),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::System {
                    title: frame.clone(),
                },
            });
        }

        // Легенда — справа от диаграммы, на светло-жёлтой подложке.
        // Раньше поле `legend` не читалось layout-движком: парсер её
        // разбирал, а в вывод она не попадала.
        if let Some(legend) = &diagram.metadata.legend {
            let width = self.config.text.width(legend, self.config.font_size)
                + SEQUENCE_LEGEND_PADDING * 2.0;
            let height = legend.lines().count() as f64 * self.config.line_height
                + SEQUENCE_LEGEND_PADDING * 2.0;

            let mut properties = std::collections::HashMap::new();
            properties.insert("fill".to_string(), SEQUENCE_LEGEND_BACKGROUND.to_string());

            trailing_elements.push(LayoutElement {
                id: "legend_bg".to_string(),
                bounds: Rect::new(
                    self.config.margin + metrics.max_x + SEQUENCE_LEGEND_GAP,
                    self.config.margin,
                    width,
                    height,
                ),
                text: None,
                properties,
                element_type: ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 0.0,
                },
            });
            trailing_elements.push(LayoutElement {
                id: "legend".to_string(),
                bounds: Rect::new(
                    self.config.margin
                        + metrics.max_x
                        + SEQUENCE_LEGEND_GAP
                        + SEQUENCE_LEGEND_PADDING,
                    self.config.margin + SEQUENCE_LEGEND_PADDING,
                    width,
                    height,
                ),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Text {
                    text: legend.clone(),
                    font_size: self.config.font_size,
                },
            });
        }

        // 10. Вставляем box элементы в начало (чтобы рендерились под остальным),
        //     а заголовки — в конец
        let mut final_elements = box_elements;
        final_elements.extend(elements);
        final_elements.extend(header_elements);
        final_elements.extend(trailing_elements);

        // 11. Вычисляем bounds
        let mut result = LayoutResult {
            elements: final_elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        // 12. Расширяем bounds для текста сообщений (PlantUML Вариант B)
        // Текст может выходить за границы участников, viewBox расширяется
        self.adjust_bounds_for_message_text(diagram, &metrics, &mut result);

        result
    }

    /// Расширяет bounds диаграммы для учёта текста сообщений, выходящего за участников
    /// PlantUML Вариант B: фиксированный spacing, но viewBox расширяется под текст
    fn adjust_bounds_for_message_text(
        &self,
        diagram: &SequenceDiagram,
        metrics: &DiagramMetrics,
        result: &mut LayoutResult,
    ) {
        let mut max_right = result.bounds.x + result.bounds.width;
        let mut max_left_overflow = 0.0_f64;

        // Проходим по всем элементам и вычисляем максимальный выход текста за границы
        for element in &diagram.elements {
            self.check_element_text_overflow(
                element,
                metrics,
                &mut max_right,
                &mut max_left_overflow,
            );
        }

        // Расширяем bounds если текст выходит за правую границу
        // Добавляем небольшой отступ (5px) для читабельности, но не полный margin
        let current_right = result.bounds.x + result.bounds.width;
        if max_right > current_right {
            result.bounds.width = max_right - result.bounds.x + 5.0;
        }

        // Расширяем bounds если текст выходит за левую границу
        if max_left_overflow > 0.0 {
            result.bounds.x -= max_left_overflow + 5.0;
            result.bounds.width += max_left_overflow + 5.0;
        }
    }

    /// Рекурсивно проверяет overflow текста сообщений
    fn check_element_text_overflow(
        &self,
        element: &SequenceElement,
        metrics: &DiagramMetrics,
        max_right: &mut f64,
        max_left_overflow: &mut f64,
    ) {
        match element {
            SequenceElement::Message(msg) => {
                let is_self_message = msg.from == msg.to;

                if is_self_message {
                    // Self-message: текст справа от петли.
                    // Берём те же константы, что и при построении геометрии:
                    // раньше здесь стояло 40.0 против 42.0 в отрисовке, из-за
                    // чего проверка переполнения и фактическая петля
                    // расходились на 2px.
                    if let Some(pm) = metrics.participants.get(&msg.from) {
                        let text_width = self.config.message_label_width(&msg.label);
                        let right_edge = pm.center_x
                            + SELF_MESSAGE_LOOP_WIDTH
                            + SELF_MESSAGE_TEXT_GAP
                            + text_width;
                        *max_right = max_right.max(right_edge);
                    }
                } else {
                    // Обычное сообщение: текст слева от стрелки (над ней)
                    // Проверяем, не выходит ли текст за левого участника
                    let from_x = metrics.lifeline_x(&msg.from, &self.config);
                    let to_x = metrics.lifeline_x(&msg.to, &self.config);
                    let left_x = from_x.min(to_x);
                    let text_start = left_x + 5.0; // отступ от lifeline
                    let text_width = self.config.message_label_width(&msg.label);
                    let text_end = text_start + text_width;

                    // Проверяем overflow вправо
                    *max_right = max_right.max(text_end);

                    // Проверяем overflow влево (если сообщение идёт справа налево)
                    let min_x = metrics
                        .participants
                        .values()
                        .map(|p| p.center_x - p.width / 2.0)
                        .fold(f64::MAX, f64::min);
                    if text_start < min_x {
                        *max_left_overflow = max_left_overflow.max(min_x - text_start);
                    }
                }
            }
            SequenceElement::Fragment(frag) => {
                for section in &frag.sections {
                    for elem in &section.elements {
                        self.check_element_text_overflow(
                            elem,
                            metrics,
                            max_right,
                            max_left_overflow,
                        );
                    }
                }
            }
            SequenceElement::Return(ret) => {
                // Return тоже может иметь label
                if let Some(label) = &ret.label {
                    let text_width = self.config.message_label_width(label);
                    // Return обычно идёт справа налево, текст над стрелкой
                    // Просто добавляем к max_right для безопасности
                    let current_max_x = metrics
                        .participants
                        .values()
                        .map(|p| p.center_x + p.width / 2.0)
                        .fold(f64::MIN, f64::max);
                    *max_right = max_right.max(current_max_x + text_width);
                }
            }
            _ => {}
        }
    }

    /// Создаёт layout для box группировок
    fn layout_boxes(
        &self,
        diagram: &SequenceDiagram,
        metrics: &DiagramMetrics,
    ) -> Vec<LayoutElement> {
        let mut elements = Vec::new();

        for (i, pbox) in diagram.boxes.iter().enumerate() {
            if pbox.participants.is_empty() {
                continue;
            }

            // Находим границы box по участникам
            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;

            for participant_id in &pbox.participants {
                if let Some(pm) = metrics.participants.get(participant_id) {
                    let left = pm.center_x - pm.width / 2.0;
                    let right = pm.center_x + pm.width / 2.0;
                    min_x = min_x.min(left);
                    max_x = max_x.max(right);
                }
            }

            if min_x == f64::MAX {
                continue;
            }

            // Добавляем отступы
            let padding = 10.0;
            min_x -= padding;
            max_x += padding;

            // Box начинается выше участников и заканчивается на footer_y + footer_height
            let box_y = 5.0; // Немного выше margin
                             // Высота будет определена позже при рендеринге (на всю высоту диаграммы)

            let mut properties = std::collections::HashMap::new();
            if let Some(title) = &pbox.title {
                properties.insert("title".to_string(), title.clone());
            }
            if let Some(color) = &pbox.color {
                properties.insert("color".to_string(), color.to_css());
            }

            let box_element = LayoutElement {
                id: format!("box_{}", i),
                bounds: Rect::new(min_x, box_y, max_x - min_x, 100.0), // Высота будет корректироваться
                element_type: ElementType::ParticipantBox,
                text: pbox.title.clone(),
                properties,
            };

            elements.push(box_element);
        }

        elements
    }

    /// Размещает участников с учётом длины текста сообщений (PlantUML стиль)
    fn layout_participants(
        &self,
        diagram: &SequenceDiagram,
        metrics: &mut DiagramMetrics,
        elements: &mut Vec<LayoutElement>,
    ) {
        // Собираем список участников в порядке появления
        let mut participant_order: Vec<String> = Vec::new();
        let mut participant_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut participant_types: std::collections::HashMap<String, ParticipantType> =
            std::collections::HashMap::new();

        for participant in &diagram.participants {
            let name = participant
                .id
                .alias
                .as_ref()
                .unwrap_or(&participant.id.name)
                .clone();
            let display_name = participant.id.name.clone();
            if !participant_order.contains(&name) {
                participant_order.push(name.clone());
                participant_names.insert(name.clone(), display_name);
                participant_types.insert(name, participant.participant_type);
            }
        }

        // Также собираем участников из сообщений
        self.collect_participants_order(diagram, &mut participant_order);

        // Если есть боксы с заголовками, сдвигаем участников вниз
        let has_box_titles = diagram.boxes.iter().any(|b| b.title.is_some());

        // При наличии актёра PlantUML сдвигает весь верхний ряд участников
        // вниз, освобождая место для стик-фигуры над прямоугольником:
        // в эталоне sequence_participants (с actor) прямоугольники стоят на
        // y=55, а в sequence_simple (без actor) — на y=10. Разница 45px.
        let has_actor = participant_types
            .values()
            .any(|t| matches!(t, ParticipantType::Actor));

        let participant_y = self.config.margin
            + if has_box_titles {
                self.config.box_title_height
            } else {
                0.0
            }
            + if has_actor { ACTOR_HEAD_OFFSET } else { 0.0 };

        // Сначала вычисляем ширины всех участников
        let participant_widths: Vec<f64> = participant_order
            .iter()
            .map(|name| {
                let display_name = participant_names.get(name).unwrap_or(name);
                self.config.participant_width_for_name(display_name)
            })
            .collect();

        // Вычисляем максимальную ширину сообщений между соседними участниками
        // Теперь с учётом реальных ширин участников
        let spacing_map =
            self.calculate_participant_spacing(diagram, &participant_order, &participant_widths);

        // Размещаем участников с вычисленными расстояниями
        let mut x = self.config.margin;

        for (i, name) in participant_order.iter().enumerate() {
            let display_name = participant_names.get(name).unwrap_or(name);
            let ptype = participant_types
                .get(name)
                .copied()
                .unwrap_or(ParticipantType::Participant);

            let width = participant_widths[i];
            let center_x = x + width / 2.0;

            // Y-координата уже посчитана выше: она учитывает и заголовки
            // боксов, и сдвиг под стик-фигуру актёра. Раньше в ветке else
            // брался config.margin, из-за чего сдвиг при наличии актёра
            // терялся и весь верхний ряд оставался на месте.
            let y = participant_y;

            let bounds = Rect::new(x, y, width, self.config.participant_height);

            metrics.participants.insert(
                name.clone(),
                ParticipantMetrics {
                    id: name.clone(),
                    display_name: display_name.to_string(),
                    center_x,
                    width,
                    header_bounds: bounds,
                    participant_type: ptype,
                },
            );

            // Создаём визуальный элемент
            let element = self.create_participant_element(name, display_name, &bounds, ptype);
            elements.push(element);

            // Расстояние до следующего участника
            if i < participant_order.len() - 1 {
                let next_name = &participant_order[i + 1];
                let key = format!("{}_{}", name, next_name);
                // Используем расстояние из spacing_map (уже рассчитано для всех пар)
                // Если пары нет — минимальный отступ 15px
                let spacing = spacing_map.get(&key).copied().unwrap_or(15.0);
                x += width + spacing;
            } else {
                x += width;
            }
        }

        metrics.max_x = x;
    }

    /// Собирает порядок участников из сообщений диаграммы
    fn collect_participants_order(&self, diagram: &SequenceDiagram, order: &mut Vec<String>) {
        for element in &diagram.elements {
            self.collect_participants_from_element_order(element, order);
        }
    }

    /// Рекурсивно собирает участников из элемента
    fn collect_participants_from_element_order(
        &self,
        element: &SequenceElement,
        order: &mut Vec<String>,
    ) {
        match element {
            SequenceElement::Message(msg) => {
                if !order.contains(&msg.from) {
                    order.push(msg.from.clone());
                }
                if !order.contains(&msg.to) {
                    order.push(msg.to.clone());
                }
            }
            SequenceElement::Fragment(frag) => {
                for section in &frag.sections {
                    for elem in &section.elements {
                        self.collect_participants_from_element_order(elem, order);
                    }
                }
            }
            _ => {}
        }
    }

    /// Вычисляет необходимое расстояние между соседними участниками на основе длины сообщений
    ///
    /// Алгоритм (см. docs/SEQUENCE_LAYOUT_ALGORITHM.md):
    /// 1. Собираем все сообщения и группируем по span (количеству сегментов)
    /// 2. Обрабатываем от коротких к длинным (span=1, затем span=2, ...)
    /// 3. Для соседних (span=1): устанавливаем spacing напрямую
    /// 4. Для несоседних (span>1): проверяем суммарную длину, увеличиваем только первый сегмент
    fn calculate_participant_spacing(
        &self,
        diagram: &SequenceDiagram,
        participant_order: &[String],
        participant_widths: &[f64],
    ) -> std::collections::HashMap<String, f64> {
        let n = participant_order.len();
        if n < 2 {
            return std::collections::HashMap::new();
        }

        // Инициализируем spacing минимальными значениями
        // Минимальный зазор между блоками участников.
        //
        // Было 50 — это значение ДОМИНИРОВАЛО над всеми парами и
        // раздувало ширину: PlantUML отводит под стрелку примерно
        // `текст + 20`, и при 50 зазор всегда упирался в минимум.
        // 30 оставляет место между блоками, но не искажает раскладку.
        // Минимальный ЗАЗОР между рамками соседних участников.
        //
        // Проверено на сервере: два длинных имени и сообщение из одной
        // буквы дают рамки 150.40 и 166.70 с зазором ровно 10.
        let min_gap = 10.0;
        // Начинаем с минимального зазора: требования сообщений только
        // УВЕЛИЧИВАЮТ его, поэтому старт с 30 не давал опуститься ниже 30
        // даже там, где эталон держит 27.
        let mut spacing: Vec<f64> = vec![min_gap; n - 1];

        // Отслеживаем пары с ПРЯМЫМИ сообщениями
        let mut direct_pairs: std::collections::HashSet<usize> = std::collections::HashSet::new();

        // Определяем есть ли autonumber
        let has_autonumber = self.diagram_has_autonumber(diagram);

        // Собираем все сообщения и группируем по span
        let mut messages_by_span: std::collections::BTreeMap<usize, Vec<(usize, usize, f64)>> =
            std::collections::BTreeMap::new();

        self.collect_messages_by_span(
            diagram,
            participant_order,
            has_autonumber,
            &mut messages_by_span,
        );

        // Обрабатываем от коротких к длинным
        for (span, messages) in messages_by_span {
            for (start_idx, end_idx, text_width) in messages {
                // Длина стрелки, необходимой под подпись.
                //
                // `text_width` приходит из `message_label_width`, которая
                // УЖЕ добавила 16px отступов. Эталон требует ещё 8:
                // «сообщение» 76.261 + 16 + 8 = 100.261 — ровно шаг
                // участников в sequence_participants. Раньше здесь стояло
                // 20, то есть на 12px больше, и все шаги были раздуты.
                let required_length = text_width + 8.0;

                if span == 1 {
                    // Соседние участники: устанавливаем spacing напрямую
                    // Стрелка идёт от центра до центра:
                    // arrow_length = spacing + (width_start + width_end) / 2
                    // required_length <= arrow_length
                    // spacing >= required_length - (width_start + width_end) / 2
                    let half_widths =
                        (participant_widths[start_idx] + participant_widths[end_idx]) / 2.0;
                    // Прямое сообщение задаёт РАССТОЯНИЕ МЕЖДУ ЦЕНТРАМИ:
                    // эталон `sequence_participants` держит шаг 100.261 при
                    // любой ширине рамок, поэтому зазор может оказаться
                    // меньше 30. Прежний порог 30 поднимал шаг до 103.17.
                    let required_spacing = (required_length - half_widths).max(min_gap);
                    spacing[start_idx] = spacing[start_idx].max(required_spacing);
                    direct_pairs.insert(start_idx);
                } else {
                    // Несоседние участники: проверяем суммарную длину
                    // arrow_length = sum(spacing[i]) + сумма ширин промежуточных участников + половины крайних
                    let current_spacing_sum: f64 = (start_idx..end_idx).map(|i| spacing[i]).sum();

                    // Ширины промежуточных участников (от start+1 до end-1) + половины крайних
                    let half_start = participant_widths[start_idx] / 2.0;
                    let half_end = participant_widths[end_idx] / 2.0;
                    let intermediate_widths: f64 = (start_idx + 1..end_idx)
                        .map(|i| participant_widths[i])
                        .sum();
                    let total_widths = half_start + intermediate_widths + half_end;

                    let current_arrow_length = current_spacing_sum + total_widths;

                    if current_arrow_length < required_length {
                        // Нужно увеличить. Увеличиваем ТОЛЬКО первый сегмент.
                        let deficit = required_length - current_arrow_length;
                        spacing[start_idx] += deficit;
                    }
                    // Отмечаем ВСЕ сегменты как используемые (чтобы не уменьшать)
                    for i in start_idx..end_idx {
                        direct_pairs.insert(i);
                    }
                }
            }
        }

        // Конвертируем в HashMap с ключами "participant1_participant2"
        let mut spacing_map: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for i in 0..n - 1 {
            let key = format!("{}_{}", participant_order[i], participant_order[i + 1]);

            // Пары без прямых сообщений уже ограничены сверху значением
            // по умолчанию, отдельная ветка не нужна.
            let final_spacing = spacing[i];
            spacing_map.insert(key, final_spacing);
        }

        spacing_map
    }

    /// Собирает все сообщения и группирует по span (количеству сегментов)
    fn collect_messages_by_span(
        &self,
        diagram: &SequenceDiagram,
        participant_order: &[String],
        has_autonumber: bool,
        messages_by_span: &mut std::collections::BTreeMap<usize, Vec<(usize, usize, f64)>>,
    ) {
        // Стек вызовов ведётся при обходе: он нужен, чтобы определить
        // участников `return` (callee -> caller)
        let mut call_stack: Vec<(String, String)> = Vec::new();
        for element in &diagram.elements {
            self.collect_message_span(
                element,
                participant_order,
                has_autonumber,
                messages_by_span,
                &mut call_stack,
            );
        }
    }

    /// Рекурсивно собирает сообщение и добавляет в группу по span
    #[allow(clippy::too_many_arguments)]
    fn collect_message_span(
        &self,
        element: &SequenceElement,
        participant_order: &[String],
        has_autonumber: bool,
        messages_by_span: &mut std::collections::BTreeMap<usize, Vec<(usize, usize, f64)>>,
        call_stack: &mut Vec<(String, String)>,
    ) {
        match element {
            SequenceElement::Message(msg) => {
                // Стек вызовов нужен, чтобы определить участников `return`
                if msg.activate {
                    call_stack.push((msg.from.clone(), msg.to.clone()));
                }
                if msg.deactivate {
                    call_stack.pop();
                }

                let text_width = self.config.message_label_width(&msg.label);
                let autonumber_width = if has_autonumber { 45.0 } else { 0.0 };
                let total_width = text_width + autonumber_width;

                let from_idx = participant_order.iter().position(|p| p == &msg.from);
                let to_idx = participant_order.iter().position(|p| p == &msg.to);

                if let (Some(from_idx), Some(to_idx)) = (from_idx, to_idx) {
                    let (start, end) = if from_idx < to_idx {
                        (from_idx, to_idx)
                    } else {
                        (to_idx, from_idx)
                    };
                    let span = end - start;
                    if span > 0 {
                        messages_by_span
                            .entry(span)
                            .or_default()
                            .push((start, end, total_width));
                    }
                }
            }
            SequenceElement::Return(ret) => {
                // Возврат — это сообщение от callee к caller, поэтому он
                // влияет на spacing так же, как обычное сообщение.
                // Участники берутся из стека вызовов. Раньше ширина текста
                // вычислялась и отбрасывалась (`let _text_width`), из-за чего
                // длинный текст возврата не влиял на раскладку.
                if let Some((caller, callee)) = call_stack.pop() {
                    let text_width = self
                        .config
                        .message_label_width(ret.label.as_deref().unwrap_or(""));

                    let from_idx = participant_order.iter().position(|p| p == &callee);
                    let to_idx = participant_order.iter().position(|p| p == &caller);

                    if let (Some(from_idx), Some(to_idx)) = (from_idx, to_idx) {
                        let (start, end) = if from_idx < to_idx {
                            (from_idx, to_idx)
                        } else {
                            (to_idx, from_idx)
                        };
                        if start != end {
                            messages_by_span
                                .entry(end - start)
                                .or_default()
                                .push((start, end, text_width));
                        }
                    }
                }
            }
            SequenceElement::Fragment(frag) => {
                for section in &frag.sections {
                    for elem in &section.elements {
                        self.collect_message_span(
                            elem,
                            participant_order,
                            has_autonumber,
                            messages_by_span,
                            call_stack,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Проверяет есть ли команда autonumber в диаграмме
    fn diagram_has_autonumber(&self, diagram: &SequenceDiagram) -> bool {
        for element in &diagram.elements {
            if let SequenceElement::Autonumber(
                AutonumberCommand::Start(_) | AutonumberCommand::Resume(_),
            ) = element
            {
                return true;
            }
        }
        false
    }

    /// Создаёт элемент участника
    fn create_participant_element(
        &self,
        id: &str,
        display_name: &str,
        bounds: &Rect,
        participant_type: ParticipantType,
    ) -> LayoutElement {
        // Фигура выступает над полосой подписи, поэтому элемент получает
        // расширенные границы: иначе рисунок вышел бы за пределы диаграммы
        // и не попал в её габариты.
        let extra = figure_extra_above(participant_type);
        let bounds = Rect::new(
            bounds.x,
            bounds.y - extra,
            bounds.width,
            bounds.height + extra,
        );

        match participant_type {
            ParticipantType::Actor => {
                // Актёр рисуется как стик-фигура (человечек), а не эллипс:
                // в PlantUML это отдельная форма, и рендерер её уже умеет
                // (ElementType::Actor). Ранее здесь стоял эллипс «для
                // упрощения», что визуально не соответствовало оригиналу.
                LayoutElement {
                    id: format!("participant_{}", id),
                    bounds,
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Actor {
                        label: display_name.to_string(),
                    },
                }
            }
            ParticipantType::Database => {
                // База данных в PlantUML — цилиндр, а не прямоугольник
                // со скруглёнными углами
                LayoutElement {
                    id: format!("participant_{}", id),
                    bounds,
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Database {
                        label: display_name.to_string(),
                    },
                }
            }
            ParticipantType::Boundary => LayoutElement {
                id: format!("participant_{}", id),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Boundary {
                    label: display_name.to_string(),
                },
            },
            ParticipantType::Control => LayoutElement {
                id: format!("participant_{}", id),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Control {
                    label: display_name.to_string(),
                },
            },
            ParticipantType::Entity => LayoutElement {
                id: format!("participant_{}", id),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Entity {
                    label: display_name.to_string(),
                },
            },
            _ => {
                // Остальные типы - обычный прямоугольник
                // PlantUML использует rx/ry = 2.5 для скругления углов
                LayoutElement {
                    id: format!("participant_{}", id),
                    bounds,
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Rectangle {
                        label: display_name.to_string(),
                        corner_radius: 2.5, // PlantUML style
                    },
                }
            }
        }
    }

    /// Обрабатывает один элемент диаграммы
    fn layout_element(
        &self,
        element: &SequenceElement,
        metrics: &mut DiagramMetrics,
        elements: &mut Vec<LayoutElement>,
    ) {
        match element {
            SequenceElement::Message(msg) => {
                self.layout_message(msg, metrics, elements);
            }
            SequenceElement::Fragment(frag) => {
                self.layout_fragment(frag, metrics, elements);
            }
            SequenceElement::Note(note) => {
                self.layout_note(note, metrics, elements);
            }
            SequenceElement::Activation(act) => {
                self.process_activation(act, metrics);
            }
            SequenceElement::Divider(div) => {
                self.layout_divider(div, metrics, elements);
            }
            SequenceElement::Delay(delay) => {
                self.layout_delay(delay, metrics, elements);
            }
            SequenceElement::Space(height) => {
                metrics.advance_y(*height as f64);
            }
            SequenceElement::Reference(reference) => {
                self.layout_reference(reference, metrics, elements);
            }
            SequenceElement::Autonumber(cmd) => {
                self.process_autonumber(cmd, metrics);
            }
            SequenceElement::Return(ret) => {
                self.layout_return(ret, metrics, elements);
            }
        }
    }

    /// Обрабатывает команду autonumber
    fn process_autonumber(&self, cmd: &AutonumberCommand, metrics: &mut DiagramMetrics) {
        match cmd {
            AutonumberCommand::Start(params) => {
                metrics.autonumber.enabled = true;
                if let Some(levels) = &params.levels {
                    // Многоуровневая нумерация: autonumber 1.1.1
                    metrics.autonumber.levels = levels.clone();
                } else if let Some(start) = params.start {
                    metrics.autonumber.set_start(start);
                } else {
                    // Если не указано, начинаем с 1
                    metrics.autonumber.levels = vec![1];
                }
                if let Some(step) = params.step {
                    metrics.autonumber.step = step;
                } else {
                    metrics.autonumber.step = 1;
                }
                metrics.autonumber.format = params.format.clone();
            }
            AutonumberCommand::Stop => {
                metrics.autonumber.enabled = false;
            }
            AutonumberCommand::Resume(params) => {
                metrics.autonumber.enabled = true;
                // При resume можно указать новые параметры
                if let Some(p) = params {
                    if let Some(levels) = &p.levels {
                        metrics.autonumber.levels = levels.clone();
                    } else if let Some(start) = p.start {
                        metrics.autonumber.set_start(start);
                    }
                    if let Some(step) = p.step {
                        metrics.autonumber.step = step;
                    }
                    if p.format.is_some() {
                        metrics.autonumber.format = p.format.clone();
                    }
                }
            }
            AutonumberCommand::Inc(level) => {
                // Поддержка многоуровневой нумерации (1.1, 1.2, etc.)
                // level = "A" -> инкремент первого уровня
                // level = "B" -> инкремент второго уровня
                if let Some(ch) = level.chars().next() {
                    metrics.autonumber.increment_level(ch);
                }
            }
        }
    }

    /// Обрабатывает return statement
    fn layout_return(
        &self,
        ret: &Return,
        metrics: &mut DiagramMetrics,
        elements: &mut Vec<LayoutElement>,
    ) {
        // Извлекаем последний вызов из стека
        if let Some((caller, callee)) = metrics.call_stack.pop() {
            // Создаём return message: callee --> caller
            let mut msg = Message::new(&callee, &caller, ret.label.clone().unwrap_or_default());
            msg.line_style = LineStyle::Dashed;
            msg.deactivate = true; // Return деактивирует callee

            // Layout return message (без autonumber)
            let y = metrics.current_y;
            metrics.last_message_y = y;

            // Деактивируем callee
            metrics.deactivate(&callee);

            let from_x = metrics.lifeline_x(&callee, &self.config);
            let to_x = metrics.lifeline_x(&caller, &self.config);

            let points = vec![Point::new(from_x, y), Point::new(to_x, y)];

            let bounds = Rect::new(
                from_x.min(to_x),
                y - 1.0,
                (to_x - from_x).abs().max(1.0),
                2.0,
            );

            let edge = LayoutElement {
                id: format!("return_{}_{}", callee, caller),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Edge {
                    points,
                    label: ret.label.clone(),
                    arrow_start: false,
                    arrow_end: true,
                    dashed: true,
                    edge_type: EdgeType::Association,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            };

            elements.push(edge);
            metrics.advance_y(self.config.message_spacing);
        }
    }

    /// Размещает сообщение
    fn layout_message(
        &self,
        msg: &Message,
        metrics: &mut DiagramMetrics,
        elements: &mut Vec<LayoutElement>,
    ) {
        // Сначала вычисляем количество строк текста
        let line_count = msg.label.matches("\\n").count() + msg.label.matches('\n').count();

        // Для многострочного текста нужно добавить место ПЕРЕД стрелкой
        // (текст идёт вверх от стрелки)
        if line_count > 0 {
            metrics.advance_y(line_count as f64 * self.config.line_height);
        }

        let y = metrics.current_y;

        // Сохраняем Y позицию этого сообщения для последующих активаций
        metrics.last_message_y = y;

        // Получаем X координаты ДО активации (чтобы стрелка шла к центру lifeline)
        let from_x = metrics.lifeline_x(&msg.from, &self.config);
        let to_x = metrics.lifeline_x(&msg.to, &self.config);

        // Обрабатываем активацию на сообщении
        // Важно: активация начинается с Y позиции ЭТОГО сообщения
        if msg.activate {
            metrics.activate_at(&msg.to, y);
            // Добавляем в call_stack для return
            metrics.call_stack.push((msg.from.clone(), msg.to.clone()));
        }
        if msg.deactivate {
            metrics.deactivate(&msg.from);
        }

        // Получаем номер autonumber (если включен)
        let autonumber = if metrics.autonumber.enabled {
            Some(metrics.autonumber.next())
        } else {
            None
        };

        // Label сообщения (без номера - он будет отдельным элементом)
        let label = msg.label.clone();

        // Создаём линию сообщения
        let is_self_message = msg.from == msg.to;

        // Вычисляем ширину текста для корректного позиционирования
        let label_width = if label.is_empty() {
            0.0
        } else {
            self.config.message_label_width(&label)
        };

        let points = if is_self_message {
            // Self-message в стиле PlantUML: горизонтальная линия, вертикальная
            // вниз и обратно. Размеры петли вынесены в константы модуля, чтобы
            // проверка переполнения и отрисовка не расходились.
            let loop_width = SELF_MESSAGE_LOOP_WIDTH;
            let loop_height = SELF_MESSAGE_LOOP_HEIGHT;
            vec![
                Point::new(from_x, y),
                Point::new(from_x + loop_width, y),
                Point::new(from_x + loop_width, y + loop_height),
                Point::new(from_x, y + loop_height),
            ]
        } else {
            vec![Point::new(from_x, y), Point::new(to_x, y)]
        };

        // Определяем стиль линии (используется при рендеринге)
        let is_dashed = msg.line_style == LineStyle::Dashed;

        // Вычисляем bounds с учётом текста
        let bounds = if is_self_message {
            // Bounds включает текст над петлёй и саму петлю
            let loop_width = SELF_MESSAGE_LOOP_WIDTH;
            let loop_height = SELF_MESSAGE_LOOP_HEIGHT;
            // Текст над петлёй - нужна ширина текста или петли (что больше)
            let total_width = loop_width.max(label_width);
            Rect::new(
                from_x,
                y - 5.0, // текст над петлёй (PlantUML: ~5px над верхней линией)
                total_width,
                loop_height + 10.0, // текст + высота петли + отступ
            )
        } else {
            Rect::new(
                from_x.min(to_x),
                y - 1.0,
                (to_x - from_x).abs().max(1.0),
                2.0,
            )
        };

        // Properties для хранения autonumber (если есть)
        let mut properties = std::collections::HashMap::new();
        if let Some(ref num) = autonumber {
            properties.insert("autonumber".to_string(), num.clone());
        }

        let edge = LayoutElement {
            id: format!("msg_{}_{}", msg.from, msg.to),
            bounds,
            text: None,
            properties,
            element_type: ElementType::Edge {
                points,
                label: if label.is_empty() && autonumber.is_none() {
                    None
                } else if label.is_empty() {
                    // Только номер без текста — не добавляем label (номер в properties)
                    None
                } else {
                    Some(label)
                },
                arrow_start: false,
                arrow_end: true,
                dashed: is_dashed, // пунктирная линия для --> (response)
                edge_type: EdgeType::Association, // стандартные стрелки sequence diagram
                from_cardinality: None,
                to_cardinality: None,
            },
        };

        elements.push(edge);

        // Продвигаем Y на базовое расстояние между сообщениями
        // (место для многострочного текста уже добавлено ПЕРЕД стрелкой)
        let height = if is_self_message {
            // PlantUML self-message: шаг между self-messages ~30px (петля 13px + отступ)
            30.0
        } else {
            self.config.message_spacing
        };
        metrics.advance_y(height);
    }

    /// Размещает фрагмент (alt, opt, loop, etc.)
    fn layout_fragment(
        &self,
        frag: &Fragment,
        metrics: &mut DiagramMetrics,
        elements: &mut Vec<LayoutElement>,
    ) {
        // Рамка фрагмента начинается ВЫШЕ точки, до которой дошёл поток.
        //
        // Измерено на трёх случаях: наш фрагмент оказывался ровно на
        // 17px ниже эталонного — и когда перед ним нет сообщения, и
        // когда есть. Значение одинаково, поэтому это постоянный отступ,
        // а не следствие высоты строки.
        let start_y = metrics.current_y - FRAGMENT_TOP_OFFSET;

        // Отступ от верха фрагмента до первого сообщения.
        //
        // Эталон `sequence_fragments`: рамка начинается на 82.43, первая
        // стрелка — на 130.69, то есть 48.26 от верха. При
        // `start_y = current_y - 17` это даёт ровно 31.26. Прежняя формула
        // `fragment_header_height + 23` = 45 растягивала блок на 13.7.
        metrics.advance_y(FRAGMENT_HEADER_GAP);

        // Обрабатываем секции
        let mut layout_sections: Vec<FragmentSection> = Vec::new();

        for (i, section) in frag.sections.iter().enumerate() {
            if i > 0 {
                // Разделитель между секциями (else):
                // 1. Текста условия else [текст] над пунктирной линией ~18px
                // 2. Разделительной линии ~5px
                // 3. Отступа от линии до первого сообщения следующей секции ~20px (увеличено!)
                // Общий отступ: 18 + 5 + 20 = 43px
                metrics.advance_y(19.0);
            }

            let section_start_y = metrics.current_y;

            let mut section_elements: Vec<LayoutElement> = Vec::new();
            for elem in &section.elements {
                self.layout_element(elem, metrics, &mut section_elements);
            }

            let section_end_y = metrics.current_y;

            layout_sections.push(FragmentSection {
                condition: section.condition.clone(),
                start_y: section_start_y,
                end_y: section_end_y,
                children: section_elements,
            });
        }

        // Низ фрагмента тоже поднимается над текущей позицией потока.
        //
        // Измерено: эталон заканчивает фрагмент на 16.06px ВЫШЕ, чем
        // дошёл поток после последнего сообщения. Прежняя формула
        // (`current_y + padding - 5`) давала низ на 21px ниже нужного.
        let end_y = metrics.current_y - FRAGMENT_BOTTOM_OFFSET;
        metrics.current_y = end_y;

        // ВАЖНО: Отступ ПОСЛЕ фрагмента до следующего элемента (между фрагментами или до footer)
        // PlantUML имеет заметный отступ между фрагментами
        metrics.advance_y(FRAGMENT_FOOTER_ADVANCE);

        // Находим границы фрагмента
        let (min_x, max_x) = self.find_fragment_x_bounds(frag, metrics);

        // Ширина рамки фрагмента задаётся УСЛОВИЕМ, а не участниками.
        //
        // Проверено на сервере двумя замерами: условие «[Успешно]» (66.96)
        // даёт рамку 162.40, «[ОченьДлинноеУсловиеВетки]» (194.89) — 290.33,
        // то есть ширина = условие + 95.44 в обоих случаях. Прежний расчёт
        // по рамкам участников давал 0…179.87 и вылезал за левый край
        // диаграммы.
        let condition_width = frag
            .sections
            .iter()
            .filter_map(|section| section.condition.as_deref())
            .map(|condition| {
                // Условие рисуется ПОЛУЖИРНЫМ, поэтому и ширина берётся
                // по жирным метрикам.
                self.config
                    .text
                    .width_bold(&format!("[{condition}]"), FRAGMENT_CONDITION_FONT_SIZE)
            })
            .fold(0.0_f64, f64::max);
        let condition_width = condition_width.max(self.config.text.width_bold(
            &format!("[{}]", frag.condition.clone().unwrap_or_default()),
            FRAGMENT_CONDITION_FONT_SIZE,
        ));

        // Именно ширина ПО УСЛОВИЮ, без оглядки на рамки участников: в
        // эталоне `sequence_fragments` рамка (162.40) УЖЕ суммы блоков
        // участников (10…175.27).
        let _ = (min_x, max_x);
        let width = condition_width + FRAGMENT_CONDITION_EXTRA;
        let fragment_bounds = Rect::new(
            min_x - self.config.fragment_padding + FRAGMENT_FRAME_LEFT_INSET,
            start_y,
            width,
            end_y - start_y,
        );

        // Формируем тип фрагмента
        let fragment_type_str = match frag.fragment_type {
            FragmentType::Alt => "alt",
            FragmentType::Opt => "opt",
            FragmentType::Loop => "loop",
            FragmentType::Par => "par",
            FragmentType::Break => "break",
            FragmentType::Critical => "critical",
            FragmentType::Group => "group",
            FragmentType::Ref => "ref",
        };

        // Условие первой секции уже должно быть установлено парсером
        // Если нет — используем условие из fragment.condition для обратной совместимости
        if !layout_sections.is_empty()
            && layout_sections[0].condition.is_none()
            && frag.condition.is_some()
        {
            layout_sections[0].condition = frag.condition.clone();
        }

        let fragment_elem = LayoutElement {
            id: format!("fragment_{}", fragment_type_str),
            bounds: fragment_bounds,
            text: None,
            // Заголовок фрагмента (alt/opt/loop) PlantUML рисует
            // ПОЛУЖИРНЫМ: в эталоне sequence_fragments все три элемента
            // («alt», «[Успешно]», «[Ошибка]») имеют font-weight="700".
            properties: [("font-weight".to_string(), "700".to_string())]
                .into_iter()
                .collect(),
            element_type: ElementType::Fragment {
                fragment_type: fragment_type_str.to_string(),
                sections: layout_sections,
            },
        };

        elements.push(fragment_elem);
    }

    /// Находит X границы фрагмента
    /// Рамка должна охватывать всех участников, задействованных в сообщениях внутри фрагмента
    fn find_fragment_x_bounds(&self, frag: &Fragment, metrics: &DiagramMetrics) -> (f64, f64) {
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;

        for section in &frag.sections {
            for elem in &section.elements {
                if let SequenceElement::Message(msg) = elem {
                    // Используем границы участника (center_x ± width/2) для полного охвата
                    if let Some(participant) = metrics.participants.get(&msg.from) {
                        let left = participant.center_x - participant.width / 2.0;
                        let right = participant.center_x + participant.width / 2.0;
                        min_x = min_x.min(left);
                        max_x = max_x.max(right);
                    }
                    if let Some(participant) = metrics.participants.get(&msg.to) {
                        let left = participant.center_x - participant.width / 2.0;
                        let right = participant.center_x + participant.width / 2.0;
                        min_x = min_x.min(left);
                        max_x = max_x.max(right);
                    }
                }
            }
        }

        // Если фрагмент пустой, используем всю ширину
        if min_x == f64::MAX {
            min_x = self.config.margin;
            max_x = metrics.max_x;
        }

        (min_x, max_x)
    }

    /// Размещает заметку
    fn layout_note(
        &self,
        note: &Note,
        metrics: &mut DiagramMetrics,
        elements: &mut Vec<LayoutElement>,
    ) {
        // Примечание после сообщения ставится СРАЗУ под ним.
        //
        // Измерено по эталону `sequence_notes`: стрелка на 70.43,
        // примечание начинается на 83.43, то есть отступ 13. Прежде
        // примечание ставилось на `current_y`, который уже сдвинут на
        // весь шаг сообщения (29.133), и оно оказывалось на 16 ниже.
        let y = if metrics.last_message_y > 0.0 {
            metrics.last_message_y + NOTE_TOP_GAP
        } else {
            metrics.current_y
        };

        // Определяем X позицию
        let x = if note.anchors.is_empty() {
            self.config.margin
        } else if note.anchors.len() == 1 {
            let anchor_x = metrics
                .participant_center_x(&note.anchors[0])
                .unwrap_or(self.config.margin);
            match note.position {
                // Отступ заметки от линии жизни. Измерено по эталону:
                // линия Bob на x=106.487, заметка начинается на 110.987,
                // то есть отступ 4.5.
                NotePosition::Left => anchor_x - self.config.note_width - NOTE_ANCHOR_GAP,
                NotePosition::Right => anchor_x + NOTE_ANCHOR_GAP,
                NotePosition::Over => anchor_x - self.config.note_width / 2.0,
                NotePosition::Top | NotePosition::Bottom => anchor_x - self.config.note_width / 2.0,
            }
        } else {
            // Over multiple participants.
            //
            // Список анкоров может быть пуст — парсер считает такую заметку
            // допустимой. Раньше здесь стояли note.anchors[0] и
            // anchors.last().unwrap(), то есть паника на валидном входе.
            let first_x = note
                .anchors
                .first()
                .and_then(|a| metrics.participant_center_x(a))
                .unwrap_or(self.config.margin);
            let last_x = note
                .anchors
                .last()
                .and_then(|a| metrics.participant_center_x(a))
                .unwrap_or(first_x);
            (first_x + last_x) / 2.0 - self.config.note_width / 2.0
        };

        let bounds = Rect::new(x, y, self.config.note_width, self.config.note_height);

        // Цвет заметки в PlantUML — светло-жёлтый (#FEFFDD).
        // Раньше заливка бралась из темы (серо-голубая #E2E2F0), из-за чего
        // заметка визуально не отличалась от участников.
        let mut properties = std::collections::HashMap::new();
        properties.insert("fill".to_string(), NOTE_BACKGROUND.to_string());
        // Текст примечания выравнивается по ЛЕВОМУ краю с отступом.
        // Измерено по эталону `sequence_notes`: рамка начинается на
        // 110.987, а текст — на 116.987, то есть отступ 6. Прежде текст
        // центрировался и при длинной подписи выходил за рамку.
        properties.insert("text-align".to_string(), "left".to_string());
        properties.insert("text-inset".to_string(), NOTE_TEXT_INSET.to_string());
        // Текст примечания PlantUML пишет кеглем 13, а не темой (14):
        // эталон `sequence_notes` даёт `font-size="13"` и textLength 73.830
        // против наших 79.509.
        properties.insert("font-size".to_string(), NOTE_FONT_SIZE.to_string());

        let note_elem = LayoutElement {
            id: format!("note_{}", y as u32),
            bounds,
            text: None,
            properties,
            element_type: ElementType::Note {
                label: note.text.clone(),
                fold: NOTE_FOLD,
            },
        };

        elements.push(note_elem);

        // Следующее сообщение идёт НИЖЕ примечания. Измерено по эталону:
        // низ примечания 108.43, следующая стрелка на 134.695.
        let note_bottom = y + self.config.note_height;
        metrics.current_y = metrics.current_y.max(note_bottom + NOTE_BOTTOM_GAP);
    }

    /// Размещает ref блок (ссылка на другую диаграмму)
    /// Ref блок в PlantUML выглядит как рамка с меткой "ref" в левом верхнем углу
    fn layout_reference(
        &self,
        reference: &Reference,
        metrics: &mut DiagramMetrics,
        elements: &mut Vec<LayoutElement>,
    ) {
        let y = metrics.current_y;

        // Определяем границы ref блока по охваченным участникам
        let (x, width) = if reference.participants.is_empty() {
            // Если участники не указаны, охватываем всю диаграмму
            (self.config.margin, metrics.max_x - self.config.margin * 2.0)
        } else if reference.participants.len() == 1 {
            // Один участник - центрируем вокруг него
            let center_x = reference
                .participants
                .first()
                .and_then(|p| metrics.participant_center_x(p))
                .unwrap_or(self.config.margin + 50.0);
            let width = self.config.participant_spacing.max(100.0);
            (center_x - width / 2.0, width)
        } else {
            // Несколько участников - от первого до последнего
            let first_x = reference
                .participants
                .first()
                .and_then(|p| metrics.participant_center_x(p))
                .unwrap_or(self.config.margin);
            let last_x = reference
                .participants
                .last()
                .and_then(|p| metrics.participant_center_x(p))
                .unwrap_or(metrics.max_x);
            let padding = 20.0;
            (first_x - padding, last_x - first_x + padding * 2.0)
        };

        // Высота ref блока зависит от текста
        let text_lines = reference.text.lines().count().max(1);
        let ref_height = self.config.fragment_header_height
            + (text_lines as f64 * self.config.line_height)
            + 10.0;

        let bounds = Rect::new(x, y, width, ref_height);

        // Создаём ref блок как фрагмент
        let ref_elem = LayoutElement {
            id: format!("ref_{}", y as u32),
            bounds,
            text: Some(reference.text.clone()),
            properties: {
                let mut props = std::collections::HashMap::new();
                props.insert("type".to_string(), "ref".to_string());
                props.insert("label".to_string(), "ref".to_string());
                props
            },
            element_type: ElementType::Fragment {
                fragment_type: "ref".to_string(),
                sections: vec![FragmentSection {
                    condition: Some(reference.text.clone()),
                    start_y: y,
                    end_y: y + ref_height,
                    children: vec![],
                }],
            },
        };

        elements.push(ref_elem);
        metrics.advance_y(ref_height + self.config.message_spacing);
    }

    /// Y-координата, на которой завершаются незакрытые активации.
    ///
    /// Совпадает с позицией нижнего блока участника минус небольшой отступ:
    /// в эталоне PlantUML активация заканчивается на 10px выше footer.
    fn activation_footer_y(&self, metrics: &DiagramMetrics) -> f64 {
        metrics.current_y + FOOTER_GAP - self.config.message_spacing - ACTIVATION_FOOTER_GAP
    }

    /// Обрабатывает активацию/деактивацию
    fn process_activation(&self, act: &Activation, metrics: &mut DiagramMetrics) {
        match act.activation_type {
            ActivationType::Activate => {
                metrics.activate(&act.participant);
            }
            ActivationType::Deactivate | ActivationType::Destroy => {
                // Завершаем активацию на той же отметке, где начинается
                // нижний блок участника, минус отступ: в эталоне PlantUML
                // активация заканчивается на 10px выше footer.
                //
                // Раньше использовался metrics.current_y, а footer
                // вычисляется как current_y + FOOTER_GAP - message_spacing,
                // то есть на 11px меньше — из-за этого активация вылезала
                // за footer ровно на эти 11px.
                let end_y = self.activation_footer_y(metrics);
                metrics.deactivate_at(&act.participant, end_y);
            }
        }
    }

    /// Размещает разделитель
    fn layout_divider(
        &self,
        div: &Divider,
        metrics: &mut DiagramMetrics,
        elements: &mut Vec<LayoutElement>,
    ) {
        let y = metrics.current_y;

        // Линия через всю диаграмму
        let divider = LayoutElement {
            id: format!("divider_{}", y as u32),
            bounds: Rect::new(
                self.config.margin,
                y,
                metrics.max_x - self.config.margin,
                self.config.divider_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Text {
                text: format!("== {} ==", div.text),
                font_size: self.config.font_size,
            },
        };

        elements.push(divider);
        metrics.advance_y(self.config.divider_height);
    }

    /// Размещает задержку
    fn layout_delay(
        &self,
        delay: &Delay,
        metrics: &mut DiagramMetrics,
        elements: &mut Vec<LayoutElement>,
    ) {
        let y = metrics.current_y;

        let text = delay.text.clone().unwrap_or_else(|| "...".to_string());

        let delay_elem = LayoutElement {
            id: format!("delay_{}", y as u32),
            bounds: Rect::new(
                self.config.margin,
                y,
                metrics.max_x - self.config.margin,
                self.config.delay_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Text {
                text,
                font_size: self.config.font_size,
            },
        };

        elements.push(delay_elem);
        metrics.advance_y(self.config.delay_height);
    }

    /// Добавляет прямоугольники активаций
    fn add_activations(&self, metrics: &DiagramMetrics, elements: &mut Vec<LayoutElement>) {
        for (i, (info, end_y)) in metrics.completed_activations.iter().enumerate() {
            if let Some(participant) = metrics.participants.get(&info.participant) {
                // Вычисляем X позицию с учётом уровня вложенности
                let offset = (info.level as f64 - 1.0) * self.config.activation_width / 2.0;
                let x = participant.center_x - self.config.activation_width / 2.0 + offset;

                let height = end_y - info.start_y;

                // Минимальная высота активации
                let height = height.max(10.0);

                let activation = LayoutElement {
                    id: format!("activation_{}_{}", info.participant, i),
                    bounds: Rect::new(x, info.start_y, self.config.activation_width, height),
                    // Используем специальный тип Activation для белого фона
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Activation,
                };

                elements.push(activation);
            }
        }
    }

    /// Добавляет lifelines
    /// Lifeline идёт от нижней границы header до верхней границы footer
    fn add_lifelines(&self, metrics: &DiagramMetrics, elements: &mut Vec<LayoutElement>) {
        // Lifeline заканчивается на верхней границе footer
        // PlantUML: отступ от последней стрелки до footer ~17px
        // current_y уже включает message_spacing (28px) после последнего сообщения
        // Нужно: last_message_y + 17 = footer_y
        // last_message_y = current_y - message_spacing (приблизительно)
        // Используем: current_y - message_spacing + 17 ≈ current_y - 11
        let footer_y = metrics.current_y + FOOTER_GAP - self.config.message_spacing;
        let end_y = footer_y;

        for (id, participant) in &metrics.participants {
            // Lifeline начинается от нижней границы header участника
            // (учитывает box_title_height если есть боксы)
            let start_y = participant.header_bounds.y + self.config.participant_height;

            let lifeline = LayoutElement {
                id: format!("lifeline_{}", id),
                bounds: Rect::new(participant.center_x - 0.5, start_y, 1.0, end_y - start_y),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Edge {
                    points: vec![
                        Point::new(participant.center_x, start_y),
                        Point::new(participant.center_x, end_y),
                    ],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: true,              // Lifelines всегда пунктирные
                    edge_type: EdgeType::Link, // линия без маркеров
                    from_cardinality: None,
                    to_cardinality: None,
                },
            };
            elements.push(lifeline);
        }
    }

    /// Добавляет нижние блоки участников
    fn add_participant_footers(&self, metrics: &DiagramMetrics, elements: &mut Vec<LayoutElement>) {
        // PlantUML: отступ от последней стрелки до footer ~17px
        // current_y уже включает message_spacing после последнего сообщения
        // Компенсируем: current_y - message_spacing + 17 ≈ current_y - 11
        let y = metrics.current_y + FOOTER_GAP - self.config.message_spacing;

        for (id, participant) in &metrics.participants {
            // Нижний блок повторяет тип участника: PlantUML рисует и сверху,
            // и снизу одну и ту же фигуру (стик-фигура, кружок, цилиндр).
            // Раньше здесь всегда стоял прямоугольник, поэтому `actor` и
            // `database` снизу выглядели неправильно.
            let extra = figure_extra_below(participant.participant_type);
            let bounds = Rect::new(
                participant.center_x - participant.width / 2.0,
                y,
                participant.width,
                self.config.participant_height + extra,
            );

            let element_type = match participant.participant_type {
                ParticipantType::Actor => ElementType::Actor {
                    label: participant.display_name.clone(),
                },
                ParticipantType::Database => ElementType::Database {
                    label: participant.display_name.clone(),
                },
                ParticipantType::Boundary => ElementType::Boundary {
                    label: participant.display_name.clone(),
                },
                ParticipantType::Control => ElementType::Control {
                    label: participant.display_name.clone(),
                },
                ParticipantType::Entity => ElementType::Entity {
                    label: participant.display_name.clone(),
                },
                _ => ElementType::Rectangle {
                    label: participant.display_name.clone(),
                    corner_radius: 2.5, // PlantUML style
                },
            };

            elements.push(LayoutElement {
                id: format!("footer_{}", id),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type,
            });
        }
    }
}

impl Default for SequenceLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::LayoutEngine for SequenceLayoutEngine {
    type Input = SequenceDiagram;

    fn layout(&self, input: &Self::Input, _config: &LayoutConfig) -> LayoutResult {
        self.layout(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::sequence::Participant;

    #[test]
    fn test_empty_diagram() {
        let engine = SequenceLayoutEngine::new();
        let diagram = SequenceDiagram::new();

        let result = engine.layout(&diagram);

        assert!(result.elements.is_empty() || result.elements.len() <= 2);
    }

    #[test]
    fn test_simple_diagram() {
        let engine = SequenceLayoutEngine::new();
        let mut diagram = SequenceDiagram::new();

        diagram.add_participant(Participant::as_participant("Alice"));
        diagram.add_participant(Participant::as_participant("Bob"));
        diagram.add_element(SequenceElement::Message(Message::new(
            "Alice", "Bob", "Hello",
        )));

        let result = engine.layout(&diagram);

        // Должны быть: 2 участника + 1 сообщение + 2 lifeline + 2 footer
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_self_message() {
        let engine = SequenceLayoutEngine::new();
        let mut diagram = SequenceDiagram::new();

        diagram.add_participant(Participant::as_participant("Alice"));
        diagram.add_element(SequenceElement::Message(Message::new(
            "Alice", "Alice", "Think",
        )));

        let result = engine.layout(&diagram);

        // Проверяем, что self-message создан
        let has_message = result.elements.iter().any(
            |e| matches!(&e.element_type, ElementType::Edge { points, .. } if points.len() > 2),
        );

        assert!(has_message, "Self-message should have more than 2 points");
    }

    #[test]
    fn test_fragment_layout() {
        let engine = SequenceLayoutEngine::new();
        let mut diagram = SequenceDiagram::new();

        diagram.add_participant(Participant::as_participant("Alice"));
        diagram.add_participant(Participant::as_participant("Bob"));

        let fragment = Fragment {
            fragment_type: FragmentType::Alt,
            condition: Some("success".to_string()),
            sections: vec![plantuml_ast::sequence::FragmentSection {
                condition: None,
                elements: vec![SequenceElement::Message(Message::new("Alice", "Bob", "OK"))],
            }],
        };

        diagram.add_element(SequenceElement::Fragment(fragment));

        let result = engine.layout(&diagram);

        // Должен быть элемент Fragment
        let has_fragment = result
            .elements
            .iter()
            .any(|e| matches!(&e.element_type, ElementType::Fragment { .. }));

        assert!(has_fragment, "Should have a fragment element");
    }

    #[test]
    fn test_activation_layout() {
        let engine = SequenceLayoutEngine::new();
        let mut diagram = SequenceDiagram::new();

        diagram.add_participant(Participant::as_participant("Alice"));
        diagram.add_participant(Participant::as_participant("Bob"));

        // Сообщение с активацией
        let mut msg = Message::new("Alice", "Bob", "Request");
        msg.activate = true;
        diagram.add_element(SequenceElement::Message(msg));

        // Ответ с деактивацией
        let mut reply = Message::new("Bob", "Alice", "Response");
        reply.deactivate = true;
        diagram.add_element(SequenceElement::Message(reply));

        let result = engine.layout(&diagram);

        // Должен быть элемент активации (Activation)
        let activation_count = result
            .elements
            .iter()
            .filter(|e| {
                e.id.starts_with("activation_")
                    && matches!(&e.element_type, ElementType::Activation)
            })
            .count();

        assert!(
            activation_count >= 1,
            "Should have at least one activation element"
        );
    }

    #[test]
    fn test_nested_activation() {
        let engine = SequenceLayoutEngine::new();
        let mut diagram = SequenceDiagram::new();

        diagram.add_participant(Participant::as_participant("Alice"));

        // Первая активация
        diagram.add_element(SequenceElement::Activation(Activation {
            participant: "Alice".to_string(),
            activation_type: ActivationType::Activate,
            color: None,
        }));

        // Вложенная активация
        diagram.add_element(SequenceElement::Activation(Activation {
            participant: "Alice".to_string(),
            activation_type: ActivationType::Activate,
            color: None,
        }));

        // Self-message
        diagram.add_element(SequenceElement::Message(Message::new(
            "Alice", "Alice", "Think",
        )));

        // Деактивации
        diagram.add_element(SequenceElement::Activation(Activation {
            participant: "Alice".to_string(),
            activation_type: ActivationType::Deactivate,
            color: None,
        }));

        diagram.add_element(SequenceElement::Activation(Activation {
            participant: "Alice".to_string(),
            activation_type: ActivationType::Deactivate,
            color: None,
        }));

        let result = engine.layout(&diagram);

        // Должно быть 2 прямоугольника активации
        let activation_count = result
            .elements
            .iter()
            .filter(|e| e.id.starts_with("activation_"))
            .count();

        assert_eq!(
            activation_count, 2,
            "Should have 2 activation rectangles for nested activations"
        );
    }

    #[test]
    fn test_participant_with_alias() {
        let engine = SequenceLayoutEngine::new();
        let mut diagram = SequenceDiagram::new();

        // Участник с alias: "Сервис Обработки" as Processor
        let mut participant = Participant::as_participant("Сервис Обработки");
        participant.id.alias = Some("Processor".to_string());
        diagram.add_participant(participant);

        // Self-message использует alias
        diagram.add_element(SequenceElement::Message(Message::new(
            "Processor",
            "Processor",
            "Инициализация",
        )));

        let result = engine.layout(&diagram);

        // Должен быть только ОДИН header участника
        let participant_count = result
            .elements
            .iter()
            .filter(|e| e.id.starts_with("participant_"))
            .count();

        assert_eq!(
            participant_count, 1,
            "Should have exactly 1 participant header, got {}",
            participant_count
        );

        // Должен быть только ОДИН footer участника
        let footer_count = result
            .elements
            .iter()
            .filter(|e| e.id.starts_with("footer_"))
            .count();

        assert_eq!(
            footer_count, 1,
            "Should have exactly 1 participant footer, got {}",
            footer_count
        );
    }
}
