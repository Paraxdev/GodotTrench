//! The automatic layout: signals flow left to right, each node one column right of everything that fires it. Only sizes
//! and edges go in, so the same wiring always gets the same picture.

use egui::{Pos2, Vec2, pos2, vec2};

const COLUMN_GAP: f32 = 90.0;
const ROW_GAP: f32 = 24.0;
const COMPONENT_GAP: f32 = 60.0;
/// Barycenter passes that untangle the order within the columns.
const SWEEPS: usize = 4;

/// Top left corners for nodes of `sizes`, laid out left to right along `edges` given as (from, to) indices. Separate
/// groups of wired nodes are stacked, nodes without wiring fill rows below them.
pub fn layered(sizes: &[Vec2], edges: &[(usize, usize)]) -> Vec<Pos2> {
    let n = sizes.len();
    let mut succ: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut pred: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(a, b) in edges {
        if a != b && a < n && b < n && !succ[a].contains(&b) {
            succ[a].push(b);
            pred[b].push(a);
        }
    }

    let mut out = vec![Pos2::ZERO; n];
    let mut component = vec![usize::MAX; n];
    let mut singles = Vec::new();
    let mut groups: Vec<(Vec<usize>, Vec2)> = Vec::new();
    for start in 0..n {
        if component[start] != usize::MAX {
            continue;
        }

        let mut members = vec![start];
        component[start] = start;
        let mut i = 0;
        while i < members.len() {
            let v = members[i];
            for &w in succ[v].iter().chain(&pred[v]) {
                if component[w] == usize::MAX {
                    component[w] = start;
                    members.push(w);
                }
            }

            i += 1;
        }

        if members.len() == 1 {
            singles.push(start);
            continue;
        }

        members.sort_unstable();
        let size = place_component(&members, sizes, &succ, &mut out);
        groups.push((members, size));
    }

    // Groups fill rows about as wide as the whole picture is tall, so it fits a panel instead of one tall column.
    let area: f32 = groups.iter().map(|(_, s)| (s.x + COMPONENT_GAP) * (s.y + COMPONENT_GAP)).sum::<f32>()
        + singles.iter().map(|s| (sizes[*s].x + COLUMN_GAP) * (sizes[*s].y + ROW_GAP)).sum::<f32>();
    let widest = groups.iter().map(|(_, s)| s.x).fold(0.0, f32::max);
    let row_width = widest.max(area.sqrt() * 1.6).max(600.0);
    let (mut x, mut y, mut row_height) = (0.0f32, 0.0f32, 0.0f32);
    for (members, size) in &groups {
        if x > 0.0 && x + size.x > row_width {
            x = 0.0;
            y += row_height + COMPONENT_GAP;
            row_height = 0.0;
        }

        for m in members {
            out[*m] += vec2(x, y);
        }

        x += size.x + COMPONENT_GAP * 2.0;
        row_height = row_height.max(size.y);
    }

    if !groups.is_empty() {
        y += row_height + COMPONENT_GAP;
    }

    (x, row_height) = (0.0, 0.0);
    for s in singles {
        if x > 0.0 && x + sizes[s].x > row_width {
            x = 0.0;
            y += row_height + ROW_GAP;
            row_height = 0.0;
        }

        out[s] = pos2(x, y);
        x += sizes[s].x + COLUMN_GAP * 0.5;
        row_height = row_height.max(sizes[s].y);
    }

    out
}

/// Lays out one connected group with its top left at the origin. Returns its size.
fn place_component(members: &[usize], sizes: &[Vec2], succ: &[Vec<usize>], out: &mut [Pos2]) -> Vec2 {
    let local = |v: usize| members.binary_search(&v).ok();
    let count = members.len();
    let mut next: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (i, v) in members.iter().enumerate() {
        next[i] = succ[*v].iter().filter_map(|w| local(*w)).collect();
    }

    let dag = break_cycles(&next);
    let mut prev: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (a, targets) in dag.iter().enumerate() {
        for b in targets {
            prev[*b].push(a);
        }
    }

    // Longest path from the sources, visited in index order so ties always fall the same way.
    let mut layer = vec![0usize; count];
    let mut waiting: Vec<usize> = prev.iter().map(Vec::len).collect();
    let mut ready: std::collections::BTreeSet<usize> = (0..count).filter(|v| waiting[*v] == 0).collect();
    let mut topo = Vec::with_capacity(count);
    while let Some(v) = ready.pop_first() {
        topo.push(v);
        for &w in &dag[v] {
            layer[w] = layer[w].max(layer[v] + 1);
            waiting[w] -= 1;
            if waiting[w] == 0 {
                ready.insert(w);
            }
        }
    }

    let depth = layer.iter().max().map_or(0, |l| l + 1);
    let mut columns: Vec<Vec<usize>> = vec![Vec::new(); depth];
    for v in &topo {
        columns[layer[*v]].push(*v);
    }

    let mut order = vec![0.0f32; count];
    let renumber = |columns: &[Vec<usize>], order: &mut [f32]| {
        for column in columns {
            for (i, v) in column.iter().enumerate() {
                order[*v] = i as f32;
            }
        }
    };
    renumber(&columns, &mut order);
    for _ in 0..SWEEPS {
        for (c, neighbours) in [(1..depth).collect::<Vec<_>>(), (0..depth.saturating_sub(1)).rev().collect()].into_iter().zip([&prev, &dag]) {
            for l in c {
                let column = &mut columns[l];
                let key = |v: usize| {
                    let ns = &neighbours[v];
                    if ns.is_empty() { order[v] } else { ns.iter().map(|w| order[*w]).sum::<f32>() / ns.len() as f32 }
                };
                let mut keyed: Vec<(f32, f32, usize)> = column.iter().map(|v| (key(*v), order[*v], *v)).collect();
                keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
                *column = keyed.into_iter().map(|k| k.2).collect();
                for (i, v) in column.iter().enumerate() {
                    order[*v] = i as f32;
                }
            }
        }
    }

    // Each node moves toward the middle of what fires it, keeping the column's order and gaps.
    let mut x = 0.0f32;
    let mut top = vec![0.0f32; count];
    let mut left = vec![0.0f32; count];
    for column in &columns {
        let mut floor = f32::MIN;
        for v in column {
            let h = sizes[members[*v]].y;
            let wanted = if prev[*v].is_empty() {
                floor.max(0.0)
            } else {
                prev[*v].iter().map(|p| top[*p] + sizes[members[*p]].y * 0.5).sum::<f32>() / prev[*v].len() as f32 - h * 0.5
            };
            top[*v] = if floor == f32::MIN { wanted } else { wanted.max(floor) };
            floor = top[*v] + h + ROW_GAP;
            left[*v] = x;
        }

        x += column.iter().map(|v| sizes[members[*v]].x).fold(0.0, f32::max) + COLUMN_GAP;
    }

    let min_y = top.iter().copied().fold(f32::MAX, f32::min);
    let mut size = Vec2::ZERO;
    for v in 0..count {
        let p = pos2(left[v], top[v] - min_y);
        out[members[v]] = p;
        size = size.max(p.to_vec2() + sizes[members[v]]);
    }

    vec2(size.x, size.y)
}

/// The edges without the ones that close a cycle, found by a depth first search in index order.
fn break_cycles(next: &[Vec<usize>]) -> Vec<Vec<usize>> {
    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        New,
        Open,
        Done,
    }

    let mut mark = vec![Mark::New; next.len()];
    let mut dag: Vec<Vec<usize>> = vec![Vec::new(); next.len()];
    for root in 0..next.len() {
        if mark[root] != Mark::New {
            continue;
        }

        mark[root] = Mark::Open;
        let mut stack = vec![(root, 0usize)];
        while let Some((v, i)) = stack.last_mut() {
            let v = *v;
            if let Some(&w) = next[v].get(*i) {
                *i += 1;
                match mark[w] {
                    Mark::Open => {}
                    Mark::Done => dag[v].push(w),
                    Mark::New => {
                        dag[v].push(w);
                        mark[w] = Mark::Open;
                        stack.push((w, 0));
                    }
                }
            } else {
                mark[v] = Mark::Done;
                stack.pop();
            }
        }
    }

    dag
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sizes(n: usize) -> Vec<Vec2> {
        (0..n).map(|i| vec2(150.0 + i as f32 * 3.0, 80.0)).collect()
    }

    #[test]
    fn flows_left_to_right() {
        let s = sizes(4);
        let p = layered(&s, &[(0, 1), (1, 2), (0, 2), (2, 3)]);
        assert!(p[0].x < p[1].x && p[1].x < p[2].x && p[2].x < p[3].x, "{p:?}");
        let r = |i: usize| egui::Rect::from_min_size(p[i], s[i]);
        for i in 0..4 {
            for j in i + 1..4 {
                assert!(!r(i).intersects(r(j)), "{i} and {j} overlap");
            }
        }
    }

    #[test]
    fn is_deterministic_and_survives_cycles() {
        let s = sizes(9);
        let edges = [(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (3, 5), (5, 3), (6, 6)];
        let first = layered(&s, &edges);
        for _ in 0..3 {
            assert_eq!(layered(&s, &edges), first);
        }

        assert!(first[0].x < first[1].x && first[1].x < first[2].x, "a loop is cut where it closes: {first:?}");
        assert!(first[6].y > first[3].y && first[7].y == first[6].y, "unwired nodes fill a row below: {first:?}");
    }

    #[test]
    fn separate_groups_do_not_overlap_and_fill_rows() {
        let n = 24;
        let s = sizes(n);
        let edges: Vec<(usize, usize)> = (0..n / 2).map(|i| (2 * i, 2 * i + 1)).collect();
        let p = layered(&s, &edges);
        let rects: Vec<egui::Rect> = (0..n).map(|i| egui::Rect::from_min_size(p[i], s[i])).collect();
        for i in 0..n {
            for j in i + 1..n {
                assert!(!rects[i].intersects(rects[j]), "{i} and {j} overlap: {p:?}");
            }
        }

        let bounds = rects.iter().fold(egui::Rect::NOTHING, |a, b| a.union(*b));
        assert!(bounds.width() > bounds.height() * 0.5, "not one tall column: {bounds:?}");
    }
}
