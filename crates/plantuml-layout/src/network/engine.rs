//! Layout engine для Network диаграмм
//!
//! Размещает сети как горизонтальные полосы с серверами внутри.
//! Серверы, принадлежащие нескольким сетям, соединяются вертикальными линиями.

use std::collections::{HashMap, HashSet};

use plantuml_ast::network::{DeviceType, NetworkDiagram, Server};
use plantuml_model::{Point, Rect};

use crate::network::config::NetworkLayoutConfig;
use crate::traits::{LayoutEngine, LayoutResult};
use crate::{EdgeType, ElementType, LayoutConfig, LayoutElement};

/// Layout engine для Network диаграмм
pub struct NetworkLayoutEngine {
    config: NetworkLayoutConfig,
}

impl NetworkLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: NetworkLayoutConfig::default(),
        }
    }

    /// Создаёт engine с указанной конфигурацией
    pub fn with_config(config: NetworkLayoutConfig) -> Self {
        Self { config }
    }

    /// Собирает информацию о серверах: в каких сетях они присутствуют
    fn collect_server_networks(&self, diagram: &NetworkDiagram) -> HashMap<String, Vec<usize>> {
        let mut server_networks: HashMap<String, Vec<usize>> = HashMap::new();

        for (net_idx, network) in diagram.networks.iter().enumerate() {
            for member in &network.members {
                server_networks
                    .entry(member.id.name.clone())
                    .or_default()
                    .push(net_idx);
            }
        }

        server_networks
    }

    /// Определяет уникальные серверы и их порядок (для согласованного X позиционирования)
    fn collect_unique_servers(&self, diagram: &NetworkDiagram) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut servers = Vec::new();

        for network in &diagram.networks {
            for member in &network.members {
                if seen.insert(member.id.name.clone()) {
                    servers.push(member.id.name.clone());
                }
            }
        }

        // Добавляем глобальные серверы
        for server in &diagram.servers {
            if seen.insert(server.id.name.clone()) {
                servers.push(server.id.name.clone());
            }
        }

        servers
    }

    /// Вычисляет X позицию для сервера по индексу
    fn server_x_position(&self, index: usize) -> f64 {
        self.config.padding + index as f64 * (self.config.server_width + self.config.server_spacing)
    }

    /// Вычисляет Y позицию для сети по индексу
    fn network_y_position(&self, index: usize) -> f64 {
        self.config.padding + index as f64 * (self.config.network_band_height + self.config.network_spacing)
    }

    /// Размещает сети
    fn layout_networks(
        &self,
        diagram: &NetworkDiagram,
        server_order: &[String],
        elements: &mut Vec<LayoutElement>,
    ) -> (f64, f64) {
        let mut max_width = 0.0_f64;
        let mut total_height = self.config.padding;

        for (net_idx, network) in diagram.networks.iter().enumerate() {
            let y = self.network_y_position(net_idx);
            
            // Ширина сети зависит от количества серверов
            let network_width = server_order.len() as f64 * (self.config.server_width + self.config.server_spacing)
                + self.config.padding;

            // Фон сети
            let network_bg = LayoutElement {
                id: format!("network_{}_bg", network.id.name),
                element_type: ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 5.0,
                },
                bounds: Rect::new(
                    self.config.padding,
                    y,
                    network_width,
                    self.config.network_band_height,
                ),
                text: None,
                properties: [
                    ("fill".to_string(), self.config.network_bg_color.to_string()),
                    ("stroke".to_string(), "#A0A0A0".to_string()),
                    ("stroke-dasharray".to_string(), "5,5".to_string()),
                ]
                .into_iter()
                .collect(),
            };
            elements.push(network_bg);

            // Название сети
            let mut label = network.id.name.clone();
            if let Some(ref addr) = network.address {
                label = format!("{}\n{}", label, addr);
            }

            let network_label = LayoutElement {
                id: format!("network_{}_label", network.id.name),
                element_type: ElementType::Text {
                    text: label.clone(),
                    font_size: self.config.font_size,
                },
                bounds: Rect::new(
                    self.config.padding + 10.0,
                    y + 5.0,
                    150.0,
                    self.config.network_header_height,
                ),
                text: Some(label),
                properties: [
                    ("fill".to_string(), "#000000".to_string()),
                    ("font-weight".to_string(), "bold".to_string()),
                ]
                .into_iter()
                .collect(),
            };
            elements.push(network_label);

            max_width = max_width.max(network_width + self.config.padding * 2.0);
            total_height = y + self.config.network_band_height;
        }

        (max_width, total_height + self.config.padding)
    }

    /// Размещает серверы
    fn layout_servers(
        &self,
        diagram: &NetworkDiagram,
        server_order: &[String],
        server_networks: &HashMap<String, Vec<usize>>,
        elements: &mut Vec<LayoutElement>,
    ) -> HashMap<String, Vec<Point>> {
        let mut server_positions: HashMap<String, Vec<Point>> = HashMap::new();

        // Создаём map для быстрого поиска данных сервера
        let mut server_data: HashMap<String, &Server> = HashMap::new();
        for network in &diagram.networks {
            for member in &network.members {
                server_data.insert(member.id.name.clone(), member);
            }
        }
        for server in &diagram.servers {
            server_data.insert(server.id.name.clone(), server);
        }

        for (srv_idx, server_name) in server_order.iter().enumerate() {
            let x = self.server_x_position(srv_idx);
            
            if let Some(net_indices) = server_networks.get(server_name) {
                let server = server_data.get(server_name);
                
                for &net_idx in net_indices {
                    let y = self.network_y_position(net_idx) 
                        + self.config.network_header_height + 10.0;

                    let server_rect = Rect::new(
                        x,
                        y,
                        self.config.server_width,
                        self.config.server_height,
                    );

                    // Иконка/форма сервера в зависимости от типа
                    let device_type = server.map(|s| s.device_type).unwrap_or(DeviceType::Server);
                    self.render_server(
                        server_name,
                        server,
                        &server_rect,
                        device_type,
                        net_idx,
                        elements,
                    );

                    // Сохраняем центр сервера для соединительных линий
                    let center = Point::new(
                        server_rect.x + server_rect.width / 2.0,
                        server_rect.y + server_rect.height / 2.0,
                    );
                    server_positions
                        .entry(server_name.clone())
                        .or_default()
                        .push(center);
                }
            }
        }

        server_positions
    }

    /// Рендерит сервер
    fn render_server(
        &self,
        name: &str,
        server: Option<&&Server>,
        bounds: &Rect,
        device_type: DeviceType,
        net_idx: usize,
        elements: &mut Vec<LayoutElement>,
    ) {
        let id = format!("server_{}_{}", name, net_idx);

        // Форма сервера
        let (element_type, fill_color) = match device_type {
            DeviceType::Database => (
                ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 0.0, // Для БД можно будет сделать cylinder
                },
                "#FFFFCC",
            ),
            DeviceType::Cloud => (
                ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 15.0,
                },
                "#CCE5FF",
            ),
            DeviceType::Firewall => (
                ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 0.0,
                },
                "#FFCCCC",
            ),
            DeviceType::Router | DeviceType::Switch => (
                ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 3.0,
                },
                "#CCFFCC",
            ),
            _ => (
                ElementType::Rectangle {
                    label: String::new(),
                    corner_radius: 3.0,
                },
                self.config.server_bg_color,
            ),
        };

        let server_shape = LayoutElement {
            id: format!("{}_bg", id),
            element_type,
            bounds: *bounds,
            text: None,
            properties: [
                ("fill".to_string(), fill_color.to_string()),
                ("stroke".to_string(), "#181818".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(server_shape);

        // Название сервера
        let server_label = LayoutElement {
            id: format!("{}_name", id),
            element_type: ElementType::Text {
                text: name.to_string(),
                font_size: self.config.font_size,
            },
            bounds: Rect::new(
                bounds.x + 5.0,
                bounds.y + 5.0,
                bounds.width - 10.0,
                20.0,
            ),
            text: Some(name.to_string()),
            properties: [
                ("fill".to_string(), "#000000".to_string()),
                ("font-weight".to_string(), "bold".to_string()),
                ("text-anchor".to_string(), "middle".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        elements.push(server_label);

        // Адрес сервера (если есть)
        if let Some(srv) = server {
            if let Some(ref addr) = srv.address {
                let addr_label = LayoutElement {
                    id: format!("{}_addr", id),
                    element_type: ElementType::Text {
                        text: addr.clone(),
                        font_size: self.config.font_size - 2.0,
                    },
                    bounds: Rect::new(
                        bounds.x + 5.0,
                        bounds.y + 25.0,
                        bounds.width - 10.0,
                        16.0,
                    ),
                    text: Some(addr.clone()),
                    properties: [
                        ("fill".to_string(), "#666666".to_string()),
                        ("text-anchor".to_string(), "middle".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                };
                elements.push(addr_label);
            }
        }
    }

    /// Рисует соединительные линии для серверов, присутствующих в нескольких сетях
    fn render_connections(
        &self,
        server_positions: &HashMap<String, Vec<Point>>,
        elements: &mut Vec<LayoutElement>,
    ) {
        for (server_name, positions) in server_positions {
            if positions.len() > 1 {
                // Соединяем все позиции вертикальной линией
                let mut sorted_positions = positions.clone();
                sorted_positions.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

                for window in sorted_positions.windows(2) {
                    let from = window[0];
                    let to = window[1];

                    let connection = LayoutElement {
                        id: format!("conn_{}_{:.0}_{:.0}", server_name, from.y, to.y),
                        element_type: ElementType::Edge {
                            points: vec![
                                Point::new(from.x, from.y + self.config.server_height / 2.0),
                                Point::new(to.x, to.y - self.config.server_height / 2.0),
                            ],
                            label: None,
                            arrow_start: false,
                            arrow_end: false,
                            dashed: false,
                            edge_type: EdgeType::Link, from_cardinality: None, to_cardinality: None,
                        },
                        bounds: Rect::from_points(from, to),
                        text: None,
                        properties: [
                            ("stroke".to_string(), "#666666".to_string()),
                            ("stroke-width".to_string(), "2".to_string()),
                        ]
                        .into_iter()
                        .collect(),
                    };
                    elements.push(connection);
                }
            }
        }
    }

    /// Рендерит группы серверов
    fn render_groups(
        &self,
        diagram: &NetworkDiagram,
        server_order: &[String],
        elements: &mut Vec<LayoutElement>,
    ) {
        for (grp_idx, group) in diagram.groups.iter().enumerate() {
            // Находим границы группы
            let mut min_x = f64::MAX;
            let mut max_x = 0.0_f64;

            for server_name in &group.servers {
                if let Some(idx) = server_order.iter().position(|s| s == server_name) {
                    let x = self.server_x_position(idx);
                    min_x = min_x.min(x);
                    max_x = max_x.max(x + self.config.server_width);
                }
            }

            if min_x < f64::MAX {
                // Рисуем фон группы
                let group_bg = LayoutElement {
                    id: format!("group_{}", grp_idx),
                    element_type: ElementType::Rectangle {
                        label: group.name.clone(),
                        corner_radius: 8.0,
                    },
                    bounds: Rect::new(
                        min_x - 10.0,
                        self.config.padding - 5.0,
                        max_x - min_x + 20.0,
                        (diagram.networks.len() as f64) * (self.config.network_band_height + self.config.network_spacing) + 10.0,
                    ),
                    text: Some(group.name.clone()),
                    properties: [
                        ("fill".to_string(), group.color.as_ref()
                            .map(|c| c.to_css())
                            .unwrap_or_else(|| self.config.group_bg_color.to_string())),
                        ("stroke".to_string(), "#888888".to_string()),
                        ("stroke-dasharray".to_string(), "3,3".to_string()),
                    ]
                    .into_iter()
                    .collect(),
                };
                // Группы рисуем на заднем плане (вставляем в начало)
                elements.insert(0, group_bg);
            }
        }
    }
}

impl Default for NetworkLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for NetworkLayoutEngine {
    type Input = NetworkDiagram;

    fn layout(&self, diagram: &Self::Input, _config: &LayoutConfig) -> LayoutResult {
        let mut elements = Vec::new();

        // Собираем информацию о серверах
        let server_networks = self.collect_server_networks(diagram);
        let server_order = self.collect_unique_servers(diagram);

        // Размещаем сети
        let (width, height) = self.layout_networks(diagram, &server_order, &mut elements);

        // Размещаем серверы
        let server_positions = self.layout_servers(
            diagram,
            &server_order,
            &server_networks,
            &mut elements,
        );

        // Рисуем соединительные линии
        self.render_connections(&server_positions, &mut elements);

        // Рисуем группы (на заднем плане)
        self.render_groups(diagram, &server_order, &mut elements);

        LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, width, height),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::network::{Network, Server};

    #[test]
    fn test_layout_simple_network() {
        let mut diagram = NetworkDiagram::new();

        let mut dmz = Network::new("dmz").with_address("192.168.1.0/24");
        dmz.add_member(Server::new("web01").with_address("192.168.1.1"));
        dmz.add_member(Server::new("web02").with_address("192.168.1.2"));
        diagram.add_network(dmz);

        let engine = NetworkLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        assert!(!result.elements.is_empty());
        assert!(result.bounds.width > 0.0);
        assert!(result.bounds.height > 0.0);
    }

    #[test]
    fn test_layout_multiple_networks() {
        let mut diagram = NetworkDiagram::new();

        let mut dmz = Network::new("dmz");
        dmz.add_member(Server::new("web01"));
        diagram.add_network(dmz);

        let mut internal = Network::new("internal");
        internal.add_member(Server::new("web01")); // Сервер в двух сетях
        internal.add_member(Server::new("db01"));
        diagram.add_network(internal);

        let engine = NetworkLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        // Должны быть элементы для обеих сетей и серверов
        assert!(result.elements.len() >= 4);
    }

    #[test]
    fn test_server_in_multiple_networks_connected() {
        let mut diagram = NetworkDiagram::new();

        let mut dmz = Network::new("dmz");
        dmz.add_member(Server::new("gateway"));
        diagram.add_network(dmz);

        let mut internal = Network::new("internal");
        internal.add_member(Server::new("gateway")); // Тот же сервер
        diagram.add_network(internal);

        let engine = NetworkLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        // Должна быть соединительная линия
        let has_connection = result.elements.iter().any(|e| e.id.starts_with("conn_"));
        assert!(has_connection);
    }
}
