//! Сетевой симплекс: целочисленные значения с минимальной суммой взвешенных длин рёбер.
//!
//! Этим алгоритмом Graphviz dot считает и ранги узлов, и их горизонтальные координаты:
//! обе задачи сводятся к `минимум Σ w·(r(v)-r(u))` при `r(v)-r(u) ≥ min_len`.
//!
//! Обход дерева написан без рекурсии: глубина графа приходит из пользовательской схемы,
//! а переполнение стека завершило бы весь процесс.

/// Ребро ограничения: `r(to) - r(from) >= min_len`.
#[derive(Debug, Clone, Copy)]
pub struct Constraint {
    pub from: usize,
    pub to: usize,
    pub min_len: i64,
    pub weight: i64,
}

/// Предел обменов рёбер. Любое промежуточное решение допустимо, поэтому остановка
/// по пределу даёт лишь чуть худшую раскладку огромной схемы, а не ошибку.
const MAX_PIVOTS: usize = 1500;

struct Solver<'a> {
    edges: &'a [Constraint],
    rank: Vec<i64>,
    /// Рёбра вокруг узла: индексы в `edges`
    incident: Vec<Vec<usize>>,
    in_tree: Vec<bool>,
    /// Родитель в дереве и ребро к нему
    parent: Vec<Option<(usize, usize)>>,
    /// Границы поддерева при обходе: узел `x` лежит в поддереве `c`, если `low[c] <= lim[x] <= lim[c]`
    lim: Vec<usize>,
    low: Vec<usize>,
    /// Разрез ребра дерева, индекс — номер ребра
    cut: Vec<i64>,
}

/// Решает задачу для `n` узлов. Граф ограничений обязан быть ациклическим.
///
/// Вернёт значения, нормированные к минимуму 0.
pub fn solve(n: usize, edges: &[Constraint]) -> Vec<i64> {
    if n == 0 {
        return Vec::new();
    }
    // Служебный корень связывает компоненты связности рёбрами нулевого веса.
    let root = n;
    let mut all: Vec<Constraint> = edges.to_vec();
    all.extend((0..n).map(|v| Constraint {
        from: root,
        to: v,
        min_len: 0,
        weight: 0,
    }));

    let total = n + 1;
    let mut solver = Solver {
        edges: &all,
        rank: longest_path(total, &all),
        incident: incident_lists(total, &all),
        in_tree: vec![false; all.len()],
        parent: vec![None; total],
        lim: vec![0; total],
        low: vec![0; total],
        cut: vec![0; all.len()],
    };
    solver.feasible_tree();
    solver.refresh_tree(root);

    for _ in 0..MAX_PIVOTS {
        let Some(leave) = solver.leaving_edge() else {
            break;
        };
        let Some(enter) = solver.entering_edge(leave) else {
            break;
        };
        solver.exchange(leave, enter, root);
    }

    let mut result = solver.rank;
    result.truncate(n);
    let min = result.iter().copied().min().unwrap_or(0);
    result.iter_mut().for_each(|r| *r -= min);
    result
}

fn incident_lists(total: usize, edges: &[Constraint]) -> Vec<Vec<usize>> {
    let mut incident = vec![Vec::new(); total];
    for (index, edge) in edges.iter().enumerate() {
        incident[edge.from].push(index);
        incident[edge.to].push(index);
    }
    incident
}

/// Начальное допустимое решение: самый длинный путь от истоков.
fn longest_path(total: usize, edges: &[Constraint]) -> Vec<i64> {
    let mut indegree = vec![0usize; total];
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); total];
    for (index, edge) in edges.iter().enumerate() {
        indegree[edge.to] += 1;
        out[edge.from].push(index);
    }
    let mut rank = vec![0i64; total];
    let mut queue: Vec<usize> = (0..total).filter(|v| indegree[*v] == 0).collect();
    while let Some(v) = queue.pop() {
        for &index in &out[v] {
            let edge = edges[index];
            rank[edge.to] = rank[edge.to].max(rank[v] + edge.min_len);
            indegree[edge.to] -= 1;
            if indegree[edge.to] == 0 {
                queue.push(edge.to);
            }
        }
    }
    rank
}

impl Solver<'_> {
    fn slack(&self, index: usize) -> i64 {
        let edge = self.edges[index];
        self.rank[edge.to] - self.rank[edge.from] - edge.min_len
    }

    /// Строит остовное дерево из рёбер с нулевым запасом, сдвигая ранги при нехватке.
    fn feasible_tree(&mut self) {
        let total = self.rank.len();
        let mut in_component = vec![false; total];
        in_component[total - 1] = true;
        let mut size = 1;
        let mut stack = vec![total - 1];

        while size < total {
            while let Some(v) = stack.pop() {
                for i in 0..self.incident[v].len() {
                    let index = self.incident[v][i];
                    let edge = self.edges[index];
                    let other = if edge.from == v { edge.to } else { edge.from };
                    if !in_component[other] && self.slack(index) == 0 {
                        in_component[other] = true;
                        self.in_tree[index] = true;
                        size += 1;
                        stack.push(other);
                    }
                }
            }
            if size == total {
                break;
            }
            // Ребро с минимальным запасом между деревом и остальным графом.
            let mut best: Option<(i64, usize)> = None;
            for index in 0..self.edges.len() {
                let edge = self.edges[index];
                if in_component[edge.from] != in_component[edge.to] {
                    let slack = self.slack(index);
                    if best.is_none_or(|(s, _)| slack < s) {
                        best = Some((slack, index));
                    }
                }
            }
            let Some((slack, index)) = best else {
                break;
            };
            let edge = self.edges[index];
            let delta = if in_component[edge.to] { -slack } else { slack };
            for (v, inside) in in_component.iter().enumerate() {
                if *inside {
                    self.rank[v] += delta;
                    stack.push(v);
                }
            }
        }
    }

    /// Пересчитывает обход дерева и разрезы.
    fn refresh_tree(&mut self, root: usize) {
        let total = self.rank.len();
        self.parent.iter_mut().for_each(|p| *p = None);
        let mut visited = vec![false; total];
        visited[root] = true;
        let mut order = Vec::with_capacity(total);
        // Стек: узел и позиция в списке его рёбер
        let mut stack = vec![(root, 0usize)];
        let mut counter = 1;
        self.low[root] = counter;

        while let Some(top) = stack.last_mut() {
            let (v, cursor) = *top;
            if cursor < self.incident[v].len() {
                top.1 += 1;
                let index = self.incident[v][cursor];
                if !self.in_tree[index] {
                    continue;
                }
                let edge = self.edges[index];
                let other = if edge.from == v { edge.to } else { edge.from };
                if !visited[other] {
                    visited[other] = true;
                    self.parent[other] = Some((v, index));
                    self.low[other] = counter;
                    stack.push((other, 0));
                }
            } else {
                self.lim[v] = counter;
                counter += 1;
                order.push(v);
                stack.pop();
            }
        }

        // Внутренние рёбра поддерева сокращаются, остаётся сумма по узлам.
        let mut net = vec![0i64; total];
        for edge in self.edges {
            net[edge.from] += edge.weight;
            net[edge.to] -= edge.weight;
        }
        for &v in &order {
            if let Some((parent, index)) = self.parent[v] {
                let edge = self.edges[index];
                self.cut[index] = if edge.from == v { net[v] } else { -net[v] };
                net[parent] += net[v];
            }
        }
    }

    fn in_subtree(&self, node: usize, top: usize) -> bool {
        self.low[top] <= self.lim[node] && self.lim[node] <= self.lim[top]
    }

    /// Ребро дерева с отрицательным разрезом.
    fn leaving_edge(&self) -> Option<usize> {
        (0..self.edges.len()).find(|&i| self.in_tree[i] && self.cut[i] < 0)
    }

    /// Ребро вне дерева, идущее из компоненты головы в компоненту хвоста, с минимальным запасом.
    fn entering_edge(&self, leave: usize) -> Option<usize> {
        let edge = self.edges[leave];
        let child = if self.lim[edge.from] < self.lim[edge.to] {
            edge.from
        } else {
            edge.to
        };
        // В поддереве лежит хвост или голова покидающего ребра.
        let tail_in_subtree = child == edge.from;
        let mut best: Option<(i64, usize)> = None;
        for index in 0..self.edges.len() {
            if self.in_tree[index] {
                continue;
            }
            let candidate = self.edges[index];
            let from_in = self.in_subtree(candidate.from, child);
            let to_in = self.in_subtree(candidate.to, child);
            let crosses = if tail_in_subtree {
                !from_in && to_in
            } else {
                from_in && !to_in
            };
            if crosses {
                let slack = self.slack(index);
                if best.is_none_or(|(s, _)| slack < s) {
                    best = Some((slack, index));
                }
            }
        }
        best.map(|(_, index)| index)
    }

    /// Меняет ребро дерева и сдвигает ранги одной из компонент.
    fn exchange(&mut self, leave: usize, enter: usize, root: usize) {
        let edge = self.edges[leave];
        let child = if self.lim[edge.from] < self.lim[edge.to] {
            edge.from
        } else {
            edge.to
        };
        let slack = self.slack(enter);
        let candidate = self.edges[enter];
        let delta = if self.in_subtree(candidate.to, child) {
            -slack
        } else {
            slack
        };
        for v in 0..self.rank.len() {
            if self.in_subtree(v, child) {
                self.rank[v] += delta;
            }
        }
        self.in_tree[leave] = false;
        self.in_tree[enter] = true;
        self.refresh_tree(root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(from: usize, to: usize) -> Constraint {
        Constraint {
            from,
            to,
            min_len: 1,
            weight: 1,
        }
    }

    fn check(n: usize, edges: &[Constraint]) -> Vec<i64> {
        let ranks = solve(n, edges);
        assert_eq!(ranks.len(), n);
        for e in edges {
            assert!(
                ranks[e.to] - ranks[e.from] >= e.min_len,
                "нарушено {e:?}: {ranks:?}"
            );
        }
        assert_eq!(ranks.iter().min(), Some(&0));
        ranks
    }

    fn cost(ranks: &[i64], edges: &[Constraint]) -> i64 {
        edges
            .iter()
            .map(|e| e.weight * (ranks[e.to] - ranks[e.from]))
            .sum()
    }

    #[test]
    fn test_chain() {
        assert_eq!(check(3, &[edge(0, 1), edge(1, 2)]), vec![0, 1, 2]);
    }

    #[test]
    fn test_empty_and_isolated() {
        assert!(solve(0, &[]).is_empty());
        assert_eq!(solve(3, &[]), vec![0, 0, 0]);
    }

    /// Самый длинный путь оставил бы лист на нижнем ранге; оптимум подтягивает его к соседу.
    #[test]
    fn test_pulls_short_branch_up() {
        // 0→1→2→3 и 0→4→3 ; плюс 4→5: узел 5 должен стоять сразу за 4.
        let edges = [
            edge(0, 1),
            edge(1, 2),
            edge(2, 3),
            edge(0, 4),
            edge(4, 3),
            edge(4, 5),
        ];
        let ranks = check(6, &edges);
        assert_eq!(ranks[5] - ranks[4], 1);
    }

    /// Классический пример: у оптимума суммарная длина меньше, чем у самого длинного пути.
    #[test]
    fn test_reaches_optimum_on_known_graph() {
        // Граф из статьи Gansner et al., 1993.
        let names = "abcdefgh";
        let idx = |c: char| names.find(c).unwrap_or(0);
        let raw = [
            ('a', 'b'),
            ('b', 'c'),
            ('c', 'd'),
            ('d', 'h'),
            ('a', 'e'),
            ('e', 'g'),
            ('g', 'h'),
            ('a', 'f'),
            ('f', 'g'),
        ];
        let edges: Vec<Constraint> = raw.iter().map(|(u, v)| edge(idx(*u), idx(*v))).collect();
        let ranks = check(8, &edges);
        assert_eq!(cost(&ranks, &edges), 10, "{ranks:?}");
    }

    #[test]
    fn test_weights_prefer_heavy_edge_short() {
        let mut heavy = edge(0, 2);
        heavy.weight = 10;
        let edges = [edge(0, 1), heavy, edge(1, 3), edge(2, 3)];
        let ranks = check(4, &edges);
        assert_eq!(ranks[2] - ranks[0], 1);
    }

    #[test]
    fn test_disconnected_components() {
        let ranks = check(4, &[edge(0, 1), edge(2, 3)]);
        assert_eq!(ranks[1] - ranks[0], 1);
        assert_eq!(ranks[3] - ranks[2], 1);
    }

    /// Большой граф укладывается быстро и без рекурсии.
    #[test]
    fn test_large_chain_does_not_overflow_stack() {
        let edges: Vec<Constraint> = (0..20_000).map(|i| edge(i, i + 1)).collect();
        let handle = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || solve(20_001, &edges))
            .map(|h| h.join());
        assert!(matches!(handle, Ok(Ok(_))));
    }
}
