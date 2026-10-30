//! Activity Diagram Layout Engine
//!
//! Flowchart-based layout algorithm для activity diagrams.

use std::collections::HashMap;

use plantuml_ast::activity::{
    Action, ActionStyle, ActivityDiagram, ActivityElement, ArrowSpec, Condition, Fork, Partition,
    RepeatLoop, Split, Switch, WhileLoop,
};
use plantuml_ast::common::Color;
use plantuml_model::{Point, Rect};

/// Сторона ромба слияния после ветвления (измерено по эталону).
/// Внутреннее поле диаграммы деятельности.
///
/// Измерено по эталону `activity_basic`: контент начинается в (16, 15)
/// при нулевом начале координат вывода.
const ACTIVITY_CONTENT_INSET_X: f64 = 16.0;
const ACTIVITY_CONTENT_INSET_Y: f64 = 15.0;

const MERGE_DIAMOND_SIZE: f64 = 24.0;

/// Отступ от конца ветвей до ромба слияния (измерено по эталону).
const MERGE_DIAMOND_GAP: f64 = 6.0;

/// Насколько конечный узел больше начального (измерено по эталону).
const END_RADIUS_EXTRA: f64 = 1.0;

/// Цвет заливки фигур activity.
///
/// PlantUML заливает действия, шестиугольник условия и ромб слияния
/// одним цветом `#F1F1F1` — он светлее темы `node_background`
/// (`#E2E2F0`), поэтому цвет передаётся свойством.
const ACTIVITY_SHAPE_BACKGROUND: &str = "#F1F1F1";

/// Зазор между соседними столбцами ветвей (эталон `elseif`: правая
/// вершина ромба и левая вершина следующего отстоят на 20).
const COLUMN_GAP: f64 = 20.0;

/// Расстояние от низа ромба до верха тела ветви в цепочке `elseif` и `switch`.
const BRANCH_DROP: f64 = 32.8;

/// Отступ линии слияния от низа самых длинных ветвей (эталон `elseif`).
const MERGE_BAR_DROP: f64 = 20.0;

/// Кегль и высота рамки предупреждения об устаревшем синтаксисе.
const WARNING_FONT_SIZE: f64 = 10.0;
const WARNING_BOX_HEIGHT: f64 = 16.641;
/// Зазор между рамкой предупреждения и схемой.
const WARNING_GAP: f64 = 7.0;

/// Выемка пятиугольника `<<input>>` и `<<output>>`.
const SIGNAL_NOTCH: f64 = 10.0;

/// Радиус скругления действий (эталон `activity_basic`: `rx=12.5`).
const ACTIVITY_ACTION_CORNER_RADIUS: f64 = 12.5;

/// Кегль подписей действий.
///
/// Эталон `activity_basic` рисует «Первый шаг» 12-м кеглем; тема даёт 14,
/// из-за чего подписи выходили на 16 % крупнее эталонных.
const ACTIVITY_ACTION_FONT_SIZE: f64 = 12.0;

/// Кегль подписей условия и ветвлений («да», «нет»).
const ACTIVITY_LABEL_FONT_SIZE: f64 = 11.0;

/// Срез боковых вершин шестиугольника условия.
///
/// Эталон `activity_basic`: шестиугольник `68.9..139.8` при плоской
/// верхней грани `80.9..127.8`, то есть срез по 12 с каждой стороны.
const CONDITION_CORNER_CUT: f64 = 12.0;

/// Насколько базис подписи ветвления выше плеча излома.
///
/// Эталон `activity_basic`: плечо на `y=120.97`, базис «да» — `118.38`.
const BRANCH_LABEL_BASELINE_OFFSET: f64 = 2.62;

use super::config::ActivityLayoutConfig;

/// Цвет заметки в PlantUML — светло-жёлтый.
const ACTIVITY_NOTE_BACKGROUND: &str = "#FEFFDD";

/// Минимальный радиус коннектора.
const CONNECTOR_MIN_RADIUS: f64 = 12.0;

/// Внутренний отступ рамки раздела.
const PARTITION_PADDING: f64 = 8.0;

/// Расстояние от низа условия `while` до тела, когда есть подпись `is (..)`.
const WHILE_LABELED_GAP: f64 = 31.7;

/// Насколько линия возврата и выхода из `while` ниже тела (эталон `activity_branch`).
const WHILE_RETURN_DROP: f64 = 10.0;

/// Высота полосы с названием раздела над потоком.
const PARTITION_HEADER_HEIGHT: f64 = 20.0;

/// Зазор между потоком и заметкой.
const ACTIVITY_NOTE_GAP: f64 = 10.0;

/// Добавка к отступу ветви за каждую петлю внутри неё.
const ACTIVITY_LOOP_OFFSET: f64 = 6.0;

/// Добавка к ширине действия: вмещает внутренние отступы.
///
/// Измерено по эталону activity_basic: «Первый шаг» 76.822 → 96.8,
/// «Последний шаг» 98.209 → 118.2.
const ACTION_TEXT_PADDING: f64 = 20.0;
use crate::{EdgeType, ElementType, LayoutElement, LayoutResult};

/// Информация о swimlane для layout
#[derive(Debug, Clone)]
struct SwimlaneInfo {
    /// Имя swimlane
    name: String,
    /// Цвет фона
    color: Option<Color>,
    /// Индекс (порядок появления)
    index: usize,
    /// X-координата центра swimlane
    center_x: f64,
}

/// Layout engine для activity diagrams
pub struct ActivityLayoutEngine {
    config: ActivityLayoutConfig,
}

impl ActivityLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: ActivityLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: ActivityLayoutConfig) -> Self {
        Self { config }
    }

    /// Свойства фигуры activity: заливка, скругление и кегль подписи.    ///
    /// `render_rectangle` читает эти свойства, поэтому движку достаточно
    /// пометить элемент — отдельная ветка рендеринга не нужна.
    fn shape_properties(font_size: f64) -> HashMap<String, String> {
        let mut properties = HashMap::new();
        properties.insert("fill".to_string(), ACTIVITY_SHAPE_BACKGROUND.to_string());
        properties.insert("rx".to_string(), ACTIVITY_ACTION_CORNER_RADIUS.to_string());
        properties.insert("font-size".to_string(), font_size.to_string());
        properties
    }

    /// Геометрия шестиугольника условия.
    ///
    /// PlantUML рисует условие `if` плоским сверху и снизу, а боковые
    /// вершины срезает: ширина зависит от подписи, высота фиксирована.
    /// Возвращает ширину и вершины в локальных координатах фигуры.
    fn condition_shape(&self, condition: &str) -> (f64, Vec<Point>) {
        let text_width = self.config.text.width(condition, ACTIVITY_LABEL_FONT_SIZE);
        let cut = CONDITION_CORNER_CUT;
        let height = self.config.diamond_height;
        let width = (text_width + cut * 2.0).max(cut * 4.0);

        let points = vec![
            Point::new(cut, 0.0),
            Point::new(width - cut, 0.0),
            Point::new(width, height / 2.0),
            Point::new(width - cut, height),
            Point::new(cut, height),
            Point::new(0.0, height / 2.0),
        ];

        (width, points)
    }

    /// Добавляет ромб условия (шестиугольник) с подписью внутри.
    fn add_condition_shape(
        &self,
        condition: &str,
        center_x: f64,
        top_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let (width, points) = self.condition_shape(condition);
        let height = self.config.diamond_height;

        elements.push(LayoutElement {
            id: format!("condition_{}", elements.len()),
            bounds: Rect::new(center_x - width / 2.0, top_y, width, height),
            text: None,
            properties: Self::shape_properties(ACTIVITY_LABEL_FONT_SIZE),
            element_type: ElementType::Polygon {
                points,
                label: Some(condition.to_string()),
                font_size: ACTIVITY_LABEL_FONT_SIZE,
            },
        });

        width
    }

    /// Добавляет подпись ветвления над плечом излома.
    ///
    /// PlantUML прижимает подпись к боковой вершине шестиугольника:
    /// «да» кончается на левой вершине, «нет» начинается на правой
    /// (эталон `activity_basic`).
    fn add_branch_label(
        &self,
        label: &str,
        anchor_x: f64,
        line_y: f64,
        align_end: bool,
        elements: &mut Vec<LayoutElement>,
    ) {
        let width = self.config.text.width(label, ACTIVITY_LABEL_FONT_SIZE);
        let x = if align_end {
            anchor_x - width
        } else {
            anchor_x
        };

        elements.push(LayoutElement {
            id: format!("branch_label_{}", elements.len()),
            bounds: Rect::new(
                x,
                line_y - BRANCH_LABEL_BASELINE_OFFSET - ACTIVITY_LABEL_FONT_SIZE,
                width,
                ACTIVITY_LABEL_FONT_SIZE,
            ),
            text: None,
            properties: HashMap::new(),
            element_type: ElementType::Text {
                text: label.to_string(),
                font_size: ACTIVITY_LABEL_FONT_SIZE,
            },
        });
    }

    /// Добавляет стрелку с одним изломом (горизонталь, затем вертикаль).
    ///
    /// PlantUML не соединяет ветви с условием по диагонали: эталон
    /// `activity_basic` даёт две отдельные линии —
    /// `(68.86,120.97)-(54.34,120.97)` и `(54.34,120.97)-(54.34,142.97)`.
    fn add_elbow_arrow(&self, points: Vec<Point>, elements: &mut Vec<LayoutElement>) {
        self.add_path(points, true, elements);
    }

    /// Ломаная линия потока, со стрелкой на конце или без неё.
    fn add_path(&self, points: Vec<Point>, arrow_end: bool, elements: &mut Vec<LayoutElement>) {
        let xs: Vec<f64> = points.iter().map(|p| p.x).collect();
        let ys: Vec<f64> = points.iter().map(|p| p.y).collect();
        let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        elements.push(LayoutElement {
            id: format!("elbow_{}", elements.len()),
            bounds: Rect::new(
                min_x,
                min_y,
                (max_x - min_x).max(1.0),
                (max_y - min_y).max(1.0),
            ),
            text: None,
            properties: HashMap::new(),
            element_type: ElementType::Edge {
                points,
                label: None,
                arrow_start: false,
                arrow_end,
                dashed: false,
                edge_type: EdgeType::Association,
                from_cardinality: None,
                to_cardinality: None,
            },
        });
    }

    /// Раскладывает тело ветви условия.
    ///
    /// Отличие от последовательного вызова `layout_element` в одном:
    /// последнее действие ветви не рисует стрелку «вниз». PlantUML ведёт
    /// поток от конца ветви прямо в ромб слияния, поэтому без этой
    /// поправки из-под излома торчал хвост длиной в `vertical_spacing`.
    /// Естественная ширина тела ветви: максимум по всем блокам внутри.
    ///
    /// Нужна ДО раскладки, потому что позиция ветви зависит от её
    /// ширины. Считается рекурсивно: у цикла берётся максимум из его
    /// тела и самого условия.
    ///
    /// Формула позиционирования снята с сервера PlantUML: при трёх
    /// замерах с одинаковой шириной блока везде получилось
    /// «шаг = w/2 + 10» и «между ветвями = w + 20» (совпадение до
    /// 0.03 px). Константа вместо этого не годится: подбор значения
    /// улучшал `activity_branch` с 77 до 22 px, но ломал точный
    /// `activity_basic` с 0 до 37 px.
    /// Число петель (`while`/`repeat`) в теле ветви.
    ///
    /// Измерено на сервере: цикл добавляет к отстуту ровно 6.00 px —
    /// одинаково при узком и широком теле и при двух циклах подряд
    /// (три независимых замера, плюс расхождение 6.00 в `activity_branch`).
    /// Число действий при этом не влияет ни на что: 1, 2 и 3 действия дали
    /// одно и то же отступ до сотых.
    ///
    /// Похоже, PlantUML резервирует место под обратную стрелку цикла.
    fn branch_loop_count(&self, body: &[ActivityElement]) -> usize {
        let mut count = 0;
        for elem in body {
            match elem {
                ActivityElement::While(while_loop) => {
                    count += 1 + self.branch_loop_count(&while_loop.body);
                }
                ActivityElement::Repeat(repeat_loop) => {
                    count += 1 + self.branch_loop_count(&repeat_loop.body);
                }
                ActivityElement::Condition(cond) => {
                    count += self.branch_loop_count(&cond.then_branch);
                    if let Some(else_branch) = &cond.else_branch {
                        count += self.branch_loop_count(else_branch);
                    }
                    for branch in &cond.elseif_branches {
                        count += self.branch_loop_count(&branch.elements);
                    }
                }
                _ => {}
            }
        }
        count
    }

    fn branch_natural_width(&self, body: &[ActivityElement]) -> f64 {
        let mut widest: f64 = 0.0;

        for elem in body {
            let width = match elem {
                ActivityElement::Action(action) => {
                    self.config.text.width(&action.label, self.config.font_size)
                        + ACTION_TEXT_PADDING
                }
                ActivityElement::Condition(cond) if !cond.elseif_branches.is_empty() => {
                    Self::columns_total_width(&self.chain_widths(cond))
                }
                ActivityElement::Condition(cond) => self.condition_shape(&cond.condition).0,
                ActivityElement::Fork(fork) => {
                    let branches: Vec<&[ActivityElement]> =
                        fork.branches.iter().map(Vec::as_slice).collect();
                    Self::columns_total_width(&self.fork_widths(&branches))
                }
                ActivityElement::Split(split) => {
                    let branches: Vec<&[ActivityElement]> = split
                        .branches
                        .iter()
                        .map(|branch| branch.elements.as_slice())
                        .collect();
                    Self::columns_total_width(&self.fork_widths(&branches))
                }
                ActivityElement::Switch(switch) => {
                    let widths: Vec<f64> = switch
                        .branches
                        .iter()
                        .map(|branch| self.branch_natural_width(&branch.elements))
                        .collect();
                    Self::columns_total_width(&widths).max(
                        self.condition_shape(switch.expression.as_deref().unwrap_or(""))
                            .0,
                    )
                }
                ActivityElement::While(while_loop) => self
                    .condition_shape(&while_loop.condition)
                    .0
                    .max(self.branch_natural_width(&while_loop.body)),
                ActivityElement::Repeat(repeat_loop) => self
                    .condition_shape(&repeat_loop.condition)
                    .0
                    .max(self.branch_natural_width(&repeat_loop.body)),
                ActivityElement::Start | ActivityElement::Stop | ActivityElement::End => {
                    self.config.bar_width
                }
                // Остальные вложенные конструкции считаем по действиям
                // верхнего уровня: их собственная ширина уже учтена, когда
                // они встретятся рекурсивно.
                _ => self.config.action_width,
            };
            widest = widest.max(width);
        }

        widest
    }

    /// Горизонтальный отступ ветви от центра условия.
    ///
    /// Измерено на сервере: `w/2 + 10`, где `w` — самая широкая ветвь
    /// условия. Ветви then и else стоят симметрично на `±S`.
    fn branch_offset(&self, widest: f64) -> f64 {
        widest / 2.0 + 10.0
    }

    fn layout_branch_body(
        &self,
        body: &[ActivityElement],
        center_x: f64,
        start_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let mut y = start_y;
        for elem in body {
            y = self.layout_element(elem, center_x, y, elements);
        }

        let is_plain_tail = elements.last().is_some_and(|element| {
            element.id.starts_with("arrow_")
                && matches!(&element.element_type, ElementType::Edge { points, .. } if points.len() == 2)
        });
        if is_plain_tail {
            elements.pop();
        }

        y
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &ActivityDiagram) -> LayoutResult {
        let mut elements = Vec::new();

        // Собираем информацию о swimlanes
        let swimlanes = self.collect_swimlanes(&diagram.elements);
        let has_swimlanes = !swimlanes.is_empty();

        // Начальная Y позиция (после заголовков swimlanes если есть)
        let content_start_y = if has_swimlanes {
            self.config.margin + self.config.swimlane_header_height
        } else {
            self.config.margin
        };

        let mut current_y = content_start_y;

        // Предупреждения об устаревшем синтаксисе PlantUML рисует рамками
        // над схемой.
        for warning in &diagram.warnings {
            self.push_warning(warning, self.config.margin, current_y, &mut elements);
            current_y += WARNING_BOX_HEIGHT + WARNING_GAP;
        }

        // Текущий swimlane
        let mut current_swimlane: Option<String> = None;

        // Определяем center_x в зависимости от наличия swimlanes
        let default_center_x = self.config.margin + self.config.action_width / 2.0;

        // Храним позицию предыдущего элемента для отрисовки стрелок
        // (center_x, bottom_y, is_start_or_action) - is_start_or_action указывает, нужна ли стрелка
        let mut prev_element_info: Option<(f64, f64, bool)> = None;
        let mut pending_arrow: Option<ArrowSpec> = None;

        // Обрабатываем элементы последовательно
        for element in &diagram.elements {
            // Обрабатываем смену swimlane
            if let ActivityElement::SwimlaneChange(swimlane) = element {
                current_swimlane = Some(swimlane.name.clone());
                continue;
            }

            // `detach` и `kill` обрывают поток: следующий элемент не
            // соединяется стрелкой с предыдущим.
            if matches!(element, ActivityElement::Detach | ActivityElement::Kill) {
                prev_element_info = None;
                continue;
            }

            // `-> текст;` оформляет следующую стрелку потока.
            if let ActivityElement::Arrow(spec) = element {
                pending_arrow = Some(spec.clone());
                continue;
            }

            // Получаем center_x для текущего swimlane
            let center_x = if let Some(ref lane_name) = current_swimlane {
                swimlanes
                    .get(lane_name)
                    .map(|info| info.center_x)
                    .unwrap_or(default_center_x)
            } else if has_swimlanes {
                // Если есть swimlanes но текущий не установлен, используем первый
                swimlanes
                    .values()
                    .next()
                    .map(|info| info.center_x)
                    .unwrap_or(default_center_x)
            } else {
                default_center_x
            };

            // Заметка не входит в поток: ни места, ни стрелки не занимает.
            if let ActivityElement::Note(note) = element {
                let note_center_x = prev_element_info.map_or(center_x, |info| info.0);
                self.push_note(note, note_center_x, current_y, &mut elements);
                continue;
            }

            // Рисуем стрелку от предыдущего элемента
            if let Some((prev_x, prev_bottom_y, needs_arrow)) = prev_element_info {
                if needs_arrow {
                    if (prev_x - center_x).abs() > 1.0 {
                        // Стрелка между разными swimlanes (с изломом)
                        self.add_cross_swimlane_arrow(
                            prev_x,
                            prev_bottom_y,
                            center_x,
                            current_y,
                            &mut elements,
                        );
                    } else {
                        // Стрелка в том же swimlane (вертикальная)
                        self.add_arrow(
                            prev_x,
                            prev_bottom_y,
                            center_x,
                            current_y,
                            None,
                            &mut elements,
                        );
                    }
                }
            }

            if let Some(spec) = pending_arrow.take() {
                self.apply_arrow_spec(&spec, &mut elements);
            }

            let old_y = current_y;

            // Определяем высоту элемента и нужна ли стрелка после него
            let (element_height, needs_arrow_after) = match element {
                ActivityElement::Start => {
                    let r = self.config.node_radius;
                    elements.push(LayoutElement {
                        id: format!("start_{}", elements.len()),
                        bounds: Rect::new(center_x - r, current_y, r * 2.0, r * 2.0),
                        text: None,
                        properties: std::collections::HashMap::new(),
                        // PlantUML заливает начальный узел чёрным: пустой
                        // круг в эталоне `activity_basic` — ошибка.
                        element_type: ElementType::InitialState,
                    });
                    (r * 2.0, true)
                }
                ActivityElement::Connector(name) => {
                    // Коннектор PlantUML рисует кружком с подписью внутри.
                    // Раньше элемент пропускался целиком (и в фильтре, и в
                    // заглушке `TODO`) — на диаграмме его не было вовсе.
                    let width =
                        self.config.text.width(name, self.config.font_size) + ACTION_TEXT_PADDING;
                    let radius = (width / 2.0).max(CONNECTOR_MIN_RADIUS);

                    elements.push(LayoutElement {
                        id: format!("connector_{}", elements.len()),
                        bounds: Rect::new(center_x - radius, current_y, radius * 2.0, radius * 2.0),
                        text: None,
                        properties: std::collections::HashMap::new(),
                        element_type: ElementType::Ellipse {
                            label: Some(name.clone()),
                        },
                    });

                    (radius * 2.0, true)
                }
                ActivityElement::Stop => {
                    // Конечный узел на 1px больше начального: в эталоне
                    // activity_basic начальный круг имеет rx=10, конечный —
                    // rx=11. PlantUML рисует его «яблочком»: внешнее кольцо
                    // без заливки плюс внутренний залитый круг.
                    let r = self.config.node_radius + END_RADIUS_EXTRA;
                    elements.push(LayoutElement {
                        id: format!("stop_{}", elements.len()),
                        bounds: Rect::new(center_x - r, current_y, r * 2.0, r * 2.0),
                        text: None,
                        properties: std::collections::HashMap::new(),
                        element_type: ElementType::FinalState,
                    });
                    (r * 2.0, false) // Stop не требует стрелки после
                }
                ActivityElement::End => {
                    // Конечный узел на 1px больше начального: в эталоне
                    // activity_basic начальный круг имеет rx=10, конечный —
                    // rx=11. PlantUML рисует его «яблочком»: внешнее кольцо
                    // без заливки плюс внутренний залитый круг.
                    let r = self.config.node_radius + END_RADIUS_EXTRA;
                    elements.push(LayoutElement {
                        id: format!("end_{}", elements.len()),
                        bounds: Rect::new(center_x - r, current_y, r * 2.0, r * 2.0),
                        text: None,
                        properties: std::collections::HashMap::new(),
                        element_type: ElementType::FinalState,
                    });
                    (r * 2.0, false)
                }
                ActivityElement::Action(action) => {
                    let h = self.push_action(action, center_x, current_y, &mut elements);
                    (h, true)
                }
                // Для сложных элементов используем старые методы
                ActivityElement::Condition(cond) => {
                    let new_y = self.layout_condition(cond, center_x, current_y, &mut elements);
                    (new_y - current_y - self.config.vertical_spacing, true)
                }
                ActivityElement::While(while_loop) => {
                    let new_y = self.layout_while(while_loop, center_x, current_y, &mut elements);
                    (new_y - current_y - self.config.vertical_spacing, true)
                }
                ActivityElement::Repeat(repeat_loop) => {
                    let new_y = self.layout_repeat(repeat_loop, center_x, current_y, &mut elements);
                    (new_y - current_y - self.config.vertical_spacing, true)
                }
                ActivityElement::Fork(fork) => {
                    let new_y = self.layout_fork(fork, center_x, current_y, &mut elements);
                    (new_y - current_y - self.config.vertical_spacing, true)
                }
                ActivityElement::Partition(partition) => {
                    let new_y =
                        self.layout_partition(partition, center_x, current_y, &mut elements);
                    (new_y - current_y - self.config.vertical_spacing, true)
                }
                ActivityElement::Switch(switch) => {
                    let new_y = self.layout_switch(switch, center_x, current_y, &mut elements);
                    (new_y - current_y - self.config.vertical_spacing, true)
                }
                ActivityElement::Split(split) => {
                    let new_y = self.layout_split(split, center_x, current_y, &mut elements);
                    (new_y - current_y - self.config.vertical_spacing, true)
                }
                _ => (0.0, false),
            };

            current_y = old_y + element_height + self.config.vertical_spacing;
            prev_element_info = Some((center_x, old_y + element_height, needs_arrow_after));
        }

        // Добавляем swimlane backgrounds и headers В НАЧАЛО (для правильного Z-порядка)
        if has_swimlanes {
            let mut swimlane_elements = Vec::new();
            self.add_swimlane_elements(
                &swimlanes,
                content_start_y,
                current_y,
                &mut swimlane_elements,
            );
            // Swimlanes должны быть первыми, чтобы рендериться позади остальных элементов
            swimlane_elements.extend(elements);
            elements = swimlane_elements;
        }

        // Приводим контент к началу координат.
        //
        // Движок оставлял отрицательные координаты (крайняя левая ветвь
        // уходила в x = −23.75), из-за чего рамка диаграммы смещалась
        // влево и вверх, а размер не совпадал с эталоном. Эталон
        // начинает контент в (0, 0), а поля добавляет рендерер.
        let min_x = elements
            .iter()
            .map(|element| element.bounds.x)
            .fold(f64::MAX, f64::min);
        let min_y = elements
            .iter()
            .map(|element| element.bounds.y)
            .fold(f64::MAX, f64::min);

        // Смещение — не в ноль, а на величину внутреннего поля: эталон
        // начинает контент в (16, 15) при `bounds.x = 0`, поэтому в
        // выводе начало координат остаётся нулевым.
        let offset_x = ACTIVITY_CONTENT_INSET_X - min_x;
        let offset_y = ACTIVITY_CONTENT_INSET_Y - min_y;

        if min_x.is_finite() && min_y.is_finite() && (offset_x != 0.0 || offset_y != 0.0) {
            for element in &mut elements {
                element.bounds.x += offset_x;
                element.bounds.y += offset_y;

                // У стрелок координаты лежат ещё и в точках линии.
                if let ElementType::Edge { points, .. } = &mut element.element_type {
                    for point in points.iter_mut() {
                        point.x += offset_x;
                        point.y += offset_y;
                    }
                }
            }
        }

        // Вычисляем bounds
        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        // Добавляем отступы.
        //
        // Начало координат остаётся НУЛЕВЫМ: внутреннее поле уже учтено
        // смещением контента выше. Иначе рендерер сдвинул бы viewBox на
        // величину поля, и вывод отличался бы от эталонного.
        result.bounds.x = 0.0;
        result.bounds.y = 0.0;
        result.bounds.width += self.config.margin;
        result.bounds.height += self.config.margin;

        result
    }

    /// Собирает информацию о всех swimlanes в диаграмме
    fn collect_swimlanes(&self, elements: &[ActivityElement]) -> HashMap<String, SwimlaneInfo> {
        let mut swimlanes = HashMap::new();
        let mut index = 0;

        for element in elements {
            if let ActivityElement::SwimlaneChange(swimlane) = element {
                if !swimlanes.contains_key(&swimlane.name) {
                    let center_x = self.config.margin
                        + self.config.swimlane_width / 2.0
                        + index as f64
                            * (self.config.swimlane_width + self.config.swimlane_spacing);

                    swimlanes.insert(
                        swimlane.name.clone(),
                        SwimlaneInfo {
                            name: swimlane.name.clone(),
                            color: swimlane.color.clone(),
                            index,
                            center_x,
                        },
                    );
                    index += 1;
                }
            }
        }

        swimlanes
    }

    /// Добавляет элементы swimlanes (заголовки и фоны)
    fn add_swimlane_elements(
        &self,
        swimlanes: &HashMap<String, SwimlaneInfo>,
        _content_start_y: f64,
        content_end_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) {
        let total_height = content_end_y - self.config.margin + self.config.margin;

        for info in swimlanes.values() {
            let x = info.center_x - self.config.swimlane_width / 2.0;

            // Фон swimlane (вертикальная полоса)
            let mut props = std::collections::HashMap::new();
            if let Some(ref color) = info.color {
                props.insert("fill".to_string(), color.to_css());
            } else {
                // Чередующиеся цвета для swimlanes без явного цвета
                let bg_color = if info.index % 2 == 0 {
                    "#FEFECE".to_string()
                } else {
                    "#E2E2F0".to_string()
                };
                props.insert("fill".to_string(), bg_color);
            }
            props.insert("opacity".to_string(), "0.3".to_string());

            elements.push(LayoutElement {
                id: format!("swimlane_bg_{}", info.name),
                bounds: Rect::new(
                    x,
                    self.config.margin,
                    self.config.swimlane_width,
                    total_height,
                ),
                text: None,
                properties: props,
                element_type: ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 0.0,
                },
            });

            // Заголовок swimlane
            let mut header_props = std::collections::HashMap::new();
            if let Some(ref color) = info.color {
                header_props.insert("fill".to_string(), color.to_css());
            }

            elements.push(LayoutElement {
                id: format!("swimlane_header_{}", info.name),
                bounds: Rect::new(
                    x,
                    self.config.margin,
                    self.config.swimlane_width,
                    self.config.swimlane_header_height,
                ),
                text: None,
                properties: header_props,
                element_type: ElementType::Rectangle {
                    label: info.name.clone(),
                    corner_radius: 0.0,
                },
            });

            // Вертикальная разделительная линия справа от swimlane
            let line_x = x + self.config.swimlane_width;
            elements.push(LayoutElement {
                id: format!("swimlane_divider_{}", info.name),
                bounds: Rect::new(line_x, self.config.margin, 1.0, total_height),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Edge {
                    points: vec![
                        Point::new(line_x, self.config.margin),
                        Point::new(line_x, self.config.margin + total_height),
                    ],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: true,
                    edge_type: EdgeType::Association,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });
        }
    }

    /// Располагает элемент и возвращает новую Y позицию (со стрелками между элементами)
    fn layout_element(
        &self,
        element: &ActivityElement,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        match element {
            ActivityElement::Start => self.layout_start(center_x, current_y, elements),
            ActivityElement::Stop => self.layout_stop(center_x, current_y, elements),
            ActivityElement::End => self.layout_end(center_x, current_y, elements),
            ActivityElement::Action(action) => {
                self.layout_action(action, center_x, current_y, elements)
            }
            ActivityElement::Condition(cond) => {
                self.layout_condition(cond, center_x, current_y, elements)
            }
            ActivityElement::While(while_loop) => {
                self.layout_while(while_loop, center_x, current_y, elements)
            }
            ActivityElement::Repeat(repeat_loop) => {
                self.layout_repeat(repeat_loop, center_x, current_y, elements)
            }
            ActivityElement::Fork(fork) => self.layout_fork(fork, center_x, current_y, elements),
            ActivityElement::Partition(partition) => {
                self.layout_partition(partition, center_x, current_y, elements)
            }
            ActivityElement::Switch(switch) => {
                self.layout_switch(switch, center_x, current_y, elements)
            }
            ActivityElement::Split(split) => {
                self.layout_split(split, center_x, current_y, elements)
            }
            ActivityElement::Detach | ActivityElement::Kill => {
                // Detach/Kill просто прерывают поток, не рисуем ничего
                current_y
            }
            ActivityElement::Note(note) => {
                self.push_note(note, center_x, current_y, elements);
                current_y
            }
            ActivityElement::Arrow(spec) => {
                self.apply_arrow_spec(spec, elements);
                current_y
            }
            ActivityElement::SwimlaneChange(_) => {
                // Обрабатывается в основном цикле layout()
                current_y
            }
            ActivityElement::Connector(name) => {
                let width =
                    self.config.text.width(name, self.config.font_size) + ACTION_TEXT_PADDING;
                let radius = (width / 2.0).max(CONNECTOR_MIN_RADIUS);

                elements.push(LayoutElement {
                    id: format!("connector_{}", elements.len()),
                    bounds: Rect::new(center_x - radius, current_y, radius * 2.0, radius * 2.0),
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Ellipse {
                        label: Some(name.clone()),
                    },
                });

                current_y + radius * 2.0 + self.config.vertical_spacing
            }
        }
    }

    /// Добавляет стрелку между swimlanes (с изломом)
    fn add_cross_swimlane_arrow(
        &self,
        from_x: f64,
        from_y: f64,
        to_x: f64,
        to_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) {
        // Рисуем стрелку с изломом: вниз, затем горизонтально, затем вниз
        let mid_y = from_y + self.config.vertical_spacing / 2.0;

        let min_x = from_x.min(to_x);
        let max_x = from_x.max(to_x);

        elements.push(LayoutElement {
            id: format!("cross_arrow_{}", elements.len()),
            bounds: Rect::new(min_x, from_y, max_x - min_x, to_y - from_y),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points: vec![
                    Point::new(from_x, from_y),
                    Point::new(from_x, mid_y),
                    Point::new(to_x, mid_y),
                    Point::new(to_x, to_y),
                ],
                label: None,
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association,
                from_cardinality: None,
                to_cardinality: None,
            },
        });
    }

    /// Располагает начальный узел (filled circle)
    fn layout_start(
        &self,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let r = self.config.node_radius;

        elements.push(LayoutElement {
            id: format!("start_{}", elements.len()),
            bounds: Rect::new(center_x - r, current_y, r * 2.0, r * 2.0),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::InitialState,
        });

        let next_y = current_y + r * 2.0 + self.config.vertical_spacing;

        // Стрелка вниз от start
        self.add_arrow(
            center_x,
            current_y + r * 2.0,
            center_x,
            next_y,
            None,
            elements,
        );

        next_y
    }

    /// Располагает конечный узел (filled circle with ring)
    fn layout_stop(&self, center_x: f64, current_y: f64, elements: &mut Vec<LayoutElement>) -> f64 {
        // Конечный узел на 1px больше начального — см. `ActivityElement::Stop`.
        let r = self.config.node_radius + END_RADIUS_EXTRA;

        elements.push(LayoutElement {
            id: format!("stop_{}", elements.len()),
            bounds: Rect::new(center_x - r, current_y, r * 2.0, r * 2.0),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::FinalState,
        });

        current_y + r * 2.0 + self.config.vertical_spacing
    }

    /// Располагает конечный узел (альтернативный)
    fn layout_end(&self, center_x: f64, current_y: f64, elements: &mut Vec<LayoutElement>) -> f64 {
        self.layout_stop(center_x, current_y, elements)
    }

    /// Свойства линии `fork`/`split`: PlantUML рисует её тёмной, без скругления.
    fn bar_properties() -> HashMap<String, String> {
        let mut properties = HashMap::new();
        properties.insert("fill".to_string(), "#181818".to_string());
        properties.insert("rx".to_string(), "0".to_string());
        properties
    }

    /// Центры столбцов ветвей вокруг `center_x`.
    ///
    /// Соседние столбцы разделяет зазор `COLUMN_GAP`. Блок центрируется по
    /// габариту ВСЕХ столбцов, а не по их центрам: так сделан эталон `elseif`
    /// (старт над серединой всей схемы, а не над первым ромбом).
    fn column_centers(widths: &[f64], center_x: f64) -> Vec<f64> {
        let mut centers = Vec::with_capacity(widths.len());
        let mut x = 0.0;
        for (i, width) in widths.iter().enumerate() {
            if i > 0 {
                x += widths[i - 1] / 2.0 + COLUMN_GAP + width / 2.0;
            }
            centers.push(x);
        }
        let (Some(first), Some(last)) = (widths.first(), widths.last()) else {
            return centers;
        };
        let left = -first / 2.0;
        let right = x + last / 2.0;
        let shift = center_x - (left + right) / 2.0;
        centers.iter().map(|c| c + shift).collect()
    }

    /// Ширины столбцов цепочки `if / elseif / else`: ромб или тело ветви,
    /// что шире; последний столбец — `else` (нулевой, если ветви нет).
    fn chain_widths(&self, cond: &Condition) -> Vec<f64> {
        let mut widths = vec![self
            .condition_shape(&cond.condition)
            .0
            .max(self.branch_natural_width(&cond.then_branch))];
        for branch in &cond.elseif_branches {
            widths.push(
                self.condition_shape(&branch.condition)
                    .0
                    .max(self.branch_natural_width(&branch.elements)),
            );
        }
        widths.push(
            cond.else_branch
                .as_ref()
                .map_or(0.0, |body| self.branch_natural_width(body)),
        );
        widths
    }

    /// Полная ширина ряда столбцов.
    fn columns_total_width(widths: &[f64]) -> f64 {
        widths.iter().sum::<f64>() + COLUMN_GAP * widths.len().saturating_sub(1) as f64
    }

    /// Ширины столбцов `fork`/`split`.
    fn fork_widths(&self, branches: &[&[ActivityElement]]) -> Vec<f64> {
        branches
            .iter()
            .map(|body| self.branch_natural_width(body))
            .collect()
    }

    /// Убирает прямую стрелку, подведённую к точке `(x, y)`, и возвращает
    /// её начало. Нужна ромбу, стоящему не на оси потока: вместо прямой
    /// стрелки рисуется излом.
    fn take_entry_arrow(elements: &mut Vec<LayoutElement>, x: f64, y: f64) -> Option<f64> {
        let ElementType::Edge { points, .. } = &elements.last()?.element_type else {
            return None;
        };
        if !elements.last()?.id.starts_with("arrow_") || points.len() != 2 {
            return None;
        }
        let (start, end) = (points[0], points[1]);
        if (end.x - x).abs() > 0.5 || (end.y - y).abs() > 0.5 || (start.x - x).abs() > 0.5 {
            return None;
        }
        elements.pop();
        Some(start.y)
    }

    /// Линия без стрелки.
    fn add_line(&self, x1: f64, y1: f64, x2: f64, y2: f64, elements: &mut Vec<LayoutElement>) {
        elements.push(LayoutElement {
            id: format!("line_{}", elements.len()),
            bounds: Rect::new(
                x1.min(x2),
                y1.min(y2),
                (x2 - x1).abs().max(1.0),
                (y2 - y1).abs().max(1.0),
            ),
            text: None,
            properties: HashMap::new(),
            element_type: ElementType::Edge {
                points: vec![Point::new(x1, y1), Point::new(x2, y2)],
                label: None,
                arrow_start: false,
                arrow_end: false,
                dashed: false,
                edge_type: EdgeType::Association,
                from_cardinality: None,
                to_cardinality: None,
            },
        });
    }

    /// Подпись у стрелки: базис на `baseline`, начало текста в `x`.
    fn add_text_at(&self, text: &str, x: f64, baseline: f64, elements: &mut Vec<LayoutElement>) {
        let width = self.config.text.width(text, ACTIVITY_LABEL_FONT_SIZE);
        elements.push(LayoutElement {
            id: format!("branch_label_{}", elements.len()),
            bounds: Rect::new(
                x,
                baseline - ACTIVITY_LABEL_FONT_SIZE,
                width,
                ACTIVITY_LABEL_FONT_SIZE,
            ),
            text: None,
            properties: HashMap::new(),
            element_type: ElementType::Text {
                text: text.to_string(),
                font_size: ACTIVITY_LABEL_FONT_SIZE,
            },
        });
    }

    /// Ромб слияния ветвей 24x24 с верхней гранью в `top`.
    fn push_merge_diamond(&self, center_x: f64, top: f64, elements: &mut Vec<LayoutElement>) {
        let half = MERGE_DIAMOND_SIZE / 2.0;
        elements.push(LayoutElement {
            id: format!("merge_diamond_{}", elements.len()),
            bounds: Rect::new(center_x - half, top, MERGE_DIAMOND_SIZE, MERGE_DIAMOND_SIZE),
            text: None,
            properties: Self::shape_properties(ACTIVITY_LABEL_FONT_SIZE),
            element_type: ElementType::Polygon {
                points: vec![
                    Point::new(half, 0.0),
                    Point::new(MERGE_DIAMOND_SIZE, half),
                    Point::new(half, MERGE_DIAMOND_SIZE),
                    Point::new(0.0, half),
                ],
                label: None,
                font_size: ACTIVITY_LABEL_FONT_SIZE,
            },
        });
    }

    /// Применяет подпись и оформление `-> текст;` к последней стрелке потока.
    fn apply_arrow_spec(&self, spec: &ArrowSpec, elements: &mut Vec<LayoutElement>) {
        let Some(last) = elements.last_mut() else {
            return;
        };
        if !(last.id.starts_with("arrow_") || last.id.starts_with("elbow_")) {
            return;
        }
        let mut anchor = None;
        if let ElementType::Edge { points, dashed, .. } = &mut last.element_type {
            if spec.dashed {
                *dashed = true;
            }
            if let (Some(first), Some(end)) = (points.first(), points.last()) {
                anchor = Some((first.x, (first.y + end.y) / 2.0));
            }
        }
        if let Some(color) = &spec.color {
            last.properties.insert("stroke".to_string(), color.to_css());
        }
        // Подпись отдельным текстом: габариты схемы учитывают её ширину.
        if let (Some(label), Some((x, y))) = (&spec.label, anchor) {
            self.add_text_at(label, x + 4.0, y + 4.0, elements);
        }
    }

    /// Рамка предупреждения об устаревшем синтаксисе над схемой.
    fn push_warning(&self, text: &str, x: f64, y: f64, elements: &mut Vec<LayoutElement>) {
        let width = self.config.text.width(text, WARNING_FONT_SIZE) + 10.0;
        let mut properties = HashMap::new();
        properties.insert("fill".to_string(), "#FFFFCC".to_string());
        properties.insert("rx".to_string(), "2.5".to_string());
        properties.insert("font-size".to_string(), WARNING_FONT_SIZE.to_string());
        elements.push(LayoutElement {
            id: format!("note_warning_{}", elements.len()),
            bounds: Rect::new(x, y, width, WARNING_BOX_HEIGHT),
            text: None,
            properties,
            element_type: ElementType::Rectangle {
                label: text.to_string(),
                corner_radius: 2.5,
            },
        });
    }

    /// Заметка рядом с предыдущим действием: справа или слева, на его высоте.
    ///
    /// Заметка не входит в поток, поэтому не сдвигает курсор и не рвёт
    /// стрелки. Раньше она занимала место действия и обрывала поток.
    fn push_note(
        &self,
        note: &plantuml_ast::common::Note,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) {
        let lines: Vec<&str> = note.text.lines().collect();
        let text_width = lines
            .iter()
            .map(|line| self.config.text.width(line, self.config.font_size))
            .fold(0.0, f64::max);
        let width = text_width + ACTION_TEXT_PADDING;
        let height = (lines.len().max(1) as f64 * 15.0 + 10.0).max(self.config.action_height);

        // Якорь: последнее действие потока; если его нет — сама ось потока.
        let anchor = elements
            .iter()
            .rev()
            .find(|e| e.id.starts_with("action_"))
            .map(|e| e.bounds);
        let (left, right, top) = match anchor {
            Some(rect) => (rect.x, rect.x + rect.width, rect.y),
            None => (
                center_x - self.config.action_width / 2.0,
                center_x + self.config.action_width / 2.0,
                current_y,
            ),
        };
        let x = if note.position == plantuml_ast::common::NotePosition::Left {
            left - ACTIVITY_NOTE_GAP - width
        } else {
            right + ACTIVITY_NOTE_GAP
        };

        let mut properties = HashMap::new();
        properties.insert("fill".to_string(), ACTIVITY_NOTE_BACKGROUND.to_string());
        elements.push(LayoutElement {
            id: format!("note_{}", elements.len()),
            bounds: Rect::new(x, top, width, height),
            text: None,
            properties,
            element_type: ElementType::Rectangle {
                label: note.text.clone(),
                corner_radius: 0.0,
            },
        });
    }

    /// Фигура действия с учётом стереотипа и цвета.
    fn push_action(
        &self,
        action: &Action,
        center_x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let text_width = self.config.text.width(&action.label, self.config.font_size);
        let h = self.config.action_height;
        let mut properties = Self::shape_properties(ACTIVITY_ACTION_FONT_SIZE);
        if let Some(color) = &action.background_color {
            properties.insert("fill".to_string(), color.to_css());
        }

        let signal = matches!(action.style, ActionStyle::Input | ActionStyle::Output);
        let w = text_width + ACTION_TEXT_PADDING + if signal { SIGNAL_NOTCH } else { 0.0 };
        let bounds = Rect::new(center_x - w / 2.0, y, w, h);

        let element_type = match action.style {
            ActionStyle::Input | ActionStyle::Output => {
                let tip = if action.style == ActionStyle::Input {
                    // вырез справа
                    Point::new(w - SIGNAL_NOTCH, h / 2.0)
                } else {
                    // острие справа
                    Point::new(w, h / 2.0)
                };
                let edge_x = if action.style == ActionStyle::Input {
                    w
                } else {
                    w - SIGNAL_NOTCH
                };
                ElementType::Polygon {
                    points: vec![
                        Point::new(0.0, 0.0),
                        Point::new(edge_x, 0.0),
                        tip,
                        Point::new(edge_x, h),
                        Point::new(0.0, h),
                    ],
                    label: Some(action.label.clone()),
                    font_size: ACTIVITY_ACTION_FONT_SIZE,
                }
            }
            ActionStyle::Procedure | ActionStyle::Task => {
                properties.insert("rx".to_string(), "0".to_string());
                ElementType::Rectangle {
                    label: action.label.clone(),
                    corner_radius: 0.0,
                }
            }
            _ => ElementType::Rectangle {
                label: action.label.clone(),
                corner_radius: self.config.action_corner_radius,
            },
        };

        let id_prefix = if matches!(element_type, ElementType::Polygon { .. }) {
            "action_poly"
        } else {
            "action"
        };
        elements.push(LayoutElement {
            id: format!("{id_prefix}_{}", elements.len()),
            bounds,
            text: None,
            properties,
            element_type,
        });
        h
    }

    /// Цепочка `if / elseif / else`: ромбы в ряд, у каждого своя ветвь.
    ///
    /// Так рисует PlantUML (эталон `feat_activity_elseif`): ромб следующего
    /// условия стоит справа от предыдущего, подпись «да» — под ромбом, ветви
    /// сливаются на общей линии, откуда один выход вниз.
    fn layout_condition_chain(
        &self,
        cond: &Condition,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let dh = self.config.diamond_height;
        let mid_y = current_y + dh / 2.0;
        let bottom_y = current_y + dh;
        let branch_y = bottom_y + BRANCH_DROP;

        let mut tests: Vec<(&str, Option<&str>, &[ActivityElement])> = vec![(
            cond.condition.as_str(),
            cond.then_label.as_deref(),
            cond.then_branch.as_slice(),
        )];
        for branch in &cond.elseif_branches {
            tests.push((
                branch.condition.as_str(),
                branch.label.as_deref(),
                branch.elements.as_slice(),
            ));
        }
        let widths = self.chain_widths(cond);
        let mut centers = Self::column_centers(&widths, center_x);
        let columns = tests.len();

        // Вход: излом от оси потока к первому ромбу. Если прямую стрелку
        // забрать нельзя, первый ромб остаётся на оси.
        let entry = Self::take_entry_arrow(elements, center_x, current_y);
        match entry {
            Some(start_y) => {
                self.add_elbow_arrow(
                    vec![
                        Point::new(center_x, start_y),
                        Point::new(center_x, start_y + 5.0),
                        Point::new(centers[0], start_y + 5.0),
                        Point::new(centers[0], current_y),
                    ],
                    elements,
                );
            }
            None => {
                let shift = center_x - centers[0];
                for c in &mut centers {
                    *c += shift;
                }
            }
        }

        let mut bottoms: Vec<f64> = Vec::with_capacity(columns + 1);
        let mut diamond_widths: Vec<f64> = Vec::with_capacity(columns);
        for (k, (text, label, body)) in tests.iter().enumerate() {
            let cx = centers[k];
            let dw = self.add_condition_shape(text, cx, current_y, elements);
            diamond_widths.push(dw);

            if let Some(label) = label {
                self.add_text_at(label, cx + 4.0, bottom_y + 10.2, elements);
            }
            if body.is_empty() {
                bottoms.push(bottom_y);
            } else {
                self.add_arrow(cx, bottom_y, cx, branch_y, None, elements);
                let end = self.layout_branch_body(body, cx, branch_y, elements);
                bottoms.push(end - self.config.vertical_spacing);
            }
        }

        // Стрелки между соседними ромбами.
        for k in 0..columns.saturating_sub(1) {
            self.add_arrow(
                centers[k] + diamond_widths[k] / 2.0,
                mid_y,
                centers[k + 1] - diamond_widths[k + 1] / 2.0,
                mid_y,
                None,
                elements,
            );
        }

        // Ветвь `else`: от правой вершины последнего ромба.
        let last_right = centers[columns - 1] + diamond_widths[columns - 1] / 2.0;
        let else_x = centers[columns];
        let else_body = cond.else_branch.as_deref().filter(|body| !body.is_empty());
        if let Some(label) = &cond.else_label {
            self.add_branch_label(label, last_right, mid_y, false, elements);
        }
        let else_bottom = if let Some(body) = else_body {
            self.add_elbow_arrow(
                vec![
                    Point::new(last_right, mid_y),
                    Point::new(else_x, mid_y),
                    Point::new(else_x, branch_y),
                ],
                elements,
            );
            let end = self.layout_branch_body(body, else_x, branch_y, elements);
            end - self.config.vertical_spacing
        } else {
            mid_y
        };
        bottoms.push(else_bottom);

        // Слияние: общая линия, куда ведут все ветви, и один выход вниз.
        let merge_y = bottoms.iter().copied().fold(f64::MIN, f64::max) + MERGE_BAR_DROP;
        for (k, bottom) in bottoms.iter().take(columns).enumerate() {
            self.add_arrow(centers[k], *bottom, centers[k], merge_y, None, elements);
        }
        if else_body.is_some() {
            self.add_arrow(else_x, else_bottom, else_x, merge_y, None, elements);
        } else {
            // `else` нет: путь «нет» уходит вправо и вниз прямо к линии.
            self.add_elbow_arrow(
                vec![
                    Point::new(last_right, mid_y),
                    Point::new(else_x, mid_y),
                    Point::new(else_x, merge_y),
                ],
                elements,
            );
        }
        let line_left = centers[0].min(center_x);
        let line_right = else_x.max(center_x);
        self.add_line(line_left, merge_y, line_right, merge_y, elements);

        merge_y + self.config.vertical_spacing
    }

    /// Располагает действие (rounded rectangle)
    fn layout_action(
        &self,
        action: &Action,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let h = self.push_action(action, center_x, current_y, elements);

        let next_y = current_y + h + self.config.vertical_spacing;

        // Стрелка вниз
        self.add_arrow(center_x, current_y + h, center_x, next_y, None, elements);

        next_y
    }

    /// Располагает условие (if/else)
    fn layout_condition(
        &self,
        cond: &Condition,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        if !cond.elseif_branches.is_empty() {
            return self.layout_condition_chain(cond, center_x, current_y, elements);
        }
        let dh = self.config.diamond_height;

        // Шестиугольник условия: ширина зависит от подписи.
        let dw = self.add_condition_shape(&cond.condition, center_x, current_y, elements);

        let after_diamond = current_y + dh + self.config.vertical_spacing / 2.0;
        let branch_start_y = after_diamond;
        let mid_y = current_y + dh / 2.0;

        // Ширина самой широкой ветви определяет отступ всех ветвей:
        // без её измерения раньше сдвиг брался константой и не зависел от
        // содержимого, из-за чего широкие диаграммы уезжали влево.
        let mut widest = self.branch_natural_width(&cond.then_branch);
        for branch in &cond.elseif_branches {
            widest = widest.max(self.branch_natural_width(&branch.elements));
        }
        if let Some(else_branch) = &cond.else_branch {
            widest = widest.max(self.branch_natural_width(else_branch));
        }
        // Отступ берётся от СРЕДНЕЙ ширины ветвей, а не от максимальной.
        //
        // Формула выведена на сервере и подтверждена на трёх замерах:
        // при одинаковой ширине везде это w/2 + 10; при разной —
        // (w1 + w2)/4 + 10. Контрольный случай: ширины ветвей 137.00 и
        // 203.36 дали измеренный отступ 95.09, а формула даёт 95.09.
        // Вариант «максимум» на тех же данных дал бы 111.68.
        let mut sum = self.branch_natural_width(&cond.then_branch);
        let mut count = 1.0;
        for branch in &cond.elseif_branches {
            sum += self.branch_natural_width(&branch.elements);
            count += 1.0;
        }
        if let Some(else_branch) = &cond.else_branch {
            sum += self.branch_natural_width(else_branch);
            count += 1.0;
        }
        // Плюс 6.00 за каждую петлю в ветвях: см. `branch_loop_count`.
        let mut loops = self.branch_loop_count(&cond.then_branch);
        for branch in &cond.elseif_branches {
            loops += self.branch_loop_count(&branch.elements);
        }
        if let Some(else_branch) = &cond.else_branch {
            loops += self.branch_loop_count(else_branch);
        }

        let offset = self.branch_offset(sum / count) + ACTIVITY_LOOP_OFFSET * loops as f64;
        let _ = widest;

        // Ширины ветвей elseif — по порядку: каждая следующая ветвь
        // отстоит от предыдущей на ширину предыдущей плюс 20.
        let branch_natural_widths: Vec<f64> = cond
            .elseif_branches
            .iter()
            .map(|branch| self.branch_natural_width(&branch.elements))
            .collect();

        // Then branch (left)
        let left_x = center_x - offset;
        let mut then_end_y = branch_start_y;

        // Стрелка от условия влево + вниз (излом, а не диагональ).
        self.add_elbow_arrow(
            vec![
                Point::new(center_x - dw / 2.0, mid_y),
                Point::new(left_x, mid_y),
                Point::new(left_x, branch_start_y),
            ],
            elements,
        );

        if let Some(label) = &cond.then_label {
            self.add_branch_label(label, center_x - dw / 2.0, mid_y, true, elements);
        }

        then_end_y = self.layout_branch_body(&cond.then_branch, left_x, then_end_y, elements);

        // Ветки elseif: раньше они полностью игнорировались — поле
        // `elseif_branches` не читалось нигде, поэтому `if / elseif / else`
        // терял промежуточные ветки, а поток на диаграмме становился
        // неверным. Размещаем их каскадом вправо-вниз между then и else.
        let mut elseif_end_y = branch_start_y;
        for (i, branch) in cond.elseif_branches.iter().enumerate() {
            // Ветви elseif каскадом вправо: каждая следующая отстоит от
            // предыдущей на свою ширину плюс 20 — так же, как в эталоне.
            let branch_x = center_x
                + offset
                + branch_natural_widths
                    .iter()
                    .take(i)
                    .map(|w| w + 20.0)
                    .sum::<f64>();

            // Стрелка от условия к ветке с её подписью
            self.add_elbow_arrow(
                vec![
                    Point::new(center_x + dw / 2.0, mid_y),
                    Point::new(branch_x, mid_y),
                    Point::new(branch_x, branch_start_y),
                ],
                elements,
            );
            self.add_branch_label(
                &branch.condition,
                center_x + dw / 2.0,
                mid_y,
                false,
                elements,
            );

            let y = self.layout_branch_body(&branch.elements, branch_x, branch_start_y, elements);
            elseif_end_y = elseif_end_y.max(y);
        }

        // Else branch (right) if exists
        let mut else_end_y = elseif_end_y;
        if let Some(else_branch) = &cond.else_branch {
            // Если есть ветки elseif, else уходит ещё правее
            let right_x =
                center_x + offset + branch_natural_widths.iter().map(|w| w + 20.0).sum::<f64>();

            // Стрелка от условия вправо + вниз
            self.add_elbow_arrow(
                vec![
                    Point::new(center_x + dw / 2.0, mid_y),
                    Point::new(right_x, mid_y),
                    Point::new(right_x, branch_start_y),
                ],
                elements,
            );
            if let Some(label) = &cond.else_label {
                self.add_branch_label(label, center_x + dw / 2.0, mid_y, false, elements);
            }

            else_end_y = self.layout_branch_body(else_branch, right_x, else_end_y, elements);
        }

        // Низ ветвей.
        //
        // `layout_element` возвращает уже продвинутый курсор — с запасом
        // `vertical_spacing` под следующую стрелку. Для точки слияния нужен
        // ФАКТИЧЕСКИЙ низ последнего элемента, иначе ветви «висят» на 20px
        // ниже своего содержимого и соединение уезжает вниз.
        let then_bottom = then_end_y - self.config.vertical_spacing;
        let else_bottom = else_end_y - self.config.vertical_spacing;

        // Точка слияния
        let merge_y = then_bottom.max(else_bottom);

        // Стрелки к точке слияния.
        //
        // PlantUML рисует их ВСЕГДА, даже когда ветви одинаковой длины:
        // в эталоне activity_basic обе кончаются на 176.94, и обе линии
        // доходят до ромба — вертикально вниз, затем горизонтально в его
        // боковую вершину.
        let merge_top = merge_y + MERGE_DIAMOND_GAP;
        let merge_center_y = merge_top + MERGE_DIAMOND_SIZE / 2.0;
        let merge_half = MERGE_DIAMOND_SIZE / 2.0;

        self.add_elbow_arrow(
            vec![
                Point::new(left_x, then_bottom),
                Point::new(left_x, merge_center_y),
                Point::new(center_x - merge_half, merge_center_y),
            ],
            elements,
        );

        let mut merge_branches: Vec<(f64, f64)> = Vec::new();
        if cond.else_branch.is_some() {
            let right_x = center_x
                + self.config.horizontal_spacing * (cond.elseif_branches.len() as f64 + 1.0);
            merge_branches.push((right_x, else_bottom));
        }
        for (i, _) in cond.elseif_branches.iter().enumerate() {
            // Ветки elseif уходят вниз в свою точку слияния отдельными
            // линиями: их курсор хранится в `elseif_end_y` как общий.
            let branch_x = center_x + self.config.horizontal_spacing * (i as f64 + 1.0);
            merge_branches.push((branch_x, elseif_end_y - self.config.vertical_spacing));
        }
        for (branch_x, bottom) in merge_branches {
            self.add_elbow_arrow(
                vec![
                    Point::new(branch_x, bottom),
                    Point::new(branch_x, merge_center_y),
                    Point::new(center_x + merge_half, merge_center_y),
                ],
                elements,
            );
        }

        // Ромб слияния.
        //
        // PlantUML ставит между ветвями и следующим действием ромб 24x24:
        // в эталоне activity_basic ветви кончаются на 176.94, ромб занимает
        // 182.94..206.94, следующее действие начинается на 226.94 —
        // то есть отступы 6 сверху и 20 снизу.
        elements.push(LayoutElement {
            id: format!("merge_diamond_{}", elements.len()),
            bounds: Rect::new(
                center_x - merge_half,
                merge_top,
                MERGE_DIAMOND_SIZE,
                MERGE_DIAMOND_SIZE,
            ),
            text: None,
            properties: Self::shape_properties(ACTIVITY_LABEL_FONT_SIZE),
            element_type: ElementType::Polygon {
                points: vec![
                    Point::new(merge_half, 0.0),
                    Point::new(MERGE_DIAMOND_SIZE, merge_half),
                    Point::new(merge_half, MERGE_DIAMOND_SIZE),
                    Point::new(0.0, merge_half),
                ],
                label: None,
                font_size: ACTIVITY_LABEL_FONT_SIZE,
            },
        });

        merge_top + MERGE_DIAMOND_SIZE + self.config.vertical_spacing
    }

    /// Располагает цикл while
    ///
    /// Шестиугольник условия сверху; подпись `is (..)` под ним, на входе в
    /// тело. Возврат идёт справа вверх к условию, выход — слева вниз и обратно
    /// на ось потока (эталон `real_activity_real`). Подпись `endwhile (..)`
    /// стоит у левой вершины.
    fn layout_while(
        &self,
        while_loop: &WhileLoop,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let dh = self.config.diamond_height;
        let dw = self.add_condition_shape(&while_loop.condition, center_x, current_y, elements);
        let mid_y = current_y + dh / 2.0;
        let hex_right = center_x + dw / 2.0;
        let hex_left = center_x - dw / 2.0;
        let bottom_y = current_y + dh;

        // Под условием место для подписи `is (..)`: в эталоне `activity_branch`
        // тело начинается на 31.7 ниже условия, а не на обычные 20.
        let body_start_y = bottom_y
            + if while_loop.end_label.is_some() {
                WHILE_LABELED_GAP
            } else {
                self.config.vertical_spacing
            };
        if let Some(label) = &while_loop.end_label {
            self.add_text_at(label, center_x + 4.0, bottom_y + 10.2, elements);
        }
        self.add_arrow(center_x, bottom_y, center_x, body_start_y, None, elements);
        let body_end_y =
            self.layout_branch_body(&while_loop.body, center_x, body_start_y, elements);
        let body_bottom = if while_loop.body.is_empty() {
            body_start_y
        } else {
            body_end_y - self.config.vertical_spacing
        };

        // Обход тела по бокам: ширина — по самой широкой части.
        let half_width = (dw / 2.0).max(self.branch_natural_width(&while_loop.body) / 2.0);
        let loop_x = center_x + half_width + 12.0;
        let exit_x = center_x - half_width - 12.0;
        let return_y = body_bottom + WHILE_RETURN_DROP;

        // Возврат: из-под тела вправо, вверх и в правую вершину условия.
        self.add_elbow_arrow(
            vec![
                Point::new(center_x, body_bottom),
                Point::new(center_x, return_y),
                Point::new(loop_x, return_y),
                Point::new(loop_x, mid_y),
                Point::new(hex_right, mid_y),
            ],
            elements,
        );

        // Выход: из левой вершины влево, вниз и обратно на ось потока.
        self.add_path(
            vec![
                Point::new(hex_left, mid_y),
                Point::new(exit_x, mid_y),
                Point::new(exit_x, return_y),
                Point::new(center_x, return_y),
            ],
            false,
            elements,
        );
        if let Some(label) = &while_loop.backward_label {
            self.add_branch_label(label, hex_left, mid_y, true, elements);
        }

        return_y + self.config.vertical_spacing
    }

    /// Располагает цикл repeat
    ///
    /// Сверху ромб входа в цикл, снизу шестиугольник условия; возврат идёт
    /// справа вверх к ромбу (эталон `ref_lang_053`). Подпись `is (..)` стоит у
    /// возврата, `not (..)` — у выхода вниз.
    fn layout_repeat(
        &self,
        repeat_loop: &RepeatLoop,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let half = MERGE_DIAMOND_SIZE / 2.0;
        let entry_center_y = current_y + half;

        // Ромб входа в цикл и стрелка от него к первому действию тела.
        self.push_merge_diamond(center_x, current_y, elements);
        let body_start_y = current_y + MERGE_DIAMOND_SIZE + self.config.vertical_spacing;
        self.add_arrow(
            center_x,
            current_y + MERGE_DIAMOND_SIZE,
            center_x,
            body_start_y,
            None,
            elements,
        );

        let mut body_end_y = body_start_y;
        for elem in &repeat_loop.body {
            body_end_y = self.layout_element(elem, center_x, body_end_y, elements);
        }

        // Шестиугольник условия внизу
        let dh = self.config.diamond_height;
        let dw = self.add_condition_shape(&repeat_loop.condition, center_x, body_end_y, elements);
        let mid_y = body_end_y + dh / 2.0;

        // Возврат правее самой широкой части тела.
        let right = (dw / 2.0).max(self.branch_natural_width(&repeat_loop.body) / 2.0);
        let loop_x = center_x + right + 12.0;
        let hex_right = center_x + dw / 2.0;

        match &repeat_loop.backward_label {
            Some(label) => {
                // `backward :текст;` — действие на пути возврата.
                let backward = Action::new(label.clone());
                let box_height = self.config.action_height;
                let box_top = (mid_y + entry_center_y) / 2.0 - box_height / 2.0;
                self.add_elbow_arrow(
                    vec![
                        Point::new(hex_right, mid_y),
                        Point::new(loop_x, mid_y),
                        Point::new(loop_x, box_top + box_height),
                    ],
                    elements,
                );
                self.push_action(&backward, loop_x, box_top, elements);
                self.add_elbow_arrow(
                    vec![
                        Point::new(loop_x, box_top),
                        Point::new(loop_x, entry_center_y),
                        Point::new(center_x + half, entry_center_y),
                    ],
                    elements,
                );
            }
            None => self.add_elbow_arrow(
                vec![
                    Point::new(hex_right, mid_y),
                    Point::new(loop_x, mid_y),
                    Point::new(loop_x, entry_center_y),
                    Point::new(center_x + half, entry_center_y),
                ],
                elements,
            ),
        }

        if let Some(label) = &repeat_loop.end_label {
            self.add_branch_label(label, hex_right, mid_y, false, elements);
        }
        if let Some(label) = &repeat_loop.not_label {
            self.add_text_at(label, center_x + 4.0, body_end_y + dh + 10.2, elements);
        }

        body_end_y + dh + self.config.vertical_spacing
    }

    /// Раскладывает раздел `partition`.
    ///
    /// PlantUML рисует раздел рамкой с названием в левом верхнем углу,
    /// внутри — обычный поток. Рамка пустая и прозрачная: поток сквозь неё
    /// идёт непрерывной стрелкой.
    fn layout_partition(
        &self,
        partition: &Partition,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let header_height = PARTITION_HEADER_HEIGHT;

        // Входная стрелка потока продлевается до первого элемента раздела.
        let content_y = current_y + header_height + PARTITION_PADDING;
        if let Some(start_y) = Self::take_entry_arrow(elements, center_x, current_y) {
            self.add_arrow(center_x, start_y, center_x, content_y, None, elements);
        }

        let mut inner: Vec<LayoutElement> = Vec::new();
        let end_y = self.layout_branch_body(&partition.elements, center_x, content_y, &mut inner);
        let content_bottom = if partition.elements.is_empty() {
            content_y
        } else {
            end_y - self.config.vertical_spacing
        };
        let frame_bottom = content_bottom + PARTITION_PADDING;

        // Ширина рамки — по самому широкому содержимому и названию.
        let name_width = self
            .config
            .text
            .width(&partition.name, self.config.font_size)
            + PARTITION_PADDING * 2.0;
        let mut half = name_width / 2.0;
        for element in &inner {
            half = half.max((element.bounds.x - center_x).abs() + PARTITION_PADDING);
            half = half.max(
                (element.bounds.x + element.bounds.width - center_x).abs() + PARTITION_PADDING,
            );
        }

        let mut properties = HashMap::new();
        properties.insert("fill".to_string(), "none".to_string());
        properties.insert("rx".to_string(), "0".to_string());
        if let Some(color) = &partition.color {
            properties.insert("stroke".to_string(), color.to_css());
        }

        // Рамка добавляется ПЕРВОЙ, чтобы её закрыли стрелки и элементы
        elements.push(LayoutElement {
            id: format!("partition_{}", elements.len()),
            bounds: Rect::new(
                center_x - half,
                current_y,
                half * 2.0,
                frame_bottom - current_y,
            ),
            text: None,
            properties,
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
        });
        self.add_text_at(
            &partition.name,
            center_x - half + PARTITION_PADDING,
            current_y + header_height - 6.0,
            elements,
        );

        elements.extend(inner);
        // Поток внутри рамки доходит до её нижней границы.
        if !partition.elements.is_empty() {
            self.add_line(center_x, content_bottom, center_x, frame_bottom, elements);
        }

        frame_bottom + self.config.vertical_spacing
    }

    /// Раскладывает множественный выбор `switch`.
    ///
    /// Шестиугольник с выражением, под ним столбцы `case`: крайние ветви
    /// выходят из боковых вершин, средняя — из низа; подпись `case` стоит у
    /// стрелки, ветви сходятся в ромбе слияния (эталон `ref_lang_052`).
    fn layout_switch(
        &self,
        switch: &Switch,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let dh = self.config.diamond_height;
        let expression = switch.expression.as_deref().unwrap_or("");
        let dw = self.add_condition_shape(expression, center_x, current_y, elements);
        let mid_y = current_y + dh / 2.0;
        let bottom_y = current_y + dh;
        let branch_y = bottom_y + BRANCH_DROP + 8.0;

        let count = switch.branches.len();
        if count == 0 {
            return bottom_y + self.config.vertical_spacing;
        }
        let widths: Vec<f64> = switch
            .branches
            .iter()
            .map(|branch| {
                let label_width = branch.label.as_deref().map_or(0.0, |label| {
                    self.config.text.width(label, ACTIVITY_LABEL_FONT_SIZE) + 8.0
                });
                self.branch_natural_width(&branch.elements).max(label_width)
            })
            .collect();
        let centers = Self::column_centers(&widths, center_x);

        let mut bottoms = Vec::with_capacity(count);
        for (k, branch) in switch.branches.iter().enumerate() {
            let cx = centers[k];
            let on_axis = (cx - center_x).abs() < 1.0;
            if count > 1 && k == 0 && !on_axis {
                self.add_elbow_arrow(
                    vec![
                        Point::new(center_x - dw / 2.0, mid_y),
                        Point::new(cx, mid_y),
                        Point::new(cx, branch_y),
                    ],
                    elements,
                );
            } else if count > 1 && k == count - 1 && !on_axis {
                self.add_elbow_arrow(
                    vec![
                        Point::new(center_x + dw / 2.0, mid_y),
                        Point::new(cx, mid_y),
                        Point::new(cx, branch_y),
                    ],
                    elements,
                );
            } else if on_axis {
                self.add_arrow(center_x, bottom_y, cx, branch_y, None, elements);
            } else {
                // Средняя ветвь не на оси: уходит вбок из-под шестиугольника.
                self.add_elbow_arrow(
                    vec![
                        Point::new(center_x, bottom_y),
                        Point::new(center_x, bottom_y + 8.0),
                        Point::new(cx, bottom_y + 8.0),
                        Point::new(cx, branch_y),
                    ],
                    elements,
                );
            }
            if let Some(label) = &branch.label {
                self.add_text_at(label, cx + 4.0, branch_y - 13.0, elements);
            }
            if branch.elements.is_empty() {
                bottoms.push(branch_y);
            } else {
                let end = self.layout_branch_body(&branch.elements, cx, branch_y, elements);
                bottoms.push(end - self.config.vertical_spacing);
            }
        }

        // Слияние в ромбе под ветвями.
        let merge_top = bottoms.iter().copied().fold(f64::MIN, f64::max) + MERGE_BAR_DROP;
        let half = MERGE_DIAMOND_SIZE / 2.0;
        let merge_center_y = merge_top + half;
        for (k, bottom) in bottoms.iter().enumerate() {
            let cx = centers[k];
            if (cx - center_x).abs() < 1.0 {
                self.add_arrow(cx, *bottom, center_x, merge_top, None, elements);
            } else {
                let side = if cx < center_x { -half } else { half };
                self.add_elbow_arrow(
                    vec![
                        Point::new(cx, *bottom),
                        Point::new(cx, merge_center_y),
                        Point::new(center_x + side, merge_center_y),
                    ],
                    elements,
                );
            }
        }
        self.push_merge_diamond(center_x, merge_top, elements);

        merge_top + MERGE_DIAMOND_SIZE + self.config.vertical_spacing
    }

    /// Раскладывает разделение потока `split`.
    ///
    /// На схеме это те же параллельные ветви между двумя линиями, что и у
    /// `fork`; подписей у веток нет.
    fn layout_split(
        &self,
        split: &Split,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let branches: Vec<&[ActivityElement]> = split
            .branches
            .iter()
            .map(|branch| branch.elements.as_slice())
            .collect();
        self.layout_parallel(&branches, center_x, current_y, elements)
    }

    /// Параллельные ветви между двумя линиями: общая часть `fork` и `split`.
    fn layout_parallel(
        &self,
        branches: &[&[ActivityElement]],
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let count = branches.len();
        if count == 0 {
            return current_y;
        }

        let widths = self.fork_widths(branches);
        let centers = Self::column_centers(&widths, center_x);
        let (bar_left, bar_right) = if count > 1 {
            (centers[0], centers[count - 1])
        } else {
            (
                center_x - self.config.bar_width / 2.0,
                center_x + self.config.bar_width / 2.0,
            )
        };
        let bar_width = bar_right - bar_left;

        elements.push(LayoutElement {
            id: format!("fork_bar_{}", elements.len()),
            bounds: Rect::new(bar_left, current_y, bar_width, self.config.bar_height),
            text: None,
            properties: Self::bar_properties(),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
        });

        let branches_start_y = current_y + self.config.bar_height + self.config.vertical_spacing;
        let mut ends: Vec<Option<f64>> = Vec::with_capacity(count);
        for (k, branch) in branches.iter().enumerate() {
            let cx = centers[k];
            self.add_arrow(
                cx,
                current_y + self.config.bar_height,
                cx,
                branches_start_y,
                None,
                elements,
            );
            let end = self.layout_branch_body(branch, cx, branches_start_y, elements);
            // Ветвь, оборванная `stop`/`end`/`detach`, в линию слияния не идёт.
            let terminated = matches!(
                branch.last(),
                Some(
                    ActivityElement::Stop
                        | ActivityElement::End
                        | ActivityElement::Detach
                        | ActivityElement::Kill
                )
            );
            ends.push((!terminated).then_some(end - self.config.vertical_spacing));
        }

        let join_y = ends
            .iter()
            .flatten()
            .copied()
            .fold(branches_start_y - self.config.vertical_spacing, f64::max)
            + self.config.vertical_spacing;
        elements.push(LayoutElement {
            id: format!("join_bar_{}", elements.len()),
            bounds: Rect::new(bar_left, join_y, bar_width, self.config.bar_height),
            text: None,
            properties: Self::bar_properties(),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
        });
        for (k, end) in ends.iter().enumerate() {
            if let Some(bottom) = end {
                self.add_arrow(centers[k], *bottom, centers[k], join_y, None, elements);
            }
        }

        join_y + self.config.bar_height + self.config.vertical_spacing
    }

    /// Располагает fork/join
    fn layout_fork(
        &self,
        fork: &Fork,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let branches: Vec<&[ActivityElement]> = fork.branches.iter().map(Vec::as_slice).collect();
        self.layout_parallel(&branches, center_x, current_y, elements)
    }

    /// Добавляет стрелку
    fn add_arrow(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        label: Option<String>,
        elements: &mut Vec<LayoutElement>,
    ) {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);

        elements.push(LayoutElement {
            id: format!("arrow_{}", elements.len()),
            bounds: Rect::new(
                min_x,
                min_y,
                (max_x - min_x).max(1.0),
                (max_y - min_y).max(1.0),
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points: vec![Point::new(x1, y1), Point::new(x2, y2)],
                label,
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association,
                from_cardinality: None,
                to_cardinality: None,
            },
        });
    }
}

impl Default for ActivityLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::activity::ElseIfBranch;

    /// Ветки `elseif` должны попадать в раскладку.
    ///
    /// Регрессия: поле `Condition::elseif_branches` не читалось нигде,
    /// поэтому конструкция `if / elseif / else` молча теряла промежуточные
    /// ветки — поток на диаграмме становился неверным.
    #[test]
    fn test_elseif_branches_are_laid_out() {
        let mut diagram = ActivityDiagram::new();
        diagram.elements.push(ActivityElement::Start);
        diagram.elements.push(ActivityElement::Condition(Condition {
            condition: "у1".to_string(),
            then_branch: vec![ActivityElement::Action(Action::new("шаг1"))],
            then_label: Some("да".to_string()),
            elseif_branches: vec![ElseIfBranch {
                condition: "у2".to_string(),
                elements: vec![ActivityElement::Action(Action::new("шаг2"))],
                label: Some("может".to_string()),
            }],
            else_branch: Some(vec![ActivityElement::Action(Action::new("шаг3"))]),
            else_label: Some("нет".to_string()),
        }));
        diagram.elements.push(ActivityElement::Stop);

        let result = ActivityLayoutEngine::new().layout(&diagram);

        // Текст ветки elseif должен присутствовать среди элементов
        let has_elseif_action = result.elements.iter().any(|e| match &e.element_type {
            ElementType::Rectangle { label, .. } => label.contains("шаг2"),
            _ => false,
        });
        assert!(has_elseif_action, "ветка elseif потеряна при раскладке");

        // Условие elseif — в собственном шестиугольнике, а подпись ветки
        // («может») — отдельным текстом под ним (эталон `feat_activity_elseif`).
        let has_elseif_label = result.elements.iter().any(|e| match &e.element_type {
            ElementType::Text { text, .. } => text.contains("может"),
            _ => false,
        });
        assert!(has_elseif_label, "метка ветки elseif потеряна");
        let has_elseif_condition = result.elements.iter().any(|e| match &e.element_type {
            ElementType::Polygon { label, .. } => label.as_deref() == Some("у2"),
            _ => false,
        });
        assert!(has_elseif_condition, "условие elseif потеряно");
    }
    use plantuml_ast::activity::ActionStyle;

    #[test]
    fn test_layout_simple() {
        let mut diagram = ActivityDiagram::new();
        diagram.elements.push(ActivityElement::Start);
        diagram.elements.push(ActivityElement::Action(Action {
            label: "Hello".to_string(),
            background_color: None,
            style: ActionStyle::Normal,
            arrow_label: None,
        }));
        diagram.elements.push(ActivityElement::Stop);

        let engine = ActivityLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должно быть: start circle + arrow + action rect + arrow + stop circle
        assert!(result.elements.len() >= 5);
    }

    #[test]
    fn test_layout_condition() {
        let mut diagram = ActivityDiagram::new();
        diagram.elements.push(ActivityElement::Start);
        diagram.elements.push(ActivityElement::Condition(Condition {
            condition: "test?".to_string(),
            then_branch: vec![ActivityElement::Action(Action::new("yes"))],
            then_label: Some("yes".to_string()),
            elseif_branches: vec![],
            else_branch: Some(vec![ActivityElement::Action(Action::new("no"))]),
            else_label: Some("no".to_string()),
        }));
        diagram.elements.push(ActivityElement::Stop);

        let engine = ActivityLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть элементы для обеих веток
        assert!(result.elements.len() >= 8);
    }

    #[test]
    fn test_layout_fork() {
        let mut diagram = ActivityDiagram::new();
        diagram.elements.push(ActivityElement::Start);
        diagram.elements.push(ActivityElement::Fork(Fork {
            branches: vec![
                vec![ActivityElement::Action(Action::new("task1"))],
                vec![ActivityElement::Action(Action::new("task2"))],
            ],
            join_type: plantuml_ast::activity::JoinType::And,
        }));
        diagram.elements.push(ActivityElement::Stop);

        let engine = ActivityLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть fork bar, 2 ветки, join bar
        assert!(result.elements.len() >= 10);
    }
}
