//! Use Case Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм вариантов использования.
//! PlantUML стиль: актёры слева, система справа с use cases внутри.

use std::collections::HashMap;

use plantuml_ast::common::Direction;
use plantuml_ast::usecase::{UseCaseDiagram, UseCaseRelationship, UseCaseRelationType};
use plantuml_model::{Point, Rect};

use super::config::UseCaseLayoutConfig;
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
        let max_actor_label_width = diagram.actors.iter()
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

        // Вычисляем размеры системы/rectangle
        let num_usecases = all_usecases.len().max(1);
        let system_inner_height = num_usecases as f64 * (self.config.usecase_height + self.config.vertical_spacing);
        let system_height = system_inner_height + self.config.package_header_height + self.config.package_padding * 2.0;
        let system_width = self.config.usecase_width + self.config.package_padding * 2.0 + 40.0;
        
        // Позиция системы (справа от актёров с учётом их label)
        let system_x = self.config.margin + actor_total_width + self.config.horizontal_spacing;
        let system_y = self.config.margin;

        // Если есть packages, создаём System элемент
        let system_name = if !diagram.packages.is_empty() {
            diagram.packages[0].name.clone()
        } else {
            "System".to_string()
        };

        // Создаём rectangle системы
        let system_bounds = Rect::new(system_x, system_y, system_width, system_height);
        let system_elem = LayoutElement {
            id: format!("system_{}", system_name.replace(' ', "_")),
            bounds: system_bounds.clone(),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::System {
                title: system_name,
            },
        };
        elements.push(system_elem);

        // Размещаем use cases внутри системы (вертикально по центру)
        let usecases_x = system_x + (system_width - self.config.usecase_width) / 2.0;
        let usecases_start_y = system_y + self.config.package_header_height + self.config.package_padding;

        for (i, (name, alias)) in all_usecases.iter().enumerate() {
            let y = usecases_start_y + i as f64 * (self.config.usecase_height + self.config.vertical_spacing);
            
            let (elem, bounds) = self.create_usecase_element(name, usecases_x, y);
            element_positions.insert(name.to_string(), bounds.clone());
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
            if diagram.actors.iter().any(|a| a.name == rel.from || a.alias.as_deref() == Some(&rel.from)) {
                actor_usecases.entry(rel.from.clone()).or_default().push(rel.to.clone());
            }
            // Проверяем, является ли to актёром
            if diagram.actors.iter().any(|a| a.name == rel.to || a.alias.as_deref() == Some(&rel.to)) {
                actor_usecases.entry(rel.to.clone()).or_default().push(rel.from.clone());
            }
        }

        // Размещаем актёров слева (actors_x уже вычислен выше с учётом ширины label)
        for actor in &diagram.actors {
            let actor_id = actor.alias.as_ref().unwrap_or(&actor.name);
            
            // Вычисляем среднюю Y позицию use cases, с которыми связан актёр
            let connected_usecases = actor_usecases.get(actor_id).or_else(|| actor_usecases.get(&actor.name));
            
            let y = if let Some(ucs) = connected_usecases {
                if !ucs.is_empty() {
                    let total_y: f64 = ucs.iter()
                        .filter_map(|uc_name| element_positions.get(uc_name))
                        .map(|rect| rect.y + rect.height / 2.0)
                        .sum();
                    let count = ucs.iter().filter(|uc| element_positions.contains_key(*uc)).count();
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

            let (elem, bounds) = self.create_actor_element(&actor.name, actors_x, y);
            element_positions.insert(actor.name.clone(), bounds.clone());
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
                bounds: bounds.clone(),
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
    fn create_usecase_element(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let bounds = Rect::new(x, y, self.config.usecase_width, self.config.usecase_height);

        (
            LayoutElement {
                id: format!("usecase_{}", name.replace(' ', "_")),
                bounds: bounds.clone(),
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
            bounds: Rect::new(min_x, min_y, (max_x - min_x).max(1.0), (max_y - min_y).max(1.0)),
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
        let from_center_x = from.x + from.width / 2.0;
        let from_center_y = from.y + from.height / 2.0;
        let to_center_x = to.x + to.width / 2.0;
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
