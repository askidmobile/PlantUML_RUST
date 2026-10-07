//! Layout engine для JSON диаграмм
//!
//! PlantUML раскладывает JSON не как дерево вложенных блоков, а как ГРАФ:
//! каждая таблица (объект или массив) — отдельный узел, а строка со
//! значением-контейнером даёт ребро «родитель → потомок» (порт `P{номер
//! строки}`). Расстановкой узлов занимается Graphviz dot: в PlantUML это его
//! Java-порт — `jsondiagram/SmetanaForJson.java` строит граф с `shape=record`
//! и вызывает `gvLayoutJobs`.
//!
//! Отсюда два следствия, которые и определяют раскладку:
//!
//! 1. Вложенная таблица НЕ удлиняет родительскую. Высота таблицы всегда
//!    `число строк x line_height`; эталон `JSON: Диаграмма` даёт контейнер
//!    `user` высотой 101.484 = 5 x 20.297 при общей высоте холста 251.
//! 2. Узлы одного уровня (ранга) стоят на ОДНОЙ координате оси ранга
//!    (в SVG это `x`), а поперечную координату (`y`) выбирает решатель dot:
//!    он минимизирует суммарную длину рёбер `Σ |y_потомок − y_порта_родителя|`
//!    при ограничении «соседи по рангу не пересекаются».

use std::collections::VecDeque;

use plantuml_ast::json::{JsonDiagram, JsonNode, JsonValue};
use plantuml_model::{Point, Rect};

use crate::json::config::JsonLayoutConfig;
use crate::traits::{LayoutEngine, LayoutResult};
use crate::{EdgeType, ElementType, LayoutConfig, LayoutElement};

/// Layout engine для JSON/YAML диаграмм
pub struct JsonLayoutEngine {
    config: JsonLayoutConfig,
}

/// Одна таблица раскладки (объект или массив JSON).
struct TableNode {
    /// Строки «ключ — значение».
    entries: Vec<(Option<String>, JsonValue)>,
    /// Дочерние таблицы: `(номер строки, индекс узла)`.
    children: Vec<(usize, usize)>,
    /// Родительская таблица.
    parent: Option<usize>,
    /// Номер строки родителя, из которой выходит ребро.
    parent_row: usize,
    /// Ширина колонки ключей.
    key_width: f64,
    /// Ширина колонки значений.
    value_width: f64,
    /// Ранг (глубина вложенности), он же уровень по оси ранга.
    rank: usize,
    /// Центр по оси ранга (в SVG это `x`).
    center_x: f64,
    /// Центр по поперечной оси в координатах dot (в SVG это `y`).
    center_y: f64,
}

impl TableNode {
    /// Ширина таблицы — она же протяжённость узла по оси ранга.
    fn width(&self) -> f64 {
        self.key_width + self.value_width
    }

    /// Высота таблицы — она же протяжённость узла по поперечной оси.
    fn height(&self) -> f64 {
        self.entries.len() as f64 * LINE_HEIGHT
    }

    /// Протяжённость узла по поперечной оси в раскладке dot.
    ///
    /// Узел записи в dot чуть выше нарисованной таблицы: измерено по эталону
    /// `JSON: Диаграмма`, где соседи по рангу стоят ровно через 59.0 =
    /// (20.297 + 0.406) + 18.0 (высота строки + запас + nodesep).
    fn cross_extent(&self) -> f64 {
        self.height() + CROSS_EXTENT_EXTRA
    }

    /// Смещение порта строки относительно центра узла.
    ///
    /// Порт (в разметке записи dot это `<P{i}>`) стоит в середине своей
    /// строки. Шаг портов НЕ равен высоте строки таблицы: dot раскладывает
    /// поля подписи по своим метрикам, и шаг выходит ровно 20.0. Измерено по
    /// двум эталонам с сервера: у таблицы из четырёх строк порт нулевой
    /// строки ставит потомка на 40.0 выше, чем порт второй строки, — это
    /// ровно `2 x 20.0`.
    fn port_offset(&self, row: usize) -> f64 {
        (row as f64 - (self.entries.len() as f64 - 1.0) / 2.0) * PORT_PITCH
    }
}

impl JsonLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: JsonLayoutConfig::default(),
        }
    }

    /// Создаёт engine с указанной конфигурацией
    pub fn with_config(config: JsonLayoutConfig) -> Self {
        Self { config }
    }

    /// Собирает таблицы JSON в узлы графа раскладки.
    ///
    /// Обход префиксный: корень получает индекс 0, дальше — потомки в порядке
    /// строк. Этот же порядок PlantUML использует при отрисовке.
    fn collect_tables(
        &self,
        node: &JsonNode,
        rank: usize,
        parent: Option<(usize, usize)>,
        out: &mut Vec<TableNode>,
    ) -> Option<usize> {
        let entries = self.table_entries(node);
        if entries.is_empty() {
            return None;
        }

        let key_width = entries
            .iter()
            .filter_map(|(key, _)| key.as_deref())
            .map(|key| self.config.text.width_bold(key, self.config.font_size))
            .fold(0.0_f64, f64::max)
            + TABLE_CELL_PADDING;

        let value_width = entries
            .iter()
            .map(|(_, value)| self.cell_text(value))
            .map(|text| self.config.text.width(&text, self.config.font_size))
            .fold(0.0_f64, f64::max)
            + TABLE_CELL_PADDING;

        // Если ключей нет (массив), таблица состоит из одной колонки.
        //
        // Ширина определяется содержимым: в эталоне таблица вложенного
        // массива со значениями «a» и «b» занимает 18.887, то есть по одному
        // символу плюс 10.
        let (key_width, value_width) = if key_width <= TABLE_CELL_PADDING {
            (0.0, value_width)
        } else {
            (key_width, value_width)
        };

        let index = out.len();
        out.push(TableNode {
            entries: entries.clone(),
            children: Vec::new(),
            parent: parent.map(|(_, parent_index)| parent_index),
            parent_row: parent.map(|(row, _)| row).unwrap_or(0),
            key_width,
            value_width,
            rank,
            center_x: 0.0,
            center_y: 0.0,
        });

        for (row, (_, value)) in entries.iter().enumerate() {
            let child_node = match value {
                JsonValue::Object(children) => Some(JsonNode {
                    key: None,
                    value: JsonValue::Object(children.clone()),
                    collapsed: false,
                    highlighted: false,
                }),
                JsonValue::Array(items) => Some(JsonNode {
                    key: None,
                    value: JsonValue::Array(items.clone()),
                    collapsed: false,
                    highlighted: false,
                }),
                _ => None,
            };
            if let Some(child_node) = child_node {
                if let Some(child) =
                    self.collect_tables(&child_node, rank + 1, Some((row, index)), out)
                {
                    out[index].children.push((row, child));
                }
            }
        }

        Some(index)
    }

    /// Кладёт узлы по оси рангов.
    ///
    /// Узлы одного ранга центрируются на общей координате, а соседние ранги
    /// раздвигаются на половину протяжённости самого крупного узла каждого
    /// ранга плюс `ranksep`. Единственный узел нулевого ранга (корень)
    /// прижат к левому отступу холста — так же, как в эталоне.
    fn assign_rank_axis(&self, nodes: &mut [TableNode]) {
        let max_rank = nodes.iter().map(|node| node.rank).max().unwrap_or(0);
        let mut rank_width = vec![0.0_f64; max_rank + 1];
        for node in nodes.iter() {
            rank_width[node.rank] = rank_width[node.rank].max(node.width());
        }

        let mut center = self.config.padding + rank_width[0] / 2.0;
        for rank in 0..=max_rank {
            if rank > 0 {
                center += (rank_width[rank - 1] + rank_width[rank]) / 2.0 + RANK_SEPARATION;
            }
            for node in nodes.iter_mut() {
                if node.rank == rank {
                    node.center_x = center;
                }
            }
        }
    }

    /// Порядок узлов внутри каждого ранга.
    ///
    /// Обход в ширину: родители обрабатываются сверху вниз, потомки — в
    /// порядке строк. Для дерева это и есть порядок, который даёт dot после
    /// минимизации пересечений.
    fn rank_order(&self, nodes: &[TableNode]) -> Vec<Vec<usize>> {
        let max_rank = nodes.iter().map(|node| node.rank).max().unwrap_or(0);
        let mut order = vec![Vec::new(); max_rank + 1];
        if nodes.is_empty() {
            return order;
        }

        let mut queue = VecDeque::new();
        queue.push_back(0_usize);
        while let Some(index) = queue.pop_front() {
            order[nodes[index].rank].push(index);
            for (_, child) in &nodes[index].children {
                queue.push_back(*child);
            }
        }
        order
    }

    /// Кладёт узлы по поперечной оси — повторяет решатель dot.
    ///
    /// Задача: минимизировать `Σ |y_потомок − y_порта_родителя|` при
    /// ограничении, что соседи по рангу не пересекаются. Решается блочным
    /// покоординатным спуском: для каждого ранга берётся медиана «идеальных»
    /// позиций соседей и проецируется на допустимое множество (L1-изотонная
    /// регрессия по цепочке ограничений). Такой спуск для дерева сходится за
    /// считанные итерации и даёт те же координаты, что и dot.
    fn assign_cross_axis(&self, nodes: &mut [TableNode], order: &[Vec<usize>]) {
        // Начальное приближение: каждый ранг — стопка от нуля.
        for rank in order {
            let mut cursor = 0.0;
            for &index in rank {
                let extent = nodes[index].cross_extent();
                nodes[index].center_y = cursor + extent / 2.0;
                cursor = nodes[index].center_y + extent / 2.0 + NODE_SEPARATION;
            }
        }

        for _ in 0..MAX_SOLVER_ITERATIONS {
            let mut shift = 0.0_f64;
            for rank in order {
                if rank.is_empty() {
                    continue;
                }

                // Идеальные позиции: медиана требований соседей.
                let targets: Vec<f64> = rank
                    .iter()
                    .map(|&index| {
                        let node = &nodes[index];
                        let mut demands = Vec::new();
                        if let Some(parent) = node.parent {
                            let parent = &nodes[parent];
                            demands.push(parent.center_y + parent.port_offset(node.parent_row));
                        }
                        for (row, child) in &node.children {
                            demands.push(nodes[*child].center_y - node.port_offset(*row));
                        }
                        if demands.is_empty() {
                            node.center_y
                        } else {
                            balanced_median(&mut demands)
                        }
                    })
                    .collect();

                // Ограничения «соседи не пересекаются» — цепочка.
                let mut prefix = vec![0.0_f64; rank.len()];
                for i in 1..rank.len() {
                    let left = nodes[rank[i - 1]].cross_extent();
                    let right = nodes[rank[i]].cross_extent();
                    prefix[i] = prefix[i - 1] + (left + right) / 2.0 + NODE_SEPARATION;
                }

                let shifted: Vec<f64> = (0..rank.len()).map(|i| targets[i] - prefix[i]).collect();
                let fitted = isotonic_l1(&shifted);
                for (i, &index) in rank.iter().enumerate() {
                    let value = fitted[i] + prefix[i];
                    shift = shift.max((value - nodes[index].center_y).abs());
                    nodes[index].center_y = value;
                }
            }
            if shift < SOLVER_EPSILON {
                break;
            }
        }

        // Сдвиг к верхнему отступу холста.
        let top = nodes
            .iter()
            .map(|node| node.center_y - node.cross_extent() / 2.0)
            .fold(f64::INFINITY, f64::min);
        for node in nodes.iter_mut() {
            node.center_y = node.center_y - top + self.config.padding;
        }
    }

    /// Рисует одну таблицу в точке `(x, y)`.
    fn draw_table(&self, node: &TableNode, x: f64, y: f64, elements: &mut Vec<LayoutElement>) {
        let total_width = node.width();
        let total_height = node.height();
        let row_height = LINE_HEIGHT;

        // Фон и рамка
        for (id, fill, stroke) in [
            (
                "json_table_bg",
                self.config.object_bg_color,
                self.config.object_bg_color,
            ),
            ("json_table_border", "none", "#000000"),
        ] {
            elements.push(LayoutElement {
                id: format!("{}_{}", id, elements.len()),
                element_type: ElementType::RoundedRectangle,
                bounds: Rect::new(x, y, total_width, total_height),
                text: None,
                properties: [
                    ("fill".to_string(), fill.to_string()),
                    ("stroke".to_string(), stroke.to_string()),
                    ("stroke-width".to_string(), "1.5".to_string()),
                    ("rx".to_string(), TABLE_CORNER_RADIUS.to_string()),
                ]
                .into_iter()
                .collect(),
            });
        }

        for (row, (key, value)) in node.entries.iter().enumerate() {
            let row_y = y + row as f64 * row_height;
            let baseline = row_y + row_height - TABLE_BASELINE_GAP;

            if let Some(key) = key {
                elements.push(LayoutElement {
                    id: format!("json_key_{}", elements.len()),
                    element_type: ElementType::Text {
                        text: key.clone(),
                        font_size: self.config.font_size,
                    },
                    bounds: Rect::new(
                        x + TABLE_CELL_PADDING / 2.0,
                        row_y,
                        node.key_width,
                        row_height,
                    ),
                    text: None,
                    properties: [
                        ("fill".to_string(), self.config.key_color.to_string()),
                        ("font-weight".to_string(), "700".to_string()),
                        ("baseline".to_string(), baseline.to_string()),
                    ]
                    .into_iter()
                    .collect(),
                });

                // Вертикальный разделитель колонок
                elements.push(LayoutElement {
                    id: format!("json_col_sep_{}", elements.len()),
                    element_type: ElementType::Edge {
                        points: vec![
                            Point::new(x + node.key_width, row_y),
                            Point::new(x + node.key_width, row_y + row_height),
                        ],
                        label: None,
                        arrow_start: false,
                        arrow_end: false,
                        dashed: false,
                        edge_type: EdgeType::Link,
                        from_cardinality: None,
                        to_cardinality: None,
                    },
                    bounds: Rect::new(x + node.key_width, row_y, 0.0, row_height),
                    text: None,
                    properties: [
                        ("stroke".to_string(), "#000000".to_string()),
                        ("stroke-width".to_string(), "1".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                });
            }

            elements.push(LayoutElement {
                id: format!("json_value_{}", elements.len()),
                element_type: ElementType::Text {
                    text: self.cell_text(value),
                    font_size: self.config.font_size,
                },
                bounds: Rect::new(
                    x + node.key_width + TABLE_CELL_PADDING / 2.0,
                    row_y,
                    node.value_width,
                    row_height,
                ),
                text: None,
                properties: [("fill".to_string(), "#000000".to_string())]
                    .into_iter()
                    .collect(),
            });

            // Горизонтальный разделитель (кроме последней строки)
            if row + 1 < node.entries.len() {
                elements.push(LayoutElement {
                    id: format!("json_row_sep_{}", elements.len()),
                    element_type: ElementType::Edge {
                        points: vec![
                            Point::new(x, row_y + row_height),
                            Point::new(x + total_width, row_y + row_height),
                        ],
                        label: None,
                        arrow_start: false,
                        arrow_end: false,
                        dashed: false,
                        edge_type: EdgeType::Link,
                        from_cardinality: None,
                        to_cardinality: None,
                    },
                    bounds: Rect::new(x, row_y + row_height, total_width, 0.0),
                    text: None,
                    properties: [
                        ("stroke".to_string(), "#000000".to_string()),
                        ("stroke-width".to_string(), "1".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                });
            }
        }
    }

    /// Строки таблицы: пары «ключ — значение».
    fn table_entries(&self, node: &JsonNode) -> Vec<(Option<String>, JsonValue)> {
        match &node.value {
            JsonValue::Object(children) => children
                .iter()
                .map(|child| (child.key.clone(), child.value.clone()))
                .collect(),
            JsonValue::Array(items) => items
                .iter()
                .map(|item| (None, item.value.clone()))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Текст ячейки значения.
    ///
    /// Для контейнеров PlantUML пишет три пробела: в эталоне `JSON:
    /// Диаграмма` колонка значений таблицы корня занимает 23.351 =
    /// 10 + 13.351, где 13.351 — ширина «   ».
    fn cell_text(&self, value: &JsonValue) -> String {
        match value {
            JsonValue::String(s) => s.clone(),
            JsonValue::Number(_n, text) => text.clone(),
            JsonValue::Boolean(b) => {
                if self.config.bool_as_text {
                    b.to_string()
                } else if *b {
                    "☑ true".to_string()
                } else {
                    "☐ false".to_string()
                }
            }
            JsonValue::Null => String::new(),
            JsonValue::Object(_) | JsonValue::Array(_) => "   ".to_string(),
        }
    }
}

/// Медиана, смещённая к середине плоской зоны.
///
/// Для чётного числа требований любой минимум L1 лежит между двумя средними
/// значениями; dot выбирает середину этой зоны, поэтому и здесь берётся
/// полусумма. На эталоне `JSON: Диаграмма` это видно по таблице корня: её
/// центр (95.242) стоит ровно между портами двух своих строк.
fn balanced_median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let middle = values.len() / 2;
    if values.len() % 2 == 1 {
        values[middle]
    } else {
        (values[middle - 1] + values[middle]) / 2.0
    }
}

/// L1-изотонная регрессия: минимум `Σ |z_i − target_i|` при `z_0 ≤ z_1 ≤ ...`.
///
/// Метод смежных нарушителей: блоки сливаются, пока медиана предыдущего
/// больше медианы следующего. Значение блока — медиана объединения.
fn isotonic_l1(targets: &[f64]) -> Vec<f64> {
    let mut blocks: Vec<Vec<f64>> = Vec::with_capacity(targets.len());
    for &target in targets {
        blocks.push(vec![target]);
        while blocks.len() >= 2 {
            let mut left = blocks[blocks.len() - 2].clone();
            let mut right = blocks[blocks.len() - 1].clone();
            if balanced_median(&mut left) <= balanced_median(&mut right) {
                break;
            }
            let mut merged = blocks.pop().unwrap_or_default();
            if let Some(previous) = blocks.last_mut() {
                previous.append(&mut merged);
            }
        }
    }

    let mut result = Vec::with_capacity(targets.len());
    for mut block in blocks {
        let value = balanced_median(&mut block);
        result.extend(std::iter::repeat_n(value, block.len()));
    }
    result
}

/// Внутренний отступ ячейки таблицы.
const TABLE_CELL_PADDING: f64 = 10.0;

/// Скругление рамки таблицы (измерено по эталону).
const TABLE_CORNER_RADIUS: f64 = 5.0;

/// Расстояние от базовой линии текста до низа строки.
const TABLE_BASELINE_GAP: f64 = 5.302;

/// Высота строки таблицы.
const LINE_HEIGHT: f64 = 20.297;

/// Расстояние между соседними рангами (`ranksep` в dot).
///
/// Измерено по эталону `json_basic`: таблица массива `tags` начинается на
/// 178.0 при правом крае корня 141.446, то есть через 36.554 — там
/// протяжённости узлов по оси ранга равны их ширинам, поэтому вся разница
/// приходится на `ranksep`.
const RANK_SEPARATION: f64 = 36.554;

/// Зазор между соседними узлами одного ранга (`nodesep` в dot, 0.25 дюйма).
const NODE_SEPARATION: f64 = 18.0;

/// Запас поперечной протяжённости узла поверх высоты таблицы.
///
/// Измерено по эталону `JSON: Диаграмма`: четыре узла второго ранга стоят
/// ровно через 59.0, то есть (20.297 + 0.406) + 18.0. Тот же запас
/// подтверждается эталоном `json_basic`: без него таблица массива `tags`
/// оказалась бы на 0.2 выше, чем в эталоне.
const CROSS_EXTENT_EXTRA: f64 = 0.406;

/// Шаг портов (строк) в разметке записи dot.
///
/// Измерено по эталонам с сервера: у таблицы из четырёх строк порт строки 0
/// и порт строки 2 дают потомку разницу ровно 40.0. Шаг НЕ равен высоте
/// строки таблицы (20.297) — dot считает его по метрикам своей подписи.
const PORT_PITCH: f64 = 20.0;

/// Предел итераций решателя поперечной оси.
const MAX_SOLVER_ITERATIONS: usize = 64;

/// Порог сходимости решателя.
const SOLVER_EPSILON: f64 = 1e-9;

impl Default for JsonLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for JsonLayoutEngine {
    type Input = JsonDiagram;

    fn layout(&self, diagram: &Self::Input, _config: &LayoutConfig) -> LayoutResult {
        let mut elements = Vec::new();
        let mut nodes = Vec::new();

        if let Some(root) = &diagram.root {
            self.collect_tables(root, 0, None, &mut nodes);
        }

        if !nodes.is_empty() {
            self.assign_rank_axis(&mut nodes);
            let order = self.rank_order(&nodes);
            self.assign_cross_axis(&mut nodes, &order);

            for node in &nodes {
                let x = node.center_x - node.width() / 2.0;
                let y = node.center_y - node.cross_extent() / 2.0;
                self.draw_table(node, x, y, &mut elements);
            }
        }

        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_simple_json() {
        let root = JsonNode::object(
            None,
            vec![
                JsonNode::string(Some("name".into()), "John"),
                JsonNode::number(Some("age".into()), 30.0),
            ],
        );
        let diagram = JsonDiagram::with_root(root);

        let engine = JsonLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        assert!(!result.elements.is_empty());
        assert!(result.bounds.width > 0.0);
        assert!(result.bounds.height > 0.0);
    }

    #[test]
    fn test_layout_nested_json() {
        let address = JsonNode::object(
            Some("address".into()),
            vec![JsonNode::string(Some("city".into()), "NYC")],
        );
        let root = JsonNode::object(None, vec![address]);
        let diagram = JsonDiagram::with_root(root);

        let engine = JsonLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        // Должны быть элементы для обоих объектов
        assert!(result.elements.len() >= 4);
    }

    #[test]
    fn test_layout_json_array() {
        let items = JsonNode::array(
            Some("items".into()),
            vec![
                JsonNode::string(None, "apple"),
                JsonNode::string(None, "banana"),
            ],
        );
        let root = JsonNode::object(None, vec![items]);
        let diagram = JsonDiagram::with_root(root);

        let engine = JsonLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_node_width_calculation() {
        let config = JsonLayoutConfig::default();
        let engine = JsonLayoutEngine::with_config(config);

        // Простая проверка что engine создаётся
        assert!(engine.config.padding > 0.0);
    }

    #[test]
    fn test_nested_table_does_not_grow_container() {
        // Контейнер из одного поля с вложенным объектом: высота внешней
        // таблицы обязана остаться одной строкой.
        let inner = JsonNode::object(
            Some("a".into()),
            vec![JsonNode::string(Some("b".into()), "c")],
        );
        let root = JsonNode::object(None, vec![inner]);
        let diagram = JsonDiagram::with_root(root);

        let engine = JsonLayoutEngine::new();
        let result = engine.layout(&diagram, &LayoutConfig::default());

        let tables: Vec<_> = result
            .elements
            .iter()
            .filter(|e| e.id.starts_with("json_table_bg"))
            .collect();
        assert_eq!(tables.len(), 2);
        for table in &tables {
            assert!((table.bounds.height - LINE_HEIGHT).abs() < 1e-6);
        }
    }

    #[test]
    fn test_isotonic_regression_respects_order() {
        let fitted = isotonic_l1(&[3.0, 1.0, 2.0]);
        assert!(fitted[0] <= fitted[1] + 1e-9);
        assert!(fitted[1] <= fitted[2] + 1e-9);
        assert!((fitted.iter().sum::<f64>() - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_balanced_median_uses_middle_of_flat_zone() {
        assert!((balanced_median(&mut [4.0, 10.0]) - 7.0).abs() < 1e-9);
        assert!((balanced_median(&mut [10.0, 4.0, 7.0]) - 7.0).abs() < 1e-9);
    }
}
