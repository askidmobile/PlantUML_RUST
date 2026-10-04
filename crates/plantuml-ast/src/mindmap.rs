//! AST типы для MindMap диаграмм
//!
//! MindMap — диаграмма в виде дерева для визуализации идей и концепций.
//!
//! Синтаксис PlantUML:
//! ```text
//! @startmindmap
//! * Root
//! ** Branch 1
//! *** Leaf 1.1
//! *** Leaf 1.2
//! ** Branch 2
//! @endmindmap
//! ```

use serde::{Deserialize, Serialize};

use crate::common::{Color, DiagramMetadata};

/// MindMap диаграмма
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MindMapDiagram {
    /// Корневой узел (может быть None если диаграмма пуста)
    pub root: Option<MindMapNode>,
    /// Метаданные диаграммы (title, caption и т.д.)
    pub metadata: DiagramMetadata,
}

impl MindMapDiagram {
    /// Создаёт новую пустую MindMap диаграмму
    pub fn new() -> Self {
        Self {
            root: None,
            metadata: DiagramMetadata::default(),
        }
    }

    /// Создаёт MindMap с корневым узлом
    pub fn with_root(root: MindMapNode) -> Self {
        Self {
            root: Some(root),
            metadata: DiagramMetadata::default(),
        }
    }

    /// Возвращает общее количество узлов в диаграмме
    pub fn node_count(&self) -> usize {
        self.root.as_ref().map_or(0, |r| r.count_all())
    }

    /// Возвращает максимальную глубину дерева
    pub fn max_depth(&self) -> usize {
        self.root.as_ref().map_or(0, |r| r.max_depth())
    }
}

impl Default for MindMapDiagram {
    fn default() -> Self {
        Self::new()
    }
}

/// Узел MindMap дерева
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MindMapNode {
    /// Уровень вложенности (1 = корень, 2 = первый уровень и т.д.)
    pub level: usize,
    /// Текст узла
    pub text: String,
    /// Направление ветки (для OrgMode стиля)
    pub direction: NodeDirection,
    /// Стиль узла
    pub style: NodeStyle,
    /// Цвет узла
    pub color: Option<Color>,
    /// Цвет фона
    pub background_color: Option<Color>,
    /// Дочерние узлы
    pub children: Vec<MindMapNode>,
}

impl MindMapNode {
    /// Создаёт новый узел с текстом
    pub fn new(level: usize, text: impl Into<String>) -> Self {
        Self {
            level,
            text: text.into(),
            direction: NodeDirection::Right,
            style: NodeStyle::Default,
            color: None,
            background_color: None,
            children: Vec::new(),
        }
    }

    /// Добавляет дочерний узел
    pub fn add_child(&mut self, child: MindMapNode) {
        self.children.push(child);
    }

    /// Возвращает общее количество узлов (включая себя)
    pub fn count_all(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_all()).sum::<usize>()
    }

    /// Возвращает максимальную глубину поддерева
    pub fn max_depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self
                .children
                .iter()
                .map(|c| c.max_depth())
                .max()
                .unwrap_or(0)
        }
    }

    /// Устанавливает стиль узла
    pub fn with_style(mut self, style: NodeStyle) -> Self {
        self.style = style;
        self
    }

    /// Устанавливает направление узла
    pub fn with_direction(mut self, direction: NodeDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Устанавливает цвет узла
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Устанавливает цвет фона
    pub fn with_background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }
}

/// Направление ветки относительно корня
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeDirection {
    /// Ветка справа от корня (по умолчанию)
    Right,
    /// Ветка слева от корня
    Left,
}

#[allow(clippy::derivable_impls)] // Требует #[default] из Rust 1.80, а MSRV = 1.75
impl Default for NodeDirection {
    fn default() -> Self {
        Self::Right
    }
}

/// Стиль отображения узла
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStyle {
    /// Стандартный стиль (прямоугольник со скруглёнными углами)
    Default,
    /// Стиль "коробка" (прямоугольник)
    Box,
    /// Без рамки (только текст)
    NoBorder,
    /// Стиль для удалённых/отменённых элементов
    Strikethrough,
}

#[allow(clippy::derivable_impls)] // Требует #[default] из Rust 1.80, а MSRV = 1.75
impl Default for NodeStyle {
    fn default() -> Self {
        Self::Default
    }
}

impl NodeStyle {
    /// Парсит стиль из строки
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "box" | "_" => Some(Self::Box),
            "noborder" | "-" => Some(Self::NoBorder),
            "strike" | "strikethrough" | ";" => Some(Self::Strikethrough),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let node = MindMapNode::new(1, "Root");
        assert_eq!(node.level, 1);
        assert_eq!(node.text, "Root");
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_node_with_children() {
        let mut root = MindMapNode::new(1, "Root");
        root.add_child(MindMapNode::new(2, "Child 1"));
        root.add_child(MindMapNode::new(2, "Child 2"));

        assert_eq!(root.children.len(), 2);
        assert_eq!(root.count_all(), 3);
        assert_eq!(root.max_depth(), 2);
    }

    #[test]
    fn test_diagram_stats() {
        let mut root = MindMapNode::new(1, "Root");
        let mut child1 = MindMapNode::new(2, "Child 1");
        child1.add_child(MindMapNode::new(3, "Leaf 1.1"));
        child1.add_child(MindMapNode::new(3, "Leaf 1.2"));
        root.add_child(child1);
        root.add_child(MindMapNode::new(2, "Child 2"));

        let diagram = MindMapDiagram::with_root(root);
        assert_eq!(diagram.node_count(), 5);
        assert_eq!(diagram.max_depth(), 3);
    }

    #[test]
    fn test_node_style_parse() {
        assert_eq!(NodeStyle::parse("box"), Some(NodeStyle::Box));
        assert_eq!(NodeStyle::parse("_"), Some(NodeStyle::Box));
        assert_eq!(NodeStyle::parse("-"), Some(NodeStyle::NoBorder));
        assert_eq!(NodeStyle::parse("strike"), Some(NodeStyle::Strikethrough));
        assert_eq!(NodeStyle::parse("unknown"), None);
    }
}
