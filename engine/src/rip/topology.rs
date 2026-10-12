use super::{Config, Rand, Edge, MIN_LINK_COST, MAX_LINK_COST};
use std::collections::BTreeSet;

// assignment presentation network edges
pub(super) fn default_edges() -> Vec<Edge> {
    const LINKS: [(u16, u16, u8); 8] = [
        (1, 2, 1), (1, 6, 5), (1, 7, 8), (2, 3, 3),
        (3, 4, 4), (4, 5, 2), (4, 7, 6), (5, 6, 1),
    ];
    LINKS.into_iter().map(|(a, b, cost)| Edge {
        a, b, cost, can_increase: false, can_decrease: false,
    }).collect()
}

pub(super) fn unique_path_costs(edges: &[Edge], n: u16) -> bool {
    fn walk(current: u16, visited: u32, cost: u16, edges: &[Edge],
            seen: &mut [BTreeSet<u16>]) -> bool {
        if !seen[current as usize].insert(cost) { return false; }
        for edge in edges {
            let other = if edge.a == current { edge.b }
                else if edge.b == current { edge.a } else { continue; };
            if visited & (1u32 << other) != 0 { continue; }
            if !walk(other, visited | (1u32 << other), cost + u16::from(edge.cost), edges, seen) {
                return false;
            }
        }
        true
    }
    for start in 1..=n {
        let mut seen = vec![BTreeSet::new(); usize::from(n) + 1];
        if !walk(start, 1u32 << start, 0, edges, &mut seen) { return false; }
    }
    true
}

pub(super) fn unique_incident_costs(edges: &[Edge]) -> bool {
    let mut seen = BTreeSet::new();
    edges.iter().all(|edge| (MIN_LINK_COST..=MAX_LINK_COST).contains(&edge.cost)
        && seen.insert((edge.a, edge.cost))
        && seen.insert((edge.b, edge.cost)))
}

pub(super) fn weighted_cost(rng: &mut Rand, used_at_a: u16, used_at_b: u16) -> Option<u8> {
    let unavailable = used_at_a | used_at_b;
    let total: usize = (MIN_LINK_COST..=MAX_LINK_COST)
        .filter(|&cost| unavailable & (1u16 << cost) == 0)
        .map(|cost| if cost <= 10 { 2 } else { 1 })
        .sum();
    if total == 0 { return None; }
    let mut pick = rng.range(0, total - 1);
    for cost in MIN_LINK_COST..=MAX_LINK_COST {
        if unavailable & (1u16 << cost) != 0 { continue; }
        let weight = if cost <= 10 { 2 } else { 1 };
        if pick < weight { return Some(cost); }
        pick -= weight;
    }
    unreachable!("weighted-metric selection exhausted.")
}

fn unique_cycle_costs(cycle: &[Edge]) -> bool {
    let total: u16 = cycle.iter().map(|edge| u16::from(edge.cost)).sum();
    if !total.is_multiple_of(2) { return true; }
    for start in 0..cycle.len() {
        let mut sum = 0u16;
        for offset in 0..cycle.len() - 1 {
            sum += u16::from(cycle[(start + offset) % cycle.len()].cost);
            if sum * 2 == total { return false; }
        }
    }
    true
}

fn assign_cycle_costs(edges: &mut [Edge], idx: usize, used: &mut [u16], rng: &mut Rand) -> bool {
    if idx == edges.len() { return unique_cycle_costs(edges); }
    let a = usize::from(edges[idx].a);
    let b = usize::from(edges[idx].b);
    let mut tried = 0u16;
    while let Some(cost) = weighted_cost(rng, used[a] | tried, used[b] | tried) {
        let bit = 1u16 << cost;
        tried |= bit;
        edges[idx].cost = cost;
        used[a] |= bit;
        used[b] |= bit;
        if assign_cycle_costs(edges, idx + 1, used, rng) { return true; }
        used[a] &= !bit;
        used[b] &= !bit;
    }
    false
}

pub(super) fn gen_edges(cfg: Config, rng: &mut Rand) -> Vec<Edge> {
    let mut order: Vec<u16> = (1..=cfg.nodes).collect();
    for i in (1..order.len()).rev() {
        let j = rng.range(0, i);
        order.swap(i, j);
    }

    let cycle_len = if cfg.nodes == 3 { 3 } else { rng.range(3, usize::from(cfg.nodes) - 1) };
    let mut edges = Vec::with_capacity(usize::from(cfg.nodes));
    for i in 0..cycle_len {
        let a = order[i];
        let b = order[(i + 1) % cycle_len];
        edges.push(Edge { a: a.min(b), b: a.max(b), cost: 1,
            can_increase: false, can_decrease: false });
    }

    let mut connected = order[..cycle_len].to_vec();
    for &nxt in &order[cycle_len..] {
        let prev = connected[rng.range(0, connected.len() - 1)];
        edges.push(Edge { a: prev.min(nxt), b: prev.max(nxt), cost: 1,
            can_increase: false, can_decrease: false });
        connected.push(nxt);
    }

    let mut used = vec![0u16; usize::from(cfg.nodes) + 1];
    assert!(assign_cycle_costs(&mut edges[..cycle_len], 0, &mut used, rng),
        "unable to assign cycle costs and unique path totals.");

    for edge in &mut edges[cycle_len..] {
        let a = usize::from(edge.a);
        let b = usize::from(edge.b);
        let cost = weighted_cost(rng, used[a], used[b])
            .expect("unable to assign a distinct tree-edge cost.");
        let bit = 1u16 << cost;
        edge.cost = cost;
        used[a] |= bit;
        used[b] |= bit;
    }

    if cycle_len >= 4 {
        let extra_count = rng.range(0, (cycle_len - 3).min(3));
        let mut candidates = Vec::new();
        for i in 0..cycle_len {
            for j in i + 2..cycle_len {
                if i != 0 || j != cycle_len - 1 { candidates.push((i, j)); }
            }
        }
        for i in (1..candidates.len()).rev() {
            let j = rng.range(0, i);
            candidates.swap(i, j);
        }
        let mut added = Vec::new();
        for (i, j) in candidates {
            if added.len() == extra_count { break; }
            if added.iter().any(|&(a, b)| (a < i && i < b && b < j)
                || (i < a && a < j && j < b)) { continue; }
            let a = usize::from(order[i]);
            let b = usize::from(order[j]);
            let Some(cost) = weighted_cost(rng, used[a], used[b]) else { continue; };
            let bit = 1u16 << cost;
            used[a] |= bit;
            used[b] |= bit;
            edges.push(Edge { a: order[i].min(order[j]), b: order[i].max(order[j]), cost,
                can_increase: false, can_decrease: false });
            added.push((i, j));
        }
    }

    edges.sort_by_key(|edge| (edge.a, edge.b));
    debug_assert!(unique_incident_costs(&edges));
    debug_assert!(edges.len() > usize::from(cfg.nodes) || unique_path_costs(&edges, cfg.nodes));
    edges
}
