//! Структуры данных графа для алгоритма Sugiyama.
//!
//! Граф строится из ClassDiagram: узлы = классы, рёбра = отношения.

use std::collections::HashMap;

use plantuml_ast::class::{ClassDiagram, Classifier, Relationship, RelationshipType};
use plantuml_model::Size;

use super::config::ClassLayoutConfig;

/// Узел графа (класс/интерфейс)
#[derive(Debug, Clone)]
pub struct Node {
    /// Уникальный идентификатор узла
    pub id: String,
    /// Индекс узла в графе
    pub index: usize,
    /// Ссылка на classifier (имя для lookup)
    pub classifier_name: String,
    /// Размер узла (вычисляется из содержимого)
    pub size: Size,
    /// Слой (вертикальный уровень)
    pub layer: usize,
    /// Позиция внутри слоя (горизонтальный порядок)
    pub position: usize,
    /// X координата (после layout)
    pub x: f64,
    /// Y координата (после layout)
    pub y: f64,
}

impl Node {
    /// Создаёт новый узел
    pub fn new(
        id: String,
        index: usize,
        classifier: &Classifier,
        config: &ClassLayoutConfig,
    ) -> Self {
        let size = Self::calculate_size(classifier, config);
        Self {
            id: id.clone(),
            index,
            classifier_name: classifier.id.name.clone(),
            size,
            layer: 0,
            position: 0,
            x: 0.0,
            y: 0.0,
        }
    }

    /// Вычисляет размер узла на основе содержимого класса
    fn calculate_size(classifier: &Classifier, config: &ClassLayoutConfig) -> Size {
        // Ширина: max(имя класса, поля, методы)
        // Добавляем место для иконки класса (~30px)
        let icon_width = 30.0;
        let name_width =
            classifier.id.name.len() as f64 * config.char_width + icon_width + config.class_padding * 2.0;

        let field_max_width = classifier
            .fields
            .iter()
            .map(|f| {
                // Учитываем тип поля: "+name: type"
                let text = if let Some(ref typ) = f.member_type {
                    format!("{}{}: {}", f.visibility.to_char(), f.name, typ)
                } else {
                    format!("{}{}", f.visibility.to_char(), f.name)
                };
                text.len() as f64 * config.char_width
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let method_max_width = classifier
            .methods
            .iter()
            .map(|m| {
                // Учитываем return type (member_type в AST): "+method(): type"
                let text = if let Some(ref ret_type) = m.member_type {
                    format!("{}{}(): {}", m.visibility.to_char(), m.name, ret_type)
                } else {
                    format!("{}{}()", m.visibility.to_char(), m.name)
                };
                text.len() as f64 * config.char_width
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Добавляем место для иконки видимости (~15px)
        let visibility_icon_width = 15.0;
        let content_width = field_max_width.max(method_max_width) + visibility_icon_width + config.class_padding * 2.0;
        let width = name_width.max(content_width).max(config.min_class_width);

        // Высота: заголовок + поля + методы
        // Заголовок включает: иконку + стереотип (если есть) + имя класса
        let has_stereotype = classifier.classifier_type != plantuml_ast::class::ClassifierType::Class;
        let header_height = if has_stereotype {
            // Стереотип + имя = больше высоты
            config.class_header_height + 12.0
        } else {
            config.class_header_height
        };

        // Секция полей
        let fields_height = if classifier.fields.is_empty() {
            0.0
        } else {
            classifier.fields.len() as f64 * config.line_height + config.class_padding
        };

        // Секция методов
        let methods_height = if classifier.methods.is_empty() {
            0.0
        } else {
            classifier.methods.len() as f64 * config.line_height + config.class_padding
        };

        let height = header_height + fields_height + methods_height + config.class_padding;
        let height = height.max(config.min_class_height);

        Size::new(width, height)
    }
}

/// Ребро графа (отношение между классами)
#[derive(Debug, Clone)]
pub struct Edge {
    /// Индекс исходного узла
    pub from: usize,
    /// Индекс целевого узла
    pub to: usize,
    /// Тип отношения
    pub relationship_type: RelationshipType,
    /// Метка
    pub label: Option<String>,
    /// Кардинальность у исходного узла (например "1")
    pub from_cardinality: Option<String>,
    /// Кардинальность у целевого узла (например "*")
    pub to_cardinality: Option<String>,
    /// Обратное ребро (для удаления циклов)
    pub reversed: bool,
}

impl Edge {
    /// Создаёт новое ребро
    pub fn new(from: usize, to: usize, rel: &Relationship) -> Self {
        Self {
            from,
            to,
            relationship_type: rel.relationship_type,
            label: rel.label.clone(),
            from_cardinality: rel.from_cardinality.clone(),
            to_cardinality: rel.to_cardinality.clone(),
            reversed: false,
        }
    }
}

/// Граф для алгоритма Sugiyama
#[derive(Debug)]
pub struct Graph {
    /// Все узлы
    pub nodes: Vec<Node>,
    /// Все рёбра
    pub edges: Vec<Edge>,
    /// Индекс узлов по имени
    node_index: HashMap<String, usize>,
    /// Списки смежности (исходящие рёбра)
    pub adjacency: Vec<Vec<usize>>,
    /// Обратные списки смежности (входящие рёбра)
    pub reverse_adjacency: Vec<Vec<usize>>,
}

impl Graph {
    /// Создаёт граф из ClassDiagram
    pub fn from_diagram(diagram: &ClassDiagram, config: &ClassLayoutConfig) -> Self {
        let mut nodes = Vec::new();
        let mut node_index = HashMap::new();

        // Создаём узлы из классификаторов
        for classifier in &diagram.classifiers {
            let id = classifier.id.name.clone();
            if !node_index.contains_key(&id) {
                let index = nodes.len();
                node_index.insert(id.clone(), index);
                nodes.push(Node::new(id, index, classifier, config));
            }
        }

        // Также добавляем узлы из пакетов (рекурсивно)
        Self::collect_classifiers_from_packages(
            &diagram.packages,
            &mut nodes,
            &mut node_index,
            config,
        );

        // Создаём фиктивные узлы для классов, упомянутых в отношениях, но не объявленных
        for rel in &diagram.relationships {
            for name in [&rel.from, &rel.to] {
                if !node_index.contains_key(name) {
                    let index = nodes.len();
                    node_index.insert(name.clone(), index);
                    // Создаём минимальный узел
                    nodes.push(Node {
                        id: name.clone(),
                        index,
                        classifier_name: name.clone(),
                        size: Size::new(config.min_class_width, config.min_class_height),
                        layer: 0,
                        position: 0,
                        x: 0.0,
                        y: 0.0,
                    });
                }
            }
        }

        // Создаём рёбра
        // В PlantUML синтаксис "A <|-- B" означает "B наследует от A" (B extends A)
        // В AST: rel.from = "B" (дочерний), rel.to = "A" (родитель)
        // Для layout: родитель должен быть на слое 0 (вверху), потомки ниже
        // Поэтому ребро в графе: от родителя к потомку (from=to_idx, to=from_idx)
        let mut edges = Vec::new();
        for rel in &diagram.relationships {
            if let (Some(&from_idx), Some(&to_idx)) =
                (node_index.get(&rel.from), node_index.get(&rel.to))
            {
                // Для наследования/реализации: в AST from=дочерний, to=родитель
                // В графе для layout: родитель → потомок (чтобы родитель был на слое 0)
                let (graph_from, graph_to) = match rel.relationship_type {
                    RelationshipType::Inheritance | RelationshipType::Realization => {
                        (to_idx, from_idx) // Родитель → Потомок (родитель на слое 0)
                    }
                    _ => (from_idx, to_idx),
                };
                edges.push(Edge::new(graph_from, graph_to, rel));
            }
        }

        // Строим списки смежности
        let n = nodes.len();
        let mut adjacency = vec![Vec::new(); n];
        let mut reverse_adjacency = vec![Vec::new(); n];

        for (edge_idx, edge) in edges.iter().enumerate() {
            adjacency[edge.from].push(edge_idx);
            reverse_adjacency[edge.to].push(edge_idx);
        }

        Self {
            nodes,
            edges,
            node_index,
            adjacency,
            reverse_adjacency,
        }
    }

    /// Собирает классификаторы из пакетов рекурсивно
    fn collect_classifiers_from_packages(
        packages: &[plantuml_ast::class::Package],
        nodes: &mut Vec<Node>,
        node_index: &mut HashMap<String, usize>,
        config: &ClassLayoutConfig,
    ) {
        for package in packages {
            for classifier in &package.classifiers {
                let id = classifier.id.name.clone();
                if !node_index.contains_key(&id) {
                    let index = nodes.len();
                    node_index.insert(id.clone(), index);
                    nodes.push(Node::new(id, index, classifier, config));
                }
            }
            // Рекурсивно обрабатываем вложенные пакеты
            Self::collect_classifiers_from_packages(&package.packages, nodes, node_index, config);
        }
    }

    /// Возвращает количество узлов
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Возвращает количество рёбер
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Возвращает исходящие рёбра узла
    pub fn outgoing_edges(&self, node: usize) -> impl Iterator<Item = &Edge> {
        self.adjacency[node].iter().map(|&idx| &self.edges[idx])
    }

    /// Возвращает входящие рёбра узла
    pub fn incoming_edges(&self, node: usize) -> impl Iterator<Item = &Edge> {
        self.reverse_adjacency[node]
            .iter()
            .map(|&idx| &self.edges[idx])
    }

    /// Возвращает узлы на указанном слое
    pub fn nodes_on_layer(&self, layer: usize) -> Vec<usize> {
        self.nodes
            .iter()
            .filter(|n| n.layer == layer)
            .map(|n| n.index)
            .collect()
    }

    /// Возвращает максимальный номер слоя
    pub fn max_layer(&self) -> usize {
        self.nodes.iter().map(|n| n.layer).max().unwrap_or(0)
    }

    /// Получает узел по имени
    pub fn get_node_by_name(&self, name: &str) -> Option<&Node> {
        self.node_index.get(name).map(|&idx| &self.nodes[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::class::{Classifier, Member, Visibility};

    #[test]
    fn test_graph_from_diagram() {
        let mut diagram = ClassDiagram::new();

        let mut animal = Classifier::new("Animal");
        animal.add_method(Member::method("eat").with_visibility(Visibility::Public));
        diagram.add_class(animal);

        let dog = Classifier::new("Dog");
        diagram.add_class(dog);

        diagram.add_relationship(Relationship::inheritance("Dog", "Animal"));

        let config = ClassLayoutConfig::default();
        let graph = Graph::from_diagram(&diagram, &config);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_node_size_calculation() {
        let config = ClassLayoutConfig::default();

        let mut classifier = Classifier::new("TestClass");
        classifier.add_field(Member::field("id", "Long").with_visibility(Visibility::Private));
        classifier.add_field(Member::field("name", "String").with_visibility(Visibility::Private));
        classifier.add_method(Member::method("getId").with_visibility(Visibility::Public));

        let node = Node::new("TestClass".to_string(), 0, &classifier, &config);

        assert!(node.size.width >= config.min_class_width);
        assert!(node.size.height >= config.min_class_height);
    }
}
