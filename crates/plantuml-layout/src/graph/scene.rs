//! Перевод раскладки графа в элементы диаграммы.
//!
//! Движок описывает фигуры готовыми элементами с нулевым положением, связи и заметки
//! обычными значениями, а сцена раскладывает их и расставляет. Так типы диаграмм делят
//! один путь: узлы, рамки кластеров, подписи рёбер, заметки.

use std::collections::HashMap;

use plantuml_ast::common::{ArrowShape, Note, NotePosition};
use plantuml_model::{Point, Rect};

use super::{layout, Direction, GraphCluster, GraphEdge, GraphInput, GraphNode};
use crate::text::TextMeasurer;
use crate::{EdgeType, ElementType, LayoutElement, LayoutResult};

const NOTE_FOLD: f64 = 10.0;
const NOTE_PADDING_X: f64 = 20.0;
const NOTE_PADDING_Y: f64 = 14.0;
const NOTE_LINE_HEIGHT: f64 = 16.0;
const NOTE_FILL: &str = "#FEFFDD";
const LABEL_FONT: f64 = 13.0;
/// Высота строки подписи ребра вместе с зазором: шаг рангов в эталоне 77 при 60 без подписи.
const LABEL_LINE: f64 = 17.0;

/// Связь между именованными фигурами.
#[derive(Debug, Clone)]
pub struct SceneEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub arrow: ArrowShape,
    pub dashed: bool,
    pub edge_type: EdgeType,
    pub properties: HashMap<String, String>,
    /// Идентификатор элемента; по умолчанию порядковый
    pub id: Option<String>,
    pub from_cardinality: Option<String>,
    pub to_cardinality: Option<String>,
    /// Влияет на раскладку, но не рисуется: регионы состояний друг под другом
    pub hidden: bool,
}

impl SceneEdge {
    /// Связь с обычной стрелкой.
    pub fn new(from: impl Into<String>, to: impl Into<String>, arrow: ArrowShape) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: None,
            arrow,
            dashed: false,
            edge_type: EdgeType::Association,
            properties: HashMap::new(),
            id: None,
            from_cardinality: None,
            to_cardinality: None,
            hidden: false,
        }
    }
}

/// Рамка кластера с готовым элементом.
#[derive(Debug, Clone)]
struct SceneCluster {
    element: LayoutElement,
}

/// Описание диаграммы для раскладки.
pub struct Scene {
    input: GraphInput,
    templates: Vec<Vec<LayoutElement>>,
    clusters: Vec<SceneCluster>,
    names: HashMap<String, usize>,
    cluster_names: HashMap<String, usize>,
    edges: Vec<SceneEdge>,
    notes: Vec<Note>,
    text: TextMeasurer,
}

impl Scene {
    /// Создаёт сцену с зазорами и полем кластера.
    pub fn new(node_sep: f64, rank_sep: f64, cluster_margin: f64, text: TextMeasurer) -> Self {
        Self {
            input: GraphInput {
                node_sep,
                rank_sep,
                cluster_margin,
                ..GraphInput::default()
            },
            templates: Vec::new(),
            clusters: Vec::new(),
            names: HashMap::new(),
            cluster_names: HashMap::new(),
            edges: Vec::new(),
            notes: Vec::new(),
            text,
        }
    }

    /// Направление раскладки.
    pub fn set_direction(&mut self, direction: Direction) {
        self.input.direction = direction;
    }

    /// Добавляет рамку. `element` описывает её вид, координаты заполнит сцена.
    pub fn add_cluster(
        &mut self,
        parent: Option<usize>,
        names: &[&str],
        title_height: f64,
        min_width: f64,
        element: LayoutElement,
    ) -> usize {
        let index = self.clusters.len();
        self.input.clusters.push(GraphCluster {
            parent,
            title_height,
            min_width,
        });
        self.clusters.push(SceneCluster { element });
        for name in names {
            self.cluster_names.insert((*name).to_string(), index);
        }
        index
    }

    /// Добавляет фигуру: первый элемент задаёт размер, остальные едут вместе с ним.
    pub fn add_node(
        &mut self,
        cluster: Option<usize>,
        names: &[&str],
        elements: Vec<LayoutElement>,
    ) -> usize {
        let index = self.input.nodes.len();
        let (width, height) = elements
            .first()
            .map_or((1.0, 1.0), |e| (e.bounds.width, e.bounds.height));
        self.input.nodes.push(GraphNode {
            width,
            height,
            cluster,
        });
        self.templates.push(elements);
        for name in names {
            self.names.insert((*name).to_string(), index);
        }
        index
    }

    /// Фигура с таким именем уже есть.
    pub fn knows(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }

    /// Кластер с таким именем уже есть.
    pub fn knows_cluster(&self, name: &str) -> bool {
        self.cluster_names.contains_key(name)
    }

    /// Индекс кластера по имени.
    pub fn cluster_named(&self, name: &str) -> Option<usize> {
        self.cluster_names.get(name).copied()
    }

    /// Добавляет связь.
    pub fn add_edge(&mut self, edge: SceneEdge) {
        self.edges.push(edge);
    }

    /// Добавляет заметку.
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    /// Первая фигура кластера: связь с рамкой идёт через неё.
    fn endpoint(&self, name: &str) -> Option<usize> {
        if let Some(index) = self.names.get(name) {
            return Some(*index);
        }
        let cluster = *self.cluster_names.get(name)?;
        let path_contains = |mut c: Option<usize>| {
            let mut guard = 0;
            while let Some(id) = c {
                if id == cluster {
                    return true;
                }
                c = self.input.clusters.get(id).and_then(|cl| cl.parent);
                guard += 1;
                if guard > self.input.clusters.len() {
                    break;
                }
            }
            false
        };
        self.input
            .nodes
            .iter()
            .position(|n| path_contains(n.cluster))
    }

    /// Имя относится к рамке, а не к узлу.
    fn frame_of(&self, name: &str) -> Option<usize> {
        if self.names.contains_key(name) {
            return None;
        }
        self.cluster_names.get(name).copied()
    }

    /// Узел лежит в кластере или во вложенном в него.
    fn contains(&self, cluster: usize, node: usize) -> bool {
        let mut current = self.input.nodes.get(node).and_then(|n| n.cluster);
        let mut guard = 0;
        while let Some(c) = current {
            if c == cluster {
                return true;
            }
            current = self.input.clusters.get(c).and_then(|cl| cl.parent);
            guard += 1;
            if guard > self.input.clusters.len() {
                break;
            }
        }
        false
    }

    fn note_element(&self, note: &Note, font_size: f64) -> LayoutElement {
        let lines: Vec<String> = note
            .text
            .replace("\\n", "\n")
            .lines()
            .map(str::to_string)
            .collect();
        let width = lines
            .iter()
            .map(|line| self.text.width(line, font_size))
            .fold(0.0, f64::max)
            + NOTE_PADDING_X;
        let height = lines.len().max(1) as f64 * NOTE_LINE_HEIGHT + NOTE_PADDING_Y;
        let mut properties = HashMap::new();
        properties.insert("fill".to_string(), NOTE_FILL.to_string());
        properties.insert("text-align".to_string(), "left".to_string());
        properties.insert("text-inset".to_string(), "6".to_string());
        properties.insert("font-size".to_string(), font_size.to_string());
        LayoutElement {
            id: "note".to_string(),
            bounds: Rect::new(0.0, 0.0, width, height),
            text: None,
            properties,
            element_type: ElementType::Note {
                label: lines.join("\n"),
                fold: NOTE_FOLD,
            },
        }
    }

    /// Размер подписи.
    fn label_size(&self, label: &str) -> (f64, f64) {
        let text = label.replace("\\n", "\n");
        let lines = text.lines().count().max(1) as f64;
        (self.text.width(&text, LABEL_FONT) + 4.0, lines * LABEL_LINE)
    }

    /// Раскладывает сцену в элементы диаграммы.
    pub fn build(mut self) -> LayoutResult {
        // Заметки — обычные узлы с пунктирной связью.
        let notes = std::mem::take(&mut self.notes);
        let mut note_edges: Vec<GraphEdge> = Vec::new();
        for (index, note) in notes.iter().enumerate() {
            let mut element = self.note_element(note, LABEL_FONT);
            element.id = format!("note_{index}");
            let cluster = note
                .anchors
                .iter()
                .find_map(|a| self.names.get(a))
                .and_then(|n| self.input.nodes[*n].cluster);
            let id = self.add_node(cluster, &[], vec![element]);
            for anchor in &note.anchors {
                let Some(target) = self.endpoint(anchor) else {
                    continue;
                };
                let (from, to, same_rank) = match note.position {
                    NotePosition::Left => (id, target, true),
                    NotePosition::Right => (target, id, true),
                    NotePosition::Top => (id, target, false),
                    NotePosition::Bottom => (target, id, false),
                    _ => (target, id, true),
                };
                note_edges.push(GraphEdge {
                    from,
                    to,
                    same_rank,
                    min_rank: if same_rank { 0 } else { 1 },
                    weight: 1,
                    ..GraphEdge::default()
                });
            }
        }

        let mut drawn: Vec<usize> = Vec::new();
        for (index, edge) in self.edges.iter().enumerate() {
            let (Some(from), Some(to)) = (self.endpoint(&edge.from), self.endpoint(&edge.to))
            else {
                continue;
            };
            let mut graph_edge = GraphEdge::from_arrow(from, to, edge.arrow);
            graph_edge.label = edge.label.as_deref().map(|l| self.label_size(l));
            // Концы связи с рамкой. Связь рамки со своим содержимым остаётся обычной.
            let source_frame = self.frame_of(&edge.from).filter(|c| !self.contains(*c, to));
            let target_frame = self.frame_of(&edge.to).filter(|c| !self.contains(*c, from));
            if graph_edge.reverse_path {
                graph_edge.from_cluster = target_frame;
                graph_edge.to_cluster = source_frame;
            } else {
                graph_edge.from_cluster = source_frame;
                graph_edge.to_cluster = target_frame;
            }
            self.input.edges.push(graph_edge);
            drawn.push(index);
        }
        let plain_edge_count = self.input.edges.len();
        self.input.edges.extend(note_edges);

        let result = layout(&self.input);
        let mut elements: Vec<LayoutElement> = Vec::new();

        // Внешние рамки рисуются раньше вложенных.
        let depth = |mut c: usize| {
            let mut d = 0;
            while let Some(parent) = self.input.clusters.get(c).and_then(|cl| cl.parent) {
                d += 1;
                c = parent;
                if d > self.input.clusters.len() {
                    break;
                }
            }
            d
        };
        let mut order: Vec<usize> = (0..self.clusters.len()).collect();
        order.sort_by_key(|c| depth(*c));
        for c in order {
            let boxed = result.clusters[c];
            let mut element = self.clusters[c].element.clone();
            element.bounds = Rect::new(boxed.x, boxed.y, boxed.width, boxed.height);
            elements.push(element);
        }

        for (index, template) in self.templates.iter().enumerate() {
            let boxed = result.nodes[index];
            for element in template {
                let mut element = element.clone();
                element.bounds.x += boxed.x;
                element.bounds.y += boxed.y;
                // Составная фигура: у её линий координаты абсолютные.
                if let ElementType::Edge { points, .. } = &mut element.element_type {
                    for point in points.iter_mut() {
                        point.x += boxed.x;
                        point.y += boxed.y;
                    }
                }
                elements.push(element);
            }
        }

        let edge_element = |id: String, path: &[(f64, f64)], edge: Option<&SceneEdge>| {
            let points: Vec<Point> = path.iter().map(|(x, y)| Point::new(*x, *y)).collect();
            let min_x = path.iter().map(|p| p.0).fold(f64::MAX, f64::min);
            let min_y = path.iter().map(|p| p.1).fold(f64::MAX, f64::min);
            let max_x = path.iter().map(|p| p.0).fold(f64::MIN, f64::max);
            let max_y = path.iter().map(|p| p.1).fold(f64::MIN, f64::max);
            LayoutElement {
                id,
                bounds: Rect::new(
                    min_x,
                    min_y,
                    (max_x - min_x).max(1.0),
                    (max_y - min_y).max(1.0),
                ),
                text: None,
                properties: {
                    let mut properties: HashMap<String, String> =
                        edge.map(|e| e.properties.clone()).unwrap_or_default();
                    properties.insert("curve".to_string(), "1".to_string());
                    properties.insert("dasharray".to_string(), "7,7".to_string());
                    properties
                },
                element_type: ElementType::Edge {
                    points,
                    label: None,
                    arrow_start: edge.is_some_and(|e| e.arrow.head_from),
                    arrow_end: edge.is_some_and(|e| e.arrow.head_to),
                    dashed: edge.is_none_or(|e| e.dashed),
                    edge_type: edge.map_or(EdgeType::Link, |e| e.edge_type),
                    from_cardinality: edge.and_then(|e| e.from_cardinality.clone()),
                    to_cardinality: edge.and_then(|e| e.to_cardinality.clone()),
                },
            }
        };

        for (graph_index, &edge_index) in drawn.iter().enumerate() {
            let edge = &self.edges[edge_index];
            let path = &result.edges[graph_index];
            if path.len() < 2 || edge.hidden {
                continue;
            }
            let id = edge
                .id
                .clone()
                .unwrap_or_else(|| format!("edge_{edge_index}"));
            elements.push(edge_element(id, path, Some(edge)));
            if let (Some(label), Some(center)) = (&edge.label, result.labels[graph_index]) {
                let (width, height) = self.label_size(label);
                elements.push(LayoutElement {
                    id: format!("edge_label_{edge_index}"),
                    bounds: Rect::new(
                        center.0 - width / 2.0 + 2.0,
                        center.1 - height / 2.0,
                        width,
                        height,
                    ),
                    text: None,
                    properties: HashMap::new(),
                    element_type: ElementType::Text {
                        text: label.clone(),
                        font_size: LABEL_FONT,
                    },
                });
            }
        }
        for (offset, path) in result.edges[plain_edge_count..].iter().enumerate() {
            if path.len() >= 2 {
                elements.push(edge_element(format!("note_link_{offset}"), path, None));
            }
        }

        let mut output = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, result.width, result.height),
        };
        output.calculate_bounds();
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(id: &str, w: f64, h: f64) -> LayoutElement {
        LayoutElement {
            id: id.to_string(),
            bounds: Rect::new(0.0, 0.0, w, h),
            text: None,
            properties: HashMap::new(),
            element_type: ElementType::Rectangle {
                label: id.to_string(),
                corner_radius: 0.0,
            },
        }
    }

    fn edge(from: &str, to: &str, label: Option<&str>) -> SceneEdge {
        let mut edge = SceneEdge::new(from, to, ArrowShape::default());
        edge.label = label.map(str::to_string);
        edge
    }

    fn scene() -> Scene {
        Scene::new(30.0, 60.0, 12.0, TextMeasurer::default())
    }

    #[test]
    fn test_nodes_edges_and_labels_are_emitted() {
        let mut s = scene();
        s.add_node(None, &["A"], vec![rect("A", 50.0, 30.0)]);
        s.add_node(None, &["B"], vec![rect("B", 50.0, 30.0)]);
        s.add_edge(edge("A", "B", Some("подпись")));
        let out = s.build();
        assert!(out.elements.iter().any(|e| e.id == "edge_0"));
        assert!(out.elements.iter().any(
            |e| matches!(&e.element_type, ElementType::Text { text, .. } if text == "подпись")
        ));
    }

    #[test]
    fn test_note_is_placed_beside_target_with_link() {
        let mut s = scene();
        s.add_node(None, &["A"], vec![rect("A", 50.0, 30.0)]);
        s.add_note(Note {
            position: NotePosition::Right,
            text: "текст".to_string(),
            anchors: vec!["A".to_string()],
            background_color: None,
        });
        let out = s.build();
        let a = out.elements.iter().find(|e| e.id == "A").map(|e| e.bounds);
        let n = out
            .elements
            .iter()
            .find(|e| e.id == "note_0")
            .map(|e| e.bounds);
        let (Some(a), Some(n)) = (a, n) else {
            panic!("нет элементов: {:?}", out.elements);
        };
        assert!(n.x >= a.x + a.width);
        assert!(out.elements.iter().any(|e| e.id == "note_link_0"));
    }

    #[test]
    fn test_edge_to_cluster_uses_its_member() {
        let mut s = scene();
        let c = s.add_cluster(None, &["P"], 20.0, 0.0, rect("P", 0.0, 0.0));
        s.add_node(Some(c), &["A"], vec![rect("A", 50.0, 30.0)]);
        s.add_node(None, &["B"], vec![rect("B", 50.0, 30.0)]);
        s.add_edge(edge("B", "P", None));
        assert!(s.build().elements.iter().any(|e| e.id == "edge_0"));
    }

    #[test]
    fn test_unknown_endpoint_is_skipped() {
        let mut s = scene();
        s.add_node(None, &["A"], vec![rect("A", 50.0, 30.0)]);
        s.add_edge(edge("A", "нет", None));
        assert!(!s.build().elements.iter().any(|e| e.id == "edge_0"));
    }
}
