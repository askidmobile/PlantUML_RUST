//! Layout engine для JSON диаграмм
//!
//! JSON визуализируется как дерево с вложенными блоками.
//! Объекты и массивы отображаются как контейнеры с заголовками.

use plantuml_ast::json::{JsonDiagram, JsonNode, JsonValue};
use plantuml_model::{Rect, Size};

use crate::json::config::JsonLayoutConfig;
use crate::traits::{LayoutEngine, LayoutResult};
use crate::{ElementType, LayoutConfig, LayoutElement};

/// Layout engine для JSON диаграмм
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

    /// Вычисляет layout для JSON узла
    fn layout_node(
        &self,
        node: &JsonNode,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> Size {
        match &node.value {
            JsonValue::Object(children) => self.layout_object(node, children, x, y, elements),
            JsonValue::Array(items) => self.layout_array(node, items, x, y, elements),
            _ => self.layout_primitive(node, x, y, elements),
        }
    }

    /// Layout для объекта
    fn layout_object(
        &self,
        node: &JsonNode,
        children: &[JsonNode],
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> Size {
        let header_height = self.config.line_height;
        let mut content_height = 0.0;
        let mut max_width = self.config.min_key_width * 2.0;

        // Заголовок объекта
        let header_text = if let Some(key) = &node.key {
            format!("{}: {{", key)
        } else {
            "{".to_string()
        };

        // Layout дочерних элементов
        let content_x = x + self.config.indent;
        let mut content_y = y + header_height;

        for child in children {
            let child_size = self.layout_node(child, content_x, content_y, elements);
            content_height += child_size.height;
            max_width = max_width.max(child_size.width + self.config.indent);
            content_y += child_size.height;
        }

        // Закрывающая скобка
        content_height += self.config.line_height;

        let total_height = header_height + content_height;
        let total_width = max_width + self.config.indent;

        // Фон объекта
        let bg_element = LayoutElement {
            id: format!("json_obj_{}", elements.len()),
            element_type: ElementType::RoundedRectangle,
            bounds: Rect::new(x, y, total_width, total_height),
            text: None,
            properties: [
                ("fill".to_string(), self.config.object_bg_color.to_string()),
                ("stroke".to_string(), "#A0A0A0".to_string()),
                ("rx".to_string(), self.config.corner_radius.to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(bg_element);

        // Заголовок
        let header_element = LayoutElement {
            id: format!("json_header_{}", elements.len()),
            element_type: ElementType::Text {
                text: header_text,
                font_size: self.config.font_size,
            },
            bounds: Rect::new(x + 5.0, y, total_width - 10.0, header_height),
            text: None,
            properties: [
                ("fill".to_string(), self.config.key_color.to_string()),
                ("font-weight".to_string(), "bold".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(header_element);

        // Закрывающая скобка
        let close_element = LayoutElement {
            id: format!("json_close_{}", elements.len()),
            element_type: ElementType::Text {
                text: "}".to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(
                x + 5.0,
                y + total_height - self.config.line_height,
                20.0,
                self.config.line_height,
            ),
            text: None,
            properties: [("fill".to_string(), "#000000".to_string())]
                .into_iter()
                .collect(),
        };
        elements.push(close_element);

        Size::new(total_width, total_height)
    }

    /// Layout для массива
    fn layout_array(
        &self,
        node: &JsonNode,
        items: &[JsonNode],
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> Size {
        let header_height = self.config.line_height;
        let mut content_height = 0.0;
        let mut max_width = self.config.min_key_width * 2.0;

        // Заголовок массива
        let header_text = if let Some(key) = &node.key {
            format!("{}: [", key)
        } else {
            "[".to_string()
        };

        // Layout элементов массива
        let content_x = x + self.config.indent;
        let mut content_y = y + header_height;

        for (i, item) in items.iter().enumerate() {
            // Для примитивов в массиве показываем индекс
            let item_with_index = if item.value.is_primitive() {
                let mut indexed = item.clone();
                indexed.key = Some(format!("[{}]", i));
                indexed
            } else {
                item.clone()
            };

            let child_size = self.layout_node(&item_with_index, content_x, content_y, elements);
            content_height += child_size.height;
            max_width = max_width.max(child_size.width + self.config.indent);
            content_y += child_size.height;
        }

        // Закрывающая скобка
        content_height += self.config.line_height;

        let total_height = header_height + content_height;
        let total_width = max_width + self.config.indent;

        // Фон массива
        let bg_element = LayoutElement {
            id: format!("json_arr_{}", elements.len()),
            element_type: ElementType::RoundedRectangle,
            bounds: Rect::new(x, y, total_width, total_height),
            text: None,
            properties: [
                ("fill".to_string(), self.config.array_bg_color.to_string()),
                ("stroke".to_string(), "#A0A0A0".to_string()),
                ("rx".to_string(), self.config.corner_radius.to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(bg_element);

        // Заголовок
        let header_element = LayoutElement {
            id: format!("json_header_{}", elements.len()),
            element_type: ElementType::Text {
                text: header_text,
                font_size: self.config.font_size,
            },
            bounds: Rect::new(x + 5.0, y, total_width - 10.0, header_height),
            text: None,
            properties: [
                ("fill".to_string(), self.config.key_color.to_string()),
                ("font-weight".to_string(), "bold".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(header_element);

        // Закрывающая скобка
        let close_element = LayoutElement {
            id: format!("json_close_{}", elements.len()),
            element_type: ElementType::Text {
                text: "]".to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(
                x + 5.0,
                y + total_height - self.config.line_height,
                20.0,
                self.config.line_height,
            ),
            text: None,
            properties: [("fill".to_string(), "#000000".to_string())]
                .into_iter()
                .collect(),
        };
        elements.push(close_element);

        Size::new(total_width, total_height)
    }

    /// Layout для примитивного значения
    fn layout_primitive(
        &self,
        node: &JsonNode,
        x: f64,
        y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> Size {
        let (value_text, color) = match &node.value {
            JsonValue::String(s) => (format!("\"{}\"", s), self.config.string_color),
            JsonValue::Number(n) => (format!("{}", n), self.config.number_color),
            JsonValue::Boolean(b) => (format!("{}", b), self.config.keyword_color),
            JsonValue::Null => ("null".to_string(), self.config.keyword_color),
            _ => unreachable!(),
        };

        let display_text = if let Some(key) = &node.key {
            format!("{}: {}", key, value_text)
        } else {
            value_text
        };

        let text_width = self.config.text.width(&display_text, self.config.font_size) + 20.0;
        let width = text_width.max(self.config.min_key_width);

        let element = LayoutElement {
            id: format!("json_val_{}", elements.len()),
            element_type: ElementType::Text {
                text: display_text,
                font_size: self.config.font_size,
            },
            bounds: Rect::new(x, y, width, self.config.line_height),
            text: None,
            properties: [("fill".to_string(), color.to_string())]
                .into_iter()
                .collect(),
        };
        elements.push(element);

        Size::new(width, self.config.line_height)
    }
}

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
            self.layout_node(
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

        // Добавляем padding
        result.bounds.width += self.config.padding * 2.0;
        result.bounds.height += self.config.padding * 2.0;

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
