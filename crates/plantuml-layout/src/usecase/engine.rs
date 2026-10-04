//! Use Case Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм вариантов использования.
//! PlantUML стиль: актёры слева, система справа с use cases внутри.

use std::collections::HashMap;

use plantuml_ast::common::Direction;
use plantuml_ast::usecase::{UseCaseDiagram, UseCaseRelationType, UseCaseRelationship};
use plantuml_model::{Point, Rect};

use super::config::UseCaseLayoutConfig;

/// Базовая ширина эллипса use case, измеренная по эталону PlantUML.
const USE_CASE_BASE_WIDTH: f64 = 55.1;
/// Прибавка к ширине на каждый символ подписи.
const USE_CASE_CHAR_WIDTH: f64 = 6.48;
/// Базовая высота эллипса use case.
const USE_CASE_BASE_HEIGHT: f64 = 21.4;
/// Прибавка к высоте на каждый символ подписи.
const USE_CASE_CHAR_HEIGHT: f64 = 0.95;
use crate::{EdgeType, ElementType, LayoutElement, LayoutResult};

/// Layout engine для use case diagrams
pub struct UseCaseLayoutEngine {
    config: UseCaseLayoutConfig,
}

impl UseCaseLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: UseCaseLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: UseCaseLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &UseCaseDiagram) -> LayoutResult {
        let mut elements = Vec::new();
        let mut element_positions: HashMap<String, Rect> = HashMap::new();

        let _is_left_to_right = diagram.direction == Direction::LeftToRight;

        // Вычисляем максимальную ширину имён актёров для правильного позиционирования
        // Кириллица занимает примерно 9 пикселей на символ (font-size 14)
        let max_actor_label_width = diagram
            .actors
            .iter()
            .map(|a| a.name.chars().count() as f64 * 9.0)
            .fold(0.0f64, f64::max);

        // Минимальная ширина для актёра с его label
        let actor_total_width = self.config.actor_width.max(max_actor_label_width);

        // Позиция актёров - центрируем по ширине их label
        let actors_x = self.config.margin + actor_total_width / 2.0;

        // Собираем все use cases (из packages и верхнего уровня)
        let mut all_usecases: Vec<(&str, Option<&str>)> = Vec::new();

        for uc in &diagram.use_cases {
            all_usecases.push((&uc.name, uc.alias.as_deref()));
        }

        for pkg in &diagram.packages {
            for uc in &pkg.use_cases {
                all_usecases.push((&uc.name, uc.alias.as_deref()));
            }
        }

        // Наибольшая ширина эллипса среди use case: по ней выравнивается
        // вся группа, как в PlantUML.
        let max_usecase_width = all_usecases
            .iter()
            .map(|(name, _)| self.usecase_natural_size(name).0)
            .fold(0.0_f64, f64::max)
            .max(self.config.usecase_width.min(60.0));

        // Вычисляем размеры области use case
        let num_usecases = all_usecases.len().max(1);
        let usecase_row = |name: &str| self.usecase_natural_size(name).1;
        let inner_height: f64 = all_usecases
            .iter()
            .map(|(name, _)| usecase_row(name))
            .sum::<f64>()
            + (num_usecases.saturating_sub(1)) as f64 * self.config.vertical_spacing;

        // Рамка системы рисуется ТОЛЬКО если в диаграмме есть package.
        // В эталоне PlantUML при отсутствии package рамки нет вовсе
        // (usecase_basic: ни одного прямоугольника).
        let has_package = !diagram.packages.is_empty();

        let (system_width, system_height) = if has_package {
            (
                max_usecase_width + self.config.package_padding * 2.0 + 40.0,
                inner_height
                    + self.config.package_header_height
                    + self.config.package_padding * 2.0,
            )
        } else {
            // Без package размеры области совпадают с содержимым
            (max_usecase_width, inner_height)
        };

        // Позиция области (справа от актёров с учётом их label)
        let system_x = self.config.margin + actor_total_width + self.config.horizontal_spacing;
        let system_y = self.config.margin;

        if has_package {
            let system_name = diagram.packages[0].name.clone();
            let system_bounds = Rect::new(system_x, system_y, system_width, system_height);
            elements.push(LayoutElement {
                id: format!("system_{}", system_name.replace(' ', "_")),
                bounds: system_bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::System { title: system_name },
            });
        }

        // Use case выравниваются по центру области
        let usecases_x = system_x + (system_width - max_usecase_width) / 2.0;
        let usecases_start_y = system_y
            + if has_package {
                self.config.package_header_height + self.config.package_padding
            } else {
                0.0
            };

        // Шаг между use case считается по их фактическим высотам, а не по
        // конфигу: высота эллипса зависит от длины подписи (см.
        // usecase_natural_size). Иначе длинные подписи наезжают друг на друга.
        let mut current_y = usecases_start_y;
        for (i, (name, alias)) in all_usecases.iter().enumerate() {
            let y = current_y;
            current_y += self.usecase_natural_size(name).1 + self.config.vertical_spacing;
            let _ = i;

            let (elem, bounds) = self.create_usecase_element(name, usecases_x, y);
            element_positions.insert(name.to_string(), bounds);
            if let Some(a) = alias {
                element_positions.insert(a.to_string(), bounds);
            }
            elements.push(elem);
        }

        // Группируем актёров по их связям с use cases
        // Находим какие актёры связаны с какими use cases
        let mut actor_usecases: HashMap<String, Vec<String>> = HashMap::new();
        for rel in &diagram.relationships {
            // Проверяем, является ли from актёром
            if diagram
                .actors
                .iter()
                .any(|a| a.name == rel.from || a.alias.as_deref() == Some(&rel.from))
            {
                actor_usecases
                    .entry(rel.from.clone())
                    .or_default()
                    .push(rel.to.clone());
            }
            // Проверяем, является ли to актёром
            if diagram
                .actors
                .iter()
                .any(|a| a.name == rel.to || a.alias.as_deref() == Some(&rel.to))
            {
                actor_usecases
                    .entry(rel.to.clone())
                    .or_default()
                    .push(rel.from.clone());
            }
        }

        // Размещаем актёров слева.
        //
        // Несколько актёров могут получить одинаковую Y (если связаны с
        // разными use case на одной высоте, либо оба не связаны ни с чем).
        // Раньше в этом случае они рисовались в одной точке и полностью
        // накладывались друг на друга. Теперь при совпадении Y актёр
        // сдвигается по X на ширину блока.
        let mut occupied: Vec<(f64, f64)> = Vec::new(); // (y, x)
        for actor in &diagram.actors {
            let actor_id = actor.alias.as_ref().unwrap_or(&actor.name);

            // Вычисляем среднюю Y позицию use cases, с которыми связан актёр
            let connected_usecases = actor_usecases
                .get(actor_id)
                .or_else(|| actor_usecases.get(&actor.name));

            let y = if let Some(ucs) = connected_usecases {
                if !ucs.is_empty() {
                    let total_y: f64 = ucs
                        .iter()
                        .filter_map(|uc_name| element_positions.get(uc_name))
                        .map(|rect| rect.y + rect.height / 2.0)
                        .sum();
                    let count = ucs
                        .iter()
                        .filter(|uc| element_positions.contains_key(*uc))
                        .count();
                    if count > 0 {
                        total_y / count as f64 - self.config.actor_height / 2.0
                    } else {
                        self.config.margin
                    }
                } else {
                    self.config.margin
                }
            } else {
                // Если актёр не связан ни с чем, размещаем внизу
                system_y + system_height / 2.0 - self.config.actor_height / 2.0
            };

            // Ищем свободную позицию по X среди актёров с такой же Y
            let mut actor_x = actors_x;
            let tolerance = self.config.actor_height / 2.0;
            while occupied.iter().any(|(oy, ox)| {
                (oy - y).abs() < tolerance && (ox - actor_x).abs() < actor_total_width
            }) {
                actor_x += actor_total_width + self.config.horizontal_spacing;
            }
            occupied.push((y, actor_x));

            let (elem, bounds) = self.create_actor_element(&actor.name, actor_x, y);
            element_positions.insert(actor.name.clone(), bounds);
            if let Some(alias) = &actor.alias {
                element_positions.insert(alias.clone(), bounds);
            }
            elements.push(elem);
        }

        // Создаём связи
        for rel in &diagram.relationships {
            if let Some(edge) = self.create_relationship_element(rel, &element_positions) {
                elements.push(edge);
            }
        }

        // Вычисляем bounds
        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        result.bounds.width += self.config.margin;
        result.bounds.height += self.config.margin;

        result
    }

    /// Создаёт элемент актёра (stick figure)
    fn create_actor_element(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let bounds = Rect::new(x, y, self.config.actor_width, self.config.actor_height);

        (
            LayoutElement {
                id: format!("actor_{}", name.replace(' ', "_")),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Actor {
                    label: name.to_string(),
                },
            },
            bounds,
        )
    }

    /// Создаёт элемент use case (эллипс)
    /// Натуральный размер эллипса use case (ширина, высота).
    ///
    /// Измерено по эталону PlantUML: «Оформить заказ» (15 символов) —
    /// 152.26 x 35.25, «Оплатить» (8 символов) — 106.92 x 29.05. Отсюда
    /// ширина ≈ 55.1 + 6.48 * n, высота ≈ 21.4 + 0.95 * n.
    fn usecase_natural_size(&self, name: &str) -> (f64, f64) {
        let chars = name.chars().count() as f64;
        (
            USE_CASE_BASE_WIDTH + USE_CASE_CHAR_WIDTH * chars,
            USE_CASE_BASE_HEIGHT + USE_CASE_CHAR_HEIGHT * chars,
        )
    }

    fn create_usecase_element(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        // Размер эллипса PlantUML зависит от длины подписи. Измерено
        // по эталону (tests/golden/reference/usecase_basic.svg):
        //   «Оформить заказ» (15 символов) — 152.26 x 35.25
        //   «Оплатить»        (8 символов) — 106.92 x 29.05
        // Отсюда ширина ≈ 55.1 + 6.48 * n, высота ≈ 21.4 + 0.95 * n.
        // Раньше размер был фиксированным (160x40), из-за чего диаграмма
        // получалась шире эталона независимо от подписей.
        let chars = name.chars().count() as f64;
        let width = (USE_CASE_BASE_WIDTH + USE_CASE_CHAR_WIDTH * chars)
            .max(self.config.usecase_width.min(60.0));
        let height = (USE_CASE_BASE_HEIGHT + USE_CASE_CHAR_HEIGHT * chars)
            .max(self.config.usecase_height.min(24.0));
        let bounds = Rect::new(x, y, width, height);

        (
            LayoutElement {
                id: format!("usecase_{}", name.replace(' ', "_")),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Ellipse {
                    label: Some(name.to_string()),
                },
            },
            bounds,
        )
    }

    /// Создаёт элемент связи
    fn create_relationship_element(
        &self,
        rel: &UseCaseRelationship,
        positions: &HashMap<String, Rect>,
    ) -> Option<LayoutElement> {
        let from_rect = positions.get(&rel.from)?;
        let to_rect = positions.get(&rel.to)?;

        let (start, end) = self.calculate_connection_points(from_rect, to_rect);

        let min_x = start.x.min(end.x);
        let min_y = start.y.min(end.y);
        let max_x = start.x.max(end.x);
        let max_y = start.y.max(end.y);

        let dashed = matches!(
            rel.relation_type,
            UseCaseRelationType::Include | UseCaseRelationType::Extend
        );

        Some(LayoutElement {
            id: format!(
                "rel_{}_{}",
                rel.from.replace(' ', "_"),
                rel.to.replace(' ', "_")
            ),
            bounds: Rect::new(
                min_x,
                min_y,
                (max_x - min_x).max(1.0),
                (max_y - min_y).max(1.0),
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points: vec![start, end],
                label: rel.label.clone(),
                arrow_start: false,
                // В PlantUML --> всегда показывает стрелку
                // Стрелки нужны для всех типов кроме undirected (которые мы пока не поддерживаем)
                arrow_end: true,
                dashed,
                edge_type: match rel.relation_type {
                    UseCaseRelationType::Generalization => EdgeType::Inheritance,
                    UseCaseRelationType::Include => EdgeType::Dependency,
                    UseCaseRelationType::Extend => EdgeType::Dependency,
                    UseCaseRelationType::Association => EdgeType::Association,
                },
                from_cardinality: None,
                to_cardinality: None,
            },
        })
    }

    /// Вычисляет точки соединения для связи
    fn calculate_connection_points(&self, from: &Rect, to: &Rect) -> (Point, Point) {
        let _from_center_x = from.x + from.width / 2.0;
        let from_center_y = from.y + from.height / 2.0;
        let _to_center_x = to.x + to.width / 2.0;
        let to_center_y = to.y + to.height / 2.0;

        // Для актёров (узкие) соединяем справа
        // Для эллипсов соединяем слева
        let start = if from.width < 50.0 {
            // Это актёр - соединяем справа
            Point::new(from.x + from.width, from_center_y)
        } else {
            // Это эллипс - соединяем слева
            Point::new(from.x, from_center_y)
        };

        let end = if to.width < 50.0 {
            // Это актёр - соединяем справа
            Point::new(to.x + to.width, to_center_y)
        } else {
            // Это эллипс - соединяем слева
            Point::new(to.x, to_center_y)
        };

        (start, end)
    }
}

impl Default for UseCaseLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::usecase::{UseCase, UseCaseActor};

    /// Актёры не должны накладываться друг на друга.
    ///
    /// Регрессия: раньше `actors_x` вычислялся один раз и использовался для
    /// всех актёров, поэтому несколько актёров на одной высоте рисовались в
    /// одной точке.
    #[test]
    fn test_actors_do_not_overlap() {
        let mut diagram = plantuml_ast::usecase::UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("A"));
        diagram.actors.push(UseCaseActor::new("B"));
        diagram.actors.push(UseCaseActor::new("C"));
        diagram.use_cases.push(UseCase::new("U"));

        let result = UseCaseLayoutEngine::new().layout(&diagram);

        let actors: Vec<&LayoutElement> = result
            .elements
            .iter()
            .filter(|e| e.id.starts_with("actor"))
            .collect();
        assert_eq!(actors.len(), 3, "не все актёры размещены");

        // Попарно проверяем, что прямоугольники не пересекаются
        for (i, a) in actors.iter().enumerate() {
            for b in actors.iter().skip(i + 1) {
                let overlap_x = a.bounds.x < b.bounds.x + b.bounds.width
                    && b.bounds.x < a.bounds.x + a.bounds.width;
                let overlap_y = a.bounds.y < b.bounds.y + b.bounds.height
                    && b.bounds.y < a.bounds.y + a.bounds.height;
                assert!(
                    !(overlap_x && overlap_y),
                    "актёры {} и {} перекрываются: {:?} и {:?}",
                    a.id,
                    b.id,
                    a.bounds,
                    b.bounds
                );
            }
        }
    }

    #[test]
    fn test_layout_simple() {
        let mut diagram = UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("User"));
        diagram.use_cases.push(UseCase::new("Login"));
        diagram
            .relationships
            .push(UseCaseRelationship::new("User", "Login"));

        let engine = UseCaseLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть: actor + usecase + система + связь
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_layout_with_package() {
        use plantuml_ast::usecase::UseCasePackage;

        let mut diagram = UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("User"));

        let mut pkg = UseCasePackage::new("System");
        pkg.use_cases.push(UseCase::new("Login"));
        pkg.use_cases.push(UseCase::new("Logout"));
        diagram.packages.push(pkg);

        let engine = UseCaseLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть: actor + system + 2 usecases
        assert!(result.elements.len() >= 4);
    }

    #[test]
    fn test_layout_include_extend() {
        let mut diagram = UseCaseDiagram::new();
        diagram.use_cases.push(UseCase::new("Login"));
        diagram.use_cases.push(UseCase::new("Authenticate"));
        diagram
            .relationships
            .push(UseCaseRelationship::include("Login", "Authenticate"));

        let engine = UseCaseLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert!(result.elements.len() >= 3);
    }
}
