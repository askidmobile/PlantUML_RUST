//! Layout engine для YAML диаграмм
//!
//! Переиспользует JSON layout engine, адаптируя его для YamlDiagram.

use plantuml_ast::json::JsonDiagram;
use plantuml_ast::yaml::YamlDiagram;

use crate::json::{JsonLayoutConfig, JsonLayoutEngine, Notation};
use crate::traits::{LayoutEngine, LayoutResult};
use crate::LayoutConfig;

/// Layout engine для YAML диаграмм
pub struct YamlLayoutEngine {
    json_engine: JsonLayoutEngine,
}

impl YamlLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            // YAML рисуется без фигурных и квадратных скобок: раньше движок
            // переиспользовал JSON-нотацию и YAML-диаграмма показывала
            // JSON-синтаксис (`key: {`), что не соответствует оригиналу.
            json_engine: JsonLayoutEngine::with_notation(Notation::Yaml),
        }
    }

    /// Создаёт engine с указанной конфигурацией
    pub fn with_config(config: JsonLayoutConfig) -> Self {
        Self {
            json_engine: JsonLayoutEngine::with_config_and_notation(config, Notation::Yaml),
        }
    }
}

impl Default for YamlLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine for YamlLayoutEngine {
    type Input = YamlDiagram;

    fn layout(&self, diagram: &Self::Input, config: &LayoutConfig) -> LayoutResult {
        // Конвертируем YAML в JSON для layout
        let json_diagram = JsonDiagram {
            root: diagram.root.clone(),
            metadata: diagram.metadata.clone(),
            highlights: diagram.highlights.clone(),
        };

        self.json_engine.layout(&json_diagram, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::json::JsonNode;

    #[test]
    fn test_yaml_layout_simple() {
        let root = JsonNode::object(
            None,
            vec![
                JsonNode::string(Some("name".into()), "John"),
                JsonNode::number(Some("age".into()), 30.0),
            ],
        );
        let diagram = YamlDiagram::with_root(root);

        let engine = YamlLayoutEngine::new();
        let config = LayoutConfig::default();
        let result = engine.layout(&diagram, &config);

        assert!(!result.elements.is_empty());
        assert!(result.bounds.width > 0.0);
        assert!(result.bounds.height > 0.0);
    }
}
