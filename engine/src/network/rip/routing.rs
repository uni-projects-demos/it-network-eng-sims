use super::{EventType, Sim, Packet, Route, INF, DELIVERY_DELAY, encode_res, decode_res};
use std::collections::{BTreeMap, BTreeSet};

struct RouteChange {
    dest: u16,
    prev: Option<Route>,
    best: Option<(u16, u8)>,
}

impl Sim {
    pub(super) fn send_tbl(&mut self, id: u16, is_triggered: bool) {
        let Some(node) = self.nodes.get(&id) else { return; };
        if !node.is_running || node.is_quiescent { return; }
        let peers: Vec<u16> = node.peers.keys().copied().collect();
        for peer_id in peers {
            let Some(node) = self.nodes.get(&id) else { continue; };
            let mut tbl = BTreeMap::new();
            for (&dest, route) in &node.routes {
                tbl.insert(dest, if route.nxt_hop == peer_id { INF } else { route.metric });
            }
            let entries = tbl.len();
            if tbl.is_empty() { continue; }
            let Ok(payload) = encode_res(id, &tbl) else { continue; };
            self.pending.push(Packet { from: id, to: peer_id, at: self.now + DELIVERY_DELAY, payload });
            self.stats.updates_sent += 1;
            self.emit(EventType::Send, id, Some(peer_id), None, None, None, None, None,
                      format!("Router #{id} sent RIP v2 response with {} entr{} ({} bytes) to router #{peer_id}{}",
                              entries, if entries == 1 { "y" } else { "ies" },
                              4 + 20 * entries,
                              if is_triggered { " (triggered)" } else { "" }));
        }
        if is_triggered { self.stats.triggered_updates += 1; }
        else { self.stats.periodic_updates += 1; }
    }

    pub(super) fn receive(&mut self, pck: Packet) {
        if !self.nodes.get(&pck.to).is_some_and(|r| r.is_running) { return; }
        if !self.nodes.get(&pck.to).is_some_and(|r| r.peers.contains_key(&pck.from)) { return; }
        let decoded = decode_res(&pck.payload, pck.from).and_then(|tbl| {
            if tbl.keys().any(|&dest| dest > self.cfg.nodes) {
                Err("RIP response contains unknown router destination".into())
            } else {
                Ok(tbl)
            }
        });
        let tbl = match decoded {
            Ok(tbl) => tbl,
            Err(_reason) => {
                self.stats.pcks_dropped += 1;
                self.emit(EventType::PacketDropped, pck.to, Some(pck.from), None, None, None, None, None,
                          format!("Router #{} discarded an invalid RIP packet from #{}",
                                  pck.to, pck.from));
                return;
            }
        };
        let entry_count = tbl.len();
        let mut is_recovered = false;
        let mut from_quiescence = false;
        if let Some(r) = self.nodes.get_mut(&pck.to) {
            if let Some(p) = r.peers.get_mut(&pck.from) {
                is_recovered = p.is_garbage || !p.is_active;
                p.is_active = true;
                p.is_garbage = false;
                p.last_seen = self.now;
            }
            if r.is_quiescent { r.is_quiescent = false; from_quiescence = true; }
            r.rcvd.insert(pck.from, tbl);
        }
        self.stats.updates_rcvd += 1;
        if is_recovered || from_quiescence {
            self.emit(EventType::Recovered, pck.to, Some(pck.from), None, None, None, None, None,
                      format!("Router #{} received RIP updates from router #{} again", pck.to, pck.from));
        }
        self.emit(EventType::Receive, pck.to, Some(pck.from), None, None, None, None, None,
                  format!("Router #{} received valid RIP v2 response from router #{} ({} entries)",
                          pck.to, pck.from, entry_count));
        if self.recompute(pck.to) { self.send_tbl(pck.to, true); }
    }

pub(super) fn recompute(&mut self, id: u16) -> bool {
        let Some(router) = self.nodes.get(&id) else { return false; };
        if !router.is_running { return false; }
        let mut dests = BTreeSet::new();
        dests.extend(router.routes.keys().copied());
        dests.extend(router.peers.keys().copied());
        for updates in router.rcvd.values() { dests.extend(updates.keys().copied()); }
        let mut changes: Vec<RouteChange> = Vec::new();
        let mut refresh: Vec<u16> = Vec::new();
        for dest in dests {
            if dest == id { continue; }
            let old = router.routes.get(&dest).cloned();
            let mut best: Option<(u16, u8)> = None;
            for (&peer_id, peer) in &router.peers {
                if !peer.is_active { continue; }
                let candidate = if peer_id == dest { Some(peer.cost) } else {
                    router.rcvd.get(&peer_id).and_then(|tbl| tbl.get(&dest))
                        .map(|metric| u16::from(*metric) + u16::from(peer.cost))
                        .and_then(|metric| (metric < u16::from(INF)).then_some(metric as u8))
                };
                if let Some(cost) = candidate
                    && cost < INF && best.is_none_or(|(best_hop, best_cost)|
                            cost < best_cost || (cost == best_cost && peer_id < best_hop)) {
                        best = Some((peer_id, cost));
                    }
            }
            match (old.as_ref(), best) {
                (None, None) => {},
                (Some(prev), None) if prev.metric == INF => {},
                (Some(prev), Some((hop, cost))) if prev.metric == cost && prev.nxt_hop == hop => {
                    refresh.push(dest);
                }
                _ => changes.push(RouteChange { dest, prev: old, best }),
            }
        }
        let mut is_triggered = false;
        for dest in refresh {
            if let Some(route) = self.nodes.get_mut(&id).and_then(|r| r.routes.get_mut(&dest)) {
                route.last_refreshed = self.now;
            }
        }
        for RouteChange { dest, prev, best } in changes {
            let (hop, metric, invalid_since) = match best {
                Some((hop, cost)) => (hop, cost, None),
                None => (prev.as_ref().map_or(0, |r| r.nxt_hop), INF, Some(self.now)),
            };
            if prev.is_none() && best.is_none() { continue; }
            if metric == INF && prev.as_ref().is_some_and(|r| r.metric == INF) { continue; }
            if let Some(node) = self.nodes.get_mut(&id) {
                if metric < INF { node.offline_dests.remove(&dest); }
                node.routes.insert(dest, Route { dest, nxt_hop: hop, metric,
                    last_refreshed: self.now, invalid_since });
            }
            let old_metric = prev.as_ref().map(|r| r.metric);
            let old_hop = prev.as_ref().map(|r| r.nxt_hop);
            let _type = if metric == INF { EventType::Invalidate }
                else if prev.is_none() || old_metric == Some(INF) { EventType::Discover } else { EventType::RouteChange };
            if metric == INF { is_triggered = true; self.stats.invalids += 1; }
            self.stats.route_changes += 1;
            let msg = if metric == INF {
                format!("Router #{id}: route to #{dest} via #{} invalidated ({} → 16)",
                        old_hop.unwrap_or(0), old_metric.unwrap_or(16))
            } else if prev.is_none() || old_metric == Some(INF) {
                format!("Router #{id}: discovered #{dest} via #{hop} (metric {metric})")
            } else {
                format!("Router #{id}: shortest path to #{dest} changed: #{} → #{hop}, metric {} → {metric}",
                        old_hop.unwrap_or(0), old_metric.unwrap_or(INF))
            };
            self.emit(_type, id, None, Some(dest), old_metric, Some(metric),
                      old_hop, Some(hop), msg);
        }
        is_triggered
    }
}
