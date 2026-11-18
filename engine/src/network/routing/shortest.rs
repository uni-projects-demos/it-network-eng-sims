use super::Edge;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShortestPath {
    pub(crate) cost: u16,
    pub(crate) next_hop: u16,
}

pub(crate) fn shortest_paths(
    nodes: &BTreeSet<u16>,
    edges: &[Edge],
    source: u16,
) -> BTreeMap<u16, ShortestPath> {
    let mut paths: BTreeMap<u16, ShortestPath> = BTreeMap::new();
    let mut visited: BTreeSet<u16> = BTreeSet::new();
    if !nodes.contains(&source) {
        return paths;
    }
    paths.insert(
        source,
        ShortestPath {
            cost: 0,
            next_hop: source,
        },
    );
    while let Some((current, path)) = paths
        .iter()
        .filter(|(id, _)| !visited.contains(*id))
        .map(|(&id, &path)| (id, path))
        .min_by_key(|(id, path)| (path.cost, *id))
    {
        visited.insert(current);
        for edge in edges {
            let other = if edge.a == current {
                edge.b
            } else if edge.b == current {
                edge.a
            } else {
                continue;
            };
            if !nodes.contains(&other) || visited.contains(&other) {
                continue;
            }
            let candidate = ShortestPath {
                cost: path.cost.saturating_add(u16::from(edge.cost)),
                next_hop: if current == source {
                    other
                } else {
                    path.next_hop
                },
            };
            if paths.get(&other).is_none_or(|previous| {
                (candidate.cost, candidate.next_hop) < (previous.cost, previous.next_hop)
            }) {
                paths.insert(other, candidate);
            }
        }
    }
    paths.remove(&source);
    paths
}
