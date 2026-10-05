//! Layout engine для Object Diagrams
//!
//! Конвертирует ObjectDiagram в структуру для рендеринга.
//! PlantUML размещает объекты вертикально.

use plantuml_ast::object::{ObjectDiagram, ObjectLinkType};
use plantuml_model::{Point, Rect};

/// Базовая ширина объекта, измеренная по эталону PlantUML.
const OBJECT_BASE_WIDTH: f64 = 12.41;
/// Прибавка к ширине объекта на каждый символ имени.
const OBJECT_CHAR_WIDTH: f64 = 8.724;

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
        let mut elements = Vec::new();
        let mut object_positions: std::collections::HashMap<String, Rect> =
            std::collections::HashMap::new();

        // 1. Размещаем объекты в сетке
        // PlantUML размещает объекты вертикально: в эталоне object_basic
        // (138x170) оба объекта стоят на разных y (7 и 120.29), а не в ряд.
        // Раньше использовалась сетка по 4 в ряд, из-за чего диаграмма
        // получалась широкой и низкой (350x70 против 138x170).
        //
        // Ширина объекта зависит от длины имени: измерено по эталону
        // («Пользователь», 12 символов — 117.1; «Заказ», 5 — 56.034),
        // отсюда ширина ≈ 12.41 + 8.724 * n.
        let widths: Vec<f64> = diagram
            .objects
            .iter()
            .map(|object| {
                OBJECT_BASE_WIDTH + OBJECT_CHAR_WIDTH * object.display_name().chars().count() as f64
            })
            .collect();
        let max_width = widths.iter().cloned().fold(0.0_f64, f64::max);

        // PlantUML выравнивает объекты по ОБЩЕЙ вертикальной оси: в эталоне
        // `object_basic` центры «Пользователь» и «Заказ» совпадают
        // (65.55), хотя ширины рамок разные (117.1 и 56.03). Без этого
        // связь между ними шла по диагонали вместо вертикали.
        let diagram_width = max_width + self.config.padding * 2.0;

        let mut y = self.config.padding;
        let mut max_x = 0.0f64;
        let mut max_y = 0.0f64;

        for (i, object) in diagram.objects.iter().enumerate() {
            // Рассчитываем высоту объекта
            let header_height = 30.0;
            let fields_height = object.fields.len() as f64 * self.config.field_height;
            let object_height = (header_height + fields_height).max(self.config.object_min_height);

            // Определяем заголовок (с подчёркиванием как в UML)
            let display_name = object.display_name();
            let obj_width = widths[i];

            // Создаём bounds: объект центрируется по общей оси диаграммы
            let x = (diagram_width - obj_width) / 2.0;
            let bounds = Rect::new(x, y, obj_width, object_height);
            object_positions.insert(object.name.clone(), bounds);

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
                        font_size: 12.0,
                    },
                });
            }

            // Габарит считается по ФАКТИЧЕСКОЙ рамке: `config.object_width`
            // — это лишь значение по умолчанию, а ширина каждого объекта
            // выводится из длины имени.
            max_x = max_x.max(x + obj_width);
            max_y = max_y.max(y + object_height);

            // Объекты идут по одному в строке: PlantUML расставляет их
            // вертикально (эталон `object_basic` — 138x170).
            y += object_height + self.config.vertical_spacing;
        }

        // 2. Добавляем связи
        for link in &diagram.links {
            if let (Some(from_bounds), Some(to_bounds)) = (
                object_positions.get(&link.from),
                object_positions.get(&link.to),
            ) {
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
                    text: None,
                    properties: std::collections::HashMap::new(),
                    element_type: ElementType::Edge {
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
