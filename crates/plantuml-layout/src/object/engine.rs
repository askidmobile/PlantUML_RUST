//! Layout engine для Object Diagrams
//!
//! Конвертирует ObjectDiagram в структуру для рендеринга.
//! PlantUML размещает объекты вертикально.

use plantuml_ast::common::Direction;
use plantuml_ast::object::{ObjectDiagram, ObjectLinkType};
use plantuml_model::{Point, Rect};

use crate::graph::scene::{Scene, SceneEdge};
use crate::graph::Direction as GraphDirection;

/// Шаг рангов с подписью 77, без неё 60: подпись резервирует сцена.
const OBJECT_LABEL_RESERVE: f64 = 17.0;

/// Базовая ширина объекта, измеренная по эталону PlantUML.
const OBJECT_BASE_WIDTH: f64 = 12.41;
/// Прибавка к ширине объекта на каждый символ имени.
const OBJECT_CHAR_WIDTH: f64 = 8.724;

/// Прибавка к ширине самого широкого текста объекта (имя или поле).
///
/// Измерено по эталону `Object: Диаграмма`: текст 211.2 -> рамка 223,
/// текст 146.4 -> рамка 158.
const OBJECT_FIELD_PADDING: f64 = 11.8;

/// Высота полосы имени объекта.
///
/// Эталон `object_basic`: рамка 36.3 высотой, подчёркивание имени — на
/// `y+20.3`, то есть имя занимает верхнюю полосу и не центрируется по
/// всей рамке.
const OBJECT_NAME_BAND: f64 = 20.3;

/// Цвет заливки объекта (эталон `object_basic`: `#F1F1F1`, а не тема).
const OBJECT_BACKGROUND: &str = "#F1F1F1";

/// Радиус скругления рамки объекта (эталон `object_basic`: `rx=2.5`).
const OBJECT_CORNER_RADIUS: f64 = 2.5;

/// Отступ подписи от левого края рамки (эталон `object_basic`: 14 при x=7).
const OBJECT_TEXT_INSET: f64 = 7.0;

/// Кегль подписи объекта (эталон `object_basic`: `font-size="14"`).
const OBJECT_NAME_FONT_SIZE: f64 = 14.0;

/// Кегль подписей полей объекта (в эталоне `font-size="12"`).
/// Кегль подписей полей объекта.
///
/// Измерено по эталону `Object: Диаграмма`: текст `email =
/// "john@example.com"` занимает 211.2, что соответствует кеглю 14 (наш
/// замер при 14 даёт 199.18, при 12 — только 170.7). Раньше поля
/// рисовались кеглем 12, из-за чего блок `john` выходил 183 при
/// эталонных 223.
const OBJECT_FIELD_FONT_SIZE: f64 = 14.0;

/// Кегль подписей полей.
fn object_field_font_size() -> f64 {
    OBJECT_FIELD_FONT_SIZE
}

/// Насколько базис подписи выше конца полосы имени.
///
/// Эталон `object_basic`: полоса кончается на `y+20.3`, базис — на `y+15`.
const OBJECT_NAME_BASELINE_INSET: f64 = 5.3;

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
        let mut scene = Scene::new(
            self.config.horizontal_spacing,
            self.config.vertical_spacing - OBJECT_LABEL_RESERVE,
            self.config.padding,
            crate::text::TextMeasurer::default(),
        );
        if diagram.metadata.direction == Some(Direction::LeftToRight) {
            scene.set_direction(GraphDirection::LeftToRight);
        }

        let widths: Vec<f64> = diagram
            .objects
            .iter()
            .map(|object| {
                let measurer = crate::text::TextMeasurer::default();
                let by_name = measurer.width(&object.display_name(), OBJECT_NAME_FONT_SIZE)
                    + OBJECT_FIELD_PADDING;
                let by_fields = object
                    .fields
                    .iter()
                    .map(|field| {
                        measurer.width(
                            &format!("{} = {}", field.name, field.value),
                            object_field_font_size(),
                        ) + OBJECT_FIELD_PADDING
                    })
                    .fold(0.0_f64, f64::max);
                by_name.max(by_fields).max(
                    OBJECT_BASE_WIDTH
                        + OBJECT_CHAR_WIDTH * object.display_name().chars().count() as f64,
                )
            })
            .collect();
        for (i, object) in diagram.objects.iter().enumerate() {
            // Рассчитываем высоту объекта
            let header_height = 30.0;
            let fields_height = object.fields.len() as f64 * self.config.field_height;
            let object_height = (header_height + fields_height).max(self.config.object_min_height);

            // Определяем заголовок (с подчёркиванием как в UML)
            let display_name = object.display_name();
            let obj_width = widths[i];

            // Координаты берутся из раскладки по уровням.
            let (x, y) = (0.0, 0.0);
            let bounds = Rect::new(x, y, obj_width, object_height);
            let mut elements = Vec::new();

            // Создаём element для объекта.
            //
            // Подпись рисуется ОТДЕЛЬНЫМ текстом, а не меткой рамки: у
            // PlantUML имя стоит в верхней полосе высотой 20.3 и подчёркнуто,
            // тогда как `render_rectangle` центрирует метку по всей рамке.
            let mut properties = std::collections::HashMap::new();
            properties.insert("fill".to_string(), OBJECT_BACKGROUND.to_string());
            properties.insert("rx".to_string(), OBJECT_CORNER_RADIUS.to_string());

            elements.push(LayoutElement {
                id: format!("object_{}", object.name),
                bounds,
                text: None,
                properties: properties.clone(),
                element_type: ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: OBJECT_CORNER_RADIUS,
                },
            });

            // Ширина подписи — по метрикам шрифта, а не по рамке: иначе
            // габарит диаграммы вырастал на отступ подписи.
            let name_width =
                crate::text::TextMeasurer::default().width(&display_name, OBJECT_NAME_FONT_SIZE);

            // Имя: слева с отступом 7, базис — рамка плюс 15 (кегль 14).
            elements.push(LayoutElement {
                id: format!("object_name_{}", object.name),
                bounds: Rect::new(
                    x + OBJECT_TEXT_INSET,
                    y + OBJECT_NAME_BAND - OBJECT_NAME_FONT_SIZE - OBJECT_NAME_BASELINE_INSET,
                    name_width,
                    OBJECT_NAME_FONT_SIZE,
                ),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Text {
                    text: display_name.clone(),
                    font_size: OBJECT_NAME_FONT_SIZE,
                },
            });

            // Подчёркивание имени: от левого края плюс 1 до правого минус 1.
            elements.push(LayoutElement {
                id: format!("object_underline_{}", object.name),
                bounds: Rect::new(x + 1.0, y + OBJECT_NAME_BAND, obj_width - 2.0, 0.0),
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Edge {
                    points: vec![
                        Point::new(x + 1.0, y + OBJECT_NAME_BAND),
                        Point::new(x + obj_width - 1.0, y + OBJECT_NAME_BAND),
                    ],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
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
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Text {
                        text: field_text,
                        font_size: OBJECT_FIELD_FONT_SIZE,
                    },
                });
            }

            // Габарит считается по ФАКТИЧЕСКОЙ рамке: `config.object_width`
            // — это лишь значение по умолчанию, а ширина каждого объекта
            // выводится из длины имени.
            scene.add_node(None, &[object.name.as_str()], elements);
        }

        // 2. Добавляем связи
        for link in &diagram.links {
            let mut edge = SceneEdge::new(link.from.clone(), link.to.clone(), link.arrow);
            edge.label = link.label.clone();
            edge.dashed = link.link_type.is_dashed();
            edge.edge_type = match link.link_type {
                ObjectLinkType::Composition => EdgeType::Composition,
                ObjectLinkType::Aggregation => EdgeType::Aggregation,
                ObjectLinkType::Dependency => EdgeType::Dependency,
                ObjectLinkType::Association => EdgeType::Association,
                ObjectLinkType::Link => EdgeType::Link,
            };
            scene.add_edge(edge);
        }
        for note in &diagram.notes {
            scene.add_note(note.clone());
        }
        scene.build()
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
