//! State Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм состояний.
//! Поддерживает вложенные (composite) состояния.

/// Зазор между двумя НАСТОЯЩИМИ состояниями в одном уровне.
///
/// Измерено на сервере: рамки на x=7 и x=92 по 50 шириной -> 35.
/// У `state_simple` в одном уровне стоят настоящее состояние и
/// псевдосостояние `[*]`, и там зазор 88.1 — см. `state_gap`.
const STATE_SIBLING_GAP: f64 = 35.0;

/// Отступ вокруг подписи состояния.
///
/// Измерено по эталону `State: Простой`: 41.9 -> 62, 76.4 -> 96,
/// 75.3 -> 95.
const STATE_WIDTH_PADDING: f64 = 20.0;

/// Минимальная ширина состояния: у `Idle` при тексте 25.5 рамка 50.
const STATE_MIN_WIDTH: f64 = 50.0;

/// Кегль подписи состояния (в эталоне `font-size="14"`).
const STATE_FONT_SIZE: f64 = 14.0;
const STATE_RIGHT_EXTRA: f64 = 3.0;
use plantuml_ast::common::Direction;
use plantuml_ast::state::{State, StateDiagram, StateType};
use plantuml_model::Rect;

use super::config::StateLayoutConfig;
use crate::graph::scene::{Scene, SceneEdge};
use crate::graph::Direction as GraphDirection;
use crate::{ElementType, LayoutElement, LayoutResult};

/// Layout engine для state diagrams
pub struct StateLayoutEngine {
    config: StateLayoutConfig,
}

/// Насколько конечный узел UML больше начального.
///
/// Эталон `state_simple`: начальный `[*]` — `rx=10`, конечный — `rx=11`.
const FINAL_NODE_RADIUS_EXTRA: f64 = 1.0;

/// Высота заголовка контейнера составного состояния.
const COMPOSITE_HEADER_HEIGHT: f64 = 30.0;

/// Кегль и межстрочный интервал описания состояния (`S1 : текст`).
const STATE_DESCRIPTION_FONT_SIZE: f64 = 12.0;
const STATE_DESCRIPTION_LINE: f64 = 16.3;
/// Полоса имени над разделителем и нижний отступ описания.
const STATE_HEADER_BAND: f64 = 26.3;
const STATE_DESCRIPTION_PAD: f64 = 8.0;

/// `S1[H]` / `S1[H*]` → (`S1`, глубокая ли история). Голый `[H]` не считается.
fn history_ref(name: &str) -> Option<(&str, bool)> {
    let (base, deep) = match name.strip_suffix("[H*]") {
        Some(base) => (base, true),
        None => (name.strip_suffix("[H]")?, false),
    };
    (!base.is_empty()).then_some((base, deep))
}

/// Имя состояния без суффикса истории.
fn endpoint_base(name: &str) -> &str {
    history_ref(name).map_or(name, |(base, _)| base)
}

/// Заметки из тела составного состояния, рекурсивно.
fn collect_notes<'a>(state: &'a State, out: &mut Vec<&'a plantuml_ast::common::Note>) {
    out.extend(state.notes.iter());
    for sub in &state.substates {
        collect_notes(sub, out);
    }
    for region in &state.regions {
        for sub in region {
            collect_notes(sub, out);
        }
    }
}

impl StateLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: StateLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: StateLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &StateDiagram) -> LayoutResult {
        let mut scene = Scene::new(
            STATE_SIBLING_GAP,
            self.config.vertical_spacing,
            self.config.margin,
            self.config_text(),
        );
        if diagram.metadata.direction == Some(Direction::LeftToRight) {
            scene.set_direction(GraphDirection::LeftToRight);
        }

        self.add_scope(
            &mut scene,
            None,
            "top",
            &diagram.states,
            &diagram.transitions,
        );

        let mut notes: Vec<&plantuml_ast::common::Note> = diagram.notes.iter().collect();
        for state in &diagram.states {
            collect_notes(state, &mut notes);
        }
        for note in notes {
            scene.add_note(note.clone());
        }

        let mut result = scene.build();
        // Поля сверх общих рендерера сняты с эталона `state_simple`.
        result.bounds.width += STATE_RIGHT_EXTRA;
        result.bounds.height += self.config.margin * 2.0;
        result
    }

    fn config_text(&self) -> crate::text::TextMeasurer {
        crate::text::TextMeasurer::default()
    }

    /// Раскладывает одну область: верхний уровень, тело составного состояния, регион.
    fn add_scope(
        &self,
        scene: &mut Scene,
        cluster: Option<usize>,
        scope: &str,
        states: &[State],
        transitions: &[plantuml_ast::state::Transition],
    ) {
        let initial = format!("{scope}#initial");
        let terminal = format!("{scope}#final");
        let mut has_initial = false;
        let mut has_final = false;
        for t in transitions {
            has_initial |= t.from == "[*]";
            has_final |= t.to == "[*]";
        }
        if has_initial {
            let (element, _) = self.create_initial_state(&initial, 0.0, 0.0);
            scene.add_node(cluster, &[initial.as_str()], vec![element]);
        }
        for state in states.iter().filter(|s| s.name != "[*]") {
            self.add_state(scene, cluster, state);
        }
        // Состояния, названные только в переходах.
        let declared_elsewhere =
            |name: &str, scene: &Scene| scene.knows(name) || scene.knows_cluster(name);
        for t in transitions {
            for raw in [&t.from, &t.to] {
                if raw == "[*]" {
                    continue;
                }
                let base = endpoint_base(raw);
                if !declared_elsewhere(base, scene) {
                    let (mut element, _) = self.create_simple_state(base, 0.0, 0.0, None);
                    element.bounds.x = 0.0;
                    scene.add_node(cluster, &[base], vec![element]);
                }
            }
        }
        // История — кружок внутри составного состояния.
        for t in transitions {
            for raw in [&t.from, &t.to] {
                let Some((base, deep)) = history_ref(raw) else {
                    continue;
                };
                if scene.knows(raw) {
                    continue;
                }
                let home = scene.cluster_named(base).or(cluster);
                let (element, _) = self.create_history_state(raw, 0.0, 0.0, deep);
                scene.add_node(home, &[raw.as_str()], vec![element]);
            }
        }
        if has_final {
            let (element, _) = self.create_final_state(&terminal, 0.0, 0.0);
            scene.add_node(cluster, &[terminal.as_str()], vec![element]);
        }

        for t in transitions {
            let end = |raw: &str, pseudo: &str| {
                if raw == "[*]" {
                    pseudo.to_string()
                } else {
                    raw.to_string()
                }
            };
            let mut edge = SceneEdge::new(end(&t.from, &initial), end(&t.to, &terminal), t.arrow);
            let label = t.label();
            edge.label = (!label.is_empty()).then_some(label);
            if let Some(color) = &t.color {
                edge.properties.insert("stroke".to_string(), color.to_css());
            }
            scene.add_edge(edge);
        }
    }

    /// Добавляет состояние: простое становится узлом, составное — кластером.
    fn add_state(&self, scene: &mut Scene, cluster: Option<usize>, state: &State) {
        let mut names = vec![state.name.as_str()];
        if let Some(alias) = &state.alias {
            names.push(alias);
        }
        // Составное состояние без содержимого рисуем обычным: рамке нечего охватывать.
        let is_empty = state.substates.is_empty()
            && state.regions.is_empty()
            && state.internal_transitions.is_empty();
        if state.state_type != StateType::Composite || is_empty {
            let (mut element, _) = self.create_state_element(
                &state.name,
                state.state_type,
                0.0,
                0.0,
                state.description.as_deref(),
            );
            element.bounds.x = 0.0;
            element.bounds.y = 0.0;
            scene.add_node(cluster, &names, vec![element]);
            return;
        }

        let measurer = self.config_text();
        let min_width =
            measurer.width_bold(&state.name, STATE_FONT_SIZE) + STATE_WIDTH_PADDING * 2.0;
        let inner = scene.add_cluster(
            cluster,
            &names,
            COMPOSITE_HEADER_HEIGHT,
            min_width,
            LayoutElement {
                id: format!("composite_{}", state.name),
                bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::CompositeState {
                    name: state.name.clone(),
                    header_height: COMPOSITE_HEADER_HEIGHT,
                },
            },
        );

        if state.regions.is_empty() {
            self.add_scope(
                scene,
                Some(inner),
                &state.name,
                &state.substates,
                &state.internal_transitions,
            );
            return;
        }

        // Первый регион разобран в `substates`, остальные лежат в `regions`.
        let regions: Vec<&[State]> = std::iter::once(state.substates.as_slice())
            .filter(|first| !first.is_empty())
            .chain(state.regions.iter().map(Vec::as_slice))
            .collect();
        // Регионы идут рядом; переход относится к региону своих концов.
        let region_of = |name: &str| {
            regions
                .iter()
                .position(|region| region.iter().any(|s| s.name == name))
        };
        for (index, region) in regions.iter().enumerate() {
            let region_cluster = scene.add_cluster(
                Some(inner),
                &[],
                0.0,
                0.0,
                LayoutElement {
                    id: format!("region_{}_{index}", state.name),
                    bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
                    text: None,
                    properties: [("cluster-shape".to_string(), "region".to_string())]
                        .into_iter()
                        .collect(),
                    element_type: ElementType::Group {
                        label: None,
                        children: Vec::new(),
                    },
                },
            );
            let transitions: Vec<plantuml_ast::state::Transition> = state
                .internal_transitions
                .iter()
                .filter(|t| {
                    let home = region_of(&t.from).or_else(|| region_of(&t.to)).unwrap_or(0);
                    home == index
                })
                .cloned()
                .collect();
            let scope = format!("{}#{index}", state.name);
            self.add_scope(scene, Some(region_cluster), &scope, region, &transitions);
        }
    }

    fn create_state_element(
        &self,
        name: &str,
        state_type: StateType,
        x: f64,
        y: f64,
        description: Option<&str>,
    ) -> (LayoutElement, Rect) {
        match state_type {
            StateType::Initial => self.create_initial_state(name, x, y),
            StateType::Final => self.create_final_state(name, x, y),
            StateType::Choice => self.create_choice_state(name, x, y),
            StateType::Fork | StateType::Join => self.create_fork_join_state(name, x, y),
            StateType::History => self.create_history_state(name, x, y, false),
            StateType::DeepHistory => self.create_history_state(name, x, y, true),
            StateType::Composite => self.create_simple_state(name, x, y, description), // Handled separately
            _ => self.create_simple_state(name, x, y, description),
        }
    }

    /// Создаёт начальное состояние
    fn create_initial_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let r = self.config.node_radius;
        // x уже указывает на левый край области для элемента
        // Центр должен быть в середине выделенной ширины (node_radius * 2)
        let cx = x + r;
        let cy = y + r;

        let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);

        (
            LayoutElement {
                id: format!("initial_{}", name.replace(['[', ']', '*', '_'], "")),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::InitialState,
            },
            bounds,
        )
    }

    /// Создаёт конечное состояние
    fn create_final_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        // Конечный узел на 1px больше начального — как в PlantUML
        // (эталон `state_simple`: начальный `rx=10`, конечный `rx=11`).
        let r = self.config.node_radius + FINAL_NODE_RADIUS_EXTRA;
        // x уже указывает на левый край области для элемента.
        // Центр остаётся в середине ВЫДЕЛЕННОЙ ячейки (`node_radius * 2`),
        // иначе узел на 1px больше сдвигал бы диаграмму вправо-вниз.
        let cx = x + self.config.node_radius;
        let cy = y + self.config.node_radius;

        let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);

        (
            LayoutElement {
                id: format!("final_{}", name.replace(['[', ']', '*', '_'], "")),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::FinalState,
            },
            bounds,
        )
    }

    /// Ширина состояния по длине имени.
    ///
    /// Измерено по эталону PlantUML
    /// (tests/golden/reference/state_simple.svg): «Active» (6 символов) —
    /// 63.552, «Inactive» (8) — 75.556. Отсюда ширина ≈ 27.54 + 6.002 * n.
    /// Раньше ширина была фиксированной (120), из-за чего диаграмма
    /// получалась заметно шире эталона.
    /// Ширина состояния: ЗАМЕР текста плюс отступ.
    ///
    /// Прежняя формула `27.54 + 6.002 * число_символов` была ОЦЕНКОЙ по
    /// длине, а не замером. На `state_simple` она совпадала с эталоном
    /// («Active» 6 символов и «Inactive» 8 — похожие слова), но на
    /// `State: Простой` отступ выходил 26, 13.5, 5.1 и 22.1 при
    /// постоянном в эталоне.
    ///
    /// Измерено по эталону `State: Простой`:
    ///   Idle 25.5 -> 50 (упёрлось в минимум), Failed 41.9 -> 62,
    ///   Completed 76.4 -> 96, Processing 75.3 -> 95,
    /// то есть `max(текст + 20, 50)`.
    fn state_natural_width(&self, name: &str) -> f64 {
        let text = crate::text::TextMeasurer::default().width(name, STATE_FONT_SIZE);
        (text + STATE_WIDTH_PADDING).max(STATE_MIN_WIDTH)
    }

    /// Создаёт простое состояние
    fn create_simple_state(
        &self,
        name: &str,
        x: f64,
        y: f64,
        description: Option<&str>,
    ) -> (LayoutElement, Rect) {
        let (width, height) = self.state_box_size(name, description);
        let bounds = Rect::new(x, y, width, height);

        (
            LayoutElement {
                id: format!("state_{}", name),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::State {
                    name: name.to_string(),
                    description: description.map(str::to_string),
                },
            },
            bounds,
        )
    }

    /// Размер состояния: имя и строки описания.
    ///
    /// Без описания — замеры эталона (`max(текст + 20, 50)` на 50). Описание
    /// добавляет строки под разделителем, рамка растёт по ним.
    fn state_box_size(&self, name: &str, description: Option<&str>) -> (f64, f64) {
        let mut width = self.state_natural_width(name);
        let mut height = self.config.state_min_height;
        if let Some(text) = description {
            let measurer = crate::text::TextMeasurer::default();
            let lines: Vec<&str> = text.lines().collect();
            for line in &lines {
                width = width
                    .max(measurer.width(line, STATE_DESCRIPTION_FONT_SIZE) + STATE_WIDTH_PADDING);
            }
            height = height.max(
                STATE_HEADER_BAND
                    + lines.len() as f64 * STATE_DESCRIPTION_LINE
                    + STATE_DESCRIPTION_PAD,
            );
        }
        (width, height)
    }

    /// Раскладывает заметки колонкой правее схемы.
    ///
    /// Заметка привязана к состоянию по имени: колонка выравнивается по его
    /// высоте, пунктир соединяет её с состоянием. Заметки ниже друг друга не
    /// накладываются.
    fn create_choice_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let size = self.config.choice_size;
        let cx = x + self.config.state_width / 2.0;
        let cy = y + size / 2.0;

        let bounds = Rect::new(cx - size / 2.0, cy - size / 2.0, size, size);

        (
            LayoutElement {
                id: format!("choice_{}", name),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Text {
                    text: "◇".to_string(),
                    font_size: 16.0,
                },
            },
            bounds,
        )
    }

    /// Создаёт fork/join bar
    fn create_fork_join_state(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let cx = x + self.config.state_width / 2.0;
        let bounds = Rect::new(
            cx - self.config.bar_width / 2.0,
            y,
            self.config.bar_width,
            self.config.bar_height,
        );

        (
            LayoutElement {
                id: format!("bar_{}", name),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 0.0,
                },
            },
            bounds,
        )
    }

    /// Создаёт history state
    fn create_history_state(
        &self,
        name: &str,
        x: f64,
        y: f64,
        deep: bool,
    ) -> (LayoutElement, Rect) {
        let r = self.config.node_radius * 0.8;
        let cx = x + self.config.state_width / 2.0;
        let cy = y + r;

        let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);

        let label = if deep { "H*" } else { "H" };

        (
            LayoutElement {
                id: format!("history_{}", name.replace(['[', ']', '*'], "")),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Ellipse {
                    label: Some(label.to_string()),
                },
            },
            bounds,
        )
    }
}

impl Default for StateLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::state::Transition;

    #[test]
    fn test_layout_simple_state_machine() {
        let mut diagram = StateDiagram::new();
        diagram.add_transition(Transition::new("[*]", "Active"));
        diagram.add_transition(Transition::new("Active", "Inactive").with_event("timeout"));
        diagram.add_transition(Transition::new("Inactive", "[*]"));

        let engine = StateLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert!(result.elements.len() >= 6);
    }

    #[test]
    fn test_layout_composite_state() {
        let mut diagram = StateDiagram::new();

        // Создаём composite состояние
        let mut composite = State::composite("Active");
        composite
            .internal_transitions
            .push(Transition::new("[*]", "Idle"));
        composite
            .internal_transitions
            .push(Transition::new("Idle", "Running").with_event("start"));
        composite
            .internal_transitions
            .push(Transition::new("Running", "Paused").with_event("pause"));
        composite
            .internal_transitions
            .push(Transition::new("Paused", "Running").with_event("resume"));
        composite
            .internal_transitions
            .push(Transition::new("Running", "Idle").with_event("stop"));

        diagram.add_state(composite);
        diagram.add_transition(Transition::new("[*]", "Active"));
        diagram.add_transition(Transition::new("Active", "Inactive").with_event("disable"));
        diagram.add_transition(Transition::new("Inactive", "Active").with_event("enable"));
        diagram.add_transition(Transition::new("Inactive", "[*]").with_event("delete"));

        let engine = StateLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должен быть composite контейнер с внутренними элементами
        let composite_elements: Vec<_> = result
            .elements
            .iter()
            .filter(|e| e.id.contains("Active"))
            .collect();

        assert!(
            !composite_elements.is_empty(),
            "Должны быть элементы для Active"
        );

        // Inactive НЕ должен быть внутри Active (проверяем, что нет элементов с prefix Active_inner)
        // Правильный паттерн: "Active_inner_state_Inactive" или "Active_inner_trans_..._Inactive"
        let inactive_in_active = result
            .elements
            .iter()
            .any(|e| e.id.starts_with("Active_inner_") && e.id.contains("Inactive"));

        assert!(!inactive_in_active, "Inactive не должен быть внутри Active");
    }
    /// Заметка, описание и история попадают в раскладку.
    #[test]
    fn test_notes_descriptions_and_history_are_laid_out() {
        let mut diagram = StateDiagram::new();
        let mut described = State::new("A");
        described.description = Some("первая\nвторая".to_string());
        diagram.add_state(described);
        diagram.add_state(State::composite("S1"));
        diagram.add_transition(Transition::new("A", "S1[H]"));
        diagram.notes.push(plantuml_ast::common::Note {
            position: plantuml_ast::common::NotePosition::Right,
            text: "строка 1\nстрока 2".to_string(),
            anchors: vec!["A".to_string()],
            background_color: None,
        });

        let result = StateLayoutEngine::new().layout(&diagram);
        let has_text = |needle: &str| {
            result.elements.iter().any(|e| match &e.element_type {
                ElementType::Note { label, .. } => label.contains(needle),
                ElementType::State { description, .. } => {
                    description.as_deref().is_some_and(|d| d.contains(needle))
                }
                _ => false,
            })
        };
        assert!(
            has_text("строка 1") && has_text("строка 2"),
            "заметка потеряна"
        );
        assert!(has_text("вторая"), "описание потеряно");
        assert!(
            result.elements.iter().any(
                |e| matches!(&e.element_type, ElementType::Ellipse { label: Some(l) } if l == "H")
            ),
            "круг истории не нарисован"
        );
    }
}
