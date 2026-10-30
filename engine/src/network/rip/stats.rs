use super::{EventType, Sim, Snapshot, NodeView, Edge, INF, MAX_LINK_COST, MIN_LINK_COST};
use std::collections::{BTreeMap, BTreeSet};

impl Sim {
    pub(super) fn has_rip_path(&self, src: u16, dest: u16) -> bool {
        let mut at = src;
        let mut visited = BTreeSet::new();
        while at != dest {
            if !visited.insert(at) { return false; }
            let Some(router) = self.nodes.get(&at).filter(|router| router.is_running) else {
                return false;
            };
            let Some(route) = router.routes.get(&dest).filter(|route| route.metric < INF) else {
                return false;
            };
            if !router.peers.get(&route.nxt_hop).is_some_and(|peer| peer.is_active && !peer.is_garbage) {
                return false;
            }
            at = route.nxt_hop;
        }
        self.nodes.get(&dest).is_some_and(|router| router.is_running)
    }

    pub(super) fn measure(&mut self) {
        let mut costs = Vec::new();
        let mut correct = 0usize;
        let mut total = 0usize;
        let mut rip_reachable = 0usize;
        let alive: Vec<u16> = self.nodes.values().filter(|r| r.is_running).map(|r| r.id).collect();
        for &src in &alive {
            let mut dist: BTreeMap<u16, u16> = alive.iter().map(|&id| (id, 10000)).collect();
            dist.insert(src, 0);
            let mut visited = BTreeSet::new();
            for _ in 0..alive.len() {
                let cur = dist.iter().filter(|(id, _)| !visited.contains(*id))
                    .min_by_key(|(_, cost)| **cost).map(|(&id, _)| id);
                let Some(cur) = cur else { break; };
                visited.insert(cur);
                let cur_cost = dist[&cur];
                for edge in &self.edges {
                    let other = if edge.a == cur { Some(edge.b) }
                        else if edge.b == cur { Some(edge.a) } else { None };
                    if let Some(other) = other
                        && let Some(current) = dist.get_mut(&other) {
                            *current = (*current).min(cur_cost + u16::from(edge.cost));
                        }
                }
            }
            for dest in 1..=self.cfg.nodes {
                if src == dest { continue; }
                let ideal = dist.get(&dest).copied().unwrap_or(10000).min(u16::from(INF)) as u8;
                let actual = self.nodes[&src].routes.get(&dest).map_or(INF, |r| r.metric);
                if self.has_rip_path(src, dest) { rip_reachable += 1; }
                total += 1;
                if actual == ideal { correct += 1; }
                if ideal < INF { costs.push(f64::from(ideal)); }
            }
        }
        self.stats.accuracy = if total == 0 { 1.0 } else { correct as f64 / total as f64 };
        self.stats.reachable_pairs = costs.len();
        let nodes = usize::from(self.cfg.nodes);
        let possible_pairs = nodes * nodes.saturating_sub(1);
        self.stats.rip_reachability = if possible_pairs == 0 { 0.0 }
            else { rip_reachable as f64 / possible_pairs as f64 };
        self.stats.shortest_path_cost_mean = if costs.is_empty() { 0.0 }
            else { costs.iter().sum::<f64>() / costs.len() as f64 };
        self.stats.shortest_path_cost_min = costs.iter().copied()
            .reduce(f64::min).unwrap_or(0.0) as u8;
        self.stats.shortest_path_cost_max = costs.iter().copied()
            .reduce(f64::max).unwrap_or(0.0) as u8;
        self.stats.is_converged = correct == total;
        if !self.stats.is_converged { self.stats.convergence_secs = None; }
        if self.stats.is_converged && self.stats.convergence_secs.is_none() {
            self.stats.convergence_secs = Some((self.now - self.topology_changed).max(0.0));
            self.emit(EventType::Converged, 0, None, None, None, None, None, None,
                      format!("Network converged after {:.2}s", self.now - self.topology_changed));
        }
    }

pub(super) fn take_snapshot(&mut self) -> Snapshot {
        let nodes = self.nodes.values().map(|node| NodeView {
            id: node.id, is_running: node.is_running, is_quiescent: node.is_quiescent,
            auto_restart_in: node.auto_restart_at.map(|time| (time - self.now).max(0.0)),
            active_nbrs: node.peers.values().filter(|p| p.is_active).count(),
            garbage_nbrs: node.peers.values().filter(|p| p.is_garbage).count(),
            routes: node.routes.values().cloned().collect(),
            offline_dests: node.offline_dests.iter().copied().collect(),
        }).collect();
        Snapshot {
            time: self.now, dur: f64::from(self.cfg.to_secs),
            hist_start: 0.0, is_complete: self.cfg.is_finished_at(self.now),
            nodes, edges: self.edges.iter().map(|edge| Edge {
                a: edge.a, b: edge.b, cost: edge.cost,
                can_increase: edge.cost < MAX_LINK_COST,
                can_decrease: edge.cost > MIN_LINK_COST,
            }).collect(),
            events: std::mem::take(&mut self.events), stats: self.stats.clone(),
        }
    }
}
