//! Layout engine для JSON диаграмм
//!
//! JSON визуализируется как дерево с вложенными блоками.
//! Объекты и массивы отображаются как контейнеры с заголовками.

use plantuml_ast::json::{JsonDiagram, JsonNode, JsonValue};
use plantuml_model::{Point, Rect, Size};

use crate::json::config::JsonLayoutConfig;
use crate::traits::{LayoutEngine, LayoutResult};
use crate::{EdgeType, ElementType, LayoutConfig, LayoutElement};

/// Layout engine для JSON диаграмм
/// Layout engine для JSON/YAML диаграмм
pub struct JsonLayoutEngine {
    config: JsonLayoutConfig,
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

    /// Рисует узел как двухколоночную таблицу.
    ///
    /// Значения-контейнеры (объект, массив) выносятся отдельной таблицей
    /// вправо от строки своего ключа — так делает PlantUML.
    fn layout_table(
        &self,
        node: &JsonNode,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> Size {
        let entries = self.table_entries(node);
        if entries.is_empty() {
            return Size::new(0.0, 0.0);
        }

        // Ширина колонок: измерено по эталону — максимум по колонке плюс 10
        // Ключи PlantUML рисует ПОЛУЖИРНЫМИ, и ширину колонки считает по
        // жирным метрикам: эталон `yaml_basic` даёт разделитель на 78.24 =
        // 10 + 58.242 («version» жирным) + 10.
        let key_width = entries
            .iter()
            .filter_map(|(key, _)| key.as_deref())
            .map(|key| self.config.text.width_bold(key, self.config.font_size))
            .fold(0.0_f64, f64::max)
            + TABLE_CELL_PADDING;

        let value_width = entries
            .iter()
            .filter_map(|(_, value)| self.scalar_text(value))
            .map(|text| self.config.text.width(&text, self.config.font_size))
            .fold(0.0_f64, f64::max)
            + TABLE_CELL_PADDING;

        let row_height = self.config.line_height;
        let rows = entries.len() as f64;

        // Если ключей нет (массив), таблица состоит из одной колонки.
        //
        // Ширина определяется содержимым: в эталоне таблица вложенного
        // массива со значениями «a» и «b» занимает 18.887, то есть по одному
        // символу плюс 10. Раньше подставлялся min_key_width (60), из-за чего
        // вложенная таблица оказывалась втрое шире эталонной.
        let (key_width, value_width) = if key_width <= TABLE_CELL_PADDING {
            (0.0, value_width)
        } else {
            (key_width, value_width)
        };

        let total_width = key_width + value_width;
        let total_height = rows * row_height;

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

        // Строки
        for (row, (key, value)) in entries.iter().enumerate() {
            let row_y = y + row as f64 * row_height;
            let baseline = row_y + row_height - TABLE_BASELINE_GAP;

            if let Some(key) = key {
                elements.push(LayoutElement {
                    id: format!("json_key_{}", elements.len()),
                    element_type: ElementType::Text {
                        text: key.clone(),
                        font_size: self.config.font_size,
                    },
                    bounds: Rect::new(x + TABLE_CELL_PADDING / 2.0, row_y, key_width, row_height),
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
                            Point::new(x + key_width, row_y),
                            Point::new(x + key_width, row_y + row_height),
                        ],
                        label: None,
                        arrow_start: false,
                        arrow_end: false,
                        dashed: false,
                        edge_type: EdgeType::Link,
                        from_cardinality: None,
                        to_cardinality: None,
                    },
                    bounds: Rect::new(x + key_width, row_y, 0.0, row_height),
                    text: None,
                    properties: [
                        ("stroke".to_string(), "#000000".to_string()),
                        ("stroke-width".to_string(), "1".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                });
            }

            if let Some(text) = self.scalar_text(value) {
                elements.push(LayoutElement {
                    id: format!("json_value_{}", elements.len()),
                    element_type: ElementType::Text {
                        text,
                        font_size: self.config.font_size,
                    },
                    bounds: Rect::new(
                        x + key_width + TABLE_CELL_PADDING / 2.0,
                        row_y,
                        value_width,
                        row_height,
                    ),
                    text: None,
                    properties: [("fill".to_string(), "#000000".to_string())]
                        .into_iter()
                        .collect(),
                });
            } else {
                // Контейнер выносится отдельной таблицей вправо
                let nested_x = x + total_width + TABLE_NESTED_GAP;
                let nested_y = row_y + row_height / 2.0;
                let child = match value {
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
                if let Some(child) = child {
                    // Вложенная таблица ЦЕНТРИРУЕТСЯ по своей строке, а не
                    // начинается с её середины. В эталоне json_basic таблица
                    // массива занимает 40.09..80.68 при строке `tags`
                    // 50.59..70.89, то есть её центр 60.385 совпадает с
                    // центром строки 60.74. Раньше верх таблицы ставился в
                    // центр строки, и она уезжала вниз на половину высоты —
                    // на 20.65px.
                    let mut probe: Vec<LayoutElement> = Vec::new();
                    let size = self.layout_table(&child, 0.0, 0.0, &mut probe);
                    let centered_y = nested_y - size.height / 2.0;
                    self.layout_table(&child, nested_x, centered_y, elements);
                }
            }

            // Горизонтальный разделитель (кроме последней строки)
            if row + 1 < entries.len() {
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

        Size::new(total_width, total_height)
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

    /// Текстовое представление скалярного значения.
    ///
    /// Для контейнеров возвращает `None` — они рисуются отдельной таблицей.
    fn scalar_text(&self, value: &JsonValue) -> Option<String> {
        match value {
            JsonValue::String(s) => Some(s.clone()),
            JsonValue::Number(n) => Some(format!("{n}")),
            JsonValue::Boolean(b) => Some(if self.config.bool_as_text {
                b.to_string()
            } else if *b {
                "☑ true".to_string()
            } else {
                "☐ false".to_string()
            }),
            JsonValue::Null => Some(String::new()),
            JsonValue::Object(_) | JsonValue::Array(_) => None,
        }
    }
}

/// Внутренний отступ ячейки таблицы.
const TABLE_CELL_PADDING: f64 = 10.0;

/// Скругление рамки таблицы (измерено по эталону).
const TABLE_CORNER_RADIUS: f64 = 5.0;

/// Расстояние от базовой ли��ии текста до низа строки.
const TABLE_BASELINE_GAP: f64 = 5.302;

/// Зазор между основной таблицей и таблицей вложенного значения.
const TABLE_NESTED_GAP: f64 = 36.554;

impl Default for JsonLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for JsonLayoutEngine {
    type Input = JsonDiagram;

    fn layout(&self, diagram: &Self::Input, _config: &LayoutConfig) -> LayoutResult {
        let mut elements = Vec::new();

        if let Some(root) = &diagram.root {
            // PlantUML выводит JSON и YAML двухколоночной таблицей: ключ слева
            // (жирным), значение справа, строки разделены горизонтальными
            // линиями, между колонками — вертикальная. Раньше движок рисовал
            // вложенные блоки со скобками, поэтому высота диаграммы расходилась
            // с эталоном почти вдвое.
            self.layout_table(
                root,
                self.config.padding,
                self.config.padding,
                &mut elements,
            );
        }

        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        // Таблица уже включает отступы, поэтому padding не добавляется повторно
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
}
