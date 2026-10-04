//! AST типы для WBS (Work Breakdown Structure) диаграмм
//!
//! WBS — иерархическая декомпозиция работ проекта.
//! Синтаксис похож на MindMap, но использует другой визуальный стиль.
//!
//! Синтаксис PlantUML:
//! ```text
//! @startwbs
//! * Project
//! ** Phase 1
//! *** Task 1.1
//! *** Task 1.2
//! ** Phase 2
//! @endwbs
//! ```

use serde::{Deserialize, Serialize};

use crate::common::DiagramMetadata;

/// WBS диаграмма
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WbsDiagram {
    /// Корневой узел
    pub root: Option<WbsNode>,
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
}

impl WbsDiagram {
    /// Создаёт новую пустую WBS диаграмму
    pub fn new() -> Self {
        Self {
            root: None,
            metadata: DiagramMetadata::default(),
        }
    }

    /// Создаёт WBS с корневым узлом
    pub fn with_root(root: WbsNode) -> Self {
        Self {
            root: Some(root),
            metadata: DiagramMetadata::default(),
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

impl Default for WbsDiagram {
    fn default() -> Self {
        Self::new()
    }
}

/// Узел WBS дерева
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WbsNode {
    /// Уровень вложенности (1 = корень)
    pub level: usize,
    /// Текст узла
    pub text: String,
    /// Стиль узла
    pub style: WbsNodeStyle,
    /// Дочерние узлы
    pub children: Vec<WbsNode>,
}

impl WbsNode {
    /// Создаёт новый узел
    pub fn new(level: usize, text: impl Into<String>) -> Self {
        Self {
            level,
            text: text.into(),
            style: WbsNodeStyle::Default,
            children: Vec::new(),
        }
    }

    /// Добавляет дочерний узел
    pub fn add_child(&mut self, child: WbsNode) {
        self.children.push(child);
    }

    /// Возвращает общее количество узлов
    pub fn count_all(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_all()).sum::<usize>()
    }

    /// Возвращает максимальную глубину
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

    /// Устанавливает стиль
    pub fn with_style(mut self, style: WbsNodeStyle) -> Self {
        self.style = style;
        self
    }
}

/// Стиль узла WBS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WbsNodeStyle {
    /// Стандартный стиль (прямоугольник)
    Default,
    /// Стиль "коробка"
    Box,
    /// Без рамки
    NoBorder,
    /// Зачёркнутый (выполнено/отменено)
    Strikethrough,
}

#[allow(clippy::derivable_impls)] // Требует #[default] из Rust 1.80, а MSRV = 1.75
impl Default for WbsNodeStyle {
    fn default() -> Self {
        Self::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let node = WbsNode::new(1, "Project");
        assert_eq!(node.level, 1);
        assert_eq!(node.text, "Project");
    }

    #[test]
    fn test_node_with_children() {
        let mut root = WbsNode::new(1, "Project");
        root.add_child(WbsNode::new(2, "Phase 1"));
        root.add_child(WbsNode::new(2, "Phase 2"));

        assert_eq!(root.children.len(), 2);
        assert_eq!(root.count_all(), 3);
    }

    #[test]
    fn test_diagram_depth() {
        let mut root = WbsNode::new(1, "Project");
        let mut phase = WbsNode::new(2, "Phase");
        phase.add_child(WbsNode::new(3, "Task"));
        root.add_child(phase);

        let diagram = WbsDiagram::with_root(root);
        assert_eq!(diagram.max_depth(), 3);
    }
}
