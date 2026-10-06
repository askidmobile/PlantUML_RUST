//! Структуры данных графа для алгоритма Sugiyama.
//!
//! Граф строится из ClassDiagram: узлы = классы, рёбра = отношения.

use std::collections::HashMap;

use plantuml_ast::class::{ClassDiagram, Classifier, Relationship, RelationshipType, Sprite};
use plantuml_model::Size;

/// Добавка к ширине строки имени: иконка класса слева и отступ справа.
///
/// Измерено по эталону class_inheritance: «Dog» 28.232 → 68.3,
/// «Animal» 48.446 → 80.446.
const CLASS_NAME_EXTRA: f64 = 32.0;

/// Ширина пикселя спрайта, вставленного в подпись.
const SPRITE_LABEL_PIXEL: f64 = 1.0;

/// Отступ после спрайта внутри подписи.
const SPRITE_LABEL_GAP: f64 = 2.0;

/// Ширина строки имени с учётом вставок спрайтов.
///
/// Вставка `<$имя>` занимает ширину РАСТРА, а не текста: прежний код
/// измерял сам текст `<$имя>`, из-за чего рамка класса распухала.
/// Проверено на сервере: для `class "X <$s*3>"` со спрайтом 4x4 эталон
/// даёт рамку 77.563, а измерение текста как есть — заметно больше.
///
/// Модификаторы масштаба разбираются так же, как в рендерере:
/// `*N`, `{scale=N}` и `,scale=N`.
fn name_width_with_sprites(name: &str, sprites: &[Sprite], config: &ClassLayoutConfig) -> f64 {
    let mut plain = String::new();
    let mut extra = 0.0;
    let mut rest = name;

    while let Some(start) = rest.find("<$") {
        let after = &rest[start + 2..];
        let Some(end) = after.find('>') else {
            break;
        };

        let body = &after[..end];
        let name_end = body.find(['*', '{', ',']).unwrap_or(body.len());
        let sprite_name = &body[..name_end];
        let scale = sprite_scale_of(&body[name_end..]);

        match sprites.iter().find(|sprite| sprite.name == sprite_name) {
            Some(sprite) => {
                plain.push_str(&rest[..start]);
                extra += sprite.width as f64 * SPRITE_LABEL_PIXEL * scale + SPRITE_LABEL_GAP;
            }
            // Неизвестный спрайт рендерер оставляет текстом — учитываем так же.
            None => plain.push_str(&rest[..start + 2 + end + 1]),
        }

        rest = &after[end + 1..];
    }

    plain.push_str(rest);
    config.text.width(&plain, config.font_size) + extra
}

/// Разбирает масштаб из модификаторов вставки спрайта.
///
/// Повторяет разбор из рендерера: крейты не могут импортировать друг
/// друга (рендерер зависит от раскладки), поэтому правило продублировано.
fn sprite_scale_of(modifiers: &str) -> f64 {
    let number_after = |text: &str| -> Option<f64> {
        let digits: String = text
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        digits.parse().ok()
    };

    if let Some(rest) = modifiers.strip_prefix('*') {
        if let Some(value) = number_after(rest) {
            return value;
        }
    }

    if let Some(index) = modifiers.find("scale=") {
        if let Some(value) = number_after(&modifiers[index + 6..]) {
            return value;
        }
    }

    1.0
}

/// Добавка к ширине строки содержимого (поля и методы).
///
/// Измерено по эталону: «bark()» 42.253 → 68.25, «meow()» 53.19 → 79.19.
const CLASS_CONTENT_EXTRA: f64 = 26.0;

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
        sprites: &[Sprite],
    ) -> Self {
        let size = Self::calculate_size(classifier, config, sprites);
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
    fn calculate_size(
        classifier: &Classifier,
        config: &ClassLayoutConfig,
        sprites: &[Sprite],
    ) -> Size {
        // Ширина: max(имя класса, поля, методы)
        // Добавляем место для иконки класса (~30px)
        // Ширина строки имени: измерено по эталону — ширина имени плюс 32
        // («Dog» 28.232 → 68.3; «Animal» 48.446 → 80.446). Добавка вмещает
        // иконку класса слева и отступ справа.
        let name_width =
            // Имя класса рисуется ЖИРНЫМ — шире обычного примерно на 8.3%.
            name_width_with_sprites(&classifier.id.name, sprites, config) + CLASS_NAME_EXTRA;

        let field_max_width = classifier
            .fields
            .iter()
            .map(|f| {
                // Учитываем тип поля: "+name: type"
                // Маркер видимости НЕ входит в измеряемый текст: PlantUML
                // рисует его отдельной иконкой, а место под неё уже учтено
                // в CLASS_CONTENT_EXTRA. Проверено по эталону:
                // «bark()» 42.253 + 26 = 68.253 — ровно ширина бокса Dog.
                // С «+» в тексте ширина выходила на 7.4px больше.
                let text = if let Some(ref typ) = f.member_type {
                    format!("{}: {}", f.name, typ)
                } else {
                    f.name.clone()
                };
                config.text.width(&text, config.font_size)
            })
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.0);

        let method_max_width = classifier
            .methods
            .iter()
            .map(|m| {
                // Учитываем return type (member_type в AST): "+method(): type"
                // См. комментарий выше: маркер видимости — иконка, а не текст.
                let text = if let Some(ref ret_type) = m.member_type {
                    format!("{}(): {}", m.name, ret_type)
                } else {
                    format!("{}()", m.name)
                };
                config.text.width(&text, config.font_size)
            })
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.0);

        // Ширина строки = отступ под иконку видимости + сам текст + правый
        // отступ. Измерено по эталону class_inheritance:
        //   «Dog»  ширина 68.3 при «bark()» 42.253
        //   «Cat»  ширина 79.2 при «meow()» 53.19
        let content_width = field_max_width.max(method_max_width) + CLASS_CONTENT_EXTRA;
        let width = name_width.max(content_width).max(config.min_class_width);

        // Высота: заголовок + поля + методы
        // Заголовок включает: иконку + стереотип (если есть) + имя класса
        let has_stereotype =
            classifier.classifier_type != plantuml_ast::class::ClassifierType::Class;
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
    /// Индекс узлов по слоям.
    ///
    /// Строится один раз после назначения слоёв. Прежде `nodes_on_layer`
    /// каждый раз просматривал ВСЕ узлы, а вызывается он на каждый слой и
    /// на каждую итерацию минимизации пересечений — на 2000 классах это
    /// давало квадратичный рост (3.5 секунды).
    layer_nodes: Vec<Vec<usize>>,
}

impl Graph {
    /// Создаёт граф из ClassDiagram
    pub fn from_diagram(diagram: &ClassDiagram, config: &ClassLayoutConfig) -> Self {
        let mut nodes = Vec::new();
        let mut node_index = HashMap::new();
        let sprites = diagram.sprites.as_slice();

        // Создаём узлы из классификаторов
        for classifier in &diagram.classifiers {
            let id = classifier.id.name.clone();
            if !node_index.contains_key(&id) {
                let index = nodes.len();
                node_index.insert(id.clone(), index);
                nodes.push(Node::new(id, index, classifier, config, sprites));
            }
        }

        // Также добавляем узлы из пакетов (рекурсивно)
        Self::collect_classifiers_from_packages(
            &diagram.packages,
            &mut nodes,
            &mut node_index,
            config,
            sprites,
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
                // Наследование НЕ разворачивается.
                //
                // Раньше здесь ребро наследования/реализации переворачивалось,
                // чтобы родитель оказался на слое 0, то есть сверху. Проверено
                // на plantuml.com: для `Child --|> Parent` PlantUML ставит
                // РЕБЁНКА сверху (y=27.85), а родителя снизу (y=135.85).
                // То есть наследование раскладывается как любая другая связь —
                // источник сверху, цель снизу.
                //
                // Подтверждается эталоном class_inheritance: Dog и Cat на
                // y=7, Animal на y=131.29.
                let (graph_from, graph_to) = (from_idx, to_idx);
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
            // Индекс слоёв пуст: слои ещё не назначены, его построит
            // `rebuild_layer_index` после `assign_layers`.
            layer_nodes: Vec::new(),
        }
    }

    /// Собирает классификаторы из пакетов рекурсивно
    fn collect_classifiers_from_packages(
        packages: &[plantuml_ast::class::Package],
        nodes: &mut Vec<Node>,
        node_index: &mut HashMap<String, usize>,
        config: &ClassLayoutConfig,
        sprites: &[Sprite],
    ) {
        for package in packages {
            for classifier in &package.classifiers {
                let id = classifier.id.name.clone();
                if !node_index.contains_key(&id) {
                    let index = nodes.len();
                    node_index.insert(id.clone(), index);
                    nodes.push(Node::new(id, index, classifier, config, sprites));
                }
            }
            // Рекурсивно обрабатываем вложенные пакеты
            Self::collect_classifiers_from_packages(
                &package.packages,
                nodes,
                node_index,
                config,
                sprites,
            );
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
        self.layer_nodes.get(layer).cloned().unwrap_or_default()
    }

    /// Перестраивает индекс узлов по слоям.
    ///
    /// Вызывать после ЛЮБОГО изменения `node.layer`.
    pub fn rebuild_layer_index(&mut self) {
        let max_layer = self.max_layer();
        let mut index = vec![Vec::new(); max_layer + 1];
        for node in &self.nodes {
            if let Some(bucket) = index.get_mut(node.layer) {
                bucket.push(node.index);
            }
        }
        self.layer_nodes = index;
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

        let node = Node::new("TestClass".to_string(), 0, &classifier, &config, &[]);

        assert!(node.size.width >= config.min_class_width);
        assert!(node.size.height >= config.min_class_height);
    }
}
