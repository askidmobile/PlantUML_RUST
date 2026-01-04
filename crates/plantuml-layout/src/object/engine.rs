//! Layout engine для Object Diagrams
//!
//! Конвертирует ObjectDiagram в структуру для рендеринга,
//! используя простой горизонтальный layout.

use plantuml_ast::object::{ObjectDiagram, ObjectLinkType};
use plantuml_model::{Point, Rect};

use super::ObjectLayoutConfig;
use crate::traits::LayoutResult;
use crate::{EdgeType, ElementType, LayoutElement};

/// Layout engine для Object Diagrams
pub struct ObjectLayoutEngine {
    config: ObjectLayoutConfig,
}

impl ObjectLayoutEngine {
    /// Создаёт новый layout engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: ObjectLayoutConfig::default(),
        }
    }

    /// Создаёт layout engine с заданной конфигурацией
    pub fn with_config(config: ObjectLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы объектов
    pub fn layout(&self, diagram: &ObjectDiagram) -> LayoutResult {
        let mut elements = Vec::new();
        let mut object_positions: std::collections::HashMap<String, Rect> =
            std::collections::HashMap::new();

        // 1. Размещаем объекты в сетке
        let objects_per_row = 4;
        let mut x = self.config.padding;
        let mut y = self.config.padding;
        let mut row_max_height = 0.0f64;
        let mut max_x = 0.0f64;
        let mut max_y = 0.0f64;

        for (i, object) in diagram.objects.iter().enumerate() {
            // Рассчитываем высоту объекта
            let header_height = 30.0;
            let fields_height = object.fields.len() as f64 * self.config.field_height;
            let object_height = (header_height + fields_height).max(self.config.object_min_height);

            // Определяем заголовок (с подчёркиванием как в UML)
            let display_name = object.display_name();

            // Создаём bounds
            let bounds = Rect::new(x, y, self.config.object_width, object_height);
            object_positions.insert(object.name.clone(), bounds.clone());

            // Создаём element для объекта
            elements.push(LayoutElement {
                id: format!("object_{}", object.name),
                bounds: bounds.clone(),
                text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Rectangle {
                    label: display_name,
                    corner_radius: 0.0, // Объекты без скруглённых углов
                },
            });

            // Добавляем поля как текст
            for (j, field) in object.fields.iter().enumerate() {
                let field_y = y + header_height + (j as f64 * self.config.field_height);
                let field_text = format!("{} = {}", field.name, field.value);

                elements.push(LayoutElement {
                    id: format!("field_{}_{}", object.name, j),
                    bounds: Rect::new(
                        x + 5.0,
                        field_y,
                        self.config.object_width - 10.0,
                        self.config.field_height,
                    ),
                    text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Text {
                        text: field_text,
                        font_size: 12.0,
                    },
                });
            }

            row_max_height = row_max_height.max(object_height);
            max_x = max_x.max(x + self.config.object_width);
            max_y = max_y.max(y + object_height);

            // Переход к следующей позиции
            if (i + 1) % objects_per_row == 0 {
                x = self.config.padding;
                y += row_max_height + self.config.vertical_spacing;
                row_max_height = 0.0;
            } else {
                x += self.config.object_width + self.config.horizontal_spacing;
            }
        }

        // 2. Добавляем связи
        for link in &diagram.links {
            if let (Some(from_bounds), Some(to_bounds)) =
                (object_positions.get(&link.from), object_positions.get(&link.to))
            {
                let from_center = from_bounds.center();
                let to_center = to_bounds.center();

                // Находим точки соединения на границах прямоугольников
                let (start, end) =
                    self.find_connection_points(from_bounds, to_bounds, from_center, to_center);

                let dashed = link.link_type.is_dashed();

                elements.push(LayoutElement {
                    id: format!("link_{}_{}", link.from, link.to),
                    bounds: Rect::new(
                        start.x.min(end.x),
                        start.y.min(end.y),
                        (end.x - start.x).abs(),
                        (end.y - start.y).abs(),
                    ),
                    text: None, properties: std::collections::HashMap::new(), element_type: ElementType::Edge {
                        points: vec![start, end],
                        label: link.label.clone(),
                        arrow_start: matches!(
                            link.link_type,
                            ObjectLinkType::Composition | ObjectLinkType::Aggregation
                        ),
                        arrow_end: !matches!(link.link_type, ObjectLinkType::Link),
                        dashed,
                        edge_type: match link.link_type {
                            ObjectLinkType::Composition => EdgeType::Composition,
                            ObjectLinkType::Aggregation => EdgeType::Aggregation,
                            ObjectLinkType::Dependency => EdgeType::Dependency,
                            ObjectLinkType::Association => EdgeType::Association,
                            ObjectLinkType::Link => EdgeType::Link,
                        },
                        from_cardinality: None,
                        to_cardinality: None,
                    },
                });
            }
        }

        // 3. Возвращаем результат
        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(
                0.0,
                0.0,
                max_x + self.config.padding,
                max_y + self.config.padding,
            ),
        };
        result.calculate_bounds();
        result
    }

    /// Находит точки соединения на границах прямоугольников
    fn find_connection_points(
        &self,
        from_bounds: &Rect,
        to_bounds: &Rect,
        from_center: Point,
        to_center: Point,
    ) -> (Point, Point) {
        let start = self.find_intersection_point(from_bounds, from_center, to_center);
        let end = self.find_intersection_point(to_bounds, to_center, from_center);
        (start, end)
    }

    /// Находит точку пересечения линии с прямоугольником
    fn find_intersection_point(&self, rect: &Rect, from: Point, to: Point) -> Point {
        let dx = to.x - from.x;
        let dy = to.y - from.y;

        // Находим пересечение с каждой стороной прямоугольника
        let mut best_t = f64::INFINITY;

        // Правая сторона
        if dx.abs() > 0.001 {
            let t = (rect.x + rect.width - from.x) / dx;
            let y = from.y + t * dy;
            if t > 0.0 && t < best_t && y >= rect.y && y <= rect.y + rect.height {
                best_t = t;
            }
        }

        // Левая сторона
        if dx.abs() > 0.001 {
            let t = (rect.x - from.x) / dx;
            let y = from.y + t * dy;
            if t > 0.0 && t < best_t && y >= rect.y && y <= rect.y + rect.height {
                best_t = t;
            }
        }

        // Нижняя сторона
        if dy.abs() > 0.001 {
            let t = (rect.y + rect.height - from.y) / dy;
            let x = from.x + t * dx;
            if t > 0.0 && t < best_t && x >= rect.x && x <= rect.x + rect.width {
                best_t = t;
            }
        }

        // Верхняя сторона
        if dy.abs() > 0.001 {
            let t = (rect.y - from.y) / dy;
            let x = from.x + t * dx;
            if t > 0.0 && t < best_t && x >= rect.x && x <= rect.x + rect.width {
                best_t = t;
            }
        }

        if best_t == f64::INFINITY {
            // Fallback: центр
            from
        } else {
            Point::new(from.x + best_t * dx, from.y + best_t * dy)
        }
    }
}

impl Default for ObjectLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::object::{Object, ObjectField, ObjectLink};

    #[test]
    fn test_layout_simple_objects() {
        let mut diagram = ObjectDiagram::new();
        diagram.add_object(Object::new("user1"));
        diagram.add_object(Object::new("user2"));

        let engine = ObjectLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть 2 объекта
        assert!(result.elements.len() >= 2);
    }

    #[test]
    fn test_layout_object_with_fields() {
        let mut diagram = ObjectDiagram::new();
        let mut obj = Object::new("user1");
        obj.add_field(ObjectField::new("name", "\"John\""));
        obj.add_field(ObjectField::new("age", "30"));
        diagram.add_object(obj);

        let engine = ObjectLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Объект + 2 поля
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_layout_with_links() {
        let mut diagram = ObjectDiagram::new();
        diagram.add_object(Object::new("user1"));
        diagram.add_object(Object::new("user2"));
        diagram.add_link(ObjectLink::new("user1", "user2").with_label("friend"));

        let engine = ObjectLayoutEngine::new();
        let result = engine.layout(&diagram);

        // 2 объекта + 1 связь
        assert!(result.elements.len() >= 3);
    }
}
