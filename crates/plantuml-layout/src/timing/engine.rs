//! Layout engine для Timing Diagrams
//!
//! Создаёт горизонтальную временную шкалу с вертикальными lanes для участников.

use std::collections::HashMap;

use plantuml_ast::timing::{ParticipantType, StateChange, TimeValue, TimingDiagram};
use plantuml_model::{Point, Rect};

use super::TimingLayoutConfig;
use crate::traits::LayoutResult;
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

        // 1. Собираем все времена для определения масштаба
        let (min_time, max_time) = self.calculate_time_range(diagram);
        let time_range = (max_time - min_time).max(100.0);

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
            changes_by_participant
                .entry(key)
                .or_default()
                .push(change);
        }

        // Сортируем изменения по времени
        for changes in changes_by_participant.values_mut() {
            changes.sort_by(|a, b| {
                a.time
                    .as_f64()
                    .partial_cmp(&b.time.as_f64())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // 4. Рисуем участников и их lanes
        let timeline_start_x = self.config.padding + self.config.participant_label_width;
        let timeline_width = time_range * self.config.time_scale;

        for (i, participant) in diagram.participants.iter().enumerate() {
            let lane_y = self.config.padding + (i as f64) * (self.config.lane_height + self.config.lane_spacing);

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
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
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
            + (diagram.participants.len() as f64) * (self.config.lane_height + self.config.lane_spacing);

        self.draw_time_axis(
            &mut elements,
            timeline_start_x,
            axis_y,
            timeline_width,
            min_time,
            max_time,
        );

        // 6. Title
        if let Some(ref title) = diagram.metadata.title {
            elements.push(LayoutElement {
                id: "title".to_string(),
                bounds: Rect::new(self.config.padding, 5.0, 500.0, 20.0),
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
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
        min_time: f64,
        changes: Option<&Vec<&StateChange>>,
    ) {
        let state_y = lane_y + (self.config.lane_height - self.config.robust_state_height) / 2.0;

        if let Some(changes) = changes {
            for (i, window) in changes.windows(2).enumerate() {
                let current = window[0];
                let next = window[1];

                let x1 = start_x + (current.time.as_f64() - min_time) * self.config.time_scale;
                let x2 = start_x + (next.time.as_f64() - min_time) * self.config.time_scale;
                let width = (x2 - x1).max(20.0);

                // Прямоугольник состояния
                elements.push(LayoutElement {
                    id: format!("state_{}_{}_{}", participant_name, participant_idx, i),
                    bounds: Rect::new(x1, state_y, width, self.config.robust_state_height),
                    text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
                        label: current.state.clone(),
                        corner_radius: 0.0,
                    },
                });
            }

            // Последнее состояние
            if let Some(last) = changes.last() {
                let x = start_x + (last.time.as_f64() - min_time) * self.config.time_scale;
                elements.push(LayoutElement {
                    id: format!("state_{}_last", participant_name),
                    bounds: Rect::new(x, state_y, 50.0, self.config.robust_state_height),
                    text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
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
        min_time: f64,
        changes: Option<&Vec<&StateChange>>,
    ) {
        let line_y = lane_y + self.config.lane_height / 2.0;

        // Базовая линия
        elements.push(LayoutElement {
            id: format!("baseline_{}_{}", participant_name, participant_idx),
            bounds: Rect::new(start_x, line_y - 1.0, width, 2.0),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                points: vec![
                    Point::new(start_x, line_y),
                    Point::new(start_x + width, line_y),
                ],
                label: None,
                arrow_start: false,
                arrow_end: false,
                dashed: false,
                edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
            },
        });

        // Метки состояний
        if let Some(changes) = changes {
            for (i, change) in changes.iter().enumerate() {
                let x = start_x + (change.time.as_f64() - min_time) * self.config.time_scale;

                // Вертикальная линия перехода
                elements.push(LayoutElement {
                    id: format!("transition_{}_{}_{}", participant_name, participant_idx, i),
                    bounds: Rect::new(x - 1.0, line_y - 10.0, 2.0, 20.0),
                    text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                        points: vec![
                            Point::new(x, line_y - 10.0),
                            Point::new(x, line_y + 10.0),
                        ],
                        label: None,
                        arrow_start: false,
                        arrow_end: false,
                        dashed: false,
                        edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
                    },
                });

                // Метка состояния
                elements.push(LayoutElement {
                    id: format!("state_label_{}_{}_{}", participant_name, participant_idx, i),
                    bounds: Rect::new(x + 5.0, line_y - 20.0, 50.0, 15.0),
                    text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
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
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                points,
                label: None,
                arrow_start: false,
                arrow_end: false,
                dashed: false,
                edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
            },
        });
    }

    /// Рисует временную ось
    fn draw_time_axis(
        &self,
        elements: &mut Vec<LayoutElement>,
        start_x: f64,
        y: f64,
        width: f64,
        min_time: f64,
        max_time: f64,
    ) {
        // Горизонтальная линия оси
        elements.push(LayoutElement {
            id: "time_axis".to_string(),
            bounds: Rect::new(start_x, y, width, 2.0),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                points: vec![
                    Point::new(start_x, y),
                    Point::new(start_x + width, y),
                ],
                label: None,
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association, from_cardinality: None, to_cardinality: None,
            },
        });

        // Деления и метки времени
        let time_range = max_time - min_time;
        let step = self.calculate_time_step(time_range);

        let mut t = (min_time / step).ceil() * step;
        while t <= max_time {
            let x = start_x + (t - min_time) * self.config.time_scale;

            // Деление
            elements.push(LayoutElement {
                id: format!("tick_{}", t),
                bounds: Rect::new(x - 0.5, y, 1.0, 5.0),
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                    points: vec![Point::new(x, y), Point::new(x, y + 5.0)],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
                },
            });

            // Метка времени
            elements.push(LayoutElement {
                id: format!("time_label_{}", t),
                bounds: Rect::new(x - 15.0, y + 8.0, 30.0, 15.0),
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
                    text: format!("{}", t as i64),
                    font_size: self.config.time_font_size,
                },
            });

            t += step;
        }
    }

    /// Вычисляет шаг времени для делений
    fn calculate_time_step(&self, range: f64) -> f64 {
        let target_ticks = 10.0;
        let rough_step = range / target_ticks;

        // Округляем до красивого числа
        let magnitude = 10.0_f64.powf(rough_step.log10().floor());
        let normalized = rough_step / magnitude;

        let nice_step = if normalized < 1.5 {
            1.0
        } else if normalized < 3.0 {
            2.0
        } else if normalized < 7.0 {
            5.0
        } else {
            10.0
        };

        nice_step * magnitude
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
        diagram.participants.push(TimingParticipant::robust("Browser"));
        diagram.participants.push(TimingParticipant::concise("Server"));

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
        diagram.state_changes.push(StateChange::new(
            "A",
            TimeValue::Absolute(50.0),
            "State1",
        ));
        diagram.state_changes.push(StateChange::new(
            "A",
            TimeValue::Absolute(200.0),
            "State2",
        ));

        let engine = TimingLayoutEngine::new();
        let (min, max) = engine.calculate_time_range(&diagram);

        assert_eq!(min, 50.0);
        assert_eq!(max, 200.0);
    }
}
