//! Component Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм компонентов.

use std::collections::HashMap;

use plantuml_ast::common::Direction;
use plantuml_ast::component::{Component, ComponentDiagram, ComponentType};
use plantuml_model::Rect;

use super::config::ComponentLayoutConfig;

/// Базовая ширина компонента, измеренная по эталону PlantUML.
/// Место под заголовок контейнера при переходе между пакетами.
///
/// Измерено на сервере: `package` -> `package` даёт шаг 122 при 106
/// внутри пакета.
/// Базовый шаг между уровнями для компонентов ВНУТРИ пакетов.
///
/// Измерено на сервере: шаг 106 = 46 (высота блока) + 60. У компонентов
/// верхнего уровня шаг другой — 123 = 46 + 77 (эталон `component_basic`).
const COMPONENT_IN_PACKAGE_SPACING: f64 = 60.0;

/// Высота строки подписи.
const LINE_HEIGHT: f64 = 16.297;

const COMPONENT_RIGHT_EXTRA: f64 = 6.81;
const COMPONENT_BOTTOM_EXTRA: f64 = 16.0;
const NODE_LEFT_EXTRA: f64 = 9.0;
const NODE_RIGHT_EXTRA: f64 = 29.0;
/// Рамка не уже этого значения и шире заголовка на столько (измерено по эталону).
const PACKAGE_MIN_WIDTH: f64 = 150.0;
const PACKAGE_TITLE_EXTRA: f64 = 76.0;

/// Диаметр кружка интерфейса (эталон: радиус 8).
const INTERFACE_DIAMETER: f64 = 16.0;

/// Сторона квадрата порта и зазор до подписи (эталон `feat_component_porty`).
const PORT_SIDE: f64 = 12.0;
const PORT_LABEL_GAP: f64 = 4.0;

/// Ширина значка boundary (скобка и кружок) и высота значка с подписью (эталон).
const ICON_MIN_WIDTH: f64 = 44.0;
const ICON_HEIGHT: f64 = 43.3;

/// Высота актёра с подписью (эталон: голова, тело и подпись под ними).
const ACTOR_HEIGHT: f64 = 74.8;

use crate::graph::scene::{Scene, SceneEdge};
use crate::graph::Direction as GraphDirection;
use crate::{ElementType, LayoutElement, LayoutResult};

pub struct ComponentLayoutEngine {
    config: ComponentLayoutConfig,
    show_stereotypes: bool,
}

impl ComponentLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: ComponentLayoutConfig::default(),
            show_stereotypes: true,
        }
    }

    /// Не выводить стереотип подписью: у Archimate он задаёт только значок.
    pub fn hide_stereotypes(mut self) -> Self {
        self.show_stereotypes = false;
        self
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: ComponentLayoutConfig) -> Self {
        Self {
            config,
            show_stereotypes: true,
        }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &ComponentDiagram) -> LayoutResult {
        let mut scene = Scene::new(
            self.config.horizontal_spacing,
            COMPONENT_IN_PACKAGE_SPACING,
            self.config.package_padding,
            self.config.text,
        );
        if diagram.metadata.direction == Some(Direction::LeftToRight) {
            scene.set_direction(GraphDirection::LeftToRight);
        }

        let sprites = &diagram.metadata.sprites;
        for comp in &diagram.components {
            self.add_component(&mut scene, None, comp, sprites);
        }
        for pkg in &diagram.packages {
            self.add_package(&mut scene, None, pkg, sprites);
        }
        for conn in &diagram.connections {
            let mut edge = SceneEdge::new(conn.from.clone(), conn.to.clone(), conn.arrow);
            edge.label = conn.label.clone();
            edge.dashed = conn.dashed;
            scene.add_edge(edge);
        }
        for note in &diagram.notes {
            scene.add_note(note.clone());
        }

        // Поля холста сверх общих рендерера сняты с эталонов: у плоских компонентов
        // справа 13.8, снизу 23; у объёмных узлов слева 16, справа 25.
        let has_clusters = !diagram.packages.is_empty();
        let mut result = scene.build();
        if has_clusters {
            result.bounds.x -= NODE_LEFT_EXTRA;
            result.bounds.width += NODE_LEFT_EXTRA + NODE_RIGHT_EXTRA;
            result.bounds.height += self.config.margin;
        } else {
            result.bounds.width += COMPONENT_RIGHT_EXTRA;
            result.bounds.height += COMPONENT_BOTTOM_EXTRA;
        }
        result
    }

    /// Рамка контейнера: вид зависит от его типа.
    fn cluster_element(&self, name: &str, kind: &str, stereotype: Option<&str>) -> LayoutElement {
        let mut properties: HashMap<String, String> =
            [("font-weight".to_string(), "700".to_string())]
                .into_iter()
                .collect();
        properties.insert("cluster-shape".to_string(), kind.to_string());
        if let Some(stereotype) = stereotype {
            properties.insert("stereotype".to_string(), stereotype.to_string());
        }
        LayoutElement {
            id: format!("package_{}", name.replace(' ', "_")),
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            text: None,
            properties,
            element_type: ElementType::Group {
                label: Some(name.to_string()),
                children: Vec::new(),
            },
        }
    }

    /// Высота заголовка рамки с учётом числа строк подписи.
    fn cluster_title_height(&self, name: &str) -> f64 {
        let lines = name.replace("\\n", "\n").lines().count().max(1) as f64;
        self.config.package_header_height + (lines - 1.0) * LINE_HEIGHT
    }

    fn add_package(
        &self,
        scene: &mut Scene,
        parent: Option<usize>,
        pkg: &plantuml_ast::component::ComponentPackage,
        sprites: &[plantuml_ast::class::Sprite],
    ) {
        use plantuml_ast::component::PackageType;
        let kind = match pkg.package_type {
            PackageType::Package => "package",
            PackageType::Node => "node",
            PackageType::Folder => "folder",
            PackageType::Frame => "frame",
            PackageType::Cloud => "cloud",
            PackageType::Rectangle => "rectangle",
            PackageType::Database => "database",
        };
        let title = self
            .config
            .text
            .width_bold(&pkg.name, self.config.font_size);
        let stereotype = pkg
            .stereotype
            .as_ref()
            .and_then(|s| s.names.first())
            .map(String::as_str);
        let cluster = scene.add_cluster(
            parent,
            &[pkg.name.as_str()],
            self.cluster_title_height(&pkg.name),
            (title + PACKAGE_TITLE_EXTRA).max(PACKAGE_MIN_WIDTH),
            self.cluster_element(&pkg.name, kind, stereotype),
        );
        for comp in &pkg.components {
            self.add_component(scene, Some(cluster), comp, sprites);
        }
        for nested in &pkg.packages {
            self.add_package(scene, Some(cluster), nested, sprites);
        }
    }

    fn add_component(
        &self,
        scene: &mut Scene,
        cluster: Option<usize>,
        comp: &Component,
        sprites: &[plantuml_ast::class::Sprite],
    ) {
        let mut names: Vec<&str> = vec![comp.name.as_str()];
        if let Some(alias) = &comp.alias {
            names.push(alias.as_str());
        }

        // Компонент с содержимым — это рамка.
        if !comp.children.is_empty() {
            let title = self
                .config
                .text
                .width_bold(&comp.name, self.config.font_size);
            let inner = scene.add_cluster(
                cluster,
                &names,
                self.cluster_title_height(&comp.name),
                title + self.config.package_padding * 2.0,
                self.cluster_element(&comp.name, "rectangle", None),
            );
            for child in &comp.children {
                self.add_component(scene, Some(inner), child, sprites);
            }
            return;
        }

        let mut element = self.create_component_element(comp);
        if !self.show_stereotypes {
            element
                .properties
                .insert("stereotype-hidden".to_string(), "1".to_string());
        }
        // Спрайт в подписи растягивает фигуру.
        for line in comp.name.replace("\\n", "\n").lines() {
            for span in crate::creole::parse(line) {
                let Some(sprite) = span.sprite.as_ref().and_then(|(name, scale)| {
                    sprites
                        .iter()
                        .find(|s| &s.name == name)
                        .map(|s| (s.width as f64 * scale, s.height as f64 * scale))
                }) else {
                    continue;
                };
                element.bounds.width = element.bounds.width.max(sprite.0 + 20.0);
                element.bounds.height += (sprite.1 - LINE_HEIGHT).max(0.0);
            }
        }
        scene.add_node(cluster, &names, vec![element]);
    }

    /// Создаёт элемент компонента: форма и размер зависят от вида.
    ///
    /// Размеры сняты с эталона `feat_component_deployment`: ширина — текст плюс
    /// поля вида, высота — строки подписи плюс поля. Сам контур рисует
    /// рендерер по свойству `shape`; движок отдаёт только границы, по которым
    /// связи находят фигуру.
    fn create_component_element(&self, comp: &Component) -> LayoutElement {
        let name = &comp.name;
        let text = name.replace("\\n", "\n");
        let lines = text.lines().count().max(1) as f64;
        let font = self.config.font_size;
        let mut text_width = text
            .lines()
            .map(|line| self.config.text.width(line, font))
            .fold(0.0_f64, f64::max);
        let stereotype = comp
            .stereotype
            .as_ref()
            .and_then(|s| s.names.first())
            .cloned();
        // Стереотип — отдельная строка над именем, она же может быть шире.
        let mut rows = lines;
        if let Some(stereotype) = &stereotype {
            rows += 1.0;
            text_width = text_width.max(self.config.text.width(&format!("«{stereotype}»"), font));
        }
        let block = rows * LINE_HEIGHT;

        let (shape, id_prefix, width, height) = match comp.component_type {
            ComponentType::Database => ("database", "database", text_width + 20.0, block + 29.0),
            ComponentType::Cloud => ("cloud", "cloud", text_width + 40.0, block + 30.0),
            ComponentType::Queue => ("queue", "queue", text_width + 20.0, block + 10.0),
            ComponentType::Node => ("node", "node", text_width + 30.0, block + 30.0),
            ComponentType::Folder => ("folder", "folder", text_width + 20.0, block + 20.0),
            ComponentType::File => ("file", "file", text_width + 20.0, block + 20.0),
            ComponentType::Card | ComponentType::Agent | ComponentType::Rectangle => {
                ("card", "card", text_width + 20.0, block + 6.0)
            }
            ComponentType::Hexagon => ("hexagon", "hexagon", text_width + 19.44, block + 10.0),
            ComponentType::Stack => ("stack", "stack", text_width + 50.0, block + 20.0),
            ComponentType::Storage => ("storage", "storage", text_width + 20.0, block + 20.0),
            ComponentType::Collections => (
                "collections",
                "collections",
                text_width + 20.0,
                block + 20.0,
            ),
            ComponentType::Artifact => ("artifact", "artifact", text_width + 30.0, block + 23.0),
            ComponentType::UseCase => (
                "usecase",
                "usecase",
                text_width * 1.15 + 14.8,
                (block + 12.1).max(28.4),
            ),
            ComponentType::Port => return self.create_port_element(name, text_width),
            ComponentType::Interface => return self.create_interface_element(name),
            ComponentType::Actor => return self.create_actor_element(name, text_width),
            ComponentType::Boundary | ComponentType::Control | ComponentType::Entity => {
                return self.create_icon_element(comp, text_width)
            }
            _ => ("component", "component", text_width + 40.0, block + 30.0),
        };

        let mut properties: HashMap<String, String> = HashMap::new();
        properties.insert("shape".to_string(), shape.to_string());
        // Цвет элемента, если он задан (например, слоем Archimate).
        if let Some(color) = &comp.color {
            properties.insert("fill".to_string(), color.to_css());
        }
        if let Some(stereotype) = stereotype {
            properties.insert("stereotype".to_string(), stereotype);
        }
        let element_type = if comp.component_type == ComponentType::Database {
            ElementType::Database {
                label: name.to_string(),
            }
        } else {
            ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: 2.5,
            }
        };
        LayoutElement {
            id: format!("{id_prefix}_{}", name.replace(' ', "_")),
            bounds: Rect::new(0.0, 0.0, width, height),
            text: None,
            properties,
            element_type,
        }
    }

    /// Порт (`port`, `portin`, `portout`): квадрат 12 на 12 с подписью справа.
    ///
    /// Положение на рамке компонента не вычисляется: порт живёт узлом внутри неё.
    fn create_port_element(&self, name: &str, text_width: f64) -> LayoutElement {
        let properties: HashMap<String, String> = [
            ("shape".to_string(), "port".to_string()),
            ("fill".to_string(), "#F1F1F1".to_string()),
            ("stroke-width".to_string(), "1.5".to_string()),
        ]
        .into_iter()
        .collect();
        LayoutElement {
            id: format!("port_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                0.0,
                0.0,
                PORT_SIDE + PORT_LABEL_GAP + text_width,
                LINE_HEIGHT,
            ),
            text: None,
            properties,
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: 0.0,
            },
        }
    }

    /// Интерфейс: кружок, подпись стоит под ним, а связь идёт к кружку.
    fn create_interface_element(&self, name: &str) -> LayoutElement {
        let properties: HashMap<String, String> = [
            ("fill".to_string(), "#F1F1F1".to_string()),
            ("stroke-width".to_string(), "0.5".to_string()),
            ("label-below".to_string(), "1".to_string()),
        ]
        .into_iter()
        .collect();
        LayoutElement {
            id: format!("interface_{}", name.replace(' ', "_")),
            bounds: Rect::new(0.0, 0.0, INTERFACE_DIAMETER, INTERFACE_DIAMETER),
            text: None,
            properties,
            element_type: ElementType::Ellipse {
                label: Some(name.to_string()),
            },
        }
    }

    /// Boundary, control и entity: значок-кружок с подписью под ним.
    fn create_icon_element(&self, comp: &Component, text_width: f64) -> LayoutElement {
        let name = &comp.name;
        let (id_prefix, element_type) = match comp.component_type {
            ComponentType::Boundary => (
                "boundary",
                ElementType::Boundary {
                    label: name.to_string(),
                },
            ),
            ComponentType::Control => (
                "control",
                ElementType::Control {
                    label: name.to_string(),
                },
            ),
            _ => (
                "entity",
                ElementType::Entity {
                    label: name.to_string(),
                },
            ),
        };
        LayoutElement {
            id: format!("{id_prefix}_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                0.0,
                0.0,
                (text_width + 6.0).max(ICON_MIN_WIDTH),
                ICON_HEIGHT,
            ),
            text: None,
            properties: [("fill".to_string(), "#F1F1F1".to_string())]
                .into_iter()
                .collect(),
            element_type,
        }
    }

    /// Актёр: человечек с подписью под ним (высота снята с эталона).
    fn create_actor_element(&self, name: &str, text_width: f64) -> LayoutElement {
        LayoutElement {
            id: format!("actor_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                0.0,
                0.0,
                (text_width + 6.0).max(32.0),
                ACTOR_HEIGHT
                    + name.replace("\\n", "\n").lines().count().saturating_sub(1) as f64
                        * LINE_HEIGHT,
            ),
            text: None,
            properties: [("fill".to_string(), "#F1F1F1".to_string())]
                .into_iter()
                .collect(),
            element_type: ElementType::Actor {
                label: name.to_string(),
            },
        }
    }
}

/// Ставит заметки правее всех фигур на высоте своей цели.
///
/// Раскладка не оставляет места сбоку от элементов, поэтому сторона
/// `left/top/bottom` пока не учитывается: важнее, чтобы текст заметки попал
/// на схему и не лёг на чужие фигуры.
/// shortcut: сторона заметки игнорируется, заменит общий послойный движок.
impl Default for ComponentLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_simple_components() {
        let mut diagram = ComponentDiagram::new();
        diagram.components.push(Component::new("API"));
        diagram.components.push(Component::database("MySQL"));
        diagram
            .connections
            .push(plantuml_ast::component::Connection::new("API", "MySQL"));

        let engine = ComponentLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть: 2 компонента + 1 связь
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_layout_with_package() {
        use plantuml_ast::component::ComponentPackage;

        let mut diagram = ComponentDiagram::new();
        let mut pkg = ComponentPackage::new("Backend");
        pkg.components.push(Component::new("API"));
        pkg.components.push(Component::new("Worker"));
        diagram.packages.push(pkg);

        let engine = ComponentLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должен быть пакет + 2 компонента
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_layout_various_types() {
        let mut diagram = ComponentDiagram::new();
        diagram.components.push(Component::new("App"));
        diagram.components.push(Component::database("PostgreSQL"));
        diagram.components.push(Component::cloud("AWS"));
        diagram.components.push(Component::node("Server"));

        let engine = ComponentLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert_eq!(result.elements.len(), 4);
    }
}
