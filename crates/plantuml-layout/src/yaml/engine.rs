//! Layout engine для YAML диаграмм
//!
//! Переиспользует JSON layout engine, адаптируя его для YamlDiagram.

use plantuml_ast::json::JsonDiagram;
use plantuml_ast::yaml::YamlDiagram;

use crate::json::{JsonLayoutConfig, JsonLayoutEngine};
use crate::traits::{LayoutEngine, LayoutResult};
use crate::LayoutConfig;

/// Layout engine для YAML диаграмм
pub struct YamlLayoutEngine {
    json_engine: JsonLayoutEngine,
}

impl YamlLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        // YAML и JSON в PlantUML выводятся одной и той же двухколоночной
        // таблицей, поэтому отдельная нотация больше не нужна. Отличие
        // одно: boolean в YAML пишется текстом, а в JSON — флажком.
        Self::with_config(JsonLayoutConfig::default())
    }

    /// Создаёт engine с указанной конфигурацией
    pub fn with_config(config: JsonLayoutConfig) -> Self {
        // YAML выводит boolean обычным текстом — флажок PlantUML рисует
        // только в JSON (проверено на сервере).
        let mut config = config;
        config.bool_as_text = true;

        Self {
            json_engine: JsonLayoutEngine::with_config(config),
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
