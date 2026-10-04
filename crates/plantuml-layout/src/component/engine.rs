//! Component Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм компонентов.

use std::collections::HashMap;

use plantuml_ast::component::{Component, ComponentDiagram, ComponentType, Connection};
use plantuml_model::{Point, Rect};

use super::config::ComponentLayoutConfig;
use crate::{EdgeType, ElementType, LayoutElement, LayoutResult};

/// Layout engine для component diagrams
pub struct ComponentLayoutEngine {
    config: ComponentLayoutConfig,
}

impl ComponentLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: ComponentLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: ComponentLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &ComponentDiagram) -> LayoutResult {
        let mut elements = Vec::new();
        let mut component_positions: HashMap<String, Rect> = HashMap::new();

        // Сначала располагаем компоненты в grid-layout
        let components: Vec<&Component> = diagram.components.iter().collect();
        let num_cols = ((components.len() as f64).sqrt().ceil() as usize).max(1);

        for (i, comp) in components.iter().enumerate() {
            let row = i / num_cols;
            let col = i % num_cols;

            let x = self.config.margin
                + col as f64 * (self.config.component_width + self.config.horizontal_spacing);
            let y = self.config.margin
                + row as f64 * (self.config.component_height + self.config.vertical_spacing);

            let (elem, bounds) = self.create_component_element(comp, x, y);

            // Сохраняем позицию по имени и алиасу
            component_positions.insert(comp.name.clone(), bounds);
            if let Some(alias) = &comp.alias {
                component_positions.insert(alias.clone(), bounds);
            }

            elements.push(elem);
        }

        // Располагаем пакеты
        let mut package_y = self.config.margin
            + ((components.len() / num_cols.max(1) + 1) as f64)
                * (self.config.component_height + self.config.vertical_spacing);

        for pkg in &diagram.packages {
            let (pkg_elements, pkg_bounds, inner_positions) =
                self.layout_package(pkg, self.config.margin, package_y);

            for elem in pkg_elements {
                elements.push(elem);
            }

            // Добавляем позиции вложенных компонентов
            for (name, rect) in inner_positions {
                component_positions.insert(name, rect);
            }

            package_y = pkg_bounds.y + pkg_bounds.height + self.config.vertical_spacing;
        }

        // Создаём связи
        for conn in &diagram.connections {
            if let Some(edge) = self.create_connection_element(conn, &component_positions) {
                elements.push(edge);
            }
        }

        // Вычисляем bounds
        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        // Добавляем отступы
        result.bounds.width += self.config.margin * 2.0;
        result.bounds.height += self.config.margin * 2.0;

        result
    }

    /// Создаёт элемент компонента
    fn create_component_element(&self, comp: &Component, x: f64, y: f64) -> (LayoutElement, Rect) {
        let bounds = Rect::new(
            x,
            y,
            self.config.component_width,
            self.config.component_height,
        );

        let elem = match comp.component_type {
            ComponentType::Database => self.create_database_element(&comp.name, x, y),
            ComponentType::Cloud => self.create_cloud_element(&comp.name, x, y),
            ComponentType::Interface => self.create_interface_element(&comp.name, x, y),
            ComponentType::Queue => self.create_queue_element(&comp.name, x, y),
            ComponentType::Node => self.create_node_element(&comp.name, x, y),
            ComponentType::Folder => self.create_folder_element(&comp.name, x, y),
            ComponentType::Actor => self.create_actor_element(&comp.name, x, y),
            _ => self.create_standard_component_element(&comp.name, x, y),
        };

        (elem, bounds)
    }

    /// Создаёт стандартный компонент
    fn create_standard_component_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("component_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: format!("⬡ {}", name), // Добавляем иконку компонента
                corner_radius: self.config.corner_radius,
            },
        }
    }

    /// Создаёт элемент базы данных (цилиндр)
    fn create_database_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("database_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: format!("🛢 {}", name),
                corner_radius: self.config.corner_radius,
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
                label: format!("☁ {}", name),
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
                label: format!("⟿ {}", name),
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
                label: format!("⬢ {}", name),
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
                label: format!("📁 {}", name),
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
            element_type: ElementType::Text {
                text: format!("👤\n{}", name),
                font_size: 12.0,
            },
        }
    }

    /// Располагает пакет и возвращает элементы, bounds и позиции вложенных компонентов
    fn layout_package(
        &self,
        pkg: &plantuml_ast::component::ComponentPackage,
        x: f64,
        y: f64,
    ) -> (Vec<LayoutElement>, Rect, HashMap<String, Rect>) {
        let mut elements = Vec::new();
        let mut positions = HashMap::new();

        // Располагаем вложенные компоненты
        let num_cols = ((pkg.components.len() as f64).sqrt().ceil() as usize).max(1);
        let mut max_row = 0;
        let mut max_col = 0;

        for (i, comp) in pkg.components.iter().enumerate() {
            let row = i / num_cols;
            let col = i % num_cols;
            max_row = max_row.max(row);
            max_col = max_col.max(col);

            let comp_x = x
                + self.config.package_padding
                + col as f64 * (self.config.component_width + self.config.horizontal_spacing / 2.0);
            let comp_y = y
                + self.config.package_header_height
                + self.config.package_padding
                + row as f64 * (self.config.component_height + self.config.vertical_spacing / 2.0);

            let (elem, bounds) = self.create_component_element(comp, comp_x, comp_y);
            positions.insert(comp.name.clone(), bounds);
            if let Some(alias) = &comp.alias {
                positions.insert(alias.clone(), bounds);
            }
            elements.push(elem);
        }

        // Вычисляем размер пакета
        let inner_width = (max_col + 1) as f64
            * (self.config.component_width + self.config.horizontal_spacing / 2.0)
            - self.config.horizontal_spacing / 2.0;
        let inner_height = (max_row + 1) as f64
            * (self.config.component_height + self.config.vertical_spacing / 2.0)
            - self.config.vertical_spacing / 2.0;

        let pkg_width = inner_width + self.config.package_padding * 2.0;
        let pkg_height =
            inner_height + self.config.package_header_height + self.config.package_padding * 2.0;

        let pkg_bounds = Rect::new(x, y, pkg_width.max(150.0), pkg_height.max(100.0));

        // Создаём элемент пакета (group)
        let pkg_elem = LayoutElement {
            id: format!("package_{}", pkg.name.replace(' ', "_")),
            bounds: pkg_bounds,
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Group {
                label: Some(pkg.name.clone()),
                children: Vec::new(),
            },
        };

        // Вставляем пакет первым (под компонентами)
        elements.insert(0, pkg_elem);

        (elements, pkg_bounds, positions)
    }

    /// Создаёт элемент связи
    fn create_connection_element(
        &self,
        conn: &Connection,
        positions: &HashMap<String, Rect>,
    ) -> Option<LayoutElement> {
        let from_rect = positions.get(&conn.from)?;
        let to_rect = positions.get(&conn.to)?;

        let (start, end) = self.calculate_connection_points(from_rect, to_rect);

        let min_x = start.x.min(end.x);
        let min_y = start.y.min(end.y);
        let max_x = start.x.max(end.x);
        let max_y = start.y.max(end.y);

        Some(LayoutElement {
            id: format!(
                "conn_{}_{}",
                conn.from.replace(' ', "_"),
                conn.to.replace(' ', "_")
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
                label: conn.label.clone(),
                arrow_start: false,
                arrow_end: true,
                dashed: conn.dashed,
                edge_type: EdgeType::Association,
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

        let dx = to_center_x - from_center_x;
        let dy = to_center_y - from_center_y;

        let start;
        let end;

        if dy.abs() > dx.abs() {
            if dy > 0.0 {
                start = Point::new(from_center_x, from.y + from.height);
                end = Point::new(to_center_x, to.y);
            } else {
                start = Point::new(from_center_x, from.y);
                end = Point::new(to_center_x, to.y + to.height);
            }
        } else {
            if dx > 0.0 {
                start = Point::new(from.x + from.width, from_center_y);
                end = Point::new(to.x, to_center_y);
            } else {
                start = Point::new(from.x, from_center_y);
                end = Point::new(to.x + to.width, to_center_y);
            }
        }

        (start, end)
    }
}

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
        diagram.connections.push(Connection::new("API", "MySQL"));

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
