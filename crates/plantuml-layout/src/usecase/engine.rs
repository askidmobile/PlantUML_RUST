//! Use Case Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм вариантов использования.
//! PlantUML стиль: актёры слева, система справа с use cases внутри.

use std::collections::HashMap;

use plantuml_ast::common::Direction;
use plantuml_ast::usecase::{UseCase, UseCaseDiagram, UseCaseRelationType};
use plantuml_model::Rect;

use super::config::UseCaseLayoutConfig;

/// Базовая ширина эллипса use case, измеренная по эталону PlantUML.
const USE_CASE_BASE_WIDTH: f64 = 55.1;
/// Прибавка к ширине на каждый символ подписи.
const USE_CASE_CHAR_WIDTH: f64 = 6.48;
/// Базовая высота эллипса use case.
const USE_CASE_BASE_HEIGHT: f64 = 21.4;
/// Прибавка к высоте на каждый символ подписи.
const USE_CASE_CHAR_HEIGHT: f64 = 0.95;

/// Заливка эллипсов вариантов использования и головы актёра.
///
/// Эталон `UseCase: Простой`: все семь эллипсов (пять вариантов и две
/// головы актёров) залиты `#F1F1F1`, тогда как тема даёт `#E2E2F0` —
/// цвет узлов class-диаграмм.
/// Зазор между узлами одного ранга.
const RANK_NODE_SEP: f64 = 35.0;
/// Поле холста сверх общего поля рендерера (эталон `usecase_basic`).
const USECASE_EXTRA: f64 = 12.0;

const USECASE_FILL: &str = "#F1F1F1";

/// Толщина обводки эллипса варианта использования (эталон: 0.5).
const USECASE_STROKE_WIDTH: f64 = 0.5;

use crate::graph::scene::{Scene, SceneEdge};
use crate::graph::Direction as GraphDirection;
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
        let mut scene = Scene::new(
            self.config.horizontal_spacing.min(RANK_NODE_SEP),
            self.config.vertical_spacing,
            self.config.package_padding,
            self.config.text.clone(),
        );
        if diagram.direction == Direction::LeftToRight
            || diagram.metadata.direction == Some(Direction::LeftToRight)
        {
            scene.set_direction(GraphDirection::LeftToRight);
        }

        for actor in &diagram.actors {
            let (element, _) =
                self.create_actor_element(&actor.name, 0.0, 0.0, self.actor_width(&actor.name));
            let mut element = element;
            element.bounds.x = 0.0;
            let mut names = vec![actor.name.as_str()];
            if let Some(alias) = &actor.alias {
                names.push(alias);
            }
            scene.add_node(None, &names, vec![element]);
        }

        for use_case in &diagram.use_cases {
            self.add_use_case(&mut scene, None, use_case);
        }
        for package in &diagram.packages {
            let title = self
                .config
                .text
                .width_bold(&package.name, self.config.font_size);
            let element = LayoutElement {
                id: format!("package_{}", package.name.replace(' ', "_")),
                bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
                text: None,
                properties: [
                    ("font-weight".to_string(), "700".to_string()),
                    ("cluster-shape".to_string(), "rectangle".to_string()),
                ]
                .into_iter()
                .collect(),
                element_type: ElementType::Group {
                    label: Some(package.name.clone()),
                    children: Vec::new(),
                },
            };
            let cluster = scene.add_cluster(
                None,
                &[package.name.as_str()],
                self.config.package_header_height,
                title + self.config.package_padding * 2.0,
                element,
            );
            for use_case in &package.use_cases {
                self.add_use_case(&mut scene, Some(cluster), use_case);
            }
        }

        for rel in &diagram.relationships {
            let dashed = matches!(
                rel.relation_type,
                UseCaseRelationType::Include | UseCaseRelationType::Extend
            ) || rel.arrow.length == 0;
            scene.add_edge(SceneEdge {
                from: rel.from.clone(),
                to: rel.to.clone(),
                label: rel.label.clone(),
                arrow: rel.arrow,
                dashed,
                edge_type: match rel.relation_type {
                    UseCaseRelationType::Generalization => EdgeType::Inheritance,
                    UseCaseRelationType::Include | UseCaseRelationType::Extend => {
                        EdgeType::Dependency
                    }
                    UseCaseRelationType::Association => EdgeType::Association,
                },
                properties: HashMap::new(),
            });
        }
        for note in &diagram.notes {
            scene.add_note(note.clone());
        }

        let mut result = scene.build();
        result.bounds.width += USECASE_EXTRA;
        result.bounds.height += USECASE_EXTRA;
        result
    }

    /// Ширина фигуры актёра: подпись шире самого человечка.
    fn actor_width(&self, name: &str) -> f64 {
        self.config
            .text
            .width(name, self.config.font_size)
            .max(self.config.actor_width)
    }

    fn add_use_case(&self, scene: &mut Scene, cluster: Option<usize>, use_case: &UseCase) {
        let (mut element, _) = self.create_usecase_element(&use_case.name, 0.0, 0.0);
        element.bounds.x = 0.0;
        let mut names = vec![use_case.name.as_str()];
        if let Some(alias) = &use_case.alias {
            names.push(alias);
        }
        scene.add_node(cluster, &names, vec![element]);
    }

    fn create_actor_element(
        &self,
        name: &str,
        x: f64,
        y: f64,
        width: f64,
    ) -> (LayoutElement, Rect) {
        let bounds = Rect::new(x - width / 2.0, y, width, self.config.actor_height);

        // Голова актёра в use case залита тем же `#F1F1F1`, что и
        // эллипсы (эталоны `usecase_basic`, `UseCase: Простой`); тема
        // даёт цвет узлов sequence — `#E2E2F0`.
        let mut properties = std::collections::HashMap::new();
        properties.insert("fill".to_string(), USECASE_FILL.to_string());

        (
            LayoutElement {
                id: format!("actor_{}", name.replace(' ', "_")),
                bounds,
                text: None,
                properties,
                element_type: ElementType::Actor {
                    label: name.to_string(),
                },
            },
            bounds,
        )
    }

    /// Создаёт элемент use case (эллипс).
    ///
    /// `x` — ЦЕНТР эллипса: в эталоне центры всех вариантов совпадают
    /// (`UseCase: Простой`: `cx = 247.626` у всех пяти).
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
        let bounds = Rect::new(x - width / 2.0, y, width, height);

        // Заливка и толщина обводки эллипса — из эталона: `#F1F1F1` и
        // 0.5 (тема даёт `#E2E2F0` и 1).
        let mut properties = std::collections::HashMap::new();
        properties.insert("fill".to_string(), USECASE_FILL.to_string());
        properties.insert("stroke-width".to_string(), USECASE_STROKE_WIDTH.to_string());

        (
            LayoutElement {
                id: format!("usecase_{}", name.replace(' ', "_")),
                bounds,
                text: None,
                properties,
                element_type: ElementType::Ellipse {
                    label: Some(name.to_string()),
                },
            },
            bounds,
        )
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
    use plantuml_ast::usecase::{UseCaseActor, UseCaseRelationship};

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

    /// Варианты без связей между собой PlantUML кладёт РЯДОМ, а не столбцом.
    ///
    /// Регрессия: прежний код всегда ставил варианты в столбец, из-за чего
    /// пример с `rectangle` и двумя актёрами выходил 240x663 вместо
    /// эталонных 1063x207 — связи шли наискось через всю фигуру, а подпись
    /// второго актёра накладывалась на заголовок рамки.
    #[test]
    fn test_usecases_without_edges_go_in_a_row() {
        let mut diagram = UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("Покупатель"));
        diagram.use_cases.push(UseCase::new("Просмотр каталога"));
        diagram.use_cases.push(UseCase::new("Добавить в корзину"));
        diagram.use_cases.push(UseCase::new("Оформить заказ"));
        diagram
            .relationships
            .push(UseCaseRelationship::new("Покупатель", "Просмотр каталога"));

        let result = UseCaseLayoutEngine::new().layout(&diagram);

        let ellipses: Vec<Rect> = result
            .elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Ellipse { .. }))
            .map(|e| e.bounds)
            .collect();
        assert_eq!(ellipses.len(), 3, "не все варианты размещены");

        // Несвязанные варианты встают на верхний ранг, как в dot; главное, что не перекрываются.
        let mut sorted = ellipses.clone();
        sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        for pair in sorted.windows(2) {
            assert!(
                pair[0].x + pair[0].width <= pair[1].x + 0.01,
                "эллипсы ряда перекрываются: {:?} и {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    /// Цепочка связей между вариантами по-прежнему даёт СТОЛБЕЦ.
    ///
    /// Эталон `usecase_basic` (`Пользователь --> UC1`, `UC1 --> UC2`):
    /// эллипсы стоят друг под другом на общей оси.
    #[test]
    fn test_usecase_chain_stays_in_a_column() {
        let mut diagram = UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("Пользователь"));
        diagram.use_cases.push(UseCase::new("Оформить заказ"));
        diagram.use_cases.push(UseCase::new("Оплатить"));
        diagram
            .relationships
            .push(UseCaseRelationship::new("Пользователь", "Оформить заказ"));
        diagram
            .relationships
            .push(UseCaseRelationship::new("Оформить заказ", "Оплатить"));

        let result = UseCaseLayoutEngine::new().layout(&diagram);
        let ellipses: Vec<Rect> = result
            .elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Ellipse { .. }))
            .map(|e| e.bounds)
            .collect();
        assert_eq!(ellipses.len(), 2);

        let cx = |r: &Rect| r.x + r.width / 2.0;
        assert!(
            (cx(&ellipses[0]) - cx(&ellipses[1])).abs() < 0.01,
            "варианты цепочки должны стоять на одной оси: {:?}",
            ellipses
        );
        assert!(
            ellipses[0].y + ellipses[0].height < ellipses[1].y,
            "второй вариант цепочки должен быть НИЖЕ первого: {:?}",
            ellipses
        );
    }
}
