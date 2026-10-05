//! Layout engine для MindMap диаграмм
//!
//! PlantUML стиль: корень слева, дети справа, вертикальное расположение.
//! Линии соединяют края узлов кривыми Безье.

use std::collections::HashMap;

use plantuml_ast::mindmap::{MindMapDiagram, MindMapNode, NodeStyle};
use plantuml_model::{Point, Rect};

use super::MindMapLayoutConfig;
use crate::traits::LayoutResult;
use crate::{ElementType, LayoutElement};

/// Layout engine для MindMap диаграмм
pub struct MindMapLayoutEngine {
    config: MindMapLayoutConfig,
}

/// Контекст рекурсивного размещения узлов дерева.
///
/// Объединяет изменяемое состояние обхода, чтобы не протаскивать
/// его через каждый вызов отдельными аргументами.
struct NodeLayoutContext<'a> {
    /// Высоты поддеревьев, ключ — путь от корня (индексы детей)
    subtree_heights: &'a HashMap<Vec<usize>, f64>,
    /// Накопленные элементы layout
    elements: &'a mut Vec<LayoutElement>,
}

impl MindMapLayoutEngine {
    /// Создаёт новый layout engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: MindMapLayoutConfig::default(),
        }
    }

    /// Создаёт layout engine с заданной конфигурацией
    pub fn with_config(config: MindMapLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout для диаграммы
    pub fn layout(&self, diagram: &MindMapDiagram) -> LayoutResult {
        let mut elements = Vec::new();

        if let Some(root) = &diagram.root {
            // Первый проход: вычисляем высоту каждого поддерева.
            // Ключ — путь от корня (индексы детей), а не сырой указатель:
            // путь устойчив к перевыделению памяти и клонированию узлов.
            let mut subtree_heights = HashMap::new();
            self.calculate_subtree_height_recursive(root, &mut Vec::new(), &mut subtree_heights);

            // Общая высота дерева
            let total_height = self.get_subtree_height(&[], &subtree_heights);

            // Корень размещается слева, вертикально по центру
            let root_x = self.config.padding;
            let root_y = self.config.padding + total_height / 2.0 - self.config.node_height / 2.0;

            // Layout всего дерева рекурсивно
            let mut ctx = NodeLayoutContext {
                subtree_heights: &subtree_heights,
                elements: &mut elements,
            };
            self.layout_node(
                root,
                &mut Vec::new(),
                root_x,
                root_y,
                self.config.padding,
                None,
                &mut ctx,
            );
        }

        // Вычисляем общие bounds
        let bounds = self.calculate_bounds(&elements);

        LayoutResult { elements, bounds }
    }

    /// Рекурсивно вычисляет высоту поддерева для узла по пути `path`
    fn calculate_subtree_height_recursive(
        &self,
        node: &MindMapNode,
        path: &mut Vec<usize>,
        heights: &mut HashMap<Vec<usize>, f64>,
    ) -> f64 {
        if node.children.is_empty() {
            let height = self.config.node_height;
            heights.insert(path.clone(), height);
            return height;
        }

        // Сумма высот всех детей + отступы между ними
        let mut children_total = 0.0;
        for (idx, child) in node.children.iter().enumerate() {
            path.push(idx);
            children_total += self.calculate_subtree_height_recursive(child, path, heights);
            path.pop();
        }
        let spacing = (node.children.len() - 1) as f64 * self.config.sibling_spacing;
        let total = children_total + spacing;

        // Высота поддерева = max(высота узла, высота детей)
        let height = total.max(self.config.node_height);
        heights.insert(path.clone(), height);
        height
    }

    fn get_subtree_height(&self, path: &[usize], heights: &HashMap<Vec<usize>, f64>) -> f64 {
        heights
            .get(path)
            .copied()
            .unwrap_or(self.config.node_height)
    }

    /// Размещает узел и его детей
    #[allow(clippy::too_many_arguments)]
    fn layout_node(
        &self,
        node: &MindMapNode,
        path: &mut Vec<usize>,
        x: f64,
        y: f64,
        children_top_y: f64,
        parent_rect: Option<&Rect>,
        ctx: &mut NodeLayoutContext<'_>,
    ) {
        // Создаём узел
        let node_width = self.calculate_node_width(&node.text);
        let node_rect = Rect::new(x, y, node_width, self.config.node_height);

        let node_id = ctx.elements.len();
        ctx.elements
            .push(self.create_node_element(node, &node_rect, node_id));

        // Создаём соединение с родителем (если есть)
        if let Some(parent) = parent_rect {
            // Линия от правого края родителя к левому краю текущего узла
            let from = Point::new(parent.x + parent.width, parent.y + parent.height / 2.0);
            let to = Point::new(node_rect.x, node_rect.y + node_rect.height / 2.0);
            ctx.elements.push(self.create_connection(from, to, node_id));
        }

        // Размещаем детей
        if !node.children.is_empty() {
            let child_x = x + node_width + self.config.level_spacing;
            let mut current_y = children_top_y;

            for (idx, child) in node.children.iter().enumerate() {
                // Полный путь от корня до ребёнка
                path.push(idx);
                let child_height = self.get_subtree_height(path, ctx.subtree_heights);

                // Y позиция ребёнка — в центре его поддерева
                let child_y = current_y + child_height / 2.0 - self.config.node_height / 2.0;

                self.layout_node(
                    child,
                    path,
                    child_x,
                    child_y,
                    current_y,
                    Some(&node_rect),
                    ctx,
                );
                path.pop();

                current_y += child_height + self.config.sibling_spacing;
            }
        }
    }

    /// Вычисляет ширину узла по тексту
    fn calculate_node_width(&self, text: &str) -> f64 {
        // Ширина по измерителю текста.
        //
        // Раньше здесь была оценка `символы * font_size * 0.6`: для
        // «Project» она давала 58.8 при реальных 48.45, то есть на 21%
        // больше. Ошибка накапливалась по уровням, потому что X каждого
        // следующего уровня отсчитывается от правого края предыдущего.
        let text_width = self.config.text.width(text, self.config.font_size);
        (text_width + self.config.node_padding_x * 2.0).max(self.config.min_node_width)
    }

    /// Создаёт элемент для узла
    fn create_node_element(&self, node: &MindMapNode, rect: &Rect, id: usize) -> LayoutElement {
        let element_type = match node.style {
            NodeStyle::Box => ElementType::Rectangle {
                label: node.text.clone(),
                corner_radius: 0.0,
            },
            NodeStyle::NoBorder => ElementType::Text {
                text: node.text.clone(),
                font_size: self.config.font_size,
            },
            NodeStyle::Strikethrough => ElementType::Rectangle {
                label: node.text.clone(),
                corner_radius: 0.0,
            },
            NodeStyle::Default => ElementType::RoundedRectangle,
        };

        let mut properties = HashMap::new();
        properties.insert("text".to_string(), node.text.clone());
        properties.insert("level".to_string(), node.level.to_string());

        if let Some(color) = &node.color {
            properties.insert("color".to_string(), color.to_css());
        }
        if let Some(bg) = &node.background_color {
            properties.insert("background_color".to_string(), bg.to_css());
        }
        if node.style == NodeStyle::Strikethrough {
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

    /// Создаёт элемент соединения (кривая Безье)
    fn create_connection(&self, from: Point, to: Point, id: usize) -> LayoutElement {
        let mut properties = HashMap::new();

        // Контрольные точки для плавной кривой
        // PlantUML стиль: горизонтальный выход, потом изгиб к цели
        let ctrl_offset = (to.x - from.x).abs() * 0.4;
        let ctrl1_x = from.x + ctrl_offset;
        let ctrl2_x = to.x - ctrl_offset;

        let path = format!(
            "M{},{} C{},{} {},{} {},{}",
            from.x, from.y, ctrl1_x, from.y, ctrl2_x, to.y, to.x, to.y
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

impl Default for MindMapLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_simple_mindmap() {
        let mut root = MindMapNode::new(1, "Root");
        root.add_child(MindMapNode::new(2, "Branch 1"));
        root.add_child(MindMapNode::new(2, "Branch 2"));

        let diagram = MindMapDiagram::with_root(root);
        let engine = MindMapLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Корень + 2 ветки + 2 соединения
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_layout_deep_hierarchy() {
        let mut root = MindMapNode::new(1, "Root");
        let mut branch = MindMapNode::new(2, "Branch");
        branch.add_child(MindMapNode::new(3, "Leaf"));
        root.add_child(branch);

        let diagram = MindMapDiagram::with_root(root);
        let engine = MindMapLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_node_width_calculation() {
        let engine = MindMapLayoutEngine::new();

        let short = engine.calculate_node_width("Hi");
        let long = engine.calculate_node_width("This is a very long text");

        assert!(long > short);
        assert!(short >= engine.config.min_node_width);
    }
}
