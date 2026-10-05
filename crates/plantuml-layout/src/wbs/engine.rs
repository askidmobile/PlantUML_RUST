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

/// Сдвигает все координаты SVG-пути на заданное смещение.
///
/// Путь хранится строкой вида `M10,20 L10,30 L40,30 L40,50`. Все числа в
/// ней — абсолютные координаты, поэтому при центрировании диаграммы их
/// нужно сдвинуть вместе с `bounds`, иначе связи «оторвутся» от узлов.
///
/// Формат поддерживается только тот, который порождает `create_connection`
/// (`M` и `L` с парами чисел), — этого достаточно и позволяет обойтись без
/// парсера путей.
fn shift_svg_path(path: &str, dx: f64, dy: f64) -> String {
    let mut out = String::with_capacity(path.len() + 16);
    let mut number = String::new();
    let mut numbers_in_pair = 0usize;

    // Записывает накопленное число со сдвигом по нужной оси
    fn flush(out: &mut String, number: &mut String, numbers_in_pair: usize, dx: f64, dy: f64) {
        if number.is_empty() {
            return;
        }
        if let Ok(value) = number.parse::<f64>() {
            // Чётные позиции в паре — X, нечётные — Y
            let shifted = if numbers_in_pair % 2 == 0 {
                value + dx
            } else {
                value + dy
            };
            // Компактная запись без лишних нулей
            if shifted.fract() == 0.0 {
                out.push_str(&format!("{}", shifted as i64));
            } else {
                out.push_str(&format!("{shifted}"));
            }
        } else {
            out.push_str(number);
        }
        number.clear();
    }

    for ch in path.chars() {
        if ch.is_ascii_digit() || ch == '-' || ch == '.' || ch == '+' {
            number.push(ch);
        } else if ch == ',' {
            flush(&mut out, &mut number, numbers_in_pair, dx, dy);
            // Разделитель координат сохраняем: без него пары чисел сливаются
            // (`15,17` превратилось бы в `1517`)
            out.push(',');
            numbers_in_pair += 1;
        } else {
            flush(&mut out, &mut number, numbers_in_pair, dx, dy);
            // Новая команда — счётчик координат начинается заново
            numbers_in_pair = 0;
            out.push(ch);
        }
    }
    flush(&mut out, &mut number, numbers_in_pair, dx, dy);

    out
}

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

            self.layout_node(
                root,
                start_x,
                start_y,
                0,
                true,
                start_y,
                &subtree_widths,
                &mut elements,
            );
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
    /// Вертикальная координата ряда.
    ///
    /// Ряд 0 стоит в начале, ряд 1 — через отступ `level_spacing`,
    /// каждый следующий — через `row_spacing`. Измерено по эталону:
    /// шаги 73.97 и 48.97 при высоте узла 33.969, то есть 40.0 и 15.0.
    fn row_y(&self, row: usize, base_y: f64) -> f64 {
        if row == 0 {
            return base_y;
        }

        let step = self.config.node_height + self.config.row_spacing;
        base_y + self.config.node_height + self.config.level_spacing + (row - 1) as f64 * step
    }

    #[allow(clippy::too_many_arguments)]
    fn layout_node(
        &self,
        node: &WbsNode,
        x: f64,
        y: f64,
        row: usize,
        is_root: bool,
        base_y: f64,
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
            let mut child_x = x;

            for (k, child) in node.children.iter().enumerate() {
                let child_subtree_width = subtree_widths
                    .get(&(child as *const WbsNode))
                    .copied()
                    .unwrap_or(self.config.min_node_width);

                // Ряд ребёнка по правилу выше.
                let child_row = if is_root { 1 } else { row + 1 + k };
                let child_y = self.row_y(child_row, base_y);

                // Ребёнок НЕ корень: его собственные дети пойдут по общему
                // правилу, поэтому `is_root` здесь всегда false.
                let child_start_id = elements.len();
                self.layout_node(
                    child,
                    child_x,
                    child_y,
                    child_row,
                    false,
                    base_y,
                    subtree_widths,
                    elements,
                );

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
        let text_width = self.config.text.width(text, self.config.font_size);
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
        // Размер шрифта подписи.  его не несёт,
        // поэтому передаём свойством: в эталоне подписи WBS нарисованы
        // при font-size=12, тогда как тема даёт 14.
        properties.insert("font-size".to_string(), self.config.font_size.to_string());
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

    /// Центрирует диаграмму.
    ///
    /// Сдвигает не только `bounds`, но и координаты внутри
    /// `properties["path"]`: связи хранятся готовой строкой с абсолютными
    /// координатами, поэтому без этого после центрирования коннекторы
    /// оставались на старом месте и расходились с узлами.
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

        // Если сдвиг не нужен, ничего не трогаем (в том числе не
        // переформатируем координаты путей)
        if offset_x == 0.0 && offset_y == 0.0 {
            return;
        }

        for element in elements.iter_mut() {
            element.bounds.x += offset_x;
            element.bounds.y += offset_y;

            if let Some(path) = element.properties.get("path").cloned() {
                element.properties.insert(
                    "path".to_string(),
                    shift_svg_path(&path, offset_x, offset_y),
                );
            }
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

    /// Сдвиг пути не должен менять его структуру и обязан смещать все координаты.
    #[test]
    fn test_shift_svg_path() {
        let path = "M10,20 L10,30 L40,30 L40,50";
        let shifted = shift_svg_path(path, 5.0, -3.0);
        assert_eq!(shifted, "M15,17 L15,27 L45,27 L45,47");
    }

    /// Дробные координаты сохраняются.
    #[test]
    fn test_shift_svg_path_fractional() {
        let shifted = shift_svg_path("M1.5,2.5 L3.5,4.5", 0.5, 0.5);
        assert_eq!(shifted, "M2,3 L4,5");
    }

    /// После центрирования координаты внутри `path` совпадают с границами.
    ///
    /// Регрессия: раньше `center_diagram` сдвигал только `element.bounds`,
    /// а строка пути оставалась со старыми абсолютными координатами, из-за
    /// чего связи рисовались мимо узлов.
    #[test]
    fn test_center_diagram_shifts_path() {
        let mut root = WbsNode::new(1, "Project");
        root.add_child(WbsNode::new(2, "Phase 1"));

        let diagram = WbsDiagram::with_root(root);
        let engine = WbsLayoutEngine::new();
        let result = engine.layout(&diagram);

        let connections: Vec<_> = result
            .elements
            .iter()
            .filter(|e| e.element_type == ElementType::Path)
            .collect();
        assert!(!connections.is_empty(), "соединения не созданы");

        for conn in connections {
            let path = conn.properties.get("path").expect("у соединения нет path");
            // Первая координата пути должна совпасть с левым верхом bounds
            let first = path
                .trim_start_matches('M')
                .split([' ', ','])
                .next()
                .and_then(|v| v.parse::<f64>().ok())
                .expect("не разобрать X пути");
            assert!(
                (first - conn.bounds.x).abs() < 0.01,
                "X пути ({first}) не совпадает с bounds.x ({}) после центрирования",
                conn.bounds.x
            );
        }
    }

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

    /// Ряды узлов соответствуют правилу, выведенному по девяти замерам.
    ///
    /// ```text
    /// ряд(корень) = 0
    /// дети корня  -> все в ряду 1
    /// ребёнок k не-корневого узла в ряду r -> ряд r + 1 + k
    /// ```
    #[test]
    fn test_row_rule() {
        // R[A[A1,A2], B[B1,B2]] — проверено на сервере.
        let mut a = WbsNode::new(1, "A");
        a.add_child(WbsNode::new(2, "A1"));
        a.add_child(WbsNode::new(3, "A2"));

        let mut b = WbsNode::new(4, "B");
        b.add_child(WbsNode::new(5, "B1"));
        b.add_child(WbsNode::new(6, "B2"));

        let mut root = WbsNode::new(0, "R");
        root.add_child(a);
        root.add_child(b);

        let diagram = WbsDiagram::with_root(root);
        let result = WbsLayoutEngine::new().layout(&diagram);

        let y_of = |text: &str| -> f64 {
            result
                .elements
                .iter()
                .find(|e| e.text.as_deref() == Some(text))
                .map(|e| e.bounds.y)
                .unwrap_or(f64::NAN)
        };

        let (r, a, b) = (y_of("R"), y_of("A"), y_of("B"));
        let (a1, b1) = (y_of("A1"), y_of("B1"));
        let (a2, b2) = (y_of("A2"), y_of("B2"));

        assert_eq!(a, b, "A и B должны делить ряд");
        assert_eq!(a1, b1, "A1 и B1 должны делить ряд");
        assert_eq!(a2, b2, "A2 и B2 должны делить ряд");
        assert!(a > r, "дети корня ниже корня");
        assert!(a1 > a, "второй ряд ниже первого");
        assert!(a2 > a1, "третий ряд ниже второго");
    }
}
