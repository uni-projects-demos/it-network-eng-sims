use super::{EventType, Config, Topology, Rand, Sim, Peer, Router, Route, Event, Stats, gen_edges, default_edges, UPDATE_PERIOD, MIN_LINK_COST, MAX_LINK_COST, MAX_PCK_LOSS_PERC, MAX_PCK_INVALID_PERC};
use std::collections::BTreeMap;

impl Sim {
    pub(super) fn new(cfg: Config) -> Result<Self, String> {
        cfg.check()?;
        let mut rng = Rand(cfg.seed);
        let edges = match cfg.topology {
            Topology::Default => default_edges(),
            Topology::Rand => gen_edges(cfg, &mut rng),
        };
        let mut nodes = BTreeMap::new();
        for id in 1..=cfg.nodes {
            let mut peers = BTreeMap::new();
            let mut routes = BTreeMap::new();
            for edge in &edges {
                let other = if edge.a == id { Some(edge.b) }
                    else if edge.b == id { Some(edge.a) } else { None };
                if let Some(peer) = other {
                    peers.insert(peer, Peer { cost: edge.cost, last_seen: 0.0, is_active: true, is_garbage: false });
                    routes.insert(peer, Route {
                        dest: peer, nxt_hop: peer, metric: edge.cost,
                        last_refreshed: 0.0, invalid_since: None,
                    });
                }
            }
            nodes.insert(id, Router {
                id, is_running: true, is_quiescent: false, peers,
                rcvd: BTreeMap::new(), routes,
                offline_dests: Default::default(),
                nxt_periodic: UPDATE_PERIOD * (0.8 + rng.float() * 0.4),
                auto_restart_at: None,
            });
        }
        let mut sim = Self {
            cfg, now: 0.0, rng, nodes, edges, pending: Vec::new(),
            events: Vec::new(), stats: Stats::default(), topology_changed: 0.0,
        };
        for id in 1..=cfg.nodes {
            sim.emit(EventType::Startup, id, None, None, None, None, None, None,
                     format!("Router #{id} initialized and listening for RIP responses"));
            sim.send_tbl(id, false);
        }
        sim.measure();
        Ok(sim)
    }

#[allow(clippy::too_many_arguments)]
    pub(super) fn emit(&mut self, _type: EventType, router: u16, peer: Option<u16>, dest: Option<u16>,
            old_metric: Option<u8>, new_metric: Option<u8>, old_hop: Option<u16>,
            new_hop: Option<u16>, msg: String) {
        if let Some(node) = self.nodes.get(&router)
            && node.is_quiescent && _type != EventType::AllGarbage && _type != EventType::Recovered
                && _type != EventType::RouterDown && _type != EventType::RouterRestart { return; }
        self.events.push(Event {
            time: (self.now * 100.0).round() / 100.0,
            _type, router, peer, dest, old_metric,
            new_metric, old_hop, new_hop, msg,
        });
    }

    pub(super) fn set_pck_loss_perc(&mut self, perc: u8) -> bool {
        if perc > MAX_PCK_LOSS_PERC || self.cfg.is_finished_at(self.now)
            || perc == self.cfg.pck_loss_perc { return false; }
        self.cfg.pck_loss_perc = perc;
        self.emit(EventType::PacketLossSetting, 0, None, None, None, None, None, None,
                  format!("RIP packet loss probability set to {perc}%"));
        true
    }

    pub(super) fn set_pck_invalid_perc(&mut self, perc: u8) -> bool {
        if perc > MAX_PCK_INVALID_PERC || self.cfg.is_finished_at(self.now)
            || perc == self.cfg.pck_invalid_perc { return false; }
        self.cfg.pck_invalid_perc = perc;
        self.emit(EventType::PacketInvalidSetting, 0, None, None, None, None, None, None,
                  format!("Invalid RIP packet probability set to {perc}%"));
        true
    }

    pub(super) fn kill(&mut self, id: u16) -> bool { self.stop_router(id, None) }

    pub(super) fn stop_router(&mut self, id: u16, outage_secs: Option<f64>) -> bool {
        let Some(node) = self.nodes.get_mut(&id) else { return false; };
        if !node.is_running || self.cfg.is_finished_at(self.now) { return false; }
        node.is_running = false;
        node.is_quiescent = false;
        node.auto_restart_at = outage_secs.map(|dur| self.now + dur);
        self.pending.retain(|p| p.from != id);
        self.stats.fails += 1;
        self.topology_changed = self.now;
        self.stats.convergence_secs = None;
        match outage_secs {
            Some(dur) => {
                self.stats.rand_fails += 1;
                self.emit(EventType::AutoFailure, id, None, None, None, None, None, None,
                    format!("Router #{id}: random inactivity for {dur:.1}s; RIP updates suspended"));
            }
            None => self.emit(EventType::RouterDown, id, None, None, None, None, None, None,
                format!("Router #{id} terminated by user")),
        }
        self.measure();
        true
    }

    pub(super) fn restart(&mut self, id: u16) -> bool { self.start_router(id, false) }

    pub(super) fn start_router(&mut self, id: u16, is_auto: bool) -> bool {
        let Some(node) = self.nodes.get_mut(&id) else { return false; };
        if node.is_running || self.cfg.is_finished_at(self.now) { return false; }
        if is_auto && node.auto_restart_at.is_none() { return false; }
        node.is_running = true;
        node.is_quiescent = false;
        node.auto_restart_at = None;
        node.rcvd.clear();
        node.routes.clear();
        node.offline_dests.clear();
        for (&peer, p) in node.peers.iter_mut() {
            p.is_active = true;
            p.is_garbage = false;
            p.last_seen = self.now;
            node.routes.insert(peer, Route { dest: peer, nxt_hop: peer,
                metric: p.cost, last_refreshed: self.now, invalid_since: None });
        }
        node.nxt_periodic = self.now + UPDATE_PERIOD;
        self.stats.restarts += 1;
        if is_auto { self.stats.rand_recovs += 1; }
        self.topology_changed = self.now;
        self.stats.convergence_secs = None;
        if is_auto {
            self.emit(EventType::AutoRecovery, id, None, None, None, None, None, None,
                format!("Router #{id}: random inactivity ended; RIP updates resumed"));
        } else {
            self.emit(EventType::RouterRestart, id, None, None, None, None, None, None,
                format!("Router #{id} restarted and advertised its routing table"));
        }
        self.send_tbl(id, false);
        self.measure();
        true
    }

    pub(super) fn set_link_cost(&mut self, a: u16, b: u16, cost: u8) -> bool {
        if self.cfg.is_finished_at(self.now) || !(MIN_LINK_COST..=MAX_LINK_COST).contains(&cost) || a == b {
            return false;
        }
        let Some(edge) = self.edges.iter_mut().find(|edge|
            (edge.a == a && edge.b == b) || (edge.a == b && edge.b == a))
            else { return false; };
        if edge.cost == cost { return false; }
        let old = edge.cost;
        edge.cost = cost;
        for (router_id, peer_id) in [(a, b), (b, a)] {
            if let Some(peer) = self.nodes.get_mut(&router_id)
                .and_then(|router| router.peers.get_mut(&peer_id)) {
                peer.cost = cost;
            }
        }
        self.emit(EventType::LinkCost, 0, None, None, Some(old), Some(cost), None, None,
                  format!("Link R{a} ↔ R{b} cost changed: {old} → {cost}"));
        self.topology_changed = self.now;
        self.stats.convergence_secs = None;
        for router_id in [a, b] {
            if self.recompute(router_id) { self.send_tbl(router_id, true); }
        }
        self.measure();
        true
    }
}
