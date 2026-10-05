//! Layout engine для Timing Diagrams
//!
//! Создаёт горизонтальную временную шкалу с вертикальными lanes для участников.

use std::collections::HashMap;

use plantuml_ast::timing::{ParticipantType, StateChange, TimeValue, TimingDiagram};
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
const TIMING_TEXT_COLOR: &str = "#333";

/// Цвет перехода состояния в concise-дорожках (эталон: `stroke:#006400`).
const TIMING_TRANSITION_COLOR: &str = "#006400";

/// Отступ метки состояния от вертикали перехода (измерено по эталону:
/// переход на 141.732, метка «Обработка» на 153.73 — разница 12).
const STATE_LABEL_OFFSET: f64 = 12.0;

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

        for (i, participant) in diagram.participants.iter().enumerate() {
            let lane_y = self.config.padding
                + (i as f64) * (self.config.lane_height + self.config.lane_spacing);

            // Метка участника
            let display_name = participant.alias.as_deref().unwrap_or(&participant.name);
            elements.push(LayoutElement {
                id: format!("participant_label_{}", i),
                bounds: Rect::new(
                    self.config.padding,
                    lane_y,
                    self.config.participant_label_width - 10.0,
                    self.config.lane_height,
                ),
                text: None,
                properties: [("text-fill".to_string(), TIMING_TEXT_COLOR.to_string())]
                    .into_iter()
                    .collect(),
                element_type: ElementType::Text {
                    text: display_name.to_string(),
                    font_size: self.config.label_font_size,
                },
            });

            // Рисуем timeline для участника
            match participant.participant_type {
                ParticipantType::Robust => {
                    self.draw_robust_timeline(
                        &mut elements,
                        i,
                        &participant.name,
                        lane_y,
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
                        timeline_start_x,
                        timeline_width,
                    );
                }
            }
        }

        // 5. Рисуем временную ось внизу
        let axis_y = self.config.padding
            + (diagram.participants.len() as f64)
                * (self.config.lane_height + self.config.lane_spacing);

        self.draw_time_axis(
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
        start_x: f64,
        _width: f64,
        _min_time: f64,
        changes: Option<&Vec<&StateChange>>,
    ) {
        let state_y = lane_y + (self.config.lane_height - self.config.robust_state_height) / 2.0;

        // Позиция = начало шкалы + ИНДЕКС события * шаг деления.
        //
        // Шкала строится по числу событий, а не по значениям времени
        // (правило выведено по замерам, см. `draw_time_axis`). Здесь
        // раньше оставался старый расчёт `(t - min_time) * time_scale`,
        // из-за чего переходы стояли на 7 меньше эталонных.
        if let Some(changes) = changes {
            for (i, window) in changes.windows(2).enumerate() {
                let current = window[0];

                let x1 = start_x + i as f64 * TIME_TICK_STEP;
                let x2 = start_x + (i + 1) as f64 * TIME_TICK_STEP;
                let width = (x2 - x1).max(20.0);

                // Прямоугольник состояния
                elements.push(LayoutElement {
                    id: format!("state_{}_{}_{}", participant_name, participant_idx, i),
                    bounds: Rect::new(x1, state_y, width, self.config.robust_state_height),
                    text: None,
                    properties: [("text-fill".to_string(), TIMING_TEXT_COLOR.to_string())]
                        .into_iter()
                        .collect(),
                    element_type: ElementType::Rectangle {
                        label: current.state.clone(),
                        corner_radius: 0.0,
                    },
                });
            }

            // Последнее состояние
            if let Some(last) = changes.last() {
                let x = start_x + changes.len().saturating_sub(1) as f64 * TIME_TICK_STEP;
                elements.push(LayoutElement {
                    id: format!("state_{}_last", participant_name),
                    bounds: Rect::new(x, state_y, 50.0, self.config.robust_state_height),
                    text: None,
                    properties: [("text-fill".to_string(), TIMING_TEXT_COLOR.to_string())]
                        .into_iter()
                        .collect(),
                    element_type: ElementType::Rectangle {
                        label: last.state.clone(),
                        corner_radius: 0.0,
                    },
                });
            }
        }
    }

    /// Рисует concise timeline (линии с переходами)
    #[allow(clippy::too_many_arguments)]
    fn draw_concise_timeline(
        &self,
        elements: &mut Vec<LayoutElement>,
        participant_idx: usize,
        participant_name: &str,
        lane_y: f64,
        start_x: f64,
        width: f64,
        _min_time: f64,
        changes: Option<&Vec<&StateChange>>,
    ) {
        let line_y = lane_y + self.config.lane_height / 2.0;

        // Базовая линия
        elements.push(LayoutElement {
            id: format!("baseline_{}_{}", participant_name, participant_idx),
            bounds: Rect::new(start_x, line_y - 1.0, width, 2.0),
            text: None,
            // Линии дорожек timing нарисованы цветом #333 (эталон).
            properties: [("stroke".to_string(), TIMING_TEXT_COLOR.to_string())]
                .into_iter()
                .collect(),
            element_type: ElementType::Edge {
                points: vec![
                    Point::new(start_x, line_y),
                    Point::new(start_x + width, line_y),
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

        // Метки состояний
        if let Some(changes) = changes {
            for (i, change) in changes.iter().enumerate() {
                let x = start_x + i as f64 * TIME_TICK_STEP;

                // Вертикальная линия перехода
                elements.push(LayoutElement {
                    id: format!("transition_{}_{}_{}", participant_name, participant_idx, i),
                    bounds: Rect::new(x - 1.0, line_y - 10.0, 2.0, 20.0),
                    text: None,
                    // Переход состояния: тёмно-зелёный толщиной 2 (эталон).
                    properties: [
                        ("stroke".to_string(), TIMING_TRANSITION_COLOR.to_string()),
                        ("stroke-width".to_string(), "2".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                    element_type: ElementType::Edge {
                        points: vec![Point::new(x, line_y - 10.0), Point::new(x, line_y + 10.0)],
                        label: None,
                        arrow_start: false,
                        arrow_end: false,
                        dashed: false,
                        edge_type: EdgeType::Link,
                        from_cardinality: None,
                        to_cardinality: None,
                    },
                });

                // Метка состояния.
                //
                // Ширина берётся по тексту: раньше здесь стояла константа
                // 50.0, и подпись «Обработка» вылезала за границы
                // диаграммы на 11px — то есть обрезалась бы.
                //
                // ВАЖНО: метки CONCISE-дорожек PlantUML рисует ЖИРНЫМИ
                // (`font-weight="700"`), а robust — обычными. Проверено
                // прямым замером: одно слово «Обработка» даёт textLength
                // 68.15 у robust и 75.088 у concise при одинаковом
                // font-size=12.
                let label_width = self
                    .config
                    .text
                    .width_bold(&change.state, self.config.label_font_size);
                // Жирность передаём свойством: `ElementType::Text` её не несёт,
                // а PlantUML рисует метки concise-дорожек полужирными.
                let mut properties = std::collections::HashMap::new();
                properties.insert("font-weight".to_string(), "700".to_string());
                properties.insert("text-fill".to_string(), TIMING_TEXT_COLOR.to_string());

                elements.push(LayoutElement {
                    id: format!("state_label_{}_{}_{}", participant_name, participant_idx, i),
                    bounds: Rect::new(x + STATE_LABEL_OFFSET, line_y - 20.0, label_width, 15.0),
                    text: None,
                    properties,
                    element_type: ElementType::Text {
                        text: change.state.clone(),
                        font_size: self.config.label_font_size,
                    },
                });
            }
        }
    }

    /// Рисует clock timeline (меандр)
    fn draw_clock_timeline(
        &self,
        elements: &mut Vec<LayoutElement>,
        participant_idx: usize,
        lane_y: f64,
        start_x: f64,
        width: f64,
    ) {
        let high_y = lane_y + 10.0;
        let low_y = lane_y + self.config.lane_height - 10.0;
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
        // Горизонтальная линия оси
        elements.push(LayoutElement {
            id: "time_axis".to_string(),
            bounds: Rect::new(start_x, y, width, 2.0),
            text: None,
            properties: [("stroke".to_string(), TIMING_TEXT_COLOR.to_string())]
                .into_iter()
                .collect(),
            element_type: ElementType::Edge {
                points: vec![Point::new(start_x, y), Point::new(start_x + width, y)],
                label: None,
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association,
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
        for (index, t) in times.iter().enumerate() {
            let t = *t;
            let x = start_x + index as f64 * TIME_TICK_STEP;

            // Деление — на 50 правее подписи этого события.
            let tick_x = x + TIME_TICK_STEP;
            elements.push(LayoutElement {
                id: format!("tick_{}", t),
                bounds: Rect::new(tick_x - 0.5, y, 1.0, 5.0),
                text: None,
                // Деления оси: цвет #333, толщина 2 (эталон).
                properties: [
                    ("stroke".to_string(), TIMING_TEXT_COLOR.to_string()),
                    ("stroke-width".to_string(), "2".to_string()),
                ]
                .into_iter()
                .collect(),
                element_type: ElementType::Edge {
                    points: vec![Point::new(tick_x, y), Point::new(tick_x, y + 5.0)],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });

            // Метка времени.
            //
            // Ширина измеряется, а не берётся константой 30: подпись
            // центрируется по делению, поэтому её правый край равен
            // `x + ширина / 2`. При константе 30 для «100» (реальная
            // ширина 20.996) правый край выходил на 15 вместо 10.5, и
            // метка вылезала за шкалу. Эталон: подпись «100» стоит на
            // x=131.24 при делении 141.732, то есть 141.732 − 20.996/2.
            let label_width = self
                .config
                .text
                .width(&format!("{}", t as i64), self.config.time_font_size);
            elements.push(LayoutElement {
                id: format!("time_label_{}", t as i64),
                bounds: Rect::new(x - label_width / 2.0, y + 8.0, label_width, 15.0),
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
