//! Layout engine для ER диаграмм
//!
//! Размещает сущности в виде таблиц с атрибутами.
//! Использует простой grid layout с оптимизацией для связей.

use std::collections::HashMap;

use plantuml_ast::er::{Entity, ErDiagram};
use plantuml_model::{Point, Rect, Size};

use crate::er::config::ErLayoutConfig;
use crate::traits::{LayoutEngine, LayoutResult};
use crate::{EdgeType, ElementType, LayoutConfig, LayoutElement};

/// Заливка блока сущности ER (эталон `er_basic`: `#F1F1F1`).
const ER_ENTITY_FILL: &str = "#F1F1F1";

/// Скругление рамки сущности (эталон: `rx="2.5"`).
const ER_ENTITY_CORNER_RADIUS: f64 = 2.5;

/// Добавка к ширине подписи при расчёте ширины шапки.
///
/// Эталон `er_basic`: «Order» 40.03 при рамке 72.03, «User» 31.91 при
/// содержимом 63.91 — то есть кружок 22, зазор 3 и поле 7.
const ER_HEADER_WIDTH_EXTRA: f64 = 32.0;

/// Добавка к ширине самого длинного атрибута.
///
/// Эталон: «name : string» 93.51 при рамке 105.51 и «id : int» 44.65 при
/// рамке 56.65 — по 6 с каждой стороны.
const ER_ATTR_WIDTH_EXTRA: f64 = 12.0;

/// Радиус кружка-иконки сущности (эталон: `rx="11"`).
const ER_SPOT_RADIUS: f64 = 11.0;

/// Смещение центра кружка от левого края рамки (эталон: 40.719 − 7).
const ER_SPOT_OFFSET_X: f64 = 33.719;

/// Смещение центра кружка от верха рамки (эталон: 23 − 7).
const ER_SPOT_OFFSET_Y: f64 = 16.0;

/// Заливка кружка-иконки (эталон: `#ADD1B2`).
const ER_SPOT_FILL: &str = "#ADD1B2";

/// Кегль буквы в кружке.
const ER_SPOT_LETTER_FONT_SIZE: f64 = 12.0;

/// Смещение подписи от центра кружка (эталон: 58.88 − 40.719).
const ER_NAME_OFFSET_X: f64 = 18.161;

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
                let type_str = a.data_type.as_deref().unwrap_or("");
                let stereo_str = a
                    .stereotype
                    .as_deref()
                    .map(|s| format!(" <<{}>>", s))
                    .unwrap_or_default();
                // Одна согласованная мера вместо двух разных констант
                // (9.0 для имени и 7.5 для атрибутов)
                self.config.text.width(
                    &format!("{}{}{}", a.name, type_str, stereo_str),
                    self.config.font_size,
                ) + 3.0
            })
            .fold(0.0, f64::max);

        // Атрибут занимает «текст + 12»: эталон даёт 105.51 при «name :
        // string» 93.51 и 56.65 при «id : int» 44.65.
        let final_width = width.max(max_attr_width + ER_ATTR_WIDTH_EXTRA);

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

        // PlantUML размещает сущности ER вертикально: в эталоне обе сущности
        // стоят на x=7 (er_basic: 54x208 при высоте 208). Раньше использовалась
        // сетка по 3 в ряд, из-за чего диаграмма получалась широкой и низкой
        // (410x114 против 54x208).
        let mut y = self.config.padding;
        let mut row_height = 0.0_f64;

        // Общая ось: эталон `er_basic` держит центры обеих сущностей на
        // 59.76 при ширинах 105.51 и 72.03. Без выравнивания связь шла по
        // диагонали, а она в ER-нотации вертикальная.
        let widths: Vec<f64> = diagram
            .entities
            .iter()
            .map(|entity| self.calculate_entity_size(entity).width)
            .collect();
        let diagram_width =
            widths.iter().cloned().fold(0.0_f64, f64::max) + self.config.padding * 2.0;

        for (entity_index, entity) in diagram.entities.iter().enumerate() {
            let size = self.calculate_entity_size(entity);

            if entity_index > 0 {
                // Сущности идут по ОДНОЙ в строке (эталон `er_basic`:
                // 127x226 при двух сущностях).
                y += row_height + self.config.vertical_spacing;
                row_height = 0.0;
            }

            let x = (diagram_width - size.width) / 2.0;
            let bounds = Rect::new(x, y, size.width, size.height);
            positions.insert(entity.id.name.clone(), bounds);
            // Связь ссылается на алиас (`user ||--o{ order`), а не на имя
            // сущности, поэтому позиция сохраняется и под алиасом. Без этого
            // render_relationships не находил концы и молча пропускал ВСЕ
            // связи — на диаграмме не было ни линии, ни кардинальностей.
            if let Some(alias) = &entity.id.alias {
                positions.insert(alias.clone(), bounds);
            }

            // Рисуем сущность
            self.render_entity(entity, &bounds, elements);

            row_height = row_height.max(size.height);
        }

        positions
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

        // Кружок-иконка сущности: эталон даёт эллипс r=11 с заливкой
        // `#ADD1B2` в точке (40.719, 23) при рамке от (7, 7).
        let spot_x = bounds.x + ER_SPOT_OFFSET_X;
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

        let letter_width = self.config.text.width_bold("E", ER_SPOT_LETTER_FONT_SIZE);
        elements.push(LayoutElement {
            id: format!("entity_{}_spot_letter", entity_id),
            element_type: ElementType::Text {
                text: "E".to_string(),
                font_size: ER_SPOT_LETTER_FONT_SIZE,
            },
            bounds: Rect::new(
                spot_x - letter_width / 2.0,
                spot_y - ER_SPOT_LETTER_FONT_SIZE / 2.0,
                letter_width,
                ER_SPOT_LETTER_FONT_SIZE,
            ),
            text: None,
            properties: [
                ("fill".to_string(), "#000000".to_string()),
                ("font-weight".to_string(), "700".to_string()),
            ]
            .into_iter()
            .collect(),
        });

        // Название — ОБЫЧНЫМ начертанием справа от кружка.
        //
        // Эталон: «User» кеглем 14 на x = 58.88 при кружке на 40.719,
        // базис — на 20.85 ниже верха рамки.
        let name_x = spot_x + ER_NAME_OFFSET_X;
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

            // Линия связи.
            //
            // Кардинальности передаются в типизированных полях
            // ElementType::Edge, как во всех остальных движках. Раньше они
            // шли строковыми ключами properties («from_card»/«to_card»),
            // что расходилось с общим контрактом: рендерер читал эти ключи
            // только в одном месте и не мог обработать их единообразно.
            let edge = LayoutElement {
                id: format!("rel_{}", i),
                element_type: ElementType::Edge {
                    points: vec![from_point, to_point],
                    label: rel.label.clone(),
                    arrow_start: false,
                    arrow_end: false,
                    // Пунктир задаёт САМ ИСХОДНИК: `..` — пунктир,
                    // `--` — сплошная. Эталон `er_basic` (`||--o{`) даёт
                    // сплошную линию.
                    dashed: rel.dashed,
                    edge_type: EdgeType::Link,
                    from_cardinality: Some(rel.from_cardinality.symbol().to_string()),
                    to_cardinality: Some(rel.to_cardinality.symbol().to_string()),
                },
                bounds: Rect::from_points(from_point, to_point),
                text: rel.label.clone(),
                properties: [("stroke".to_string(), "#181818".to_string())]
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
