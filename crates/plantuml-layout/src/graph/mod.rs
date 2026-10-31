//! Послойная раскладка графовых диаграмм по этапам Graphviz dot.
//!
//! 1. Убираем циклы, разворачивая обратные рёбра.
//! 2. Назначаем ранги сетевым симплексом ([`simplex`]).
//! 3. Длинные рёбра заменяем цепочками фиктивных узлов.
//! 4. Упорядочиваем узлы внутри рангов: медиана и перестановки соседних ([`order`]).
//! 5. Считаем координаты вторым сетевым симплексом, выпрямляющим рёбра.
//! 6. Строим рамки кластеров и ломаные рёбер.
//!
//! Модуль ничего не знает о типах диаграмм: движок описывает узлы размерами, а
//! результат переводит в свои элементы. Рекурсии нет: глубина приходит из схемы.

pub mod order;
pub mod scene;
pub mod simplex;

use simplex::{solve, Constraint};

/// Направление раскладки
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    TopToBottom,
    LeftToRight,
}

/// Узел: размер в исходной ориентации
#[derive(Debug, Clone, Default)]
pub struct GraphNode {
    pub width: f64,
    pub height: f64,
    /// Кластер, в котором лежит узел
    pub cluster: Option<usize>,
}

/// Ребро между узлами
#[derive(Debug, Clone, Default)]
pub struct GraphEdge {
    pub from: usize,
    pub to: usize,
    /// Размер подписи: она получает своё место, как в dot
    pub label: Option<(f64, f64)>,
    /// Минимальная разница рангов; 0 — узлы могут стоять на одном ранге
    pub min_rank: i64,
    /// Концы на одном ранге: `A -right-> B`, заметка сбоку
    pub same_rank: bool,
    pub weight: i64,
    /// Начало связи — рамка кластера, а не узел: все её узлы выше конца
    pub from_cluster: Option<usize>,
    /// Конец связи — рамка кластера: все её узлы ниже начала
    pub to_cluster: Option<usize>,
    /// Выводить ломаную от `to` к `from`: `A -up-> B` раскладывается как `B` над `A`, но идёт от `A`
    pub reverse_path: bool,
}

impl GraphEdge {
    /// Ребро по записи стрелки PlantUML: длина линии и явное направление.
    pub fn from_arrow(from: usize, to: usize, arrow: plantuml_ast::common::ArrowShape) -> Self {
        use plantuml_ast::common::ArrowHint;
        let span = (i64::from(arrow.length) - 1).max(1);
        let (from_node, to_node, same_rank, reverse_path) = match arrow.hint {
            Some(ArrowHint::Up) => (to, from, false, true),
            Some(ArrowHint::Down) => (from, to, false, false),
            Some(ArrowHint::Right) => (from, to, true, false),
            Some(ArrowHint::Left) => (to, from, true, true),
            None => (from, to, arrow.length <= 1, false),
        };
        Self {
            from: from_node,
            to: to_node,
            label: None,
            min_rank: if same_rank { 0 } else { span },
            same_rank,
            weight: 1,
            reverse_path,
            from_cluster: None,
            to_cluster: None,
        }
    }
}

/// Кластер: пакет, составное состояние
#[derive(Debug, Clone, Default)]
pub struct GraphCluster {
    pub parent: Option<usize>,
    /// Высота заголовка над содержимым
    pub title_height: f64,
    /// Минимальная ширина: заголовок должен помещаться
    pub min_width: f64,
}

/// Вход раскладки
#[derive(Debug, Clone, Default)]
pub struct GraphInput {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub clusters: Vec<GraphCluster>,
    pub direction: Direction,
    /// Зазор между узлами одного ранга
    pub node_sep: f64,
    /// Зазор между рангами
    pub rank_sep: f64,
    /// Поле кластера вокруг содержимого
    pub cluster_margin: f64,
}

/// Точка результата
pub type Point = (f64, f64);

/// Прямоугольник результата: левый верхний угол и размеры
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Boxed {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Результат раскладки. Индексы совпадают с индексами входа.
#[derive(Debug, Clone, Default)]
pub struct GraphLayout {
    pub nodes: Vec<Boxed>,
    pub clusters: Vec<Boxed>,
    /// Ломаная ребра от начала к концу
    pub edges: Vec<Vec<Point>>,
    /// Центр подписи ребра
    pub labels: Vec<Option<Point>>,
    pub width: f64,
    pub height: f64,
}

/// Зазор между горизонтальной линией и подписью над ней.
const LABEL_GAP: f64 = 2.0;

/// Вершина слоя: настоящий узел или фиктивный на длинном ребре.
#[derive(Debug, Clone)]
struct Vertex {
    width: f64,
    height: f64,
    /// Самый глубокий кластер, которому принадлежит вершина
    cluster: Option<usize>,
    is_real: bool,
    /// Место под подпись сбоку от линии: положительное справа, отрицательное слева.
    /// Как в dot: линия остаётся прямой, а подпись стоит рядом, а не на ней.
    label_side: f64,
}

impl Vertex {
    fn left(&self) -> f64 {
        self.width / 2.0 + (-self.label_side).max(0.0)
    }

    fn right(&self) -> f64 {
        self.width / 2.0 + self.label_side.max(0.0)
    }
}

/// Участок ребра между соседними рангами.
#[derive(Debug, Clone, Copy)]
struct Segment {
    upper: usize,
    lower: usize,
}

/// Выполняет раскладку.
pub fn layout(input: &GraphInput) -> GraphLayout {
    if input.nodes.is_empty() {
        return GraphLayout::default();
    }
    let horizontal = input.direction == Direction::LeftToRight;
    let swap = |w: f64, h: f64| if horizontal { (h, w) } else { (w, h) };

    let flipped = find_back_edges(input);
    let ranks = assign_ranks(input, &flipped);

    // Подписи занимают отдельный ранг, поэтому настоящие ранги удваиваются.
    let mut vertices: Vec<Vertex> = input
        .nodes
        .iter()
        .map(|n| {
            let (width, height) = swap(n.width, n.height);
            Vertex {
                width,
                height,
                cluster: n.cluster,
                is_real: true,
                label_side: 0.0,
            }
        })
        .collect();
    let mut vertex_rank: Vec<i64> = ranks.iter().map(|r| r * 2).collect();
    let mut segments: Vec<Segment> = Vec::new();
    let mut chains: Vec<Vec<usize>> = vec![Vec::new(); input.edges.len()];
    let mut label_vertex: Vec<Option<usize>> = vec![None; input.edges.len()];

    let endpoints = choose_endpoints(input, &flipped, &ranks);
    for (index, edge) in input.edges.iter().enumerate() {
        if edge.from == edge.to || edge.from >= input.nodes.len() || edge.to >= input.nodes.len() {
            continue;
        }
        let (upper, lower) = endpoints[index];
        let (ru, rl) = (vertex_rank[upper], vertex_rank[lower]);
        if ru >= rl {
            continue; // одинаковый ранг: ребро идёт по прямой, без цепочки
        }
        let shared = common_cluster(
            input,
            input.nodes[upper].cluster,
            input.nodes[lower].cluster,
        );
        let span = rl - ru;
        let label_at = ru + span / 2;
        let mut previous = upper;
        for rank in (ru + 1)..rl {
            let (label_w, label_h) = if rank == label_at {
                edge.label.map(|(w, h)| swap(w, h)).unwrap_or((0.0, 0.0))
            } else {
                (0.0, 0.0)
            };
            let id = vertices.len();
            vertices.push(Vertex {
                width: 0.0,
                height: label_h,
                cluster: shared,
                is_real: false,
                // После поворота раскладки «слева» становится «над линией».
                label_side: if horizontal {
                    -(label_w + LABEL_GAP)
                } else {
                    label_w
                },
            });
            vertex_rank.push(rank);
            if rank == label_at && edge.label.is_some() {
                label_vertex[index] = Some(id);
            }
            segments.push(Segment {
                upper: previous,
                lower: id,
            });
            chains[index].push(id);
            previous = id;
        }
        segments.push(Segment {
            upper: previous,
            lower,
        });
    }

    let rank_count = vertex_rank.iter().copied().max().unwrap_or(0) as usize + 1;
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); rank_count];
    for (id, rank) in vertex_rank.iter().enumerate() {
        rows[*rank as usize].push(id);
    }

    let pairs: Vec<(usize, usize)> = segments.iter().map(|s| (s.upper, s.lower)).collect();
    let paths = cluster_paths(input, &vertices);
    order::minimize_crossings(&mut rows, &pairs, &paths, vertices.len());
    apply_same_rank_order(input, &mut rows);

    let positions = assign_positions(input, &vertices, &rows, &segments, &paths);
    let along = assign_rows(input, &vertices, &rows);

    finish(
        input,
        &vertices,
        &positions,
        &along,
        &vertex_rank,
        &chains,
        &label_vertex,
        &flipped,
        &endpoints,
    )
}

/// Концы рёбер сверху вниз. Связь с рамкой идёт к её крайнему по рангу узлу.
fn choose_endpoints(input: &GraphInput, flipped: &[bool], ranks: &[i64]) -> Vec<(usize, usize)> {
    input
        .edges
        .iter()
        .enumerate()
        .map(|(index, edge)| {
            let (upper, lower, upper_cluster, lower_cluster) = if flipped[index] {
                (edge.to, edge.from, edge.to_cluster, edge.from_cluster)
            } else {
                (edge.from, edge.to, edge.from_cluster, edge.to_cluster)
            };
            let upper = upper_cluster
                .and_then(|c| {
                    members_of(input, c)
                        .into_iter()
                        .max_by_key(|m| ranks.get(*m).copied().unwrap_or(0))
                })
                .unwrap_or(upper);
            let lower = lower_cluster
                .and_then(|c| {
                    members_of(input, c)
                        .into_iter()
                        .min_by_key(|m| ranks.get(*m).copied().unwrap_or(0))
                })
                .unwrap_or(lower);
            (upper, lower)
        })
        .collect()
}

/// Рёбра, которые надо развернуть, чтобы граф стал ациклическим (обход без рекурсии).
fn find_back_edges(input: &GraphInput) -> Vec<bool> {
    let n = input.nodes.len();
    let mut flipped = vec![false; input.edges.len()];
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (index, edge) in input.edges.iter().enumerate() {
        if edge.from < n && edge.to < n && edge.from != edge.to && !edge.same_rank {
            out[edge.from].push(index);
        }
    }
    // 0 — не посещён, 1 — в обходе, 2 — завершён
    let mut state = vec![0u8; n];
    for start in 0..n {
        if state[start] != 0 {
            continue;
        }
        state[start] = 1;
        let mut stack = vec![(start, 0usize)];
        while let Some(top) = stack.last_mut() {
            let (v, cursor) = *top;
            if cursor < out[v].len() {
                top.1 += 1;
                let index = out[v][cursor];
                let to = input.edges[index].to;
                match state[to] {
                    0 => {
                        state[to] = 1;
                        stack.push((to, 0));
                    }
                    1 => flipped[index] = true,
                    _ => {}
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
    flipped
}

/// Ранги: узлы одного ранга склеиваются, потом решается симплекс.
fn assign_ranks(input: &GraphInput, flipped: &[bool]) -> Vec<i64> {
    let n = input.nodes.len();

    // Система непересекающихся множеств для `same_rank`.
    let mut group: Vec<usize> = (0..n).collect();
    fn find(group: &mut [usize], mut v: usize) -> usize {
        while group[v] != v {
            group[v] = group[group[v]];
            v = group[v];
        }
        v
    }
    for edge in &input.edges {
        if edge.same_rank && edge.from < n && edge.to < n {
            let (a, b) = (find(&mut group, edge.from), find(&mut group, edge.to));
            group[a] = b;
        }
    }
    let leaders: Vec<usize> = (0..n).map(|v| find(&mut group, v)).collect();

    // Склейка может вернуть цикл: обратные дуги снимаем повторной проверкой.
    let mut constraints: Vec<Constraint> = Vec::new();
    for (index, edge) in input.edges.iter().enumerate() {
        if edge.same_rank || edge.from >= n || edge.to >= n || edge.from == edge.to {
            continue;
        }
        let (from, to) = if flipped[index] {
            (edge.to, edge.from)
        } else {
            (edge.from, edge.to)
        };
        let (from, to) = (leaders[from], leaders[to]);
        if from != to {
            constraints.push(Constraint {
                from,
                to,
                min_len: edge.min_rank.max(0),
                weight: edge.weight.max(1),
            });
        }
        // Связь с рамкой обязана обойти все её узлы, иначе рамка накроет другой конец.
        let (upper_cluster, lower_cluster) = if flipped[index] {
            (edge.to_cluster, edge.from_cluster)
        } else {
            (edge.from_cluster, edge.to_cluster)
        };
        if let Some(cluster) = upper_cluster {
            for member in members_of(input, cluster) {
                if leaders[member] != to {
                    constraints.push(Constraint {
                        from: leaders[member],
                        to,
                        min_len: edge.min_rank.max(0),
                        weight: 0,
                    });
                }
            }
        }
        if let Some(cluster) = lower_cluster {
            for member in members_of(input, cluster) {
                if leaders[member] != from {
                    constraints.push(Constraint {
                        from,
                        to: leaders[member],
                        min_len: edge.min_rank.max(0),
                        weight: 0,
                    });
                }
            }
        }
    }
    let constraints = drop_cycles(n, constraints);
    let solved = solve(n, &constraints);
    (0..n).map(|v| solved[leaders[v]]).collect()
}

/// Настоящие узлы кластера вместе с вложенными.
fn members_of(input: &GraphInput, cluster: usize) -> Vec<usize> {
    let inside = |mut current: Option<usize>| {
        let mut guard = 0;
        while let Some(c) = current {
            if c == cluster {
                return true;
            }
            current = input.clusters.get(c).and_then(|cl| cl.parent);
            guard += 1;
            if guard > input.clusters.len() {
                break;
            }
        }
        false
    };
    input
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| inside(node.cluster))
        .map(|(index, _)| index)
        .collect()
}

/// Убирает дуги, замыкающие цикл после склейки узлов одного ранга.
fn drop_cycles(n: usize, constraints: Vec<Constraint>) -> Vec<Constraint> {
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (index, c) in constraints.iter().enumerate() {
        out[c.from].push(index);
    }
    let mut state = vec![0u8; n];
    let mut keep = vec![true; constraints.len()];
    for start in 0..n {
        if state[start] != 0 {
            continue;
        }
        state[start] = 1;
        let mut stack = vec![(start, 0usize)];
        while let Some(top) = stack.last_mut() {
            let (v, cursor) = *top;
            if cursor < out[v].len() {
                top.1 += 1;
                let index = out[v][cursor];
                let to = constraints[index].to;
                match state[to] {
                    0 => {
                        state[to] = 1;
                        stack.push((to, 0));
                    }
                    1 => keep[index] = false,
                    _ => {}
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
    constraints
        .into_iter()
        .zip(keep)
        .filter_map(|(c, k)| k.then_some(c))
        .collect()
}

/// Путь кластеров от корня к вершине.
fn cluster_paths(input: &GraphInput, vertices: &[Vertex]) -> Vec<Vec<usize>> {
    vertices
        .iter()
        .map(|v| {
            let mut path = Vec::new();
            let mut current = v.cluster;
            // Предел защищает от цикла родителей в некорректном входе.
            while let Some(c) = current.filter(|_| path.len() <= input.clusters.len()) {
                path.push(c);
                current = input.clusters.get(c).and_then(|cluster| cluster.parent);
            }
            path.reverse();
            path
        })
        .collect()
}

/// Самый глубокий кластер, содержащий оба.
fn common_cluster(input: &GraphInput, a: Option<usize>, b: Option<usize>) -> Option<usize> {
    let chain = |mut c: Option<usize>| {
        let mut path = Vec::new();
        while let Some(id) = c.filter(|_| path.len() <= input.clusters.len()) {
            path.push(id);
            c = input.clusters.get(id).and_then(|cluster| cluster.parent);
        }
        path
    };
    let (pa, pb) = (chain(a), chain(b));
    pa.into_iter().find(|c| pb.contains(c))
}

/// Для `A -right-> B` начало стоит левее конца.
fn apply_same_rank_order(input: &GraphInput, rows: &mut [Vec<usize>]) {
    for edge in input.edges.iter().filter(|e| e.same_rank) {
        for row in rows.iter_mut() {
            let from = row.iter().position(|v| *v == edge.from);
            let to = row.iter().position(|v| *v == edge.to);
            if let (Some(a), Some(b)) = (from, to) {
                if a > b {
                    let moved = row.remove(a);
                    row.insert(b, moved);
                }
            }
        }
    }
}

/// Вес выпрямления ребра: длинные цепочки держим прямее, как в dot.
fn straighten_weight(upper_real: bool, lower_real: bool) -> i64 {
    match (upper_real, lower_real) {
        (true, true) => 1,
        (false, false) => 8,
        _ => 2,
    }
}

/// Сколько рамок кластеров разделяет соседей в ряду.
fn boundaries_between(a: &[usize], b: &[usize]) -> usize {
    let common = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    (a.len() - common) + (b.len() - common)
}

/// Координаты поперёк рангов и границы кластеров.
struct Positions {
    x: Vec<f64>,
    /// Левая и правая граница кластера поперёк рангов
    clusters: Vec<Option<(f64, f64)>>,
}

/// Координаты поперёк рангов: второй симплекс, как в dot.
///
/// У кластера две служебные вершины: левая и правая граница. Его участники лежат
/// между ними, а соседи ряда отталкиваются от границ. Поэтому рамка, охватывающая
/// несколько рангов, не накрывает чужой узел.
fn assign_positions(
    input: &GraphInput,
    vertices: &[Vertex],
    rows: &[Vec<usize>],
    segments: &[Segment],
    paths: &[Vec<usize>],
) -> Positions {
    let n = vertices.len();
    let horizontal = input.direction == Direction::LeftToRight;
    let cluster_count = input.clusters.len();
    let left_of = |c: usize| n + c * 2;
    let right_of = |c: usize| n + c * 2 + 1;
    let mut used = vec![false; cluster_count];
    let mut constraints: Vec<Constraint> = Vec::new();
    let fixed = |from: usize, to: usize, gap: f64| Constraint {
        from,
        to,
        min_len: gap.ceil() as i64,
        weight: 0,
    };

    for (v, path) in paths.iter().enumerate() {
        let (left, right) = (vertices[v].left(), vertices[v].right());
        for &c in path {
            used[c] = true;
            // В горизонтальной раскладке заголовок лежит поперёк рангов.
            let title = if horizontal {
                input.clusters[c].title_height
            } else {
                0.0
            };
            constraints.push(fixed(left_of(c), v, left + input.cluster_margin + title));
            constraints.push(fixed(v, right_of(c), right + input.cluster_margin));
        }
    }
    for (c, cluster) in input.clusters.iter().enumerate() {
        if !used[c] {
            continue;
        }
        let min_width = if horizontal { 0.0 } else { cluster.min_width };
        // Вес стягивает рамку; без него ширина не входила бы в оптимум.
        constraints.push(Constraint {
            from: left_of(c),
            to: right_of(c),
            min_len: min_width.ceil() as i64,
            weight: 1,
        });
        if let Some(parent) = cluster.parent.filter(|p| *p < cluster_count) {
            constraints.push(fixed(left_of(parent), left_of(c), input.cluster_margin));
            constraints.push(fixed(right_of(c), right_of(parent), input.cluster_margin));
        }
    }

    for row in rows {
        for pair in row.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let common = paths[a]
                .iter()
                .zip(&paths[b])
                .take_while(|(x, y)| x == y)
                .count();
            // Соседями в ряду оказываются внешние кластеры, расходящиеся на этом уровне.
            let (from, a_half) = match paths[a].get(common) {
                Some(c) => (right_of(*c), 0.0),
                None => (a, vertices[a].right()),
            };
            let (to, b_half) = match paths[b].get(common) {
                Some(c) => (left_of(*c), 0.0),
                None => (b, vertices[b].left()),
            };
            constraints.push(fixed(from, to, a_half + b_half + input.node_sep));
        }
    }

    // Рёбра одного ранга держат порядок и оставляют место подписи.
    for edge in input
        .edges
        .iter()
        .filter(|e| e.same_rank && e.from < n && e.to < n)
    {
        let label = edge
            .label
            .map_or(0.0, |(w, h)| if horizontal { h } else { w });
        let gap =
            (vertices[edge.from].width + vertices[edge.to].width) / 2.0 + input.node_sep + label;
        constraints.push(fixed(edge.from, edge.to, gap));
    }

    // Порядок кластеров в разных рядах может расходиться; такие дуги дают цикл.
    let base = n + cluster_count * 2;
    let mut constraints = drop_cycles(base, constraints);

    // Вспомогательная вершина на каждое ребро: штраф за |x(верх) - x(низ)|.
    let mut total = base;
    for segment in segments {
        let aux = total;
        total += 1;
        let weight = straighten_weight(
            vertices[segment.upper].is_real,
            vertices[segment.lower].is_real,
        );
        for target in [segment.upper, segment.lower] {
            constraints.push(Constraint {
                from: aux,
                to: target,
                min_len: 0,
                weight,
            });
        }
    }

    let solved = solve(total, &constraints);
    let mut x: Vec<f64> = solved[..n].iter().map(|v| *v as f64).collect();
    let clusters: Vec<Option<(f64, f64)>> = (0..cluster_count)
        .map(|c| used[c].then(|| (solved[left_of(c)] as f64, solved[right_of(c)] as f64)))
        .collect();
    balance(&mut x, input, vertices, rows, segments, paths, &clusters);
    Positions { x, clusters }
}

/// Выравнивает вершины, у которых оптимум симплекса не единственный.
///
/// Для родителя двух детей любое положение между ними стоит одинаково, и симплекс
/// прижимает его к одному из них. Dot для этого делает отдельный шаг: ставит такую
/// вершину посередине, если это не нарушает зазоры и не удлиняет рёбра.
fn balance(
    x: &mut [f64],
    input: &GraphInput,
    vertices: &[Vertex],
    rows: &[Vec<usize>],
    segments: &[Segment],
    paths: &[Vec<usize>],
    frames: &[Option<(f64, f64)>],
) {
    let horizontal = input.direction == Direction::LeftToRight;
    let mut ups = vec![0_usize; x.len()];
    let mut downs = vec![0_usize; x.len()];
    let mut neighbours: Vec<Vec<(usize, usize)>> = vec![Vec::new(); x.len()];
    for segment in segments {
        let (upper, lower) = (segment.upper, segment.lower);
        downs[upper] += 1;
        ups[lower] += 1;
        let weight = straighten_weight(vertices[upper].is_real, vertices[lower].is_real) as usize;
        neighbours[upper].push((lower, weight));
        neighbours[lower].push((upper, weight));
    }
    // Столбец: вершины на одной линии, связанные единственными участками. Двигать надо
    // и его целиком: перекладина fork стоит под предком, по отдельности любой её сдвиг
    // хуже, а вместе с предком сдвиг в середину между детьми ничего не стоит.
    let mut parent: Vec<usize> = (0..x.len()).collect();
    fn find(parent: &mut [usize], mut v: usize) -> usize {
        while parent[v] != v {
            parent[v] = parent[parent[v]];
            v = parent[v];
        }
        v
    }
    for segment in segments {
        let (upper, lower) = (segment.upper, segment.lower);
        if downs[upper] == 1 && ups[lower] == 1 && (x[upper] - x[lower]).abs() < 0.5 {
            let (a, b) = (find(&mut parent, upper), find(&mut parent, lower));
            parent[b] = a;
        }
    }
    let mut columns: Vec<Vec<usize>> = vec![Vec::new(); x.len()];
    for v in 0..x.len() {
        let root = find(&mut parent, v);
        columns[root].push(v);
    }
    let mut groups: Vec<Vec<usize>> = columns.into_iter().filter(|c| c.len() > 1).collect();
    groups.extend((0..x.len()).map(|v| vec![v]));
    let mut inside = vec![usize::MAX; x.len()];
    let mut place = vec![(0_usize, 0_usize); x.len()];
    for (r, row) in rows.iter().enumerate() {
        for (index, &v) in row.iter().enumerate() {
            place[v] = (r, index);
        }
    }
    let gap = |a: usize, b: usize| {
        vertices[a].right()
            + vertices[b].left()
            + input.node_sep
            + boundaries_between(&paths[a], &paths[b]) as f64 * input.cluster_margin
    };
    for _ in 0..10 {
        for (group_index, group) in groups.iter().enumerate() {
            for &v in group {
                inside[v] = group_index;
            }
            // Цена сдвига кусочно-линейна. Вес повторяем: при чётной сумме весов минимум
            // занимает целый отрезок, и его середина раскладку не ухудшает.
            let mut offsets: Vec<f64> = Vec::new();
            for &v in group {
                for &(u, weight) in &neighbours[v] {
                    if inside[u] != group_index {
                        offsets.extend(std::iter::repeat_n(x[u] - x[v], weight));
                    }
                }
            }
            if offsets.is_empty() || offsets.len() % 2 == 1 {
                continue;
            }
            offsets.sort_by(f64::total_cmp);
            let middle = offsets.len() / 2;
            if offsets[middle] - offsets[middle - 1] < 0.5 {
                continue;
            }
            let target = (offsets[middle - 1] + offsets[middle]) / 2.0;

            let mut low = f64::MIN;
            let mut high = f64::MAX;
            for &v in group {
                let (r, index) = place[v];
                if let Some(&left) = index.checked_sub(1).and_then(|i| rows[r].get(i)) {
                    low = low.max(x[left] + gap(left, v) - x[v]);
                }
                if let Some(&right) = rows[r].get(index + 1) {
                    high = high.min(x[right] - gap(v, right) - x[v]);
                }
                for &c in &paths[v] {
                    let Some((left, right)) = frames.get(c).copied().flatten() else {
                        continue;
                    };
                    let title = if horizontal {
                        input.clusters[c].title_height
                    } else {
                        0.0
                    };
                    let margin = input.cluster_margin;
                    low = low.max(left + vertices[v].left() + margin + title - x[v]);
                    high = high.min(right - vertices[v].right() - margin - x[v]);
                }
            }
            if low <= high {
                let shift = target.clamp(low, high);
                for &v in group {
                    x[v] += shift;
                }
            }
        }
    }
}

/// Координаты вдоль рангов: высота ряда и зазоры с учётом рамок кластеров.
fn assign_rows(input: &GraphInput, vertices: &[Vertex], rows: &[Vec<usize>]) -> Vec<f64> {
    let mut along = vec![0.0; vertices.len()];
    let mut cursor = 0.0;
    let paths = cluster_paths(input, vertices);

    for (index, row) in rows.iter().enumerate() {
        let height = row.iter().map(|v| vertices[*v].height).fold(0.0, f64::max);

        // Перед рядом резервируем заголовки кластеров, которые на нём начинаются.
        let previous: Vec<usize> = index
            .checked_sub(1)
            .and_then(|i| rows.get(i))
            .map(|r| r.iter().flat_map(|v| paths[*v].iter().copied()).collect())
            .unwrap_or_default();
        let opening = row
            .iter()
            .flat_map(|v| paths[*v].iter().copied())
            .filter(|c| !previous.contains(c))
            .collect::<std::collections::BTreeSet<usize>>();
        let extra: f64 = opening
            .iter()
            .map(|c| {
                input
                    .clusters
                    .get(*c)
                    .map_or(0.0, |cluster| cluster.title_height + input.cluster_margin)
            })
            .sum();
        cursor += extra;

        for v in row {
            along[*v] = cursor + height / 2.0;
        }
        cursor += height;
        // Удвоенные ранги: половина зазора на шаг.
        cursor += input.rank_sep / 2.0;
    }
    along
}

/// Итог: рамки, ломаные, сдвиг в ноль, поворот для горизонтальной раскладки.
#[allow(clippy::too_many_arguments)]
fn finish(
    input: &GraphInput,
    vertices: &[Vertex],
    positions: &Positions,
    along: &[f64],
    _ranks: &[i64],
    chains: &[Vec<usize>],
    label_vertex: &[Option<usize>],
    flipped: &[bool],
    endpoints: &[(usize, usize)],
) -> GraphLayout {
    let horizontal = input.direction == Direction::LeftToRight;

    let center = |v: usize| (positions.x[v], along[v]);
    let half = |v: usize| (vertices[v].width / 2.0, vertices[v].height / 2.0);

    // Рамки кластеров: снизу вверх по вложенности.
    let mut cluster_boxes: Vec<Option<(f64, f64, f64, f64)>> = vec![None; input.clusters.len()];
    let paths = cluster_paths(input, vertices);
    let grow = |slot: &mut Option<(f64, f64, f64, f64)>, b: (f64, f64, f64, f64)| {
        *slot = Some(match *slot {
            None => b,
            Some(a) => (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)),
        });
    };
    for v in 0..vertices.len() {
        if !vertices[v].is_real {
            continue;
        }
        let (cx, cy) = center(v);
        let (hw, hh) = half(v);
        for &cluster in &paths[v] {
            grow(
                &mut cluster_boxes[cluster],
                (cx - hw, cy - hh, cx + hw, cy + hh),
            );
        }
    }
    // Вложенные расширяют родителей; идём от глубоких к мелким.
    let depth = |mut c: usize| {
        let mut d = 0;
        while let Some(parent) = input.clusters.get(c).and_then(|cl| cl.parent) {
            d += 1;
            c = parent;
            if d > input.clusters.len() {
                break;
            }
        }
        d
    };
    let mut by_depth: Vec<usize> = (0..input.clusters.len()).collect();
    by_depth.sort_by_key(|c| std::cmp::Reverse(depth(*c)));
    let mut finished: Vec<Boxed> = vec![Boxed::default(); input.clusters.len()];
    for &c in &by_depth {
        let cluster = &input.clusters[c];
        let m = input.cluster_margin;
        let (l, t, r, b) = cluster_boxes[c].unwrap_or((0.0, 0.0, cluster.min_width, 0.0));
        let title_along = if horizontal {
            0.0
        } else {
            cluster.title_height
        };
        // Границы поперёк рангов посчитал симплекс; вдоль рангов берём содержимое.
        let (left, right) = positions.clusters[c].unwrap_or((l - m, r + m));
        let mut boxed = (left, t - m - title_along, right, b + m);
        if horizontal {
            let need = cluster.min_width - (boxed.3 - boxed.1);
            if need > 0.0 {
                boxed.1 -= need / 2.0;
                boxed.3 += need / 2.0;
            }
        }
        cluster_boxes[c] = Some(boxed);
        finished[c] = Boxed {
            x: boxed.0,
            y: boxed.1,
            width: boxed.2 - boxed.0,
            height: boxed.3 - boxed.1,
        };
        if let Some(parent) = cluster.parent {
            if parent < cluster_boxes.len() {
                grow(&mut cluster_boxes[parent], boxed);
            }
        }
    }

    let n_real = input.nodes.len();
    let mut nodes: Vec<Boxed> = (0..n_real)
        .map(|v| {
            let (cx, cy) = center(v);
            let (hw, hh) = half(v);
            Boxed {
                x: cx - hw,
                y: cy - hh,
                width: vertices[v].width,
                height: vertices[v].height,
            }
        })
        .collect();

    let mut edges: Vec<Vec<Point>> = Vec::with_capacity(input.edges.len());
    let mut labels: Vec<Option<Point>> = vec![None; input.edges.len()];
    for (index, edge) in input.edges.iter().enumerate() {
        if edge.from >= n_real || edge.to >= n_real {
            edges.push(Vec::new());
            continue;
        }
        if edge.from == edge.to {
            let b = nodes[edge.from];
            let right = b.x + b.width;
            let top = b.y + b.height * 0.25;
            let bottom = b.y + b.height * 0.75;
            let far = right + input.node_sep.max(20.0);
            edges.push(vec![
                (right, top),
                (far, top),
                (far, bottom),
                (right, bottom),
            ]);
            if edge.label.is_some() {
                labels[index] = Some((far + 4.0, b.y + b.height / 2.0));
            }
            continue;
        }
        let (upper, lower) = endpoints[index];
        let mut points: Vec<Point> = Vec::with_capacity(chains[index].len() + 2);
        points.push(center(upper));
        for v in &chains[index] {
            points.push(center(*v));
        }
        points.push(center(lower));
        if let Some(v) = label_vertex[index] {
            let (cx, cy) = center(v);
            let side = vertices[v].label_side;
            labels[index] = Some(if horizontal {
                (cx + (side + LABEL_GAP) / 2.0 - LABEL_GAP, cy)
            } else {
                (cx + side / 2.0, cy)
            });
        } else if let Some((_, label_h)) = edge.label {
            let (ax, ay) = center(upper);
            let (bx, by) = center(lower);
            // Концы на одном ранге: подпись над линией.
            let lift = if edge.same_rank {
                label_h / 2.0 + 2.0
            } else {
                0.0
            };
            labels[index] = Some(((ax + bx) / 2.0, (ay + by) / 2.0 - lift));
        }
        clip_ends(&mut points, nodes[upper], nodes[lower]);
        let (upper_cluster, lower_cluster) = if flipped[index] {
            (edge.to_cluster, edge.from_cluster)
        } else {
            (edge.from_cluster, edge.to_cluster)
        };
        if let Some(cluster) = upper_cluster.and_then(|c| finished.get(c)) {
            points.reverse();
            cut_at_box(&mut points, *cluster);
            points.reverse();
        }
        if let Some(cluster) = lower_cluster.and_then(|c| finished.get(c)) {
            cut_at_box(&mut points, *cluster);
        }
        if flipped[index] != edge.reverse_path {
            points.reverse();
        }
        edges.push(points);
    }

    let mut clusters = finished;

    // Сдвиг в ноль.
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut see = |x0: f64, y0: f64, x1: f64, y1: f64| {
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x1);
        max_y = max_y.max(y1);
    };
    for b in nodes.iter().chain(clusters.iter()) {
        see(b.x, b.y, b.x + b.width, b.y + b.height);
    }
    for path in &edges {
        for (x, y) in path {
            see(*x, *y, *x, *y);
        }
    }
    for (index, label) in labels.iter().enumerate() {
        if let (Some((x, y)), Some((w, h))) = (label, input.edges[index].label) {
            let (w, h) = if horizontal { (h, w) } else { (w, h) };
            see(x - w / 2.0, y - h / 2.0, x + w / 2.0, y + h / 2.0);
        }
    }

    let place = |x: f64, y: f64| (x - min_x, y - min_y);
    for b in nodes.iter_mut().chain(clusters.iter_mut()) {
        (b.x, b.y) = place(b.x, b.y);
    }
    for path in edges.iter_mut() {
        for p in path.iter_mut() {
            *p = place(p.0, p.1);
        }
    }
    for p in labels.iter_mut().flatten() {
        *p = place(p.0, p.1);
    }
    let mut width = max_x - min_x;
    let mut height = max_y - min_y;

    if horizontal {
        for b in nodes.iter_mut().chain(clusters.iter_mut()) {
            *b = Boxed {
                x: b.y,
                y: b.x,
                width: b.height,
                height: b.width,
            };
        }
        for path in edges.iter_mut() {
            for p in path.iter_mut() {
                *p = (p.1, p.0);
            }
        }
        for p in labels.iter_mut().flatten() {
            *p = (p.1, p.0);
        }
        std::mem::swap(&mut width, &mut height);
    }

    GraphLayout {
        nodes,
        clusters,
        edges,
        labels,
        width,
        height,
    }
}

/// Точка выхода из прямоугольника по лучу из центра.
fn border_point(b: Boxed, toward: Point) -> Point {
    let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
    let (dx, dy) = (toward.0 - cx, toward.1 - cy);
    if dx == 0.0 && dy == 0.0 {
        return (cx, cy);
    }
    let sx = if dx != 0.0 {
        (b.width / 2.0) / dx.abs()
    } else {
        f64::MAX
    };
    let sy = if dy != 0.0 {
        (b.height / 2.0) / dy.abs()
    } else {
        f64::MAX
    };
    let s = sx.min(sy);
    (cx + dx * s, cy + dy * s)
}

/// Обрывает ломаную на первом входе в прямоугольник: стрелка упирается в рамку.
fn cut_at_box(points: &mut Vec<Point>, b: Boxed) {
    let inside = |p: Point| p.0 > b.x && p.0 < b.x + b.width && p.1 > b.y && p.1 < b.y + b.height;
    let Some(first_inside) = points.iter().position(|p| inside(*p)) else {
        return;
    };
    if first_inside == 0 {
        return;
    }
    let (a, c) = (points[first_inside - 1], points[first_inside]);
    // Ищем пересечение с рамкой по параметру отрезка.
    let mut t: f64 = 1.0;
    let mut try_edge = |value: f64, from: f64, to: f64| {
        if (to - from).abs() > f64::EPSILON {
            let candidate = (value - from) / (to - from);
            if candidate > 0.0 && candidate < t {
                t = candidate;
            }
        }
    };
    try_edge(b.x, a.0, c.0);
    try_edge(b.x + b.width, a.0, c.0);
    try_edge(b.y, a.1, c.1);
    try_edge(b.y + b.height, a.1, c.1);
    let hit = (a.0 + (c.0 - a.0) * t, a.1 + (c.1 - a.1) * t);
    points.truncate(first_inside);
    points.push(hit);
}

/// Концы ломаной ставит на границу узлов.
fn clip_ends(points: &mut [Point], first: Boxed, last: Boxed) {
    let len = points.len();
    if len < 2 {
        return;
    }
    let toward_first = points[1];
    points[0] = border_point(first, toward_first);
    let toward_last = points[len - 2];
    points[len - 1] = border_point(last, toward_last);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(cluster: Option<usize>) -> GraphNode {
        GraphNode {
            width: 80.0,
            height: 40.0,
            cluster,
        }
    }

    fn edge(from: usize, to: usize) -> GraphEdge {
        GraphEdge {
            from,
            to,
            min_rank: 1,
            weight: 1,
            ..GraphEdge::default()
        }
    }

    #[test]
    fn test_arrow_length_and_hints_choose_placement() {
        use plantuml_ast::common::ArrowShape;
        let short = GraphEdge::from_arrow(0, 1, ArrowShape::parse("->"));
        assert!(short.same_rank);
        let long = GraphEdge::from_arrow(0, 1, ArrowShape::parse("--->"));
        assert_eq!(long.min_rank, 2);
        let up = GraphEdge::from_arrow(0, 1, ArrowShape::parse("-up->"));
        assert_eq!((up.from, up.to), (1, 0));
        assert!(up.reverse_path);
        let left = GraphEdge::from_arrow(0, 1, ArrowShape::parse("-left->"));
        assert!(left.same_rank && left.reverse_path);

        let r = layout(&input(
            vec![node(None), node(None)],
            vec![GraphEdge::from_arrow(0, 1, ArrowShape::parse("-up->"))],
        ));
        assert!(r.nodes[1].y + r.nodes[1].height <= r.nodes[0].y, "{r:?}");
        let path = &r.edges[0];
        assert!(
            path[0].1 > path[path.len() - 1].1,
            "ломаная идёт от начала связи: {path:?}"
        );
    }

    fn input(nodes: Vec<GraphNode>, edges: Vec<GraphEdge>) -> GraphInput {
        GraphInput {
            nodes,
            edges,
            node_sep: 30.0,
            rank_sep: 60.0,
            cluster_margin: 12.0,
            ..GraphInput::default()
        }
    }

    fn overlap(a: Boxed, b: Boxed) -> bool {
        a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
    }

    fn no_overlaps(result: &GraphLayout) {
        for (i, a) in result.nodes.iter().enumerate() {
            for b in result.nodes.iter().skip(i + 1) {
                assert!(!overlap(*a, *b), "узлы перекрылись: {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn test_empty() {
        assert!(layout(&GraphInput::default()).nodes.is_empty());
    }

    #[test]
    fn test_parent_above_child() {
        let r = layout(&input(vec![node(None), node(None)], vec![edge(0, 1)]));
        assert!(r.nodes[0].y + r.nodes[0].height < r.nodes[1].y);
        // Центры на одной вертикали: ребро прямое.
        assert!((r.nodes[0].x - r.nodes[1].x).abs() < 1.0);
    }

    #[test]
    fn test_siblings_do_not_overlap_and_parent_is_centered() {
        let r = layout(&input(
            vec![node(None), node(None), node(None)],
            vec![edge(0, 1), edge(0, 2)],
        ));
        no_overlaps(&r);
        let middle = (r.nodes[1].x + r.nodes[2].x) / 2.0;
        assert!((r.nodes[0].x - middle).abs() <= 1.0, "{r:?}");
    }

    #[test]
    fn test_edge_ends_lie_on_borders() {
        let r = layout(&input(vec![node(None), node(None)], vec![edge(0, 1)]));
        let path = &r.edges[0];
        assert!((path[0].1 - (r.nodes[0].y + r.nodes[0].height)).abs() < 0.01);
        assert!((path[path.len() - 1].1 - r.nodes[1].y).abs() < 0.01);
    }

    #[test]
    fn test_cycle_is_laid_out_and_direction_kept() {
        let r = layout(&input(
            vec![node(None), node(None), node(None)],
            vec![edge(0, 1), edge(1, 2), edge(2, 0)],
        ));
        no_overlaps(&r);
        // Ломаная идёт от начала к концу, даже у развёрнутого ребра.
        let back = &r.edges[2];
        let start = back[0];
        let top = r.nodes[2];
        assert!(
            start.1 <= top.y + top.height + 0.01 && start.1 >= top.y - 0.01,
            "{r:?}"
        );
    }

    #[test]
    fn test_long_edge_goes_around_middle_node() {
        let r = layout(&input(
            vec![node(None), node(None), node(None)],
            vec![edge(0, 1), edge(1, 2), edge(0, 2)],
        ));
        no_overlaps(&r);
        let long = &r.edges[2];
        assert!(long.len() > 2, "длинное ребро без цепочки: {long:?}");
        let mid = r.nodes[1];
        for (x, y) in long {
            let inside =
                *x > mid.x && *x < mid.x + mid.width && *y > mid.y && *y < mid.y + mid.height;
            assert!(!inside, "ребро прошло сквозь узел: {long:?}");
        }
    }

    #[test]
    fn test_chain_above_fork_is_centered_over_children() {
        // start -> fork -> (a, b): столбец start-fork стоит между детьми.
        let r = layout(&input(
            vec![node(None), node(None), node(None), node(None)],
            vec![edge(0, 1), edge(1, 2), edge(1, 3)],
        ));
        let center = |b: Boxed| b.x + b.width / 2.0;
        let middle = (center(r.nodes[2]) + center(r.nodes[3])) / 2.0;
        assert!((center(r.nodes[1]) - middle).abs() < 1.0, "{:?}", r.nodes);
        assert!((center(r.nodes[0]) - middle).abs() < 1.0, "{:?}", r.nodes);
    }

    #[test]
    fn test_label_stands_beside_the_line() {
        let mut labeled = edge(0, 1);
        labeled.label = Some((60.0, 16.0));
        let r = layout(&input(vec![node(None), node(None)], vec![labeled]));
        let (x, _) = r.labels[0].expect("есть подпись");
        let (line_x, _) = r.edges[0][0];
        assert!(
            r.edges[0].iter().all(|p| (p.0 - line_x).abs() < 0.5),
            "линия прямая: {:?}",
            r.edges[0]
        );
        assert!(x - 30.0 >= line_x - 0.01, "подпись справа от линии");
    }

    #[test]
    fn test_label_gets_space_between_ranks() {
        let mut labeled = edge(0, 1);
        labeled.label = Some((60.0, 16.0));
        let r = layout(&input(vec![node(None), node(None)], vec![labeled]));
        let (_, y) = r.labels[0].expect("есть подпись");
        assert!(y - 8.0 >= r.nodes[0].y + r.nodes[0].height - 0.01);
        assert!(y + 8.0 <= r.nodes[1].y + 0.01);
    }

    #[test]
    fn test_cluster_surrounds_its_nodes_only() {
        let mut data = input(
            vec![node(Some(0)), node(Some(0)), node(None)],
            vec![edge(0, 1), edge(2, 1)],
        );
        data.clusters = vec![GraphCluster {
            parent: None,
            title_height: 20.0,
            min_width: 0.0,
        }];
        let r = layout(&data);
        no_overlaps(&r);
        let c = r.clusters[0];
        for b in &r.nodes[..2] {
            assert!(b.x >= c.x && b.x + b.width <= c.x + c.width, "{r:?}");
            assert!(
                b.y >= c.y + 20.0 - 0.01 && b.y + b.height <= c.y + c.height,
                "{r:?}"
            );
        }
        assert!(!overlap(r.nodes[2], c), "чужой узел внутри рамки: {r:?}");
    }

    #[test]
    fn test_nested_clusters() {
        let mut data = input(vec![node(Some(1)), node(Some(0))], vec![edge(0, 1)]);
        data.clusters = vec![
            GraphCluster {
                parent: None,
                title_height: 20.0,
                min_width: 0.0,
            },
            GraphCluster {
                parent: Some(0),
                title_height: 20.0,
                min_width: 0.0,
            },
        ];
        let r = layout(&data);
        let (outer, inner) = (r.clusters[0], r.clusters[1]);
        assert!(outer.x <= inner.x && outer.y <= inner.y, "{r:?}");
        assert!(outer.x + outer.width >= inner.x + inner.width, "{r:?}");
        assert!(outer.y + outer.height >= inner.y + inner.height, "{r:?}");
    }

    /// Связь с рамкой обходит все её узлы и обрывается на границе.
    #[test]
    fn test_edge_to_cluster_stops_at_its_border() {
        let mut to_frame = edge(0, 1);
        to_frame.to_cluster = Some(0);
        let mut from_frame = edge(2, 3);
        from_frame.from_cluster = Some(0);
        let mut data = input(
            vec![node(None), node(Some(0)), node(Some(0)), node(None)],
            vec![edge(1, 2), to_frame, from_frame],
        );
        data.clusters = vec![GraphCluster {
            parent: None,
            title_height: 20.0,
            min_width: 0.0,
        }];
        let r = layout(&data);
        let c = r.clusters[0];
        assert!(
            r.nodes[0].y + r.nodes[0].height <= c.y,
            "начало над рамкой: {r:?}"
        );
        assert!(r.nodes[3].y >= c.y + c.height, "конец под рамкой: {r:?}");
        let into = &r.edges[1];
        let tip = into[into.len() - 1];
        assert!(
            (tip.1 - c.y).abs() < 1.0,
            "стрелка упирается в рамку: {into:?} {c:?}"
        );
        let out = &r.edges[2];
        assert!(
            (out[0].1 - (c.y + c.height)).abs() < 1.0,
            "выход от рамки: {out:?} {c:?}"
        );
    }

    #[test]
    fn test_same_rank_edge_keeps_order() {
        let mut flat = edge(1, 0);
        flat.same_rank = true;
        flat.min_rank = 0;
        let r = layout(&input(vec![node(None), node(None)], vec![flat]));
        assert_eq!(r.nodes[0].y, r.nodes[1].y);
        assert!(r.nodes[1].x + r.nodes[1].width <= r.nodes[0].x, "{r:?}");
    }

    #[test]
    fn test_left_to_right_swaps_axes() {
        let mut data = input(vec![node(None), node(None)], vec![edge(0, 1)]);
        data.direction = Direction::LeftToRight;
        let r = layout(&data);
        assert_eq!(
            r.nodes[0].width, 80.0,
            "размер узла не должен поворачиваться"
        );
        assert!(r.nodes[0].x + r.nodes[0].width < r.nodes[1].x, "{r:?}");
        assert!((r.nodes[0].y - r.nodes[1].y).abs() < 1.0, "{r:?}");
    }

    #[test]
    fn test_self_loop_stays_outside_node() {
        let r = layout(&input(vec![node(None)], vec![edge(0, 0)]));
        let b = r.nodes[0];
        assert!(r.edges[0].iter().any(|(x, _)| *x > b.x + b.width));
    }

    /// Несколько рёбер между одной парой не сливаются в невидимую линию.
    #[test]
    fn test_parallel_edges_all_have_paths() {
        let r = layout(&input(
            vec![node(None), node(None)],
            vec![edge(0, 1), edge(0, 1)],
        ));
        assert!(r.edges.iter().all(|path| path.len() >= 2));
    }

    #[test]
    fn test_large_graph_is_fast_and_safe() {
        let nodes: Vec<GraphNode> = (0..800).map(|_| node(None)).collect();
        let edges: Vec<GraphEdge> = (0..800).map(|i| edge(i, (i * 7 + 1) % 800)).collect();
        let started = std::time::Instant::now();
        let r = layout(&input(nodes, edges));
        assert_eq!(r.nodes.len(), 800);
        assert!(started.elapsed().as_secs() < 5, "{:?}", started.elapsed());
    }
}
