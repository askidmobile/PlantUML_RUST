//! Layout engine для ER диаграмм
//!
//! Размещает сущности в виде таблиц с атрибутами.
//! Использует простой grid layout с оптимизацией для связей.

use std::collections::HashMap;

use plantuml_ast::common::{ArrowShape, Direction};
use plantuml_ast::er::{Entity, ErDiagram};
use plantuml_model::{Point, Rect, Size};

use crate::er::config::ErLayoutConfig;
use crate::graph::scene::{Scene, SceneEdge};
use crate::graph::Direction as GraphDirection;
use crate::text::TextMeasurer;
use crate::traits::{LayoutEngine, LayoutResult};
use crate::{EdgeType, ElementType, LayoutConfig, LayoutElement};

/// Зазор между сущностями одного ранга.
const ER_NODE_SEP: f64 = 35.0;

/// Заливка блока сущности ER (эталон `er_basic`: `#F1F1F1`).
const ER_ENTITY_FILL: &str = "#F1F1F1";

/// Скругление рамки сущности (эталон: `rx="2.5"`).
const ER_ENTITY_CORNER_RADIUS: f64 = 2.5;

/// Добавка к ширине подписи при расчёте ширины шапки.
///
/// Эталон `er_basic`: «Order» 40.03 при рамке 72.03, «User» 31.91 при
/// содержимом 63.91 — то есть кружок 22, зазор 3 и поле 7.
const ER_HEADER_WIDTH_EXTRA: f64 = 32.0;

/// Радиус кружка-иконки сущности (эталон: `rx="11"`).
const ER_SPOT_RADIUS: f64 = 11.0;

/// Смещение центра кружка от верха рамки (эталон: 23 − 7).
const ER_SPOT_OFFSET_Y: f64 = 16.0;

/// Заливка кружка-иконки (эталон: `#ADD1B2`).
const ER_SPOT_FILL: &str = "#ADD1B2";

/// Зазор между кружком и подписью (эталон: 58.88 − 40.719 − 11).
const ER_SPOT_NAME_GAP: f64 = 7.161;

/// Опускание буквы в кружке от его центра до базиса (как у классов).
const ER_SPOT_GLYPH_DROP: f64 = 5.6;

/// Толщина разделителя `--` между секциями тела (эталон `real_er_real`).
const ER_SECTION_LINE_WIDTH: f64 = 1.0;

/// Низ секции атрибутов до следующей черты.
const ER_SECTION_TAIL: f64 = 8.0;

/// Базис подписи от верха рамки (эталон: 27.85 − 7).
const ER_NAME_BASELINE: f64 = 20.85;

/// Высота полосы имени — по разделителю под шапкой (эталон: 39 − 7).
const ER_NAME_BAND: f64 = 32.0;

/// Базис первой строки атрибутов от конца полосы имени.
///
/// Эталон `er_basic`: «id : int» на y = 55.99 при рамке от 7 и полосе до
/// 39, то есть 16.99 ниже разделителя.
const ER_ATTR_BASELINE: f64 = 16.99;

/// Радиус кружка, помечающего обязательный атрибут.
///
/// Измерено по эталону `er_basic`: `rx=3`, `fill=#000`.
const ER_REQUIRED_MARKER_RADIUS: f64 = 3.0;

/// Положение центра кружка обязательного атрибута от левого края рамки.
const ER_REQUIRED_MARKER_X: f64 = 11.0;

/// Смещение кружка по вертикали внутри строки атрибута.
///
/// Эталон: кружок на 45.648 от верха рамки, подпись на 48.995 —
/// то есть кружок на 3.347 выше базовой линии.
const ER_REQUIRED_MARKER_DY: f64 = 13.648;

/// Отступ подписи обязательного атрибута от края рамки.
const ER_REQUIRED_TEXT_OFFSET: f64 = 20.0;

/// Отступ подписи обычного атрибута от края рамки.
const ER_PLAIN_TEXT_OFFSET: f64 = 6.0;

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
        // Ширина шапки: кружок, зазор и подпись.
        //
        // Проверено по эталону `er_basic`: «Order» (40.03) даёт 72.03 =
        // 40.03 + 32, «User» (31.91) — 63.91, но у «User» шире атрибут,
        // поэтому блок берёт максимум из двух слагаемых.
        let name_width = self
            .config
            .text
            .width(&entity.id.name, self.config.font_size);
        let header_width = name_width + ER_HEADER_WIDTH_EXTRA;
        let width = self.config.min_entity_width.max(header_width);

        // Находим максимальную ширину атрибута
        let max_attr_width = entity
            .attributes
            .iter()
            .map(|a| {
                // ВАЖНО: измеряем РОВНО ту строку, которая рисуется.
                // Здесь терялся разделитель « : », из-за чего «name :
                // string» мерилось как «namestring» — блок «User» выходил
                // 94.89 вместо эталонных 105.51.
                let type_str = a
                    .data_type
                    .as_deref()
                    .map(|t| format!(" : {t}"))
                    .unwrap_or_default();
                let stereo_str = a
                    .stereotype
                    .as_deref()
                    // PlantUML рисует стереотип КАВЫЧКАМИ-ЁЛОЧКАМИ, а не
                    // угловыми скобками: в эталоне `ER: Диаграмма` поле
                    // выглядит как `id : int «PK»` шириной 83.8, тогда как
                    // `id : int <<PK>>` даёт 171.3 — почти вдвое больше.
                    .map(|s| format!(" «{s}»"))
                    .unwrap_or_default();
                // Одна согласованная мера вместо двух разных констант
                // (9.0 для имени и 7.5 для атрибутов)
                // Без добавочных 3: эталон даёт ровно «текст + 12»
                // (105.51 при «name : string» 93.51).
                let text_width = self.config.text.width(
                    &format!("{}{}{}", a.name, type_str, stereo_str),
                    self.config.font_size,
                );
                // Отступ слева у обязательного атрибута больше: перед ним кружок.
                let left = if a.is_required {
                    ER_REQUIRED_TEXT_OFFSET
                } else {
                    ER_PLAIN_TEXT_OFFSET
                };
                text_width + left + ER_PLAIN_TEXT_OFFSET
            })
            .fold(0.0, f64::max);

        // Атрибут занимает «текст + 12»: эталон даёт 105.51 при «name :
        // string» 93.51 и 56.65 при «id : int» 44.65.
        let final_width = width.max(max_attr_width);

        // Лишние секции (`--` больше одного раза) добавляют по такому же хвосту.
        let height = self.config.entity_header_height
            + entity.attributes.len() as f64 * self.config.attribute_height
            + self.config.entity_padding
            + ER_SECTION_TAIL * entity.separators.len().saturating_sub(1) as f64;

        Size::new(final_width, height)
    }

    /// Рендерит одну сущность
    fn render_entity(&self, entity: &Entity, bounds: &Rect, elements: &mut Vec<LayoutElement>) {
        let entity_id = &entity.id.name;

        // Рамка сущности — ОДИН блок, как у класса.
        //
        // Эталон `er_basic`: рамка 105.51x80.59, `fill="#F1F1F1"`,
        // `rx="2.5"`, толщина 0.5. Прежний код рисовал два прямоугольника
        // (жёлтая заливка + цветная шапка) и подпись по центру.
        let bg = LayoutElement {
            id: format!("entity_{}_bg", entity_id),
            element_type: ElementType::Rectangle {
                label: String::new(),
                corner_radius: ER_ENTITY_CORNER_RADIUS,
            },
            bounds: *bounds,
            text: None,
            properties: [
                ("fill".to_string(), ER_ENTITY_FILL.to_string()),
                ("rx".to_string(), ER_ENTITY_CORNER_RADIUS.to_string()),
                ("stroke".to_string(), "#181818".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(bg);

        // Кружок-иконка с буквой «E», как у классов: пара «кружок + подпись»
        // стоит по центру шапки. Буква — `<text data-glyph="1">`: PlantUML
        // рисует её контуром, и сверка подписей её словом не считает.
        let name_width = self
            .config
            .text
            .width(&entity.id.name, self.config.font_size);
        let pair_width = ER_SPOT_RADIUS * 2.0 + ER_SPOT_NAME_GAP + name_width;
        let pair_left = bounds.x + ((bounds.width - pair_width) / 2.0).max(2.0);
        let spot_x = pair_left + ER_SPOT_RADIUS;
        let spot_y = bounds.y + ER_SPOT_OFFSET_Y;
        elements.push(LayoutElement {
            id: format!("entity_{}_spot", entity_id),
            element_type: ElementType::Ellipse { label: None },
            bounds: Rect::new(
                spot_x - ER_SPOT_RADIUS,
                spot_y - ER_SPOT_RADIUS,
                ER_SPOT_RADIUS * 2.0,
                ER_SPOT_RADIUS * 2.0,
            ),
            text: None,
            properties: [
                ("fill".to_string(), ER_SPOT_FILL.to_string()),
                ("stroke".to_string(), "#181818".to_string()),
            ]
            .into_iter()
            .collect(),
        });
        elements.push(LayoutElement {
            id: format!("entity_{}_spot_letter", entity_id),
            element_type: ElementType::Text {
                text: "E".to_string(),
                font_size: 17.0,
            },
            // Базис буквы — низ `bounds`: так он едет вместе с узлом при переносе.
            bounds: Rect::new(spot_x, spot_y + ER_SPOT_GLYPH_DROP - 17.0, 0.0, 17.0),
            text: None,
            properties: [("data-glyph".to_string(), "1".to_string())]
                .into_iter()
                .collect(),
        });

        // Название — ОБЫЧНЫМ начертанием справа от кружка.
        //
        // Эталон: «User» кеглем 14 на x = 58.88 при кружке на 40.719,
        // базис — на 20.85 ниже верха рамки.
        let name_x = pair_left + ER_SPOT_RADIUS * 2.0 + ER_SPOT_NAME_GAP;
        elements.push(LayoutElement {
            id: format!("entity_{}_name", entity_id),
            element_type: ElementType::Text {
                text: entity.id.name.clone(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(
                name_x,
                bounds.y + ER_NAME_BASELINE - self.config.font_size,
                self.config
                    .text
                    .width(&entity.id.name, self.config.font_size),
                self.config.font_size,
            ),
            text: None,
            properties: [("fill".to_string(), "#000000".to_string())]
                .into_iter()
                .collect(),
        });

        // Разделитель под шапкой: эталон — линия на 32 ниже верха.
        elements.push(LayoutElement {
            id: format!("entity_{}_separator", entity_id),
            element_type: ElementType::Edge {
                points: vec![
                    Point::new(bounds.x + 1.0, bounds.y + ER_NAME_BAND),
                    Point::new(bounds.x + bounds.width - 1.0, bounds.y + ER_NAME_BAND),
                ],
                label: None,
                arrow_start: false,
                arrow_end: false,
                dashed: false,
                edge_type: EdgeType::Link,
                from_cardinality: None,
                to_cardinality: None,
            },
            bounds: Rect::new(
                bounds.x + 1.0,
                bounds.y + ER_NAME_BAND,
                bounds.width - 2.0,
                0.0,
            ),
            text: None,
            properties: HashMap::new(),
        });

        // Атрибуты
        let mut attr_y = bounds.y + self.config.entity_header_height;
        for (i, attr) in entity.attributes.iter().enumerate() {
            // `--` делит тело на секции: жирная черта через всю ширину.
            if entity.separators.contains(&i) {
                let line_y = attr_y + ER_SECTION_TAIL;
                elements.push(LayoutElement {
                    id: format!("entity_{}_section_{}", entity_id, i),
                    element_type: ElementType::Edge {
                        points: vec![
                            Point::new(bounds.x + 1.0, line_y),
                            Point::new(bounds.x + bounds.width - 1.0, line_y),
                        ],
                        label: None,
                        arrow_start: false,
                        arrow_end: false,
                        dashed: false,
                        edge_type: EdgeType::Link,
                        from_cardinality: None,
                        to_cardinality: None,
                    },
                    bounds: Rect::new(bounds.x + 1.0, line_y, bounds.width - 2.0, 0.0),
                    text: None,
                    properties: [(
                        "stroke-width".to_string(),
                        ER_SECTION_LINE_WIDTH.to_string(),
                    )]
                    .into_iter()
                    .collect(),
                });
                attr_y = line_y;
            }
            let type_str = attr
                .data_type
                .as_ref()
                .map(|t| format!(" : {}", t))
                .unwrap_or_default();
            let stereo_str = attr
                .stereotype
                .as_ref()
                .map(|s| format!(" «{s}»"))
                .unwrap_or_default();

            // Обязательный атрибут помечается КРУЖКОМ, а не звёздочкой в тексте.
            //
            // Измерено по эталону `er_basic`: для `*id : int` нарисован
            // чёрный кружок r=3 в точке (рамка + 11), а подпись — «id : int»
            // без звёздочки, с отступом 20 от края рамки. Прежний код
            // подставлял «* » прямо в текст.
            let text_offset = if attr.is_required {
                ER_REQUIRED_TEXT_OFFSET
            } else {
                ER_PLAIN_TEXT_OFFSET
            };

            if attr.is_required {
                elements.push(LayoutElement {
                    id: format!("entity_{}_attr_{}_marker", entity_id, i),
                    element_type: ElementType::Ellipse { label: None },
                    bounds: Rect::new(
                        bounds.x + ER_REQUIRED_MARKER_X - ER_REQUIRED_MARKER_RADIUS,
                        attr_y + ER_REQUIRED_MARKER_DY - ER_REQUIRED_MARKER_RADIUS,
                        ER_REQUIRED_MARKER_RADIUS * 2.0,
                        ER_REQUIRED_MARKER_RADIUS * 2.0,
                    ),
                    text: None,
                    properties: [("fill".to_string(), "#000000".to_string())]
                        .into_iter()
                        .collect(),
                });
            }

            let attr_text = format!("{}{}{}", attr.name, type_str, stereo_str);

            let attr_element = LayoutElement {
                id: format!("entity_{}_attr_{}", entity_id, i),
                element_type: ElementType::Text {
                    text: attr_text.clone(),
                    font_size: self.config.font_size,
                },
                // Базис первой строки — эталонные 48.99 от верха рамки,
                // то есть 16.99 от конца полосы имени.
                bounds: Rect::new(
                    bounds.x + text_offset,
                    attr_y + ER_ATTR_BASELINE - self.config.font_size,
                    bounds.width - text_offset - self.config.entity_padding,
                    self.config.font_size,
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
}

impl Default for ErLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for ErLayoutEngine {
    type Input = ErDiagram;

    fn layout(&self, diagram: &Self::Input, _config: &LayoutConfig) -> LayoutResult {
        let mut scene = Scene::new(
            ER_NODE_SEP,
            self.config.vertical_spacing,
            self.config.padding,
            TextMeasurer::default(),
        );
        if diagram.metadata.direction == Some(Direction::LeftToRight) {
            scene.set_direction(GraphDirection::LeftToRight);
        }
        for entity in &diagram.entities {
            let size = self.calculate_entity_size(entity);
            let mut elements = Vec::new();
            let bounds = Rect::new(0.0, 0.0, size.width, size.height);
            self.render_entity(entity, &bounds, &mut elements);
            let mut names = vec![entity.id.name.as_str()];
            if let Some(alias) = &entity.id.alias {
                names.push(alias);
            }
            scene.add_node(None, &names, elements);
        }
        for (index, rel) in diagram.relationships.iter().enumerate() {
            let mut edge = SceneEdge::new(rel.from.clone(), rel.to.clone(), ArrowShape::default());
            edge.id = Some(format!("rel_{index}"));
            edge.label = rel.label.clone();
            edge.dashed = rel.dashed;
            edge.edge_type = EdgeType::Link;
            edge.from_cardinality = Some(rel.from_cardinality.symbol().to_string());
            edge.to_cardinality = Some(rel.to_cardinality.symbol().to_string());
            edge.properties
                .insert("stroke".to_string(), "#181818".to_string());
            scene.add_edge(edge);
        }
        scene.build()
    }
}

#[cfg(test)]
mod tests {
    /// Связь должна находить концы по алиасу.
    ///
    /// Регрессия: `positions` заполнялся только именами сущностей, а связь
    /// ссылается на алиас (`user ||--o{ order`). Из-за этого
    /// `render_relationships` не находил ни одного конца и молча пропускал
    /// ВСЕ связи — на диаграмме не было ни линии, ни кардинальностей.
    #[test]
    fn test_relationships_use_alias() {
        use plantuml_ast::er::{Cardinality, Entity, ErDiagram, ErRelationship};

        let mut diagram = ErDiagram::new();
        let mut user = Entity::new("User");
        user.id.alias = Some("user".to_string());
        let mut order = Entity::new("Order");
        order.id.alias = Some("order".to_string());
        diagram.entities.push(user);
        diagram.entities.push(order);
        diagram.relationships.push(ErRelationship {
            from: "user".to_string(),
            to: "order".to_string(),
            from_cardinality: Cardinality::One,
            to_cardinality: Cardinality::ZeroOrMany,
            label: None,
            is_identifying: true,
            dashed: false,
        });

        let result = ErLayoutEngine::new().layout(&diagram, &Default::default());

        // Считаем только СВЯЗИ: разделитель под шапкой сущности — тоже
        // `ElementType::Edge`, поэтому фильтруем по идентификатору.
        let edges: Vec<_> = result
            .elements
            .iter()
            .filter(|e| e.id.starts_with("rel_"))
            .collect();
        assert_eq!(edges.len(), 1, "связь потеряна: концы не найдены по алиасу");
    }

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
