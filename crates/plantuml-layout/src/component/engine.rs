//! Component Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм компонентов.

use std::collections::HashMap;

use plantuml_ast::common::Direction;
use plantuml_ast::component::{Component, ComponentDiagram, ComponentType};
use plantuml_model::{Point, Rect};

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

const COMPONENT_BASE_WIDTH: f64 = 4.6;
/// Прибавка к ширине компонента на каждый символ подписи.
const COMPONENT_CHAR_WIDTH: f64 = 11.43;
/// Высота компонента по эталону.
const COMPONENT_HEIGHT: f64 = 46.297;

/// Добавка к ширине артефакта (измерено по эталону deployment_basic).
const ARTIFACT_TEXT_PADDING: f64 = 30.0;

/// Высота артефакта (измерено по эталону).
const ARTIFACT_HEIGHT: f64 = 39.297;
use crate::graph::scene::{Scene, SceneEdge};
use crate::graph::Direction as GraphDirection;
use crate::{EdgeType, ElementType, LayoutElement, LayoutResult};

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

        // Стереотип — первая строка подписи, спрайт в подписи растягивает фигуру.
        let stereotype = comp
            .stereotype
            .as_ref()
            .and_then(|s| s.names.first())
            .filter(|_| self.show_stereotypes);
        let mut shown = comp.clone();
        if let Some(stereotype) = stereotype {
            shown.name = format!("//«{stereotype}»//\\n{}", comp.name);
        }
        let (mut element, mut bounds) = self.create_component_element(&shown, 0.0, 0.0);
        let label = shown.name.replace("\\n", "\n");
        for line in label.lines() {
            for span in crate::creole::parse(line) {
                let Some(sprite) = span.sprite.as_ref().and_then(|(name, scale)| {
                    sprites
                        .iter()
                        .find(|s| &s.name == name)
                        .map(|s| (s.width as f64 * scale, s.height as f64 * scale))
                }) else {
                    continue;
                };
                bounds.width = bounds.width.max(sprite.0 + 20.0);
                bounds.height += (sprite.1 - LINE_HEIGHT).max(0.0);
            }
        }
        element.bounds = bounds;
        if let Some(stereotype) = comp.stereotype.as_ref().and_then(|s| s.names.first()) {
            element
                .properties
                .insert("stereotype".to_string(), stereotype.clone());
        }
        let mut elements = vec![element];
        if comp.component_type == ComponentType::Artifact {
            elements.extend(self.create_artifact_icon(bounds));
        }
        scene.add_node(cluster, &names, elements);
    }

    /// Создаёт элемент компонента
    /// Создаёт элемент компонента и возвращает его **фактические** границы.
    ///
    /// Раньше возвращался прямоугольник фиксированного размера
    /// (`component_width × component_height`, 140×60) независимо от того,
    /// что реально нарисовано. Для `Interface` (эллипс 20×20), `Actor`
    /// (0.6 ширины) и `Cloud` (1.2 ширины) это неверно: связи привязывались
    /// к несуществующему блоку и визуально «висели» мимо фигуры.
    fn create_component_element(&self, comp: &Component, x: f64, y: f64) -> (LayoutElement, Rect) {
        let elem = match comp.component_type {
            ComponentType::Database => self.create_database_element(&comp.name, x, y),
            ComponentType::Cloud => self.create_cloud_element(&comp.name, x, y),
            ComponentType::Interface => self.create_interface_element(&comp.name, x, y),
            ComponentType::Queue => self.create_queue_element(&comp.name, x, y),
            ComponentType::Node => self.create_node_element(&comp.name, x, y),
            ComponentType::Folder => self.create_folder_element(&comp.name, x, y),
            ComponentType::Actor => self.create_actor_element(&comp.name, x, y),
            // Артефакт имеет свои пропорции: измерено по эталону
            // deployment_basic — «app.jar» при тексте 49.027 занимает
            // 79.027x39.297, то есть текст плюс 30.
            ComponentType::Artifact => self.create_artifact_element(&comp.name, x, y),
            _ => self.create_standard_component_element(comp, x, y),
        };

        // Границы берём у фактически созданного элемента, а не назначаем
        // по конфигу: иначе связи не попадают в фигуру.
        let bounds = elem.bounds;

        (elem, bounds)
    }

    /// Создаёт стандартный компонент
    fn create_standard_component_element(&self, comp: &Component, x: f64, y: f64) -> LayoutElement {
        let name = &comp.name;
        let mut properties = std::collections::HashMap::new();

        // Цвет элемента, если он задан (например, слоем Archimate).
        // Раньше поле `color` не читалось вовсе, поэтому archimate-элементы
        // всех слоёв выглядели одинаково белыми.
        if let Some(color) = &comp.color {
            properties.insert("fill".to_string(), color.to_css());
        }

        // Размер компонента зависит от длины подписи: измерено по эталону
        // («Веб-интерфейс», 13 символов — 153.189x46.297; «Сервис API», 10 —
        // 118.9x46.297). Отсюда ширина ≈ 4.6 + 11.43 * n, высота 46.297.
        // Раньше размер был фиксированным (40x40), из-за чего диаграмма
        // расходилась с эталоном.
        let (width, height) = self.label_box(name);

        LayoutElement {
            id: format!("component_{}", name.replace(' ', "_")),
            bounds: Rect::new(x, y, width, height),
            text: None,
            properties,
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: self.config.corner_radius,
            },
        }
    }

    /// Создаёт артефакт.
    ///
    /// Измерено по эталону deployment_basic: «app.jar» при тексте 49.027
    /// занимает 79.027x39.297, то есть ширина — текст плюс 30, высота 39.297.
    /// Размер фигуры по подписи; многострочная подпись растягивает её.
    fn label_box(&self, name: &str) -> (f64, f64) {
        let text = name.replace("\\n", "\n");
        let lines = text.lines().count().max(1);
        if lines == 1 {
            let chars = name.chars().count() as f64;
            let width = (COMPONENT_BASE_WIDTH + COMPONENT_CHAR_WIDTH * chars)
                .max(self.config.component_width);
            return (width, self.config.component_height.max(COMPONENT_HEIGHT));
        }
        (
            self.config.text.width(&text, self.config.font_size) + 30.0,
            COMPONENT_HEIGHT + (lines - 1) as f64 * LINE_HEIGHT,
        )
    }

    fn create_artifact_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        let width = self.config.text.width(name, self.config.font_size) + ARTIFACT_TEXT_PADDING;

        // Заливка и толщина рамки — как в эталоне `deployment_basic`:
        // `fill="#F1F1F1"`, `stroke-width:0.5`.
        LayoutElement {
            id: format!("artifact_{}", name.replace(' ', "_")),
            bounds: Rect::new(x, y, width, ARTIFACT_HEIGHT),
            text: None,
            properties: [
                ("fill".to_string(), "#F1F1F1".to_string()),
                ("stroke".to_string(), "#181818".to_string()),
                ("stroke-width".to_string(), "0.5".to_string()),
            ]
            .into_iter()
            .collect(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: 2.5,
            },
        }
    }

    /// Иконка документа в правом верхнем углу артефакта.
    ///
    /// Геометрия снята с эталона `deployment_basic`: рамка артефакта
    /// 79.027x39.297 в точке (57.49, 46), иконка 12x14 в точке
    /// (119.517, 51) и загнутый уголок 6x6.
    fn create_artifact_icon(&self, bounds: Rect) -> Vec<LayoutElement> {
        /// Отступ иконки от правого края рамки.
        const INSET_RIGHT: f64 = 17.0;
        /// Отступ иконки от верхнего края рамки.
        const INSET_TOP: f64 = 5.0;
        /// Ширина и высота иконки.
        const WIDTH: f64 = 12.0;
        const HEIGHT: f64 = 14.0;
        /// Сторона загнутого уголка.
        const FOLD: f64 = 6.0;

        let x = bounds.x + bounds.width - INSET_RIGHT;
        let y = bounds.y + INSET_TOP;

        let outline = LayoutElement {
            id: "artifact_icon".to_string(),
            bounds: Rect::new(x, y, WIDTH, HEIGHT),
            text: None,
            properties: [
                ("fill".to_string(), "#F1F1F1".to_string()),
                ("stroke".to_string(), "#181818".to_string()),
                ("stroke-width".to_string(), "0.5".to_string()),
            ]
            .into_iter()
            .collect(),
            // Вершины — в ЛОКАЛЬНЫХ координатах `bounds`: рендерер
            // прибавляет к ним начало рамки.
            element_type: ElementType::Polygon {
                points: vec![
                    Point::new(0.0, 0.0),
                    Point::new(0.0, HEIGHT),
                    Point::new(WIDTH, HEIGHT),
                    Point::new(WIDTH, FOLD),
                    Point::new(WIDTH - FOLD, 0.0),
                ],
                label: None,
                font_size: 0.0,
            },
        };

        // Загнутый уголок: вертикаль и горизонталь от точки сгиба.
        // Двумя отрезками, потому что `ElementType::Path` — единичный
        // вариант без геометрии, а ломаная здесь из двух звеньев.
        let mut fold = Vec::new();
        for (index, (from, to)) in [
            (
                Point::new(x + WIDTH - FOLD, y),
                Point::new(x + WIDTH - FOLD, y + FOLD),
            ),
            (
                Point::new(x + WIDTH - FOLD, y + FOLD),
                Point::new(x + WIDTH, y + FOLD),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            fold.push(LayoutElement {
                id: format!("artifact_icon_fold_{index}"),
                bounds: Rect::new(from.x, from.y, (to.x - from.x).abs(), (to.y - from.y).abs()),
                text: None,
                properties: [
                    ("fill".to_string(), "none".to_string()),
                    ("stroke".to_string(), "#181818".to_string()),
                    ("stroke-width".to_string(), "0.5".to_string()),
                ]
                .into_iter()
                .collect(),
                element_type: ElementType::Edge {
                    points: vec![from, to],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });
        }

        let mut result = vec![outline];
        result.extend(fold);
        result
    }

    /// Создаёт элемент базы данных (цилиндр)
    fn create_database_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        let (width, height) = self.label_box(name);

        LayoutElement {
            id: format!("database_{}", name.replace(' ', "_")),
            bounds: Rect::new(x, y, width, height),
            text: None,
            properties: std::collections::HashMap::new(),
            // База данных рисуется цилиндром, а не эмодзи: символ 🛢
            // зависит от наличия шрифта в системе и визуально не совпадает
            // с оригиналом PlantUML
            element_type: ElementType::Database {
                label: name.to_string(),
            },
        }
    }

    /// Создаёт элемент облака
    fn create_cloud_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("cloud_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width * 1.2,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: self.config.component_height / 2.0,
            },
        }
    }

    /// Создаёт элемент интерфейса (кружок)
    fn create_interface_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        let r = self.config.interface_radius;
        LayoutElement {
            id: format!("interface_{}", name.replace(' ', "_")),
            bounds: Rect::new(x, y, r * 2.0, r * 2.0),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Ellipse {
                label: Some(name.to_string()),
            },
        }
    }

    /// Создаёт элемент очереди
    fn create_queue_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("queue_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: self.config.component_height / 4.0,
            },
        }
    }

    /// Создаёт элемент node
    fn create_node_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("node_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: 0.0, // Node — с углами
            },
        }
    }

    /// Создаёт элемент folder
    fn create_folder_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("folder_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: self.config.corner_radius,
            },
        }
    }

    /// Создаёт элемент actor
    fn create_actor_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("actor_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width * 0.6,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            // Актёр — стик-фигура, а не эмодзи
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
