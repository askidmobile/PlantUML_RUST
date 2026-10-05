//! Layout engine для Timing Diagrams
//!
//! Создаёт горизонтальную временную шкалу с вертикальными lanes для участников.

use std::collections::HashMap;

use plantuml_ast::timing::{
    ParticipantType, StateChange, TimeValue, TimingDiagram, TimingParticipant,
};
use plantuml_model::{Point, Rect};

use super::TimingLayoutConfig;
use crate::traits::LayoutResult;

/// Смещение первого деления от начала оси.
///
/// Сверено по двум независимым замерам: в `timing_basic` ось на 91.732,
/// деления на 141.732 и 191.732; в пробной диаграмме `@0/@100` ось на
/// 32.635, деления на 82.635 и 132.635. В обоих случаях первое деление
/// отстоит от оси ровно на 50, следующее — на 100.
const TIME_FIRST_TICK_OFFSET: f64 = 50.0;

/// Цвет подписей timing.
///
/// PlantUML рисует ВСЕ подписи timing цветом `#333`, тогда как остальные
/// типы диаграмм используют `#000`. Проверено по эталону: все восемь
/// текстовых элементов имеют `fill="#333"`.
/// Отступ подписи дорожки от её левого края.
///
/// Измерено по эталону `timing_basic`: дорожка начинается на x = 20,
/// а подпись стоит на x = 25.
const TIMING_LABEL_INSET: f64 = 5.0;

/// Высота дорожки `robust`.
///
/// Измерено по эталону `timing_basic`: дорожка занимает 20.000..85.297.
const ROBUST_LANE_HEIGHT: f64 = 65.297;

/// Высота дорожки `concise`.
///
/// Измерено по эталону: 85.297..141.594. Дорожки РАЗНОЙ высоты, поэтому
/// единая `lane_height` давала ось на y = 150 вместо 141.594.
const CONCISE_LANE_HEIGHT: f64 = 56.297;

const TIMING_TEXT_COLOR: &str = "#333";

/// Цвет перехода состояния в concise-дорожках (эталон: `stroke:#006400`).
const TIMING_TRANSITION_COLOR: &str = "#006400";

/// Заливка блока состояния concise (эталон: `fill="#E2E2F0"`).
const TIMING_CONCISE_FILL: &str = "#E2E2F0";

/// Кегль подписи участника (эталон `timing_basic`: `font-size="14"`,
/// полужирная).
const TIMING_PARTICIPANT_FONT_SIZE: f64 = 14.0;

/// Базис подписи участника от верха дорожки (эталон: 32.995 при 20).
const TIMING_PARTICIPANT_BASELINE: f64 = 12.995;

/// Смещение «флажка» подписи участника от верха дорожки.
///
/// PlantUML подчёркивает имя участника линией со СКОШЕННЫМ концом: эталон
/// даёт горизонталь на `y = верх + 17.297` от левой рамки до конца имени
/// и отрезок вверх-вправо длиной 10.
const TIMING_FLAG_DROP: f64 = 17.297;

/// Горизонтальный вылет скоса «флажка».
const TIMING_FLAG_SLANT: f64 = 10.0;

/// Отступ конца «флажка» от конца имени участника.
const TIMING_FLAG_TAIL: f64 = 1.0;

/// Базис метки состояния от её линии (эталон: 72.451 при линии 67.297).
const TIMING_STATE_LABEL_BASELINE: f64 = 5.154;

/// Смещение нижнего уровня состояния от низа дорожки `robust`.
///
/// Эталон `timing_basic`: дорожка 20…85.297, нижний уровень — 67.297.
const ROBUST_STATE_BOTTOM_OFFSET: f64 = 18.0;

/// Расстояние между уровнями состояний (эталон: 67.297 и 47.297).
const ROBUST_STATE_LEVEL_STEP: f64 = 20.0;

/// Смещение центра блока состояния от низа дорожки `concise`.
///
/// Эталон `timing_basic`: дорожка 85.297…141.594, центр блока — 119.594.
const CONCISE_STATE_BOTTOM_OFFSET: f64 = 22.0;

/// Полувысота блока состояния concise (эталон: 107.594…131.594).
const CONCISE_STATE_HALF_HEIGHT: f64 = 12.0;

/// Горизонтальный скос блока состояния concise (эталон: 91.732 → 103.732).
const CONCISE_STATE_SLANT: f64 = 12.0;

/// Толщина обводки блока состояния concise (эталон: `stroke-width:1.5`).
const CONCISE_STATE_STROKE_WIDTH: &str = "1.5";

/// Базис подписи времени от оси (эталон: 157.804 при оси 141.594).
const TIMING_TIME_LABEL_BASELINE: f64 = 16.21;

/// Длина деления оси (эталон: 141.594 → 146.594).
const TIME_TICK_LENGTH: f64 = 5.0;

/// Шаг делений шкалы. Постоянный: не зависит ни от значений времени,
/// ни от числа событий (проверено на пяти замерах с plantuml.com).
const TIME_TICK_STEP: f64 = 50.0;

/// Насколько шкала выступает правее последнего деления (измерено).
const TIME_AXIS_TAIL: f64 = 5.0;
use crate::{EdgeType, ElementType, LayoutElement};

/// Layout engine для Timing Diagrams
pub struct TimingLayoutEngine {
    config: TimingLayoutConfig,
}

impl TimingLayoutEngine {
    /// Создаёт новый layout engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: TimingLayoutConfig::default(),
        }
    }

    /// Создаёт layout engine с заданной конфигурацией
    pub fn with_config(config: TimingLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &TimingDiagram) -> LayoutResult {
        let mut elements = Vec::new();

        // 0. Нормализуем именованные моменты времени в числовые.
        //    Делается один раз здесь, чтобы не протаскивать карту имён
        //    через все методы отрисовки. Раньше `TimeValue::Named` давал 0,
        //    из-за чего все именованные моменты схлопывались в одну точку.
        let named = Self::named_times(diagram);
        let mut diagram = diagram.clone();
        for change in &mut diagram.state_changes {
            if let TimeValue::Named(name) = &change.time {
                if let Some(value) = named.get(name) {
                    change.time = TimeValue::Absolute(*value);
                }
            }
        }
        let diagram = &diagram;

        // 1. Собираем все времена для определения масштаба
        // Верхняя граница диапазона больше не нужна: шкала строится по
        // числу событий, а не по значениям времени (см. `timeline_width`).
        // Нижняя используется при отрисовке ломаных состояний.
        let (min_time, _max_time) = self.calculate_time_range(diagram);

        // 2. Создаём mapping участников к их lane индексам
        let participant_map: HashMap<String, usize> = diagram
            .participants
            .iter()
            .enumerate()
            .flat_map(|(i, p)| {
                let mut mappings = vec![(p.name.clone(), i)];
                if let Some(ref alias) = p.alias {
                    mappings.push((alias.clone(), i));
                }
                mappings
            })
            .collect();

        // 3. Группируем state_changes по участникам
        let mut changes_by_participant: HashMap<String, Vec<&StateChange>> = HashMap::new();
        for change in &diagram.state_changes {
            let key = participant_map
                .get(&change.participant)
                .map(|&i| diagram.participants[i].name.clone())
                .unwrap_or_else(|| change.participant.clone());
            changes_by_participant.entry(key).or_default().push(change);
        }

        // Сортируем изменения по времени
        for changes in changes_by_participant.values_mut() {
            // total_cmp: корректно упорядочивает и при NaN, без unwrap
            changes.sort_by(|a, b| a.time.as_f64().total_cmp(&b.time.as_f64()));
        }

        // 4. Рисуем участников и их lanes
        let timeline_start_x = self.config.padding + self.config.participant_label_width;

        // Длина шкалы.
        //
        // PlantUML строит её по ЧИСЛУ СОБЫТИЙ, а не по диапазону времени:
        // первое деление отстоит от оси на 50, каждое следующее ещё на 50,
        // а сама шкала кончается на 5 правее последнего деления.
        //
        // Проверено на трёх независимых замерах:
        //   timing_basic (2 события): ось 91.732, деления 141.732/191.732,
        //                             правый край 196.732 = 191.732 + 5
        //   @0/@100      (2 события): ось  32.635, деления  82.635/132.635,
        //                             правый край 137.635 = 132.635 + 5
        //   @0/@25/@50/@75 (4):       последнее деление 232.63, край 237.63
        //
        // Раньше длина считалась как `time_range * time_scale`, то есть
        // масштабировалась по значениям времени: при диапазоне 100 и
        // масштабе 0.43 выходило 43 против эталонных 105.
        let events = self.collect_time_values(diagram).len().max(1);
        let timeline_width = TIME_FIRST_TICK_OFFSET
            + (events.saturating_sub(1)) as f64 * TIME_TICK_STEP
            + TIME_AXIS_TAIL;

        // Высоты дорожек зависят от типа: в эталоне robust занимает
        // 65.297, а concise — 56.297, поэтому позиции считаются
        // накопительно, а не умножением на общую высоту.
        let lane_heights: Vec<f64> = diagram
            .participants
            .iter()
            .map(|participant| self.lane_height_of(participant))
            .collect();
        let mut lane_tops: Vec<f64> = Vec::with_capacity(lane_heights.len());
        let mut running_y = self.config.padding;
        for height in &lane_heights {
            lane_tops.push(running_y);
            running_y += height + self.config.lane_spacing;
        }

        for (i, participant) in diagram.participants.iter().enumerate() {
            let lane_y = lane_tops[i];
            let lane_height = lane_heights[i];

            // Метка участника
            // PlantUML показывает ОТОБРАЖАЕМОЕ имя, а не псевдоним:
            // `robust "Веб-браузер" as WB` подписывает дорожку «Веб-браузер».
            // Здесь приоритет был обратным, и в вывод попадало «WB».
            let display_name = if participant.name.is_empty() {
                participant.alias.as_deref().unwrap_or(&participant.name)
            } else {
                &participant.name
            };
            let label_x = self.config.padding + TIMING_LABEL_INSET;
            let label_width = self
                .config
                .text
                .width_bold(display_name, TIMING_PARTICIPANT_FONT_SIZE);
            let mut label_properties = HashMap::new();
            label_properties.insert("text-fill".to_string(), TIMING_TEXT_COLOR.to_string());
            label_properties.insert("font-weight".to_string(), "700".to_string());
            elements.push(LayoutElement {
                id: format!("participant_label_{}", i),
                bounds: Rect::new(
                    label_x,
                    lane_y + TIMING_PARTICIPANT_BASELINE - TIMING_PARTICIPANT_FONT_SIZE,
                    label_width,
                    TIMING_PARTICIPANT_FONT_SIZE,
                ),
                text: None,
                properties: label_properties,
                element_type: ElementType::Text {
                    text: display_name.to_string(),
                    font_size: TIMING_PARTICIPANT_FONT_SIZE,
                },
            });

            // «Флажок» под именем участника: горизонталь от левой рамки до
            // конца имени и скос вверх-вправо. Эталон `timing_basic`:
            // (20,37.297)-(127.773,37.297) и (127.773,37.297)-(137.773,20)
            // при имени «Веб-браузер» шириной 101.773.
            let flag_right = label_x + label_width + TIMING_FLAG_TAIL;
            let flag_y = lane_y + TIMING_FLAG_DROP;
            for (id, x1, y1, x2, y2) in [
                (
                    "timing_flag_line",
                    self.config.padding,
                    flag_y,
                    flag_right,
                    flag_y,
                ),
                (
                    "timing_flag_slant",
                    flag_right,
                    flag_y,
                    flag_right + TIMING_FLAG_SLANT,
                    lane_y,
                ),
            ] {
                elements.push(LayoutElement {
                    id: format!("{id}_{i}"),
                    bounds: Rect::new(x1, y1.min(y2), (x2 - x1).abs(), (y2 - y1).abs()),
                    text: None,
                    properties: [
                        ("stroke".to_string(), TIMING_TEXT_COLOR.to_string()),
                        ("stroke-width".to_string(), "0.5".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                    element_type: ElementType::Edge {
                        points: vec![Point::new(x1, y1), Point::new(x2, y2)],
                        label: None,
                        arrow_start: false,
                        arrow_end: false,
                        dashed: false,
                        edge_type: EdgeType::Link,
                        from_cardinality: None,
                        to_cardinality: None,
                    },
                });
            }

            // Рисуем timeline для участника
            match participant.participant_type {
                ParticipantType::Robust => {
                    self.draw_robust_timeline(
                        &mut elements,
                        i,
                        &participant.name,
                        lane_y,
                        lane_height,
                        timeline_start_x,
                        timeline_width,
                        min_time,
                        changes_by_participant.get(&participant.name),
                    );
                }
                ParticipantType::Concise | ParticipantType::Binary => {
                    self.draw_concise_timeline(
                        &mut elements,
                        i,
                        &participant.name,
                        lane_y,
                        lane_height,
                        timeline_start_x,
                        timeline_width,
                        min_time,
                        changes_by_participant.get(&participant.name),
                    );
                }
                ParticipantType::Clock => {
                    self.draw_clock_timeline(
                        &mut elements,
                        i,
                        lane_y,
                        lane_height,
                        timeline_start_x,
                        timeline_width,
                    );
                }
            }
        }

        // 5. Рисуем временную ось внизу
        let axis_y = lane_tops
            .last()
            .map(|last| last + lane_heights.last().copied().unwrap_or(0.0))
            .unwrap_or(self.config.padding)
            + self.config.lane_spacing;

        self.draw_time_axis(
            &mut elements,
            diagram,
            timeline_start_x,
            axis_y,
            timeline_width,
        );

        // Вертикальные пунктирные линии времени.
        //
        // Эталон `timing_basic`: по одной на каждое деление, от верхней
        // рамки до оси, цвет #333, толщина 0.5, штрих 3,5. Раньше их
        // не было вовсе.
        let events = self.collect_time_values(diagram).len();
        let frame_bottom = axis_y;
        for index in 0..=events {
            let x = timeline_start_x + index as f64 * TIME_TICK_STEP;
            elements.push(LayoutElement {
                id: format!("lifeline_{index}"),
                bounds: Rect::new(
                    x,
                    self.config.padding,
                    0.0,
                    frame_bottom - self.config.padding,
                ),
                text: None,
                properties: [
                    ("stroke".to_string(), TIMING_TEXT_COLOR.to_string()),
                    ("stroke-width".to_string(), "0.5".to_string()),
                ]
                .into_iter()
                .collect(),
                element_type: ElementType::Edge {
                    points: vec![
                        Point::new(x, self.config.padding),
                        Point::new(x, frame_bottom),
                    ],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: true,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });
        }

        // РАМКА диаграммы.
        //
        // Эталон рисует вертикали по краям дорожек и горизонтали сверху
        // и между дорожками. Без них наш контент был на 10px уже:
        // крайние точки рамки (x = 20 и x = 196.732) как раз задают
        // границы содержимого.
        self.draw_frame_lines(
            &mut elements,
            diagram,
            timeline_start_x,
            axis_y,
            timeline_width,
        );

        // 6. Title
        if let Some(ref title) = diagram.metadata.title {
            elements.push(LayoutElement {
                id: "title".to_string(),
                bounds: Rect::new(self.config.padding, 5.0, 500.0, 20.0),
                text: None,
                properties: [("text-fill".to_string(), TIMING_TEXT_COLOR.to_string())]
                    .into_iter()
                    .collect(),
                element_type: ElementType::Text {
                    text: title.clone(),
                    font_size: 14.0,
                },
            });
        }

        // 7. Возвращаем результат
        let total_width = timeline_start_x + timeline_width + self.config.padding;
        let total_height = axis_y + 30.0 + self.config.padding;

        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, total_width, total_height),
        };
        result.calculate_bounds();
        result
    }

    /// Сопоставляет именованные моменты времени числовым позициям.
    ///
    /// PlantUML позволяет задавать моменты именами, а не числами. Раньше
    /// `TimeValue::Named` возвращал 0 (`as_f64`), поэтому все именованные
    /// моменты схлопывались в одну точку и диаграмма получалась пустой.
    ///
    /// Имена нумеруются в порядке появления: первое получает 0, второе — 1
    /// и так далее. Этого достаточно для относительного расположения
    /// состояний.
    fn named_times(diagram: &TimingDiagram) -> std::collections::HashMap<String, f64> {
        let mut map = std::collections::HashMap::new();
        let mut next = 0.0_f64;

        let register = |value: &TimeValue,
                        map: &mut std::collections::HashMap<String, f64>,
                        next: &mut f64| {
            if let TimeValue::Named(name) = value {
                map.entry(name.clone()).or_insert_with(|| {
                    let v = *next;
                    *next += 1.0;
                    v
                });
            }
        };

        for change in &diagram.state_changes {
            register(&change.time, &mut map, &mut next);
        }

        map
    }

    /// Собирает уникальные значения времени в порядке появления.
    ///
    /// Нужен для шкалы: PlantUML ставит подпись на КАЖДОЕ событие, а не
    /// через равные интервалы значений (см. `draw_time_axis`).
    fn collect_time_values(&self, diagram: &TimingDiagram) -> Vec<f64> {
        let mut result: Vec<f64> = Vec::new();
        let mut cumulative = 0.0;

        for change in &diagram.state_changes {
            let t = match &change.time {
                TimeValue::Absolute(t) => {
                    cumulative = *t;
                    *t
                }
                TimeValue::Relative(delta) => {
                    cumulative += delta;
                    cumulative
                }
                TimeValue::Named(_) => continue,
            };

            if !result.iter().any(|v| (v - t).abs() < f64::EPSILON) {
                result.push(t);
            }
        }

        result
    }

    /// Вычисляет диапазон времени
    fn calculate_time_range(&self, diagram: &TimingDiagram) -> (f64, f64) {
        let mut min_time = f64::INFINITY;
        let mut max_time = f64::NEG_INFINITY;

        for change in &diagram.state_changes {
            let t = change.time.as_f64();
            min_time = min_time.min(t);
            max_time = max_time.max(t);
        }

        // Обработка относительного времени
        let mut cumulative = 0.0;
        for change in &diagram.state_changes {
            match &change.time {
                TimeValue::Absolute(t) => cumulative = *t,
                TimeValue::Relative(delta) => cumulative += delta,
                TimeValue::Named(_) => {}
            }
            max_time = max_time.max(cumulative);
        }

        if min_time == f64::INFINITY {
            min_time = 0.0;
        }
        if max_time == f64::NEG_INFINITY {
            max_time = 100.0;
        }

        (min_time, max_time)
    }

    /// Рисует robust timeline (прямоугольники состояний)
    #[allow(clippy::too_many_arguments)]
    fn draw_robust_timeline(
        &self,
        elements: &mut Vec<LayoutElement>,
        participant_idx: usize,
        participant_name: &str,
        lane_y: f64,
        lane_height: f64,
        start_x: f64,
        _width: f64,
        _min_time: f64,
        changes: Option<&Vec<&StateChange>>,
    ) {
        let Some(changes) = changes else {
            return;
        };
        if changes.is_empty() {
            return;
        }

        // Уровни состояний: первое объявленное состояние — нижнее
        // (эталон `timing_basic`: «Ожидание» на 67.297, «Работа» на 47.297).
        let levels = state_levels(changes);
        let base_y = lane_y + lane_height - ROBUST_STATE_BOTTOM_OFFSET;
        let level_y = |state: &str| {
            let level = levels.get(state).copied().unwrap_or(0);
            base_y - level as f64 * ROBUST_STATE_LEVEL_STEP
        };

        let step = |index: usize| start_x + index as f64 * TIME_TICK_STEP;
        let line_to = |index: usize| {
            if index + 1 == changes.len() {
                step(index) + TIME_TICK_STEP
            } else {
                step(index + 1)
            }
        };

        // Горизонтальные отрезки состояний.
        for (i, change) in changes.iter().enumerate() {
            let y = level_y(&change.state);
            let x1 = step(i);
            let x2 = line_to(i);
            elements.push(LayoutElement {
                id: format!("state_{}_{}_{}", participant_name, participant_idx, i),
                bounds: Rect::new(x1, y, x2 - x1, 2.0),
                text: None,
                properties: [
                    ("stroke".to_string(), TIMING_TRANSITION_COLOR.to_string()),
                    ("stroke-width".to_string(), "2".to_string()),
                ]
                .into_iter()
                .collect(),
                element_type: ElementType::Edge {
                    points: vec![Point::new(x1, y), Point::new(x2, y)],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });

            // Имя состояния — в колонке меток, на уровне своей линии.
            let label_width = self
                .config
                .text
                .width(&change.state, self.config.label_font_size);
            elements.push(LayoutElement {
                id: format!("state_name_{}_{}_{}", participant_name, participant_idx, i),
                bounds: Rect::new(
                    self.config.padding + TIMING_LABEL_INSET,
                    y + TIMING_STATE_LABEL_BASELINE - self.config.label_font_size,
                    label_width,
                    self.config.label_font_size,
                ),
                text: None,
                properties: [("text-fill".to_string(), TIMING_TEXT_COLOR.to_string())]
                    .into_iter()
                    .collect(),
                element_type: ElementType::Text {
                    text: change.state.clone(),
                    font_size: self.config.label_font_size,
                },
            });
        }

        // Вертикальные переходы между уровнями.
        for i in 1..changes.len() {
            let x = step(i);
            let y1 = level_y(&changes[i - 1].state);
            let y2 = level_y(&changes[i].state);
            if (y1 - y2).abs() < f64::EPSILON {
                continue;
            }
            elements.push(LayoutElement {
                id: format!("transition_{}_{}_{}", participant_name, participant_idx, i),
                bounds: Rect::new(x, y1.min(y2), 2.0, (y2 - y1).abs()),
                text: None,
                properties: [
                    ("stroke".to_string(), TIMING_TRANSITION_COLOR.to_string()),
                    ("stroke-width".to_string(), "2".to_string()),
                ]
                .into_iter()
                .collect(),
                element_type: ElementType::Edge {
                    points: vec![Point::new(x, y1), Point::new(x, y2)],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });
        }
    }

    /// Рисует concise timeline — блоки состояний.
    ///
    /// PlantUML рисует их не прямоугольниками, а фигурами со скошенными
    /// концами: промежуточное состояние — шестиугольник, последнее —
    /// пятиугольник (правый край остаётся прямым, потому что состояние
    /// продолжается до конца шкалы). Эталон `timing_basic`:
    /// `103.7,107.6 129.7,107.6 141.7,119.6 129.7,131.6 103.7,131.6 91.7,119.6`.
    #[allow(clippy::too_many_arguments)]
    fn draw_concise_timeline(
        &self,
        elements: &mut Vec<LayoutElement>,
        participant_idx: usize,
        participant_name: &str,
        lane_y: f64,
        lane_height: f64,
        start_x: f64,
        _width: f64,
        _min_time: f64,
        changes: Option<&Vec<&StateChange>>,
    ) {
        let Some(changes) = changes else {
            return;
        };
        if changes.is_empty() {
            return;
        }

        let center_y = lane_y + lane_height - CONCISE_STATE_BOTTOM_OFFSET;
        let top = center_y - CONCISE_STATE_HALF_HEIGHT;
        let bottom = center_y + CONCISE_STATE_HALF_HEIGHT;
        let slant = CONCISE_STATE_SLANT;

        let step = |index: usize| start_x + index as f64 * TIME_TICK_STEP;
        let line_to = |index: usize| {
            if index + 1 == changes.len() {
                step(index) + TIME_TICK_STEP
            } else {
                step(index + 1)
            }
        };

        for (i, change) in changes.iter().enumerate() {
            let x1 = step(i);
            let x2 = line_to(i);
            let is_last = i + 1 == changes.len();
            let flat_left = x1 + slant;
            let flat_right = if is_last { x2 } else { x2 - slant };

            // Вершины — в ЛОКАЛЬНЫХ координатах `bounds`: рендерер
            // прибавляет к ним левый верхний угол фигуры.
            let local = |x: f64, y: f64| Point::new(x - x1, y - top);
            let mut points = vec![
                local(flat_left, top),
                local(flat_right, top),
                local(x2, center_y),
                local(flat_right, bottom),
                local(flat_left, bottom),
                local(x1, center_y),
            ];
            if !is_last {
                points.push(local(flat_left, top));
            }

            // Заливка. У последнего состояния обводку рисует отдельный
            // незамкнутый путь: PlantUML не обводит его правый край.
            elements.push(LayoutElement {
                id: format!("state_block_{}_{}_{}", participant_name, participant_idx, i),
                bounds: Rect::new(x1, top, x2 - x1, bottom - top),
                text: None,
                properties: [
                    ("fill".to_string(), TIMING_CONCISE_FILL.to_string()),
                    (
                        "stroke".to_string(),
                        if is_last {
                            TIMING_CONCISE_FILL.to_string()
                        } else {
                            TIMING_TRANSITION_COLOR.to_string()
                        },
                    ),
                    (
                        "stroke-width".to_string(),
                        CONCISE_STATE_STROKE_WIDTH.to_string(),
                    ),
                ]
                .into_iter()
                .collect(),
                element_type: ElementType::Polygon {
                    points,
                    label: None,
                    font_size: 0.0,
                },
            });

            if is_last {
                let mut outline = std::collections::HashMap::new();
                outline.insert("path".to_string(), format!(
                    "M{x2},{top} L{flat_left},{top} L{x1},{center_y} L{flat_left},{bottom} L{x2},{bottom}"
                ));
                outline.insert("stroke".to_string(), TIMING_TRANSITION_COLOR.to_string());
                outline.insert(
                    "stroke-width".to_string(),
                    CONCISE_STATE_STROKE_WIDTH.to_string(),
                );
                outline.insert("fill".to_string(), TIMING_CONCISE_FILL.to_string());
                elements.push(LayoutElement {
                    id: format!(
                        "state_outline_{}_{}_{}",
                        participant_name, participant_idx, i
                    ),
                    bounds: Rect::new(x1, top, x2 - x1, bottom - top),
                    text: None,
                    properties: outline,
                    element_type: ElementType::Path,
                });
            }

            // Подпись: у промежуточных состояний — по центру плоской части,
            // у последнего — от её левого края (эталон: «Ожидание» на
            // 80.144 при центре 116.73, «Обработка» на 153.732).
            let label_width = self
                .config
                .text
                .width_bold(&change.state, self.config.label_font_size);
            let label_x = if is_last {
                flat_left
            } else {
                (flat_left + flat_right) / 2.0 - label_width / 2.0
            };
            let mut properties = std::collections::HashMap::new();
            properties.insert("text-fill".to_string(), TIMING_TEXT_COLOR.to_string());
            properties.insert("font-weight".to_string(), "700".to_string());
            elements.push(LayoutElement {
                id: format!("state_label_{}_{}_{}", participant_name, participant_idx, i),
                bounds: Rect::new(
                    label_x,
                    center_y + TIMING_STATE_LABEL_BASELINE - self.config.label_font_size,
                    label_width,
                    self.config.label_font_size,
                ),
                text: None,
                properties,
                element_type: ElementType::Text {
                    text: change.state.clone(),
                    font_size: self.config.label_font_size,
                },
            });
        }
    }

    /// Рисует clock timeline (меандр)
    fn draw_clock_timeline(
        &self,
        elements: &mut Vec<LayoutElement>,
        participant_idx: usize,
        lane_y: f64,
        lane_height: f64,
        start_x: f64,
        width: f64,
    ) {
        let high_y = lane_y + 10.0;
        let low_y = lane_y + lane_height - 10.0;
        let period = 40.0; // Период clock

        let mut points = Vec::new();
        let mut x = start_x;
        let mut is_high = true;

        while x < start_x + width {
            let y = if is_high { high_y } else { low_y };
            points.push(Point::new(x, y));

            // Вертикальный переход
            let next_y = if is_high { low_y } else { high_y };
            points.push(Point::new(x, next_y));

            x += period / 2.0;
            is_high = !is_high;
        }

        elements.push(LayoutElement {
            id: format!("clock_{}", participant_idx),
            bounds: Rect::new(start_x, high_y, width, low_y - high_y),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points,
                label: None,
                arrow_start: false,
                arrow_end: false,
                dashed: false,
                edge_type: EdgeType::Link,
                from_cardinality: None,
                to_cardinality: None,
            },
        });
    }

    /// Рисует временную ось
    fn draw_time_axis(
        &self,
        elements: &mut Vec<LayoutElement>,
        diagram: &TimingDiagram,
        start_x: f64,
        y: f64,
        width: f64,
    ) {
        // Горизонтальная линия оси.
        //
        // PlantUML НЕ ставит на ней наконечник: эталон `timing_basic` даёт
        // `(91.732,141.594)-(191.732,141.594)` толщиной 2 цветом #333,
        // то есть линия кончается на последнем делении, а не на правом
        // крае шкалы.
        let axis_end = start_x + width - TIME_TICK_LENGTH;
        elements.push(LayoutElement {
            id: "time_axis".to_string(),
            bounds: Rect::new(start_x, y, axis_end - start_x, 2.0),
            text: None,
            properties: [
                ("stroke".to_string(), TIMING_TEXT_COLOR.to_string()),
                ("stroke-width".to_string(), "2".to_string()),
            ]
            .into_iter()
            .collect(),
            element_type: ElementType::Edge {
                points: vec![Point::new(start_x, y), Point::new(axis_end, y)],
                label: None,
                arrow_start: false,
                arrow_end: false,
                dashed: false,
                edge_type: EdgeType::Link,
                from_cardinality: None,
                to_cardinality: None,
            },
        });

        // Деления и метки времени.
        //
        // Правило снято с plantuml.com по пяти замерам (2, 3, 4 и 5
        // событий, а также диапазоны @0/@50 и @0/@200):
        //
        //   * первое деление ВСЕГДА на x = 32.635 при колонке меток 20,
        //     то есть шаг до него фиксирован;
        //   * шаг делений ВСЕГДА 50.0 и не зависит ни от значений
        //     времени, ни от их числа;
        //   * подпись ставится на КАЖДОЕ событие: для 0/25/50/75 подписи
        //     стоят на 29.14, 75.64, 125.64, 175.64 — шаг 50, а не
        //     пропорционально значениям;
        //   * диапазон времени на геометрию не влияет.
        //
        // Прежний код считал `(t - min_time) * time_scale`, то есть
        // масштабировал по значениям времени — PlantUML так не делает.
        let times = self.collect_time_values(diagram);

        // Подпись ставится на САМУ ОСЬ и далее каждые +50, по одной на
        // событие; деления идут с +50. Сверено по двум замерам:
        //   timing_basic: ось 91.732, подпись «0» центрирована на 91.73,
        //                 «100» на 141.73 (ось + 50)
        //   @0/@100:      ось 32.635, подпись «0» на 32.635,
        //                 «100» на 82.635 (ось + 50)
        // Деления: эталон `timing_basic` при двух событиях даёт ТРИ деления —
        // на 91.732, 141.732 и 191.732, то есть по одному на каждую границу
        // интервала, а не по одному на событие.
        for index in 0..=times.len() {
            let tick_x = start_x + index as f64 * TIME_TICK_STEP;
            elements.push(LayoutElement {
                id: format!("tick_{}", index),
                bounds: Rect::new(tick_x - 0.5, y, 0.0, TIME_TICK_LENGTH),
                text: None,
                // Деления оси: цвет #333, толщина 2 (эталон).
                properties: [
                    ("stroke".to_string(), TIMING_TEXT_COLOR.to_string()),
                    ("stroke-width".to_string(), "2".to_string()),
                ]
                .into_iter()
                .collect(),
                element_type: ElementType::Edge {
                    points: vec![
                        Point::new(tick_x, y),
                        Point::new(tick_x, y + TIME_TICK_LENGTH),
                    ],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });
        }

        // Метки времени — на каждом событии, по центру его деления.
        //
        // Ширина измеряется: эталон даёт «100» на x=131.235 при делении
        // 141.732, то есть `141.732 − 20.996 / 2`.
        for (index, t) in times.iter().enumerate() {
            let t = *t;
            let tick_x = start_x + index as f64 * TIME_TICK_STEP;
            let label_width = self
                .config
                .text
                .width(&format!("{}", t as i64), self.config.time_font_size);
            elements.push(LayoutElement {
                id: format!("time_label_{}", t as i64),
                bounds: Rect::new(
                    tick_x - label_width / 2.0,
                    y + TIMING_TIME_LABEL_BASELINE - self.config.time_font_size,
                    label_width,
                    self.config.time_font_size,
                ),
                text: None,
                properties: [("text-fill".to_string(), TIMING_TEXT_COLOR.to_string())]
                    .into_iter()
                    .collect(),
                element_type: ElementType::Text {
                    text: format!("{}", t as i64),
                    font_size: self.config.time_font_size,
                },
            });
        }
    }
    /// Рисует рамку диаграммы времени: вертикали по краям дорожек,
    /// горизонталь сверху и разделители между дорожками.
    ///
    /// Измерено по эталону `timing_basic`: левая вертикаль на x = 20,
    /// правая на x = 196.732, верхняя горизонталь на y = 20,
    /// разделитель дорожек на y = 85.297.
    fn draw_frame_lines(
        &self,
        elements: &mut Vec<LayoutElement>,
        diagram: &TimingDiagram,
        timeline_start_x: f64,
        axis_y: f64,
        timeline_width: f64,
    ) {
        let left = self.config.padding;
        let right = timeline_start_x + timeline_width;
        let top = self.config.padding;
        let mut push_line = |x1: f64, y1: f64, x2: f64, y2: f64| {
            elements.push(LayoutElement {
                id: format!("timing_frame_{}", elements.len()),
                bounds: Rect::new(
                    x1.min(x2),
                    y1.min(y2),
                    (x2 - x1).abs().max(1.0),
                    (y2 - y1).abs().max(1.0),
                ),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Edge {
                    points: vec![Point::new(x1, y1), Point::new(x2, y2)],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });
        };

        // Верхняя горизонталь и вертикали по краям.
        push_line(left, top, right, top);
        push_line(left, top, left, axis_y);
        push_line(right, top, right, axis_y);

        // Разделители между дорожками — по накопительным высотам,
        // потому что дорожки разной высоты.
        let mut y = top;
        for participant in diagram
            .participants
            .iter()
            .take(diagram.participants.len().saturating_sub(1))
        {
            y += self.lane_height_of(participant) + self.config.lane_spacing;
            push_line(left, y, right, y);
        }
    }

    /// Высота дорожки участника.
    ///
    /// В эталоне `timing_basic` дорожки РАЗНОЙ высоты: robust занимает
    /// 65.297, concise — 56.297. Единая высота давала ось на y = 150
    /// вместо эталонных 141.594.
    fn lane_height_of(&self, participant: &TimingParticipant) -> f64 {
        match participant.participant_type {
            ParticipantType::Robust => ROBUST_LANE_HEIGHT,
            ParticipantType::Concise | ParticipantType::Binary | ParticipantType::Clock => {
                CONCISE_LANE_HEIGHT
            }
        }
    }
}

/// Присваивает состояниям уровни по порядку первого появления.
///
/// PlantUML укладывает состояния дорожки `robust` «стопкой»: первое
/// объявленное оказывается НИЖНИМ. В эталоне `timing_basic` «Ожидание»
/// (объявлено на `@0`) стоит на 67.297, «Работа» (`@100`) — на 47.297.
fn state_levels(changes: &[&StateChange]) -> std::collections::HashMap<String, usize> {
    let mut levels = std::collections::HashMap::new();
    for change in changes {
        let next = levels.len();
        levels.entry(change.state.clone()).or_insert(next);
    }
    levels
}

impl Default for TimingLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::timing::TimingParticipant;

    #[test]
    fn test_layout_simple_timing() {
        let mut diagram = TimingDiagram::new();
        diagram
            .participants
            .push(TimingParticipant::robust("Browser"));
        diagram
            .participants
            .push(TimingParticipant::concise("Server"));

        diagram.state_changes.push(StateChange::new(
            "Browser",
            TimeValue::Absolute(0.0),
            "Idle",
        ));
        diagram.state_changes.push(StateChange::new(
            "Browser",
            TimeValue::Absolute(100.0),
            "Running",
        ));

        let engine = TimingLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть элементы для участников и состояний
        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_time_range_calculation() {
        let mut diagram = TimingDiagram::new();
        diagram
            .state_changes
            .push(StateChange::new("A", TimeValue::Absolute(50.0), "State1"));
        diagram
            .state_changes
            .push(StateChange::new("A", TimeValue::Absolute(200.0), "State2"));

        let engine = TimingLayoutEngine::new();
        let (min, max) = engine.calculate_time_range(&diagram);

        assert_eq!(min, 50.0);
        assert_eq!(max, 200.0);
    }
}
