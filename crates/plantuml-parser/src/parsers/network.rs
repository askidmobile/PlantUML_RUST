//! Парсер Network (nwdiag) диаграмм
//!
//! Парсит Network диаграммы PlantUML.

use pest::Parser;
use pest_derive::Parser;

use plantuml_ast::common::Color;
use plantuml_ast::network::{DeviceType, Network, NetworkDiagram, Server, ServerGroup};

use crate::error::ParseError;

#[derive(Parser)]
#[grammar = "grammars/network.pest"]
struct NetworkParser;

/// Парсит Network диаграмму
pub fn parse_network(source: &str) -> crate::Result<NetworkDiagram> {
    // Извлекаем содержимое nwdiag блока
    let nwdiag_content = extract_nwdiag_content(source)?;

    let pairs = NetworkParser::parse(Rule::network_diagram, &nwdiag_content)
        .map_err(|e| ParseError::GrammarError(format!("Ошибка парсинга Network: {}", e)))?;

    let mut diagram = NetworkDiagram::new();

    for pair in pairs {
        if pair.as_rule() == Rule::network_diagram {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::nwdiag_block {
                    parse_nwdiag_block(inner, &mut diagram)?;
                }
            }
        }
    }

    Ok(diagram)
}

/// Извлекает содержимое nwdiag блока из исходного кода
fn extract_nwdiag_content(source: &str) -> crate::Result<String> {
    // Удаляем @startuml/@enduml и извлекаем содержимое
    let source = source.trim();

    // Проверяем наличие @startuml
    let content = if source.starts_with("@startuml") {
        let end_idx = source.rfind("@enduml").unwrap_or(source.len());
        let start_idx = source.find('\n').map(|i| i + 1).unwrap_or(9);
        &source[start_idx..end_idx]
    } else {
        source
    };

    Ok(content.trim().to_string())
}

/// Парсит блок nwdiag
fn parse_nwdiag_block(
    pair: pest::iterators::Pair<Rule>,
    diagram: &mut NetworkDiagram,
) -> crate::Result<()> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::diagram_content {
            for element in inner.into_inner() {
                match element.as_rule() {
                    Rule::network_definition => {
                        let network = parse_network_definition(element)?;
                        diagram.add_network(network);
                    }
                    Rule::group_definition => {
                        let group = parse_group_definition(element)?;
                        diagram.groups.push(group);
                    }
                    Rule::server_definition => {
                        let server = parse_server_definition(element)?;
                        diagram.add_server(server);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// Парсит определение сети
fn parse_network_definition(pair: pest::iterators::Pair<Rule>) -> crate::Result<Network> {
    let mut name = String::new();
    let mut network = Network::new("");

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                name = inner.as_str().to_string();
            }
            Rule::network_content => {
                for element in inner.into_inner() {
                    match element.as_rule() {
                        Rule::network_address => {
                            if let Some(addr) = element.into_inner().next() {
                                if let Some(content) = addr.into_inner().next() {
                                    network.address = Some(content.as_str().to_string());
                                }
                            }
                        }
                        Rule::server_in_network => {
                            let server = parse_server_in_network(element)?;
                            network.add_member(server);
                        }
                        Rule::description_attr => {
                            if let Some(val) = element.into_inner().next() {
                                if let Some(content) = val.into_inner().next() {
                                    network.description = Some(content.as_str().to_string());
                                }
                            }
                        }
                        Rule::color_attr => {
                            if let Some(val) = element.into_inner().next() {
                                network.color = Some(parse_color_value(val.as_str()));
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    network.id.name = name;
    Ok(network)
}

/// Парсит сервер внутри сети
fn parse_server_in_network(pair: pest::iterators::Pair<Rule>) -> crate::Result<Server> {
    let mut name = String::new();
    let mut server = Server::new("");

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                name = inner.as_str().to_string();
            }
            Rule::server_attributes => {
                parse_server_attributes(inner, &mut server)?;
            }
            _ => {}
        }
    }

    server.id.name = name;
    Ok(server)
}

/// Парсит определение сервера вне сети
fn parse_server_definition(pair: pest::iterators::Pair<Rule>) -> crate::Result<Server> {
    let mut name = String::new();
    let mut server = Server::new("");

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                name = inner.as_str().to_string();
            }
            Rule::server_attributes => {
                parse_server_attributes(inner, &mut server)?;
            }
            _ => {}
        }
    }

    server.id.name = name;
    Ok(server)
}

/// Парсит атрибуты сервера
fn parse_server_attributes(
    pair: pest::iterators::Pair<Rule>,
    server: &mut Server,
) -> crate::Result<()> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::address_attr => {
                if let Some(val) = inner.into_inner().next() {
                    if let Some(content) = val.into_inner().next() {
                        server.address = Some(content.as_str().to_string());
                    }
                }
            }
            Rule::description_attr => {
                if let Some(val) = inner.into_inner().next() {
                    if let Some(content) = val.into_inner().next() {
                        server.description = Some(content.as_str().to_string());
                    }
                }
            }
            Rule::color_attr => {
                if let Some(val) = inner.into_inner().next() {
                    server.color = Some(parse_color_value(val.as_str()));
                }
            }
            Rule::type_attr => {
                if let Some(val) = inner.into_inner().next() {
                    if let Some(dt) = DeviceType::parse(val.as_str()) {
                        server.device_type = dt;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Парсит определение группы
fn parse_group_definition(pair: pest::iterators::Pair<Rule>) -> crate::Result<ServerGroup> {
    let mut name = String::new();
    let mut group = ServerGroup::new("");

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                name = inner.as_str().to_string();
            }
            Rule::group_content => {
                for element in inner.into_inner() {
                    match element.as_rule() {
                        Rule::color_attr => {
                            if let Some(val) = element.into_inner().next() {
                                group.color = Some(parse_color_value(val.as_str()));
                            }
                        }
                        Rule::description_attr => {
                            if let Some(val) = element.into_inner().next() {
                                if let Some(content) = val.into_inner().next() {
                                    group.description = Some(content.as_str().to_string());
                                }
                            }
                        }
                        Rule::group_member => {
                            group.servers.push(element.as_str().to_string());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if name.is_empty() {
        group.name = format!("group_{}", group.servers.len());
    } else {
        group.name = name;
    }

    Ok(group)
}

/// Парсит значение цвета
fn parse_color_value(s: &str) -> Color {
    let s = s.trim().trim_matches('"');
    if s.starts_with('#') {
        Color::from_hex(s.to_string())
    } else {
        Color::Named(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_network() {
        let source = r#"@startuml
nwdiag {
  network dmz {
    address = "210.x.x.x/24"
    web01 [address = "210.x.x.1"]
    web02 [address = "210.x.x.2"]
  }
}
@enduml"#;

        let diagram = parse_network(source).unwrap();
        assert_eq!(diagram.networks.len(), 1);

        let dmz = &diagram.networks[0];
        assert_eq!(dmz.id.name, "dmz");
        assert_eq!(dmz.address, Some("210.x.x.x/24".to_string()));
        assert_eq!(dmz.members.len(), 2);
        assert_eq!(dmz.members[0].id.name, "web01");
        assert_eq!(dmz.members[0].address, Some("210.x.x.1".to_string()));
    }

    #[test]
    fn test_parse_multiple_networks() {
        let source = r#"@startuml
nwdiag {
  network dmz {
    web01
    web02
  }
  network internal {
    web01
    db01
  }
}
@enduml"#;

        let diagram = parse_network(source).unwrap();
        assert_eq!(diagram.networks.len(), 2);
        assert_eq!(diagram.networks[0].id.name, "dmz");
        assert_eq!(diagram.networks[1].id.name, "internal");
    }

    #[test]
    fn test_parse_group() {
        let source = r##"@startuml
nwdiag {
  network dmz {
    web01
    web02
  }
  group {
    color = "#FFAAAA"
    web01
    web02
  }
}
@enduml"##;

        let diagram = parse_network(source).unwrap();
        assert_eq!(diagram.groups.len(), 1);
        assert_eq!(diagram.groups[0].servers.len(), 2);
    }

    #[test]
    fn test_parse_server_type() {
        let source = r#"@startuml
nwdiag {
  network dmz {
    firewall01 [type = firewall]
    router01 [type = router]
  }
}
@enduml"#;

        let diagram = parse_network(source).unwrap();
        assert_eq!(
            diagram.networks[0].members[0].device_type,
            DeviceType::Firewall
        );
        assert_eq!(
            diagram.networks[0].members[1].device_type,
            DeviceType::Router
        );
    }
}
