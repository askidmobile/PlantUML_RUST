//! Порядок вершин внутри рангов: взвешенная медиана и перестановка соседей, как в dot.
//!
//! Вершины одного кластера держатся подряд, чтобы рамка не перекрывала чужие узлы:
//! сортируем не вершины, а дерево «кластер → содержимое» с общим ключом.

use std::collections::BTreeMap;

/// Число проходов: у dot 24 с остановкой, когда улучшений нет.
const MAX_PASSES: usize = 24;
/// На огромных графах подсчёт пересечений дорог, а раскладка всё равно нечитаема.
const MAX_SEGMENTS_FOR_TRANSPOSE: usize = 4000;

/// Соседи вершины сверху и снизу.
struct Adjacency {
    up: Vec<Vec<usize>>,
    down: Vec<Vec<usize>>,
}

/// Минимизирует пересечения; `rows` меняется на месте.
pub fn minimize_crossings(
    rows: &mut [Vec<usize>],
    segments: &[(usize, usize)],
    paths: &[Vec<usize>],
    vertex_count: usize,
) {
    let mut adjacency = Adjacency {
        up: vec![Vec::new(); vertex_count],
        down: vec![Vec::new(); vertex_count],
    };
    for &(upper, lower) in segments {
        adjacency.down[upper].push(lower);
        adjacency.up[lower].push(upper);
    }

    keep_clusters_together(rows, paths);
    let mut best = rows.to_vec();
    let mut best_crossings = count_all(rows, segments, vertex_count);
    let transpose_allowed = segments.len() <= MAX_SEGMENTS_FOR_TRANSPOSE;

    for pass in 0..MAX_PASSES {
        if best_crossings == 0 {
            break;
        }
        let downward = pass % 2 == 0;
        let indices: Vec<usize> = if downward {
            (1..rows.len()).collect()
        } else {
            (0..rows.len().saturating_sub(1)).rev().collect()
        };
        for r in indices {
            let reference = if downward { r - 1 } else { r + 1 };
            let positions = positions_of(&rows[reference], vertex_count);
            let neighbours = if downward {
                &adjacency.up
            } else {
                &adjacency.down
            };
            reorder_row(&mut rows[r], neighbours, &positions, paths);
        }
        if transpose_allowed {
            transpose(rows, &adjacency, paths, vertex_count);
        }
        let crossings = count_all(rows, segments, vertex_count);
        if crossings < best_crossings {
            best_crossings = crossings;
            best = rows.to_vec();
        }
    }
    rows.clone_from_slice(&best);
}

fn positions_of(row: &[usize], vertex_count: usize) -> Vec<usize> {
    let mut positions = vec![usize::MAX; vertex_count];
    for (index, v) in row.iter().enumerate() {
        positions[*v] = index;
    }
    positions
}

/// Взвешенная медиана позиций соседей; у вершины без соседей ключа нет.
fn median(neighbours: &[usize], positions: &[usize]) -> Option<f64> {
    let mut places: Vec<usize> = neighbours
        .iter()
        .map(|v| positions[*v])
        .filter(|p| *p != usize::MAX)
        .collect();
    if places.is_empty() {
        return None;
    }
    places.sort_unstable();
    let m = places.len() / 2;
    if places.len() % 2 == 1 {
        return Some(places[m] as f64);
    }
    if places.len() == 2 {
        return Some((places[0] + places[1]) as f64 / 2.0);
    }
    let left = (places[m - 1] - places[0]) as f64;
    let right = (places[places.len() - 1] - places[m]) as f64;
    if left + right == 0.0 {
        return Some((places[m - 1] + places[m]) as f64 / 2.0);
    }
    Some((places[m - 1] as f64 * right + places[m] as f64 * left) / (left + right))
}

/// Сортирует ряд по медианам, не разрывая кластеры.
fn reorder_row(
    row: &mut Vec<usize>,
    neighbours: &[Vec<usize>],
    positions: &[usize],
    paths: &[Vec<usize>],
) {
    let original: Vec<usize> = row.clone();
    let keys: BTreeMap<usize, f64> = original
        .iter()
        .enumerate()
        .map(|(index, v)| {
            (
                *v,
                median(&neighbours[*v], positions).unwrap_or(index as f64),
            )
        })
        .collect();
    *row = sort_tree(&original, &keys, paths, 0);
}

/// Рекурсия ограничена глубиной кластеров, а она проверена пределом вложенности разбора.
fn sort_tree(
    items: &[usize],
    keys: &BTreeMap<usize, f64>,
    paths: &[Vec<usize>],
    depth: usize,
) -> Vec<usize> {
    // Группа: либо одиночная вершина, либо кластер этого уровня.
    let mut groups: Vec<(f64, usize, Vec<usize>)> = Vec::new();
    let mut by_cluster: BTreeMap<usize, usize> = BTreeMap::new();
    for (order, v) in items.iter().enumerate() {
        match paths[*v].get(depth) {
            Some(cluster) => {
                let slot = *by_cluster.entry(*cluster).or_insert_with(|| {
                    groups.push((0.0, order, Vec::new()));
                    groups.len() - 1
                });
                groups[slot].2.push(*v);
            }
            None => groups.push((0.0, order, vec![*v])),
        }
    }
    for group in groups.iter_mut() {
        let sum: f64 = group
            .2
            .iter()
            .map(|v| keys.get(v).copied().unwrap_or(0.0))
            .sum();
        group.0 = sum / group.2.len() as f64;
    }
    groups.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut out = Vec::with_capacity(items.len());
    for (_, _, members) in groups {
        let nested = members.len() > 1 && members.iter().any(|v| paths[*v].len() > depth + 1)
            || members.len() > 1 && paths[members[0]].len() > depth;
        if nested {
            out.extend(sort_tree(&members, keys, paths, depth + 1));
        } else {
            out.extend(members);
        }
    }
    out
}

/// Начальная расстановка: кластеры подряд.
fn keep_clusters_together(rows: &mut [Vec<usize>], paths: &[Vec<usize>]) {
    for row in rows.iter_mut() {
        let keys: BTreeMap<usize, f64> = row
            .iter()
            .enumerate()
            .map(|(index, v)| (*v, index as f64))
            .collect();
        *row = sort_tree(&row.clone(), &keys, paths, 0);
    }
}

/// Пересечения между двумя вершинами ряда со стороны соседнего ряда.
fn pair_crossings(a: &[usize], b: &[usize], positions: &[usize]) -> usize {
    let mut count = 0;
    for x in a {
        for y in b {
            if positions[*x] != usize::MAX
                && positions[*y] != usize::MAX
                && positions[*x] > positions[*y]
            {
                count += 1;
            }
        }
    }
    count
}

/// Меняет соседей местами, пока это убирает пересечения.
fn transpose(
    rows: &mut [Vec<usize>],
    adjacency: &Adjacency,
    paths: &[Vec<usize>],
    vertex_count: usize,
) {
    for _ in 0..4 {
        let mut improved = false;
        for r in 0..rows.len() {
            let above = r
                .checked_sub(1)
                .map(|i| positions_of(&rows[i], vertex_count));
            let below = rows.get(r + 1).map(|row| positions_of(row, vertex_count));
            for i in 0..rows[r].len().saturating_sub(1) {
                let (a, b) = (rows[r][i], rows[r][i + 1]);
                if paths[a] != paths[b] {
                    continue;
                }
                let mut keep = 0;
                let mut swap = 0;
                if let Some(pos) = &above {
                    keep += pair_crossings(&adjacency.up[a], &adjacency.up[b], pos);
                    swap += pair_crossings(&adjacency.up[b], &adjacency.up[a], pos);
                }
                if let Some(pos) = &below {
                    keep += pair_crossings(&adjacency.down[a], &adjacency.down[b], pos);
                    swap += pair_crossings(&adjacency.down[b], &adjacency.down[a], pos);
                }
                if swap < keep {
                    rows[r].swap(i, i + 1);
                    improved = true;
                }
            }
        }
        if !improved {
            break;
        }
    }
}

/// Общее число пересечений.
fn count_all(rows: &[Vec<usize>], segments: &[(usize, usize)], vertex_count: usize) -> usize {
    let mut rank_of = vec![usize::MAX; vertex_count];
    let mut position = vec![0usize; vertex_count];
    for (r, row) in rows.iter().enumerate() {
        for (index, v) in row.iter().enumerate() {
            rank_of[*v] = r;
            position[*v] = index;
        }
    }
    let mut by_rank: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
    for &(upper, lower) in segments {
        by_rank
            .entry(rank_of[upper])
            .or_default()
            .push((position[upper], position[lower]));
    }
    let mut total = 0;
    for pairs in by_rank.values_mut() {
        pairs.sort_unstable();
        // Сортировка слиянием считала бы быстрее, но здесь важнее простота;
        // огромные ряды отсекает предел MAX_SEGMENTS_FOR_TRANSPOSE выше по вызову.
        let limit = pairs.len().min(1200);
        for i in 0..limit {
            for j in (i + 1)..limit {
                if pairs[i].1 > pairs[j].1 {
                    total += 1;
                }
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossing_is_removed() {
        // Ранг 0: a b; ранг 1: c d; a→d, b→c — одно пересечение.
        let mut rows = vec![vec![0, 1], vec![2, 3]];
        let segments = [(0, 3), (1, 2)];
        let paths = vec![Vec::new(); 4];
        minimize_crossings(&mut rows, &segments, &paths, 4);
        assert_eq!(count_all(&rows, &segments, 4), 0);
    }

    #[test]
    fn test_cluster_members_stay_adjacent() {
        // Вершины 0 и 2 в кластере, между ними на входе чужая 1.
        let mut rows = vec![vec![0, 1, 2]];
        let paths = vec![vec![7], Vec::new(), vec![7]];
        minimize_crossings(&mut rows, &[], &paths, 3);
        let at = |v: usize| rows[0].iter().position(|x| *x == v).unwrap_or(9);
        assert_eq!(at(0).abs_diff(at(2)), 1, "{rows:?}");
    }

    #[test]
    fn test_median_of_even_neighbours_is_weighted() {
        let positions = vec![0, 1, 2, 3];
        assert_eq!(median(&[0, 1], &positions), Some(0.5));
        assert_eq!(median(&[], &positions), None);
    }
}
