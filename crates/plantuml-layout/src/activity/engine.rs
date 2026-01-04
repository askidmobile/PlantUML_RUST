//! Activity Diagram Layout Engine
//!
//! Flowchart-based layout algorithm для activity diagrams.

use plantuml_ast::activity::{
    Action, ActivityDiagram, ActivityElement, Condition, Fork, RepeatLoop, WhileLoop,
};
use plantuml_model::{Point, Rect};

use super::config::ActivityLayoutConfig;
use crate::{EdgeType, ElementType, LayoutElement, LayoutResult};

/// Layout engine для activity diagrams
pub struct ActivityLayoutEngine {
    config: ActivityLayoutConfig,
}

impl ActivityLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: ActivityLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: ActivityLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &ActivityDiagram) -> LayoutResult {
        let mut elements = Vec::new();
        let mut current_y = self.config.margin;
        
        // Центр диаграммы по X
        let center_x = self.config.margin + self.config.action_width / 2.0;

        // Обрабатываем элементы последовательно
        for element in &diagram.elements {
            current_y = self.layout_element(element, center_x, current_y, &mut elements);
        }

        // Вычисляем bounds
        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        // Добавляем отступы
        result.bounds.width += self.config.margin;
        result.bounds.height += self.config.margin;

        result
    }

    /// Располагает элемент и возвращает новую Y позицию
    fn layout_element(
        &self,
        element: &ActivityElement,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        match element {
            ActivityElement::Start => self.layout_start(center_x, current_y, elements),
            ActivityElement::Stop => self.layout_stop(center_x, current_y, elements),
            ActivityElement::End => self.layout_end(center_x, current_y, elements),
            ActivityElement::Action(action) => {
                self.layout_action(action, center_x, current_y, elements)
            }
            ActivityElement::Condition(cond) => {
                self.layout_condition(cond, center_x, current_y, elements)
            }
            ActivityElement::While(while_loop) => {
                self.layout_while(while_loop, center_x, current_y, elements)
            }
            ActivityElement::Repeat(repeat_loop) => {
                self.layout_repeat(repeat_loop, center_x, current_y, elements)
            }
            ActivityElement::Fork(fork) => {
                self.layout_fork(fork, center_x, current_y, elements)
            }
            ActivityElement::Detach | ActivityElement::Kill => {
                // Detach/Kill просто прерывают поток, не рисуем ничего
                current_y
            }
            ActivityElement::Note(_) => {
                // TODO: реализовать заметки
                current_y
            }
            ActivityElement::SwimlaneChange(_) | ActivityElement::Connector(_) => {
                // TODO: swimlanes и коннекторы
                current_y
            }
        }
    }

    /// Располагает начальный узел (filled circle)
    fn layout_start(
        &self,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let r = self.config.node_radius;
        
        elements.push(LayoutElement {
            id: format!("start_{}", elements.len()),
            bounds: Rect::new(center_x - r, current_y, r * 2.0, r * 2.0),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Ellipse { label: None },
        });

        let next_y = current_y + r * 2.0 + self.config.vertical_spacing;

        // Стрелка вниз от start
        self.add_arrow(
            center_x,
            current_y + r * 2.0,
            center_x,
            next_y,
            None,
            elements,
        );

        next_y
    }

    /// Располагает конечный узел (filled circle with ring)
    fn layout_stop(
        &self,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let r = self.config.node_radius;
        
        elements.push(LayoutElement {
            id: format!("stop_{}", elements.len()),
            bounds: Rect::new(center_x - r, current_y, r * 2.0, r * 2.0),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Ellipse { 
                label: Some("●".to_string()) // Внутренний круг
            },
        });

        current_y + r * 2.0 + self.config.vertical_spacing
    }

    /// Располагает конечный узел (альтернативный)
    fn layout_end(
        &self,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        self.layout_stop(center_x, current_y, elements)
    }

    /// Располагает действие (rounded rectangle)
    fn layout_action(
        &self,
        action: &Action,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let w = self.config.action_width;
        let h = self.config.action_height;

        elements.push(LayoutElement {
            id: format!("action_{}", elements.len()),
            bounds: Rect::new(center_x - w / 2.0, current_y, w, h),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
                label: action.label.clone(),
                corner_radius: self.config.action_corner_radius,
            },
        });

        let next_y = current_y + h + self.config.vertical_spacing;

        // Стрелка вниз
        self.add_arrow(center_x, current_y + h, center_x, next_y, None, elements);

        next_y
    }

    /// Располагает условие (if/else)
    fn layout_condition(
        &self,
        cond: &Condition,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let dw = self.config.diamond_width;
        let dh = self.config.diamond_height;

        // Ромб условия
        elements.push(LayoutElement {
            id: format!("diamond_{}", elements.len()),
            bounds: Rect::new(center_x - dw / 2.0, current_y, dw, dh),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
                text: cond.condition.clone(),
                font_size: 12.0,
            },
        });

        let after_diamond = current_y + dh + self.config.vertical_spacing / 2.0;
        let branch_start_y = after_diamond;

        // Then branch (left)
        let left_x = center_x - self.config.horizontal_spacing;
        let mut then_end_y = branch_start_y;
        
        // Стрелка от ромба влево + вниз
        self.add_arrow(
            center_x - dw / 2.0,
            current_y + dh / 2.0,
            left_x,
            branch_start_y,
            cond.then_label.clone(),
            elements,
        );

        for elem in &cond.then_branch {
            then_end_y = self.layout_element(elem, left_x, then_end_y, elements);
        }

        // Else branch (right) if exists
        let mut else_end_y = branch_start_y;
        if let Some(else_branch) = &cond.else_branch {
            let right_x = center_x + self.config.horizontal_spacing;
            
            // Стрелка от ромба вправо + вниз
            self.add_arrow(
                center_x + dw / 2.0,
                current_y + dh / 2.0,
                right_x,
                branch_start_y,
                cond.else_label.clone(),
                elements,
            );

            for elem in else_branch {
                else_end_y = self.layout_element(elem, right_x, else_end_y, elements);
            }
        }

        // Точка слияния
        let merge_y = then_end_y.max(else_end_y);
        
        // Стрелки к точке слияния
        if then_end_y < merge_y {
            self.add_arrow(left_x, then_end_y, center_x, merge_y, None, elements);
        }
        if cond.else_branch.is_some() && else_end_y < merge_y {
            let right_x = center_x + self.config.horizontal_spacing;
            self.add_arrow(right_x, else_end_y, center_x, merge_y, None, elements);
        }

        merge_y + self.config.vertical_spacing
    }

    /// Располагает цикл while
    fn layout_while(
        &self,
        while_loop: &WhileLoop,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let dw = self.config.diamond_width;
        let dh = self.config.diamond_height;

        // Ромб условия
        elements.push(LayoutElement {
            id: format!("while_diamond_{}", elements.len()),
            bounds: Rect::new(center_x - dw / 2.0, current_y, dw, dh),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
                text: while_loop.condition.clone(),
                font_size: 12.0,
            },
        });

        let body_start_y = current_y + dh + self.config.vertical_spacing;
        let mut body_end_y = body_start_y;

        // Тело цикла
        for elem in &while_loop.body {
            body_end_y = self.layout_element(elem, center_x, body_end_y, elements);
        }

        // Обратная стрелка (loop back)
        let loop_x = center_x - self.config.horizontal_spacing - 20.0;
        
        // Вниз -> влево -> вверх -> вправо к ромбу
        elements.push(LayoutElement {
            id: format!("while_loop_{}", elements.len()),
            bounds: Rect::new(loop_x, current_y, center_x - loop_x, body_end_y - current_y),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                points: vec![
                    Point::new(center_x, body_end_y - self.config.vertical_spacing),
                    Point::new(loop_x, body_end_y - self.config.vertical_spacing),
                    Point::new(loop_x, current_y + dh / 2.0),
                    Point::new(center_x - dw / 2.0, current_y + dh / 2.0),
                ],
                label: while_loop.backward_label.clone(),
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association, from_cardinality: None, to_cardinality: None,
            },
        });

        // Стрелка выхода из цикла (вправо)
        let exit_x = center_x + self.config.horizontal_spacing;
        self.add_arrow(
            center_x + dw / 2.0,
            current_y + dh / 2.0,
            exit_x,
            body_end_y,
            while_loop.end_label.clone(),
            elements,
        );

        body_end_y + self.config.vertical_spacing
    }

    /// Располагает цикл repeat
    fn layout_repeat(
        &self,
        repeat_loop: &RepeatLoop,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let body_start_y = current_y;
        let mut body_end_y = body_start_y;

        // Тело цикла (выполняется первым)
        for elem in &repeat_loop.body {
            body_end_y = self.layout_element(elem, center_x, body_end_y, elements);
        }

        // Ромб условия внизу
        let dw = self.config.diamond_width;
        let dh = self.config.diamond_height;

        elements.push(LayoutElement {
            id: format!("repeat_diamond_{}", elements.len()),
            bounds: Rect::new(center_x - dw / 2.0, body_end_y, dw, dh),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
                text: repeat_loop.condition.clone(),
                font_size: 12.0,
            },
        });

        // Обратная стрелка
        let loop_x = center_x + self.config.horizontal_spacing + 20.0;
        
        elements.push(LayoutElement {
            id: format!("repeat_loop_{}", elements.len()),
            bounds: Rect::new(center_x, body_start_y, loop_x - center_x, body_end_y - body_start_y + dh),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                points: vec![
                    Point::new(center_x + dw / 2.0, body_end_y + dh / 2.0),
                    Point::new(loop_x, body_end_y + dh / 2.0),
                    Point::new(loop_x, body_start_y),
                    Point::new(center_x, body_start_y),
                ],
                label: repeat_loop.backward_label.clone(),
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association, from_cardinality: None, to_cardinality: None,
            },
        });

        body_end_y + dh + self.config.vertical_spacing
    }

    /// Располагает fork/join
    fn layout_fork(
        &self,
        fork: &Fork,
        center_x: f64,
        current_y: f64,
        elements: &mut Vec<LayoutElement>,
    ) -> f64 {
        let num_branches = fork.branches.len();
        if num_branches == 0 {
            return current_y;
        }

        // Fork bar
        let total_width = (num_branches as f64 - 1.0) * self.config.horizontal_spacing + 
                          self.config.action_width;
        let fork_bar_x = center_x - total_width / 2.0;

        elements.push(LayoutElement {
            id: format!("fork_bar_{}", elements.len()),
            bounds: Rect::new(
                fork_bar_x,
                current_y,
                total_width,
                self.config.bar_height,
            ),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
        });

        let branches_start_y = current_y + self.config.bar_height + self.config.vertical_spacing;
        let mut max_branch_end_y = branches_start_y;

        // Располагаем каждую ветку
        let branch_spacing = if num_branches > 1 {
            total_width / (num_branches as f64 - 1.0)
        } else {
            0.0
        };

        for (i, branch) in fork.branches.iter().enumerate() {
            let branch_x = if num_branches > 1 {
                fork_bar_x + i as f64 * branch_spacing
            } else {
                center_x
            };

            // Стрелка от fork bar к началу ветки
            self.add_arrow(
                branch_x,
                current_y + self.config.bar_height,
                branch_x,
                branches_start_y,
                None,
                elements,
            );

            let mut branch_y = branches_start_y;
            for elem in branch {
                branch_y = self.layout_element(elem, branch_x, branch_y, elements);
            }

            max_branch_end_y = max_branch_end_y.max(branch_y);
        }

        // Join bar
        let join_y = max_branch_end_y;
        
        elements.push(LayoutElement {
            id: format!("join_bar_{}", elements.len()),
            bounds: Rect::new(
                fork_bar_x,
                join_y,
                total_width,
                self.config.bar_height,
            ),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
        });

        // Стрелки от веток к join bar
        for i in 0..num_branches {
            let branch_x = if num_branches > 1 {
                fork_bar_x + i as f64 * branch_spacing
            } else {
                center_x
            };

            self.add_arrow(
                branch_x,
                max_branch_end_y - self.config.vertical_spacing,
                branch_x,
                join_y,
                None,
                elements,
            );
        }

        join_y + self.config.bar_height + self.config.vertical_spacing
    }

    /// Добавляет стрелку
    fn add_arrow(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        label: Option<String>,
        elements: &mut Vec<LayoutElement>,
    ) {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);

        elements.push(LayoutElement {
            id: format!("arrow_{}", elements.len()),
            bounds: Rect::new(min_x, min_y, (max_x - min_x).max(1.0), (max_y - min_y).max(1.0)),
            text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                points: vec![Point::new(x1, y1), Point::new(x2, y2)],
                label,
                arrow_start: false,
                arrow_end: true,
                dashed: false,
                edge_type: EdgeType::Association, from_cardinality: None, to_cardinality: None,
            },
        });
    }
}

impl Default for ActivityLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::activity::ActionStyle;

    #[test]
    fn test_layout_simple() {
        let mut diagram = ActivityDiagram::new();
        diagram.elements.push(ActivityElement::Start);
        diagram.elements.push(ActivityElement::Action(Action {
            label: "Hello".to_string(),
            background_color: None,
            style: ActionStyle::Normal,
            arrow_label: None,
        }));
        diagram.elements.push(ActivityElement::Stop);

        let engine = ActivityLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должно быть: start circle + arrow + action rect + arrow + stop circle
        assert!(result.elements.len() >= 5);
    }

    #[test]
    fn test_layout_condition() {
        let mut diagram = ActivityDiagram::new();
        diagram.elements.push(ActivityElement::Start);
        diagram.elements.push(ActivityElement::Condition(Condition {
            condition: "test?".to_string(),
            then_branch: vec![ActivityElement::Action(Action::new("yes"))],
            then_label: Some("yes".to_string()),
            elseif_branches: vec![],
            else_branch: Some(vec![ActivityElement::Action(Action::new("no"))]),
            else_label: Some("no".to_string()),
        }));
        diagram.elements.push(ActivityElement::Stop);

        let engine = ActivityLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть элементы для обеих веток
        assert!(result.elements.len() >= 8);
    }

    #[test]
    fn test_layout_fork() {
        let mut diagram = ActivityDiagram::new();
        diagram.elements.push(ActivityElement::Start);
        diagram.elements.push(ActivityElement::Fork(Fork {
            branches: vec![
                vec![ActivityElement::Action(Action::new("task1"))],
                vec![ActivityElement::Action(Action::new("task2"))],
            ],
            join_type: plantuml_ast::activity::JoinType::And,
        }));
        diagram.elements.push(ActivityElement::Stop);

        let engine = ActivityLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть fork bar, 2 ветки, join bar
        assert!(result.elements.len() >= 10);
    }
}
