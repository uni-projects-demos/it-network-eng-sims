use super::{EventType, Sim, INF, EPS, UPDATE_PERIOD, TO_FACTOR, GARBAGE_FACTOR, CLOCK_TICK, MAX_ADVANCE};

impl Sim {
    fn timers(&mut self) {
        let timeout = UPDATE_PERIOD * TO_FACTOR;
        let collect = UPDATE_PERIOD * GARBAGE_FACTOR;
        let ids: Vec<u16> = self.nodes.keys().copied().collect();
        for id in ids {
            if !self.nodes.get(&id).is_some_and(|r| r.is_running) { continue; }
            let mut expired = Vec::new();
            let mut collected = Vec::new();
            let mut is_all_garbage = false;
            if let Some(node) = self.nodes.get_mut(&id) {
                for (&peer, p) in node.peers.iter_mut() {
                    if p.is_active && self.now - p.last_seen + EPS >= timeout {
                        p.is_active = false;
                        node.rcvd.remove(&peer);
                        expired.push(peer);
                    }
                    if !p.is_garbage && self.now - p.last_seen + EPS >= timeout + collect {
                        p.is_garbage = true;
                        collected.push(peer);
                    }
                }
                if !node.is_quiescent && !node.peers.is_empty() && node.peers.values().all(|p| p.is_garbage) {
                    node.is_quiescent = true;
                    is_all_garbage = true;
                }
            }
            for peer in expired {
                self.emit(EventType::Timeout, id, Some(peer), None, None, None, None, None,
                          format!("Router #{id}: neighbour #{peer} timed-out after {timeout:.0}s."));
            }
            if is_all_garbage {
                self.emit(EventType::AllGarbage, id, None, None, None, None, None, None,
                          format!("Router #{id}: all direct neighbours garbage-collected."));
            } else {
                for peer in collected {
                    self.emit(EventType::NeighborGarbage, id, Some(peer), None, None, None, None, None,
                              format!("Router #{id}: neighbour #{peer} garbage-collected"));
                }
            }
            if self.recompute(id) { self.send_tbl(id, true); }
            let mut gone = Vec::new();
            let now = self.now;
            if let Some(node) = self.nodes.get_mut(&id) {
                node.routes.retain(|&dest, route| {
                    let gc = route.metric == INF && route.invalid_since.is_some_and(|since| now - since + EPS >= collect);
                    if gc { gone.push(dest); }
                    !gc
                });
            }
            for dest in gone {
                if let Some(node) = self.nodes.get_mut(&id) { node.offline_dests.insert(dest); }
                self.stats.garbage_collections += 1;
                self.emit(EventType::RouteGarbage, id, None, Some(dest), None, None, None, None,
                          format!("Router #{id}: route to #{dest} garbage-collected"));
            }
            let due = self.nodes.get(&id).is_some_and(|r| self.now + EPS >= r.nxt_periodic);
            if due {
                let nxt = self.now + UPDATE_PERIOD * (0.8 + 0.4 * self.rng.float());
                if let Some(node) = self.nodes.get_mut(&id) { node.nxt_periodic = nxt; }
                if self.nodes.get(&id).is_some_and(|r| !r.is_quiescent) {
                    self.emit(EventType::Periodic, id, None, None, None, None, None, None,
                              format!("Router #{id}: periodic RIP routing-table update"));
                    self.send_tbl(id, false);
                }
            }
        }
    }

pub(super) fn advance(&mut self, delta: f64) {
        if !delta.is_finite() || delta <= 0.0 || self.cfg.is_finished_at(self.now) { return; }
        let end = self.cfg.end_time().map_or(self.now + delta.min(MAX_ADVANCE),
            |limit| (self.now + delta.min(MAX_ADVANCE)).min(limit));
        while self.now + EPS < end {
            let tick_secs = (end - self.now).min(CLOCK_TICK);
            self.now += tick_secs;
            let (due, waiting): (Vec<_>, Vec<_>) = std::mem::take(&mut self.pending)
                .into_iter().partition(|pck| pck.at <= self.now + EPS);
            self.pending = waiting;
            for mut pck in due {
                let eligible = self.nodes.get(&pck.to).is_some_and(|router|
                    router.is_running && router.peers.contains_key(&pck.from));
                let lost = eligible && self.cfg.pck_loss_perc > 0
                    && self.rng.float() < f64::from(self.cfg.pck_loss_perc) / 100.0;
                if lost {
                    self.stats.pcks_lost += 1;
                    self.emit(EventType::PacketLost, pck.to, Some(pck.from),
                              None, None, None, None, None,
                              format!("Router #{}: RIP response from #{} lost in transit",
                                      pck.to, pck.from));
                } else {
                    if eligible && self.cfg.pck_invalid_perc > 0
                        && self.rng.float() < f64::from(self.cfg.pck_invalid_perc) / 100.0
                        && pck.payload.len() >= 2 { pck.payload[1] = 0; }
                    self.receive(pck);
                }
            }
            self.timers();
            self.rand_faults(tick_secs);
        }
        self.measure();
        if self.cfg.is_finished_at(self.now) {
            self.emit(EventType::Complete, 0, None, None, None, None, None, None,
                      format!("Simulation reached the {:.0}-minute ({:.0}-second) limit", f64::from(self.cfg.to_secs) / 60.0, f64::from(self.cfg.to_secs)));
        }
    }
}
