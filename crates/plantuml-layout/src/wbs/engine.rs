//! Layout engine для WBS диаграмм
//!
//! WBS использует вертикальное дерево (сверху вниз),
//! в отличие от MindMap который горизонтальный.

use std::collections::HashMap;

use plantuml_ast::wbs::{WbsDiagram, WbsNode, WbsNodeStyle};
use plantuml_model::{Point, Rect};

use super::WbsLayoutConfig;
use crate::traits::LayoutResult;
use crate::{ElementType, LayoutElement};

/// Layout engine для WBS диаграмм
pub struct WbsLayoutEngine {
    config: WbsLayoutConfig,
}

impl WbsLayoutEngine {
    /// Создаёт новый layout engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: WbsLayoutConfig::default(),
        }
    }

    /// Создаёт layout engine с заданной конфигурацией
    pub fn with_config(config: WbsLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout для диаграммы
    pub fn layout(&self, diagram: &WbsDiagram) -> LayoutResult {
        let mut elements = Vec::new();

        if let Some(root) = &diagram.root {
            // Первый проход: вычисляем ширину каждого поддерева
            let subtree_widths = self.calculate_subtree_widths(root);

            // Второй проход: располагаем узлы
            let start_x = self.config.padding;
            let start_y = self.config.padding;

            self.layout_node(root, start_x, start_y, &subtree_widths, &mut elements);
        }

        // Центрируем диаграмму
        self.center_diagram(&mut elements);

        // Вычисляем общие bounds
        let bounds = self.calculate_bounds(&elements);

        LayoutResult { elements, bounds }
    }

    /// Вычисляет ширину поддерева для каждого узла
    fn calculate_subtree_widths(&self, node: &WbsNode) -> HashMap<*const WbsNode, f64> {
        let mut widths = HashMap::new();
        self.calculate_subtree_width_recursive(node, &mut widths);
        widths
    }

    fn calculate_subtree_width_recursive(
        &self,
        node: &WbsNode,
        widths: &mut HashMap<*const WbsNode, f64>,
    ) -> f64 {
        let node_width = self.calculate_node_width(&node.text);

        if node.children.is_empty() {
            widths.insert(node as *const WbsNode, node_width);
            return node_width;
        }

        let children_width: f64 = node
            .children
            .iter()
            .map(|c| self.calculate_subtree_width_recursive(c, widths))
            .sum();
        let spacing = (node.children.len() - 1) as f64 * self.config.sibling_spacing;
        let total_children_width = children_width + spacing;

        let subtree_width = node_width.max(total_children_width);
        widths.insert(node as *const WbsNode, subtree_width);
        subtree_width
    }

    /// Располагает узел и его детей
    fn layout_node(
        &self,
        node: &WbsNode,
        x: f64,
        y: f64,
        subtree_widths: &HashMap<*const WbsNode, f64>,
        elements: &mut Vec<LayoutElement>,
    ) {
        let subtree_width = subtree_widths
            .get(&(node as *const WbsNode))
            .copied()
            .unwrap_or(self.config.min_node_width);

        let node_width = self.calculate_node_width(&node.text);

        // Центрируем узел в пределах его поддерева
        let node_x = x + (subtree_width - node_width) / 2.0;
        let node_rect = Rect::new(node_x, y, node_width, self.config.node_height);

        let node_id = elements.len();
        elements.push(self.create_node_element(node, &node_rect, node_id));

        // Располагаем детей
        if !node.children.is_empty() {
            let child_y = y + self.config.node_height + self.config.level_spacing;
            let mut child_x = x;

            for child in &node.children {
                let child_subtree_width = subtree_widths
                    .get(&(child as *const WbsNode))
                    .copied()
                    .unwrap_or(self.config.min_node_width);

                // Рекурсивно располагаем ребёнка
                let child_start_id = elements.len();
                self.layout_node(child, child_x, child_y, subtree_widths, elements);

                // Рисуем связь от родителя к ребёнку
                let child_node_width = self.calculate_node_width(&child.text);
                let child_center_x = child_x
                    + (child_subtree_width - child_node_width) / 2.0
                    + child_node_width / 2.0;

                elements.push(self.create_connection(
                    node_rect.bottom_center(),
                    Point::new(child_center_x, child_y),
                    child_start_id,
                ));

                child_x += child_subtree_width + self.config.sibling_spacing;
            }
        }
    }

    /// Вычисляет ширину узла
    fn calculate_node_width(&self, text: &str) -> f64 {
        let char_width = self.config.font_size * 0.6;
        let text_width = text.len() as f64 * char_width;
        (text_width + self.config.node_padding_x * 2.0).max(self.config.min_node_width)
    }

    /// Создаёт элемент для узла
    fn create_node_element(&self, node: &WbsNode, rect: &Rect, id: usize) -> LayoutElement {
        let element_type = match node.style {
            WbsNodeStyle::Box => ElementType::Rectangle {
                label: node.text.clone(),
                corner_radius: 0.0,
            },
            WbsNodeStyle::NoBorder => ElementType::Text {
                text: node.text.clone(),
                font_size: self.config.font_size,
            },
            WbsNodeStyle::Strikethrough => ElementType::Rectangle {
                label: node.text.clone(),
                corner_radius: 0.0,
            },
            WbsNodeStyle::Default => ElementType::Rectangle {
                label: node.text.clone(),
                corner_radius: 3.0,
            },
        };

        let mut properties = HashMap::new();
        properties.insert("level".to_string(), node.level.to_string());
        if node.style == WbsNodeStyle::Strikethrough {
            properties.insert("strikethrough".to_string(), "true".to_string());
        }

        LayoutElement {
            id: format!("node_{}", id),
            bounds: *rect,
            element_type,
            text: Some(node.text.clone()),
            properties,
        }
    }

    /// Создаёт элемент соединения
    fn create_connection(&self, from: Point, to: Point, id: usize) -> LayoutElement {
        let mut properties = HashMap::new();

        // Рисуем прямую вертикальную линию с изломом
        let mid_y = (from.y + to.y) / 2.0;
        let path = format!(
            "M{},{} L{},{} L{},{} L{},{}",
            from.x, from.y, from.x, mid_y, to.x, mid_y, to.x, to.y
        );
        properties.insert("path".to_string(), path);

        LayoutElement {
            id: format!("conn_{}", id),
            bounds: Rect::from_points(from, to),
            element_type: ElementType::Path,
            text: None,
            properties,
        }
    }

    /// Центрирует диаграмму
    fn center_diagram(&self, elements: &mut [LayoutElement]) {
        if elements.is_empty() {
            return;
        }

        let min_x = elements
            .iter()
            .map(|e| e.bounds.x)
            .fold(f64::INFINITY, f64::min);
        let min_y = elements
            .iter()
            .map(|e| e.bounds.y)
            .fold(f64::INFINITY, f64::min);

        let offset_x = self.config.padding - min_x;
        let offset_y = self.config.padding - min_y;

        for element in elements.iter_mut() {
            element.bounds.x += offset_x;
            element.bounds.y += offset_y;
        }
    }

    /// Вычисляет общие bounds диаграммы
    fn calculate_bounds(&self, elements: &[LayoutElement]) -> Rect {
        if elements.is_empty() {
            return Rect::new(0.0, 0.0, 100.0, 100.0);
        }

        let min_x = elements
            .iter()
            .map(|e| e.bounds.x)
            .fold(f64::INFINITY, f64::min);
        let min_y = elements
            .iter()
            .map(|e| e.bounds.y)
            .fold(f64::INFINITY, f64::min);
        let max_x = elements
            .iter()
            .map(|e| e.bounds.x + e.bounds.width)
            .fold(f64::NEG_INFINITY, f64::max);
        let max_y = elements
            .iter()
            .map(|e| e.bounds.y + e.bounds.height)
            .fold(f64::NEG_INFINITY, f64::max);

        Rect::new(
            min_x - self.config.padding,
            min_y - self.config.padding,
            max_x - min_x + self.config.padding * 2.0,
            max_y - min_y + self.config.padding * 2.0,
        )
    }
}

impl Default for WbsLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_simple_wbs() {
        let mut root = WbsNode::new(1, "Project");
        root.add_child(WbsNode::new(2, "Phase 1"));
        root.add_child(WbsNode::new(2, "Phase 2"));

        let diagram = WbsDiagram::with_root(root);
        let engine = WbsLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Корень + 2 фазы + 2 соединения
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_layout_deep_wbs() {
        let mut root = WbsNode::new(1, "Project");
        let mut phase = WbsNode::new(2, "Phase");
        phase.add_child(WbsNode::new(3, "Task"));
        root.add_child(phase);

        let diagram = WbsDiagram::with_root(root);
        let engine = WbsLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_node_width_calculation() {
        let engine = WbsLayoutEngine::new();

        let short = engine.calculate_node_width("Hi");
        let long = engine.calculate_node_width("This is a very long text");

        assert!(long > short);
        assert!(short >= engine.config.min_node_width);
    }
}
