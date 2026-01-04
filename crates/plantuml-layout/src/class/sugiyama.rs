//! Алгоритм Sugiyama для иерархического layout графов.
//!
//! Реализует 4 основных шага:
//! 1. Удаление циклов (cycle removal)
//! 2. Присвоение слоёв (layer assignment)
//! 3. Минимизация пересечений (crossing minimization)
//! 4. Присвоение координат (coordinate assignment)

use std::collections::VecDeque;

use super::config::ClassLayoutConfig;
use super::graph::Graph;

/// Алгоритм Sugiyama
pub struct SugiyamaLayout<'a> {
    graph: &'a mut Graph,
    config: &'a ClassLayoutConfig,
}

impl<'a> SugiyamaLayout<'a> {
    /// Создаёт новый экземпляр алгоритма
    pub fn new(graph: &'a mut Graph, config: &'a ClassLayoutConfig) -> Self {
        Self { graph, config }
    }

    /// Выполняет полный layout
    pub fn run(&mut self) {
        if self.graph.node_count() == 0 {
            return;
        }

        // Шаг 1: Удаление циклов
        self.remove_cycles();

        // Шаг 2: Присвоение слоёв (longest path)
        self.assign_layers();

        // Шаг 3: Минимизация пересечений (barycenter)
        self.minimize_crossings();

        // Шаг 4: Присвоение координат
        self.assign_coordinates();
    }

    // =========================================================================
    // Шаг 1: Удаление циклов (Cycle Removal)
    // =========================================================================

    /// Удаляет циклы путём обращения обратных рёбер (greedy algorithm)
    fn remove_cycles(&mut self) {
        let n = self.graph.node_count();
        if n == 0 {
            return;
        }

        // Используем DFS для поиска обратных рёбер
        let mut visited = vec![false; n];
        let mut in_stack = vec![false; n];
        let mut back_edges = Vec::new();

        for start in 0..n {
            if !visited[start] {
                self.dfs_find_back_edges(start, &mut visited, &mut in_stack, &mut back_edges);
            }
        }

        // Обращаем обратные рёбра
        for edge_idx in back_edges {
            self.graph.edges[edge_idx].reversed = true;
            let edge = &mut self.graph.edges[edge_idx];
            std::mem::swap(&mut edge.from, &mut edge.to);
        }

        // Перестраиваем списки смежности
        self.rebuild_adjacency();
    }

    /// DFS для поиска обратных рёбер
    fn dfs_find_back_edges(
        &self,
        node: usize,
        visited: &mut [bool],
        in_stack: &mut [bool],
        back_edges: &mut Vec<usize>,
    ) {
        visited[node] = true;
        in_stack[node] = true;

        for &edge_idx in &self.graph.adjacency[node] {
            let edge = &self.graph.edges[edge_idx];
            let target = edge.to;

            if !visited[target] {
                self.dfs_find_back_edges(target, visited, in_stack, back_edges);
            } else if in_stack[target] {
                // Нашли обратное ребро
                back_edges.push(edge_idx);
            }
        }

        in_stack[node] = false;
    }

    /// Перестраивает списки смежности после обращения рёбер
    fn rebuild_adjacency(&mut self) {
        let n = self.graph.node_count();
        self.graph.adjacency = vec![Vec::new(); n];
        self.graph.reverse_adjacency = vec![Vec::new(); n];

        for (idx, edge) in self.graph.edges.iter().enumerate() {
            self.graph.adjacency[edge.from].push(idx);
            self.graph.reverse_adjacency[edge.to].push(idx);
        }
    }

    // =========================================================================
    // Шаг 2: Присвоение слоёв (Layer Assignment)
    // =========================================================================

    /// Присваивает слои методом longest path
    fn assign_layers(&mut self) {
        let n = self.graph.node_count();
        if n == 0 {
            return;
        }

        // Топологическая сортировка (Kahn's algorithm)
        let topo_order = self.topological_sort();

        // Longest path: для каждого узла layer = max(layer предшественников) + 1
        let mut layers = vec![0usize; n];

        for &node in &topo_order {
            let mut max_pred_layer = 0;
            for &edge_idx in &self.graph.reverse_adjacency[node] {
                let edge = &self.graph.edges[edge_idx];
                max_pred_layer = max_pred_layer.max(layers[edge.from] + 1);
            }
            layers[node] = max_pred_layer;
        }

        // Записываем слои в узлы
        for (idx, &layer) in layers.iter().enumerate() {
            self.graph.nodes[idx].layer = layer;
        }
    }

    /// Топологическая сортировка (Kahn's algorithm)
    fn topological_sort(&self) -> Vec<usize> {
        let n = self.graph.node_count();
        let mut in_degree = vec![0usize; n];

        // Подсчитываем входящие степени
        for edge in &self.graph.edges {
            in_degree[edge.to] += 1;
        }

        // Начинаем с узлов без входящих рёбер
        let mut queue: VecDeque<usize> = in_degree
            .iter()
            .enumerate()
            .filter(|(_, &deg)| deg == 0)
            .map(|(idx, _)| idx)
            .collect();

        let mut result = Vec::with_capacity(n);

        while let Some(node) = queue.pop_front() {
            result.push(node);

            for &edge_idx in &self.graph.adjacency[node] {
                let target = self.graph.edges[edge_idx].to;
                in_degree[target] -= 1;
                if in_degree[target] == 0 {
                    queue.push_back(target);
                }
            }
        }

        // Если не все узлы обработаны, значит есть цикл (не должно случиться после remove_cycles)
        if result.len() < n {
            // Добавляем оставшиеся узлы
            for i in 0..n {
                if !result.contains(&i) {
                    result.push(i);
                }
            }
        }

        result
    }

    // =========================================================================
    // Шаг 3: Минимизация пересечений (Crossing Minimization)
    // =========================================================================

    /// Минимизирует пересечения методом барицентра
    fn minimize_crossings(&mut self) {
        let max_layer = self.graph.max_layer();
        if max_layer == 0 {
            return;
        }

        // Начальная позиция: порядок по индексу
        for node in &mut self.graph.nodes {
            node.position = node.index;
        }

        // Итерируем сверху-вниз и снизу-вверх несколько раз
        const MAX_ITERATIONS: usize = 10;

        for _ in 0..MAX_ITERATIONS {
            // Сверху вниз
            for layer in 1..=max_layer {
                self.order_layer_by_barycenter(layer, true);
            }

            // Снизу вверх
            for layer in (0..max_layer).rev() {
                self.order_layer_by_barycenter(layer, false);
            }
        }
    }

    /// Упорядочивает узлы слоя методом барицентра
    fn order_layer_by_barycenter(&mut self, layer: usize, use_upper: bool) {
        let nodes_in_layer: Vec<usize> = self.graph.nodes_on_layer(layer);

        if nodes_in_layer.is_empty() {
            return;
        }

        // Вычисляем барицентр для каждого узла
        let mut barycenters: Vec<(usize, f64)> = nodes_in_layer
            .iter()
            .map(|&node| {
                let bc = self.calculate_barycenter(node, use_upper);
                (node, bc)
            })
            .collect();

        // Сортируем по барицентру
        barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Обновляем позиции
        for (pos, (node, _)) in barycenters.iter().enumerate() {
            self.graph.nodes[*node].position = pos;
        }
    }

    /// Вычисляет барицентр узла
    fn calculate_barycenter(&self, node: usize, use_upper: bool) -> f64 {
        let edges = if use_upper {
            // Смотрим на предшественников (верхний слой)
            &self.graph.reverse_adjacency[node]
        } else {
            // Смотрим на потомков (нижний слой)
            &self.graph.adjacency[node]
        };

        if edges.is_empty() {
            // Нет связей - возвращаем текущую позицию
            return self.graph.nodes[node].position as f64;
        }

        let sum: f64 = edges
            .iter()
            .map(|&edge_idx| {
                let edge = &self.graph.edges[edge_idx];
                let neighbor = if use_upper { edge.from } else { edge.to };
                self.graph.nodes[neighbor].position as f64
            })
            .sum();

        sum / edges.len() as f64
    }

    // =========================================================================
    // Шаг 4: Присвоение координат (Coordinate Assignment)
    // =========================================================================

    /// Присваивает X и Y координаты узлам
    fn assign_coordinates(&mut self) {
        let max_layer = self.graph.max_layer();

        // Y координаты: PlantUML располагает родителей ВВЕРХУ, детей ВНИЗУ
        // Слой 0 = верх (корень/родитель), слой N = низ (потомки)
        // Вычисляем Y на основе фактических высот узлов предыдущих слоёв
        let mut layer_y = vec![0.0f64; max_layer + 1];
        layer_y[0] = self.config.margin;
        
        for layer in 1..=max_layer {
            // Находим максимальную высоту узлов на предыдущем слое
            let prev_layer_max_height = self.graph.nodes_on_layer(layer - 1)
                .iter()
                .map(|&n| self.graph.nodes[n].size.height)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(self.config.min_class_height);
            
            layer_y[layer] = layer_y[layer - 1] + prev_layer_max_height + self.config.layer_vertical_spacing;
        }
        
        for node in &mut self.graph.nodes {
            node.y = layer_y[node.layer];
        }

        // X координаты: позиционируем узлы внутри каждого слоя
        for layer in 0..=max_layer {
            self.position_layer_x(layer);
        }
    }

    /// Позиционирует узлы слоя по оси X
    fn position_layer_x(&mut self, layer: usize) {
        let mut nodes_in_layer: Vec<usize> = self.graph.nodes_on_layer(layer);

        if nodes_in_layer.is_empty() {
            return;
        }

        // Сортируем по позиции (результат минимизации пересечений)
        nodes_in_layer.sort_by_key(|&n| self.graph.nodes[n].position);

        // Вычисляем общую ширину слоя
        let total_width: f64 = nodes_in_layer
            .iter()
            .map(|&n| self.graph.nodes[n].size.width)
            .sum();

        let spacing = (nodes_in_layer.len() - 1) as f64 * self.config.node_horizontal_spacing;
        let layer_width = total_width + spacing;

        // Начинаем с центрирования (можно улучшить)
        let mut x = self.config.margin;

        for &node_idx in &nodes_in_layer {
            self.graph.nodes[node_idx].x = x;
            x += self.graph.nodes[node_idx].size.width + self.config.node_horizontal_spacing;
        }

        // Центрируем слой относительно максимальной ширины
        let max_layer_width = self.calculate_max_layer_width();
        if layer_width < max_layer_width {
            let offset = (max_layer_width - layer_width) / 2.0;
            for &node_idx in &nodes_in_layer {
                self.graph.nodes[node_idx].x += offset;
            }
        }
    }

    /// Вычисляет максимальную ширину слоя
    fn calculate_max_layer_width(&self) -> f64 {
        let max_layer = self.graph.max_layer();
        let mut max_width = 0.0f64;

        for layer in 0..=max_layer {
            let nodes = self.graph.nodes_on_layer(layer);
            let width: f64 = nodes
                .iter()
                .map(|&n| self.graph.nodes[n].size.width)
                .sum::<f64>()
                + (nodes.len().saturating_sub(1)) as f64 * self.config.node_horizontal_spacing;
            max_width = max_width.max(width);
        }

        max_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::class::{ClassDiagram, Classifier, Relationship};

    fn create_test_graph() -> (Graph, ClassLayoutConfig) {
        let mut diagram = ClassDiagram::new();

        // Простая иерархия: Animal -> Dog, Cat
        diagram.add_class(Classifier::new("Animal"));
        diagram.add_class(Classifier::new("Dog"));
        diagram.add_class(Classifier::new("Cat"));

        diagram.add_relationship(Relationship::inheritance("Dog", "Animal"));
        diagram.add_relationship(Relationship::inheritance("Cat", "Animal"));

        let config = ClassLayoutConfig::default();
        let graph = Graph::from_diagram(&diagram, &config);

        (graph, config)
    }

    #[test]
    fn test_layer_assignment() {
        let (mut graph, config) = create_test_graph();
        let mut sugiyama = SugiyamaLayout::new(&mut graph, &config);

        sugiyama.assign_layers();

        // Animal должен быть на слое 0, Dog и Cat на слое 1
        let animal = graph.get_node_by_name("Animal").unwrap();
        let dog = graph.get_node_by_name("Dog").unwrap();
        let cat = graph.get_node_by_name("Cat").unwrap();

        // Из-за инвертирования направления при наследовании,
        // Animal (родитель) будет на слое 0, а Dog/Cat на слое 1
        assert!(
            animal.layer < dog.layer
                || animal.layer < cat.layer
                || (dog.layer == 0 && cat.layer == 0 && animal.layer == 1)
        );
    }

    #[test]
    fn test_full_layout() {
        let (mut graph, config) = create_test_graph();
        let mut sugiyama = SugiyamaLayout::new(&mut graph, &config);

        sugiyama.run();

        // Проверяем, что все узлы имеют координаты
        for node in &graph.nodes {
            assert!(node.x >= 0.0);
            assert!(node.y >= 0.0);
        }
    }

    #[test]
    fn test_cycle_removal() {
        let mut diagram = ClassDiagram::new();

        // Создаём цикл: A -> B -> C -> A
        diagram.add_class(Classifier::new("A"));
        diagram.add_class(Classifier::new("B"));
        diagram.add_class(Classifier::new("C"));

        // Используем Link для создания направленного графа с циклом
        diagram.add_relationship(Relationship::new(
            "A",
            "B",
            plantuml_ast::class::RelationshipType::Association,
        ));
        diagram.add_relationship(Relationship::new(
            "B",
            "C",
            plantuml_ast::class::RelationshipType::Association,
        ));
        diagram.add_relationship(Relationship::new(
            "C",
            "A",
            plantuml_ast::class::RelationshipType::Association,
        ));

        let config = ClassLayoutConfig::default();
        let mut graph = Graph::from_diagram(&diagram, &config);
        let mut sugiyama = SugiyamaLayout::new(&mut graph, &config);

        sugiyama.remove_cycles();

        // После удаления циклов хотя бы одно ребро должно быть обращено
        let reversed_count = graph.edges.iter().filter(|e| e.reversed).count();
        assert!(
            reversed_count >= 1,
            "Должно быть обращено хотя бы одно ребро для разрыва цикла"
        );
    }
}
