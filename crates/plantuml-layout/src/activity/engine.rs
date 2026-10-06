//! Activity Diagram Layout Engine
//!
//! Flowchart-based layout algorithm для activity diagrams.

use std::collections::HashMap;

use plantuml_ast::activity::{
    Action, ActivityDiagram, ActivityElement, Condition, Fork, Partition, RepeatLoop, Split,
    Switch, WhileLoop,
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
                arrow_end: true,
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
                ActivityElement::Condition(cond) => self.condition_shape(&cond.condition).0,
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

        // Текущий swimlane
        let mut current_swimlane: Option<String> = None;

        // Определяем center_x в зависимости от наличия swimlanes
        let default_center_x = self.config.margin + self.config.action_width / 2.0;

        // Храним позицию предыдущего элемента для отрисовки стрелок
        // (center_x, bottom_y, is_start_or_action) - is_start_or_action указывает, нужна ли стрелка
        let mut prev_element_info: Option<(f64, f64, bool)> = None;

        // Обрабатываем элементы последовательно
        for element in &diagram.elements {
            // Обрабатываем смену swimlane
            if let ActivityElement::SwimlaneChange(swimlane) = element {
                current_swimlane = Some(swimlane.name.clone());
                continue;
            }

            // Пропускаем элементы, которые не требуют layout
            if matches!(element, ActivityElement::Detach | ActivityElement::Kill) {
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
                ActivityElement::Note(note) => {
                    // Заметка рисуется справа от потока, светло-жёлтым
                    // (#FEFFDD) — как в PlantUML. Раньше здесь не было ветки
                    // вовсе, и заметка не попадала в вывод.
                    let width = self.config.text.width(&note.text, self.config.font_size)
                        + ACTION_TEXT_PADDING;
                    let x = center_x + self.config.action_width / 2.0 + ACTIVITY_NOTE_GAP;

                    let mut properties = std::collections::HashMap::new();
                    properties.insert("fill".to_string(), ACTIVITY_NOTE_BACKGROUND.to_string());

                    elements.push(LayoutElement {
                        id: format!("note_{}", elements.len()),
                        bounds: Rect::new(x, current_y, width, self.config.action_height),
                        text: None,
                        properties,
                        element_type: ElementType::Rectangle {
                            label: note.text.clone(),
                            corner_radius: 0.0,
                        },
                    });

                    // Заметка не прерывает поток: стрелка после неё не нужна
                    (self.config.action_height, false)
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
                    // Ширина действия зависит от длины подписи: измерено по
                    // эталону («Первый шаг» 76.822 → 96.8, «Последний шаг»
                    // 98.209 → 118.2), то есть текст плюс 20.
                    let w = self.config.text.width(&action.label, self.config.font_size)
                        + ACTION_TEXT_PADDING;
                    let h = self.config.action_height;
                    elements.push(LayoutElement {
                        id: format!("action_{}", elements.len()),
                        bounds: Rect::new(center_x - w / 2.0, current_y, w, h),
                        text: None,
                        properties: Self::shape_properties(ACTIVITY_ACTION_FONT_SIZE),
                        element_type: ElementType::Rectangle {
                            label: action.label.clone(),
                            corner_radius: self.config.action_corner_radius,
                        },
                    });
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
                // Заметка рисуется справа от потока, светло-жёлтым (#FEFFDD) —
                // как в PlantUML. Раньше здесь стояла заглушка `TODO`,
                // и заметка не появлялась в выводе вовсе.
                let width =
                    self.config.text.width(&note.text, self.config.font_size) + ACTION_TEXT_PADDING;
                let x = center_x + self.config.action_width / 2.0 + ACTIVITY_NOTE_GAP;

                let mut properties = std::collections::HashMap::new();
                properties.insert("fill".to_string(), ACTIVITY_NOTE_BACKGROUND.to_string());

                elements.push(LayoutElement {
                    id: format!("note_{}", elements.len()),
                    bounds: Rect::new(x, current_y, width, self.config.action_height),
                    text: None,
                    properties,
                    element_type: ElementType::Rectangle {
                        label: note.text.clone(),
                        corner_radius: 0.0,
                    },
                });

                current_y + self.config.action_height + self.config.vertical_spacing
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

    /// Располагает действие (rounded rectangle)
    fn layout_action(
        &self,
        action: &Action,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        // Ширина по содержимому — см. ACTION_TEXT_PADDING
        let w = self.config.text.width(&action.label, self.config.font_size) + ACTION_TEXT_PADDING;
        let h = self.config.action_height;

        elements.push(LayoutElement {
            id: format!("action_{}", elements.len()),
            bounds: Rect::new(center_x - w / 2.0, current_y, w, h),
            text: None,
            properties: Self::shape_properties(ACTIVITY_ACTION_FONT_SIZE),
            element_type: ElementType::Rectangle {
                label: action.label.clone(),
                corner_radius: self.config.action_corner_radius,
            },
        });

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
    fn layout_while(
        &self,
        while_loop: &WhileLoop,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let dh = self.config.diamond_height;

        // Шестиугольник условия цикла
        let dw = self.add_condition_shape(&while_loop.condition, center_x, current_y, elements);

        let body_start_y = current_y + dh + self.config.vertical_spacing;
        let mut body_end_y = body_start_y;

        // Тело цикла
        for elem in &while_loop.body {
            body_end_y = self.layout_element(elem, center_x, body_end_y, elements);
        }

        // Обратная стрелка (loop back)
        let loop_x = center_x - self.config.horizontal_spacing - 20.0;

        // Вниз -> влево -> вверх -> вправо к ромбу
        elements.push(LayoutElement {
            id: format!("while_loop_{}", elements.len()),
            bounds: Rect::new(loop_x, current_y, center_x - loop_x, body_end_y - current_y),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points: vec![
                    Point::new(center_x, body_end_y - self.config.vertical_spacing),
                    Point::new(loop_x, body_end_y - self.config.vertical_spacing),
                    Point::new(loop_x, current_y + dh / 2.0),
                    Point::new(center_x - dw / 2.0, current_y + dh / 2.0),
                ],
                label: while_loop.backward_label.clone(),
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association,
                from_cardinality: None,
                to_cardinality: None,
            },
        });

        // Стрелка выхода из цикла (вправо)
        let exit_x = center_x + self.config.horizontal_spacing;
        self.add_arrow(
            center_x + dw / 2.0,
            current_y + dh / 2.0,
            exit_x,
            body_end_y,
            while_loop.end_label.clone(),
            elements,
        );

        body_end_y + self.config.vertical_spacing
    }

    /// Располагает цикл repeat
    fn layout_repeat(
        &self,
        repeat_loop: &RepeatLoop,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let body_start_y = current_y;
        let mut body_end_y = body_start_y;

        // Тело цикла (выполняется первым)
        for elem in &repeat_loop.body {
            body_end_y = self.layout_element(elem, center_x, body_end_y, elements);
        }

        // Ромб условия внизу
        let dh = self.config.diamond_height;

        let dw = self.add_condition_shape(&repeat_loop.condition, center_x, body_end_y, elements);

        // Обратная стрелка
        let loop_x = center_x + self.config.horizontal_spacing + 20.0;

        elements.push(LayoutElement {
            id: format!("repeat_loop_{}", elements.len()),
            bounds: Rect::new(
                center_x,
                body_start_y,
                loop_x - center_x,
                body_end_y - body_start_y + dh,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points: vec![
                    Point::new(center_x + dw / 2.0, body_end_y + dh / 2.0),
                    Point::new(loop_x, body_end_y + dh / 2.0),
                    Point::new(loop_x, body_start_y),
                    Point::new(center_x, body_start_y),
                ],
                label: repeat_loop.backward_label.clone(),
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association,
                from_cardinality: None,
                to_cardinality: None,
            },
        });

        body_end_y + dh + self.config.vertical_spacing
    }

    /// Располагает fork/join
    /// Раскладывает раздел `partition`.
    ///
    /// PlantUML рисует раздел рамкой с подписью в левом верхнем углу,
    /// внутри — обычный поток. Раньше раздел не поддерживался вовсе:
    /// в основном цикле он попадал в catch-all и пропускался вместе со
    /// всем содержимым.
    fn layout_partition(
        &self,
        partition: &Partition,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let header_height = self.config.action_height;

        // Содержимое раздела раскладывается ниже его заголовка
        let content_y = current_y + header_height + self.config.vertical_spacing;
        let mut inner: Vec<LayoutElement> = Vec::new();
        let end_y = self.layout_elements(&partition.elements, center_x, content_y, &mut inner);

        // Ширина рамки — по самому широкому содержимому
        let mut width = self
            .config
            .text
            .width(&partition.name, self.config.font_size)
            + ACTION_TEXT_PADDING;
        for element in &inner {
            let right = (element.bounds.x + element.bounds.width - center_x).abs() * 2.0;
            width = width.max(right + PARTITION_PADDING * 2.0);
        }

        let mut properties = std::collections::HashMap::new();
        if let Some(color) = &partition.color {
            properties.insert("stroke".to_string(), color.to_css());
        }

        // Рамка добавляется ПЕРВОЙ, чтобы её закрыли стрелки и элементы
        elements.push(LayoutElement {
            id: format!("partition_{}", elements.len()),
            bounds: Rect::new(
                center_x - width / 2.0,
                current_y,
                width,
                end_y - current_y + PARTITION_PADDING,
            ),
            text: None,
            properties,
            element_type: ElementType::Rectangle {
                label: partition.name.clone(),
                corner_radius: 0.0,
            },
        });

        elements.extend(inner);

        end_y + PARTITION_PADDING
    }

    /// Раскладывает множественный выбор `switch`.
    ///
    /// Ветки ставятся вертикально, каждая со своей подписью условия —
    /// так же, как PlantUML рисует ромбы выбора.
    fn layout_switch(
        &self,
        switch: &Switch,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let mut y = current_y;

        for (index, branch) in switch.branches.iter().enumerate() {
            let label = branch
                .label
                .clone()
                .unwrap_or_else(|| format!("case {}", index + 1));
            let width = self.config.text.width(&label, self.config.font_size) + ACTION_TEXT_PADDING;

            elements.push(LayoutElement {
                id: format!("switch_case_{}_{}", elements.len(), index),
                bounds: Rect::new(center_x - width / 2.0, y, width, self.config.action_height),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Rectangle {
                    label,
                    corner_radius: 0.0,
                },
            });

            y += self.config.action_height + self.config.vertical_spacing;
            y = self.layout_elements(&branch.elements, center_x, y, elements);
        }

        y
    }

    /// Раскладывает разделение потока `split`.
    ///
    /// Ветки идут вертикально, как и в `switch`: горизонтальная раскладка
    /// потребовала бы переработки всего потока, а PlantUML в этом случае
    /// тоже разносит ветки по вертикали.
    fn layout_split(
        &self,
        split: &Split,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let mut y = current_y;

        for (index, branch) in split.branches.iter().enumerate() {
            let label = branch
                .label
                .clone()
                .unwrap_or_else(|| format!("ветка {}", index + 1));
            let width = self.config.text.width(&label, self.config.font_size) + ACTION_TEXT_PADDING;

            elements.push(LayoutElement {
                id: format!("split_branch_{}_{}", elements.len(), index),
                bounds: Rect::new(center_x - width / 2.0, y, width, self.config.action_height),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Rectangle {
                    label,
                    corner_radius: 0.0,
                },
            });

            y += self.config.action_height + self.config.vertical_spacing;
            y = self.layout_elements(&branch.elements, center_x, y, elements);
        }

        y
    }

    /// Раскладывает последовательность элементов с заданной вертикали.
    ///
    /// Нужен вложенным конструкциям (раздел, ветки `switch`/`split`).
    fn layout_elements(
        &self,
        elements: &[ActivityElement],
        center_x: f64,
        start_y: f64,
        out: &mut Vec<LayoutElement>,
    ) -> f64 {
        let mut y = start_y;

        for element in elements {
            if let ActivityElement::Action(action) = element {
                let width = self.config.text.width(&action.label, self.config.font_size)
                    + ACTION_TEXT_PADDING;
                out.push(LayoutElement {
                    id: format!("action_{}", out.len()),
                    bounds: Rect::new(center_x - width / 2.0, y, width, self.config.action_height),
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Rectangle {
                        label: action.label.clone(),
                        corner_radius: self.config.action_corner_radius,
                    },
                });
                y += self.config.action_height + self.config.vertical_spacing;
            } else if let ActivityElement::Note(note) = element {
                let width =
                    self.config.text.width(&note.text, self.config.font_size) + ACTION_TEXT_PADDING;
                let mut properties = std::collections::HashMap::new();
                properties.insert("fill".to_string(), ACTIVITY_NOTE_BACKGROUND.to_string());
                out.push(LayoutElement {
                    id: format!("note_{}", out.len()),
                    bounds: Rect::new(
                        center_x + self.config.action_width / 2.0 + ACTIVITY_NOTE_GAP,
                        y,
                        width,
                        self.config.action_height,
                    ),
                    text: None,
                    properties,
                    element_type: ElementType::Rectangle {
                        label: note.text.clone(),
                        corner_radius: 0.0,
                    },
                });
                y += self.config.action_height + self.config.vertical_spacing;
            } else {
                // Остальные конструкции внутри раздела раскладываются
                // через общий метод: он умеет всё, что умеет основной цикл.
                y = self.layout_element(element, center_x, y, out);
            }
        }

        y
    }

    fn layout_fork(
        &self,
        fork: &Fork,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let num_branches = fork.branches.len();
        if num_branches == 0 {
            return current_y;
        }

        // Fork bar
        let total_width =
            (num_branches as f64 - 1.0) * self.config.horizontal_spacing + self.config.action_width;
        let fork_bar_x = center_x - total_width / 2.0;

        elements.push(LayoutElement {
            id: format!("fork_bar_{}", elements.len()),
            bounds: Rect::new(fork_bar_x, current_y, total_width, self.config.bar_height),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
        });

        let branches_start_y = current_y + self.config.bar_height + self.config.vertical_spacing;
        let mut max_branch_end_y = branches_start_y;

        // Располагаем каждую ветку
        let branch_spacing = if num_branches > 1 {
            total_width / (num_branches as f64 - 1.0)
        } else {
            0.0
        };

        for (i, branch) in fork.branches.iter().enumerate() {
            let branch_x = if num_branches > 1 {
                fork_bar_x + i as f64 * branch_spacing
            } else {
                center_x
            };

            // Стрелка от fork bar к началу ветки
            self.add_arrow(
                branch_x,
                current_y + self.config.bar_height,
                branch_x,
                branches_start_y,
                None,
                elements,
            );

            let mut branch_y = branches_start_y;
            for elem in branch {
                branch_y = self.layout_element(elem, branch_x, branch_y, elements);
            }

            max_branch_end_y = max_branch_end_y.max(branch_y);
        }

        // Join bar
        let join_y = max_branch_end_y;

        elements.push(LayoutElement {
            id: format!("join_bar_{}", elements.len()),
            bounds: Rect::new(fork_bar_x, join_y, total_width, self.config.bar_height),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
        });

        // Стрелки от веток к join bar
        for i in 0..num_branches {
            let branch_x = if num_branches > 1 {
                fork_bar_x + i as f64 * branch_spacing
            } else {
                center_x
            };

            self.add_arrow(
                branch_x,
                max_branch_end_y - self.config.vertical_spacing,
                branch_x,
                join_y,
                None,
                elements,
            );
        }

        join_y + self.config.bar_height + self.config.vertical_spacing
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

        // И её метка условия — отдельным текстом над плечом излома
        // (PlantUML прижимает подписи к боковым вершинам шестиугольника,
        // поэтому метка больше не висит на самой стрелке).
        let has_elseif_label = result.elements.iter().any(|e| match &e.element_type {
            ElementType::Text { text, .. } => text.contains("у2"),
            _ => false,
        });
        assert!(has_elseif_label, "метка ветки elseif потеряна");
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
