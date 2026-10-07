//! AST типы для JSON диаграмм
//!
//! JSON диаграммы визуализируют JSON данные в виде дерева.
//!
//! Синтаксис PlantUML:
//! ```text
//! @startjson
//! {
//!   "name": "John",
//!   "age": 30,
//!   "address": {
//!     "city": "New York"
//!   },
//!   "hobbies": ["reading", "gaming"]
//! }
//! @endjson
//! ```

use serde::{Deserialize, Serialize};

use crate::common::DiagramMetadata;

/// JSON диаграмма
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonDiagram {
    /// Корневой JSON узел
    pub root: Option<JsonNode>,
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Подсветка определённых путей (highlight)
    pub highlights: Vec<String>,
}

impl JsonDiagram {
    /// Создаёт новую пустую JSON диаграмму
    pub fn new() -> Self {
        Self {
            root: None,
            metadata: DiagramMetadata::default(),
            highlights: Vec::new(),
        }
    }

    /// Создаёт JSON диаграмму с корневым узлом
    pub fn with_root(root: JsonNode) -> Self {
        Self {
            root: Some(root),
            metadata: DiagramMetadata::default(),
            highlights: Vec::new(),
        }
    }

    /// Возвращает общее количество узлов
    pub fn node_count(&self) -> usize {
        self.root.as_ref().map_or(0, |r| r.count_all())
    }

    /// Возвращает максимальную глубину
    pub fn max_depth(&self) -> usize {
        self.root.as_ref().map_or(0, |r| r.max_depth())
    }
}

impl Default for JsonDiagram {
    fn default() -> Self {
        Self::new()
    }
}

/// Узел JSON дерева
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonNode {
    /// Ключ (для элементов объекта) или индекс (для элементов массива)
    pub key: Option<String>,
    /// Значение узла
    pub value: JsonValue,
    /// Является ли узел свёрнутым (collapsed)
    pub collapsed: bool,
    /// Подсвечен ли узел
    pub highlighted: bool,
}

impl JsonNode {
    /// Создаёт узел с ключом и значением
    pub fn new(key: Option<String>, value: JsonValue) -> Self {
        Self {
            key,
            value,
            collapsed: false,
            highlighted: false,
        }
    }

    /// Создаёт узел-объект
    pub fn object(key: Option<String>, children: Vec<JsonNode>) -> Self {
        Self::new(key, JsonValue::Object(children))
    }

    /// Создаёт узел-массив
    pub fn array(key: Option<String>, items: Vec<JsonNode>) -> Self {
        Self::new(key, JsonValue::Array(items))
    }

    /// Создаёт узел-строку
    pub fn string(key: Option<String>, value: impl Into<String>) -> Self {
        Self::new(key, JsonValue::String(value.into()))
    }

    /// Создаёт узел-число
    pub fn number(key: Option<String>, value: f64) -> Self {
        Self::new(key, JsonValue::Number(value, format!("{value}")))
    }

    /// То же, но с СОХРАНЕНИЕМ исходного литерала числа.
    pub fn number_literal(key: Option<String>, value: f64, text: String) -> Self {
        Self::new(key, JsonValue::Number(value, text))
    }

    /// Создаёт узел-булево значение
    pub fn boolean(key: Option<String>, value: bool) -> Self {
        Self::new(key, JsonValue::Boolean(value))
    }

    /// Создаёт узел-null
    pub fn null(key: Option<String>) -> Self {
        Self::new(key, JsonValue::Null)
    }

    /// Возвращает общее количество узлов
    pub fn count_all(&self) -> usize {
        match &self.value {
            JsonValue::Object(children) => {
                1 + children.iter().map(|c| c.count_all()).sum::<usize>()
            }
            JsonValue::Array(items) => 1 + items.iter().map(|i| i.count_all()).sum::<usize>(),
            _ => 1,
        }
    }

    /// Возвращает максимальную глубину
    pub fn max_depth(&self) -> usize {
        match &self.value {
            JsonValue::Object(children) => {
                1 + children.iter().map(|c| c.max_depth()).max().unwrap_or(0)
            }
            JsonValue::Array(items) => 1 + items.iter().map(|i| i.max_depth()).max().unwrap_or(0),
            _ => 1,
        }
    }

    /// Устанавливает состояние свёрнутости
    pub fn with_collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Устанавливает подсветку
    pub fn with_highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }
}

/// Значение JSON узла
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsonValue {
    /// Объект с дочерними узлами
    Object(Vec<JsonNode>),
    /// Массив с элементами
    Array(Vec<JsonNode>),
    /// Строка
    String(String),
    /// Число
    /// Число: значение и ИСХОДНЫЙ ЛИТЕРАЛ.
    ///
    /// Второй компонент нужен, потому что PlantUML печатает число так, как
    /// оно записано в исходнике: `150.00` остаётся `150.00`, а не `150`.
    /// Проверено по эталону `JSON: Диаграмма`: он рисует «150.00» и
    /// «75.50», а мы без сохранения литерала рисовали «150» и «75.5».
    Number(f64, String),
    /// Булево значение
    Boolean(bool),
    /// Null
    Null,
}

impl JsonValue {
    /// Проверяет, является ли значение примитивом
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            JsonValue::String(_) | JsonValue::Number(..) | JsonValue::Boolean(_) | JsonValue::Null
        )
    }

    /// Проверяет, является ли значение контейнером
    pub fn is_container(&self) -> bool {
        matches!(self, JsonValue::Object(_) | JsonValue::Array(_))
    }

    /// Возвращает тип значения как строку
    pub fn type_name(&self) -> &'static str {
        match self {
            JsonValue::Object(_) => "object",
            JsonValue::Array(_) => "array",
            JsonValue::String(_) => "string",
            JsonValue::Number(..) => "number",
            JsonValue::Boolean(_) => "boolean",
            JsonValue::Null => "null",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_simple_json() {
        let root = JsonNode::object(
            None,
            vec![
                JsonNode::string(Some("name".into()), "John"),
                JsonNode::number(Some("age".into()), 30.0),
            ],
        );

        let diagram = JsonDiagram::with_root(root);
        assert_eq!(diagram.node_count(), 3);
        assert_eq!(diagram.max_depth(), 2);
    }

    #[test]
    fn test_nested_json() {
        let address = JsonNode::object(
            Some("address".into()),
            vec![JsonNode::string(Some("city".into()), "New York")],
        );

        let root = JsonNode::object(None, vec![address]);

        assert_eq!(root.count_all(), 3);
        assert_eq!(root.max_depth(), 3);
    }

    #[test]
    fn test_json_array() {
        let hobbies = JsonNode::array(
            Some("hobbies".into()),
            vec![
                JsonNode::string(None, "reading"),
                JsonNode::string(None, "gaming"),
            ],
        );

        assert_eq!(hobbies.count_all(), 3);
    }

    #[test]
    fn test_json_value_types() {
        assert!(JsonValue::String("test".into()).is_primitive());
        assert!(JsonValue::Number(42.0, "42".into()).is_primitive());
        assert!(JsonValue::Boolean(true).is_primitive());
        assert!(JsonValue::Null.is_primitive());

        assert!(JsonValue::Object(vec![]).is_container());
        assert!(JsonValue::Array(vec![]).is_container());
    }
}
