//! Layout engine для ER диаграмм
//!
//! Размещает сущности в виде таблиц с атрибутами.
//! Использует простой grid layout с оптимизацией для связей.

use std::collections::HashMap;

use plantuml_ast::er::{ErDiagram, Entity};
use plantuml_model::{Point, Rect, Size};

use crate::er::config::ErLayoutConfig;
use crate::traits::{LayoutEngine, LayoutResult};
use crate::{EdgeType, ElementType, LayoutConfig, LayoutElement};

/// Layout engine для ER диаграмм
pub struct ErLayoutEngine {
    config: ErLayoutConfig,
}

impl ErLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: ErLayoutConfig::default(),
        }
    }

    /// Создаёт engine с указанной конфигурацией
    pub fn with_config(config: ErLayoutConfig) -> Self {
        Self { config }
    }

    /// Вычисляет размер сущности
    fn calculate_entity_size(&self, entity: &Entity) -> Size {
        let width = self.config.min_entity_width.max(
            entity.id.name.len() as f64 * 9.0 + self.config.entity_padding * 2.0,
        );

        // Находим максимальную ширину атрибута
        let max_attr_width = entity
            .attributes
            .iter()
            .map(|a| {
                let type_str = a.data_type.as_deref().unwrap_or("");
                let stereo_str = a.stereotype.as_deref().map(|s| format!(" <<{}>>", s)).unwrap_or_default();
                (a.name.len() + type_str.len() + stereo_str.len() + 3) as f64 * 7.5
            })
            .fold(0.0, f64::max);

        let final_width = width.max(max_attr_width + self.config.entity_padding * 2.0);

        let height = self.config.entity_header_height
            + entity.attributes.len() as f64 * self.config.attribute_height
            + self.config.entity_padding;

        Size::new(final_width, height)
    }

    /// Размещает сущности в grid
    fn layout_entities(
        &self,
        diagram: &ErDiagram,
        elements: &mut Vec<LayoutElement>,
    ) -> HashMap<String, Rect> {
        let mut positions: HashMap<String, Rect> = HashMap::new();

        // Простой grid layout: размещаем сущности в ряд
        let entities_per_row = 3;
        let mut x = self.config.padding;
        let mut y = self.config.padding;
        let mut row_height = 0.0_f64;
        let mut col = 0;

        for entity in &diagram.entities {
            let size = self.calculate_entity_size(entity);

            if col >= entities_per_row {
                col = 0;
                x = self.config.padding;
                y += row_height + self.config.vertical_spacing;
                row_height = 0.0;
            }

            let bounds = Rect::new(x, y, size.width, size.height);
            positions.insert(entity.id.name.clone(), bounds);

            // Рисуем сущность
            self.render_entity(entity, &bounds, elements);

            x += size.width + self.config.horizontal_spacing;
            row_height = row_height.max(size.height);
            col += 1;
        }

        positions
    }

    /// Рендерит одну сущность
    fn render_entity(
        &self,
        entity: &Entity,
        bounds: &Rect,
        elements: &mut Vec<LayoutElement>,
    ) {
        let entity_id = &entity.id.name;

        // Фон сущности
        let bg = LayoutElement {
            id: format!("entity_{}_bg", entity_id),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
            bounds: *bounds,
            text: None,
            properties: [
                ("fill".to_string(), self.config.entity_bg_color.to_string()),
                ("stroke".to_string(), "#181818".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(bg);

        // Заголовок
        let header_bounds = Rect::new(
            bounds.x,
            bounds.y,
            bounds.width,
            self.config.entity_header_height,
        );

        let header_bg = LayoutElement {
            id: format!("entity_{}_header_bg", entity_id),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: 0.0,
            },
            bounds: header_bounds,
            text: None,
            properties: [
                ("fill".to_string(), self.config.header_bg_color.to_string()),
                ("stroke".to_string(), "#181818".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(header_bg);

        // Название сущности
        let header_text = LayoutElement {
            id: format!("entity_{}_name", entity_id),
            element_type: ElementType::Text {
                text: entity.id.name.clone(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(
                bounds.x + self.config.entity_padding,
                bounds.y + 5.0,
                bounds.width - self.config.entity_padding * 2.0,
                self.config.entity_header_height - 10.0,
            ),
            text: Some(entity.id.name.clone()),
            properties: [
                ("fill".to_string(), "#000000".to_string()),
                ("font-weight".to_string(), "bold".to_string()),
                ("text-anchor".to_string(), "middle".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(header_text);

        // Атрибуты
        let mut attr_y = bounds.y + self.config.entity_header_height;
        for (i, attr) in entity.attributes.iter().enumerate() {
            let prefix = if attr.is_required { "* " } else { "  " };
            let type_str = attr
                .data_type
                .as_ref()
                .map(|t| format!(" : {}", t))
                .unwrap_or_default();
            let stereo_str = attr
                .stereotype
                .as_ref()
                .map(|s| format!(" <<{}>>", s))
                .unwrap_or_default();

            let attr_text = format!("{}{}{}{}", prefix, attr.name, type_str, stereo_str);

            let attr_element = LayoutElement {
                id: format!("entity_{}_attr_{}", entity_id, i),
                element_type: ElementType::Text {
                    text: attr_text.clone(),
                    font_size: self.config.font_size - 1.0,
                },
                bounds: Rect::new(
                    bounds.x + self.config.entity_padding,
                    attr_y,
                    bounds.width - self.config.entity_padding * 2.0,
                    self.config.attribute_height,
                ),
                text: Some(attr_text),
                properties: [("fill".to_string(), "#000000".to_string())]
                    .into_iter()
                    .collect(),
            };
            elements.push(attr_element);

            attr_y += self.config.attribute_height;
        }
    }

    /// Рендерит связи между сущностями
    fn render_relationships(
        &self,
        diagram: &ErDiagram,
        positions: &HashMap<String, Rect>,
        elements: &mut Vec<LayoutElement>,
    ) {
        for (i, rel) in diagram.relationships.iter().enumerate() {
            let from_bounds = match positions.get(&rel.from) {
                Some(b) => b,
                None => continue,
            };
            let to_bounds = match positions.get(&rel.to) {
                Some(b) => b,
                None => continue,
            };

            // Вычисляем точки соединения
            let from_center = Point::new(
                from_bounds.x + from_bounds.width / 2.0,
                from_bounds.y + from_bounds.height / 2.0,
            );
            let to_center = Point::new(
                to_bounds.x + to_bounds.width / 2.0,
                to_bounds.y + to_bounds.height / 2.0,
            );

            // Определяем точки на границах
            let (from_point, to_point) =
                self.calculate_connection_points(from_bounds, to_bounds, from_center, to_center);

            // Линия связи
            let edge = LayoutElement {
                id: format!("rel_{}", i),
                element_type: ElementType::Edge {
                    points: vec![from_point, to_point],
                    label: rel.label.clone(),
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
                },
                bounds: Rect::from_points(from_point, to_point),
                text: rel.label.clone(),
                properties: [
                    ("stroke".to_string(), "#181818".to_string()),
                    (
                        "from_card".to_string(),
                        rel.from_cardinality.symbol().to_string(),
                    ),
                    (
                        "to_card".to_string(),
                        rel.to_cardinality.symbol().to_string(),
                    ),
                ]
                .into_iter()
                .collect(),
            };
            elements.push(edge);
        }
    }

    /// Вычисляет точки соединения на границах прямоугольников
    fn calculate_connection_points(
        &self,
        from: &Rect,
        to: &Rect,
        from_center: Point,
        to_center: Point,
    ) -> (Point, Point) {
        // Простой расчёт: соединяем ближайшие стороны
        let dx = to_center.x - from_center.x;
        let dy = to_center.y - from_center.y;

        let from_point = if dx.abs() > dy.abs() {
            // Горизонтальное соединение
            if dx > 0.0 {
                Point::new(from.x + from.width, from_center.y)
            } else {
                Point::new(from.x, from_center.y)
            }
        } else {
            // Вертикальное соединение
            if dy > 0.0 {
                Point::new(from_center.x, from.y + from.height)
            } else {
                Point::new(from_center.x, from.y)
            }
        };

        let to_point = if dx.abs() > dy.abs() {
            if dx > 0.0 {
                Point::new(to.x, to_center.y)
            } else {
                Point::new(to.x + to.width, to_center.y)
            }
        } else {
            if dy > 0.0 {
                Point::new(to_center.x, to.y)
            } else {
                Point::new(to_center.x, to.y + to.height)
            }
        };

        (from_point, to_point)
    }
}

impl Default for ErLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for ErLayoutEngine {
    type Input = ErDiagram;

    fn layout(&self, diagram: &Self::Input, _config: &LayoutConfig) -> LayoutResult {
        let mut elements = Vec::new();

        // Размещаем сущности
        let positions = self.layout_entities(diagram, &mut elements);

        // Рисуем связи
        self.render_relationships(diagram, &positions, &mut elements);

        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        // Добавляем padding
        result.bounds.width += self.config.padding;
        result.bounds.height += self.config.padding;

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::er::{Attribute, ErRelationship};

    #[test]
    fn test_layout_simple_er() {
        let mut diagram = ErDiagram::new();

        let mut user = Entity::new("User");
        user.add_attribute(Attribute::new("id").with_type("int").as_primary_key());
        user.add_attribute(Attribute::new("name").with_type("varchar"));
        diagram.add_entity(user);

        let engine = ErLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        assert!(!result.elements.is_empty());
        assert!(result.bounds.width > 0.0);
        assert!(result.bounds.height > 0.0);
    }

    #[test]
    fn test_layout_er_with_relationship() {
        let mut diagram = ErDiagram::new();

        diagram.add_entity(Entity::new("User"));
        diagram.add_entity(Entity::new("Order"));
        diagram.add_relationship(ErRelationship::new("User", "Order"));

        let engine = ErLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        // Должны быть элементы для обеих сущностей и связи
        assert!(result.elements.len() >= 5);
    }
}
