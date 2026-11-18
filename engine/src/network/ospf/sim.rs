use super::*;

impl Sim {
    pub(super) fn new(cfg: Config) -> Result<Self, String> {
        cfg.check()?;
        let mut rng = Rand(cfg.seed);
        let edges = match cfg.topology {
            super::super::routing::Topology::Default => default_edges(),
            super::super::routing::Topology::Rand => gen_edges(cfg, &mut rng),
        };
        let mut nodes = BTreeMap::new();
        for id in 1..=cfg.nodes {
            let peers = edges
                .iter()
                .filter_map(|edge| {
                    let other = if edge.a == id {
                        Some(edge.b)
                    } else if edge.b == id {
                        Some(edge.a)
                    } else {
                        None
                    };
                    other.map(|other| {
                        (
                            other,
                            Peer {
                                cost: edge.cost,
                                is_active: true,
                                last_hello: 0.0,
                            },
                        )
                    })
                })
                .collect();
            nodes.insert(
                id,
                Router {
                    id,
                    is_running: true,
                    peers,
                    database: BTreeMap::new(),
                    sequence: 0,
                    routes: BTreeMap::new(),
                    next_hello: HELLO_PERIOD,
                    next_refresh: LSA_REFRESH_PERIOD,
                    spf_due: None,
                    auto_restart_at: None,
                },
            );
        }
        let mut sim = Self {
            cfg,
            now: 0.0,
            rng,
            nodes,
            edges,
            pending: Vec::new(),
            events: Vec::new(),
            stats: Stats::default(),
            topology_changed: 0.0,
        };
        for id in 1..=cfg.nodes {
            sim.log(
                EventType::Startup,
                id,
                None,
                None,
                None,
                None,
                None,
                None,
                format!("Router #{id} initialized OSPF link-state database"),
            );
            sim.originate(id, false);
        }
        sim.measure();
        Ok(sim)
    }

    #[allow(clippy::too_many_arguments)]
    fn log(
        &mut self,
        _type: EventType,
        router: u16,
        peer: Option<u16>,
        dest: Option<u16>,
        old_metric: Option<u16>,
        new_metric: Option<u16>,
        old_hop: Option<u16>,
        new_hop: Option<u16>,
        msg: String,
    ) {
        self.events.push(Event {
            time: (self.now * 100.0).round() / 100.0,
            _type,
            router,
            peer,
            dest,
            old_metric,
            new_metric,
            old_hop,
            new_hop,
            msg,
        });
    }

    fn neighbors(&self, id: u16) -> Vec<u16> {
        self.nodes
            .get(&id)
            .filter(|router| router.is_running)
            .map_or_else(Vec::new, |router| {
                router
                    .peers
                    .iter()
                    .filter_map(|(&peer, state)| state.is_active.then_some(peer))
                    .collect()
            })
    }

    fn mark_changed(&mut self) {
        self.topology_changed = self.now;
        self.stats.is_converged = false;
        self.stats.convergence_secs = None;
    }

    fn schedule_spf(&mut self, id: u16) {
        if let Some(router) = self.nodes.get_mut(&id).filter(|router| router.is_running)
            && router.spf_due.is_none()
        {
            router.spf_due = Some(self.now + SPF_DELAY);
        }
    }

    fn send_message(&mut self, from: u16, to: u16, message: Message) {
        if !self.nodes.get(&from).is_some_and(|router| {
            router.is_running
                && router
                    .peers
                    .get(&to)
                    .is_some_and(|peer| peer.is_active || matches!(&message, Message::Hello))
        }) {
            return;
        }
        match &message {
            Message::Hello => self.stats.hello_sent += 1,
            Message::Lsa(lsa) => {
                self.stats.updates_sent += 1;
                self.log(
                    EventType::Send,
                    from,
                    Some(to),
                    None,
                    None,
                    None,
                    None,
                    None,
                    format!(
                        "Router #{from} flooded LSA #{} sequence {} to router #{to}",
                        lsa.origin, lsa.sequence
                    ),
                );
            }
        }
        self.pending.push(Packet {
            from,
            to,
            at: self.now + DELIVERY_DELAY,
            message,
        });
    }

    fn send_hello(&mut self, id: u16) {
        let peers: Vec<u16> = self
            .nodes
            .get(&id)
            .map_or_else(Vec::new, |router| router.peers.keys().copied().collect());
        for peer in peers {
            self.send_message(id, peer, Message::Hello);
        }
    }

    fn send_database(&mut self, from: u16, to: u16) {
        let lsas: Vec<Lsa> = self.nodes.get(&from).map_or_else(Vec::new, |router| {
            router.database.values().cloned().collect()
        });
        for lsa in lsas {
            self.send_message(from, to, Message::Lsa(lsa));
        }
    }

    fn originate(&mut self, id: u16, triggered: bool) {
        let neighbors = self.neighbors(id);
        let Some(router) = self.nodes.get_mut(&id).filter(|router| router.is_running) else {
            return;
        };
        router.sequence += 1;
        let lsa = Lsa {
            origin: id,
            sequence: router.sequence,
            links: router
                .peers
                .iter()
                .filter_map(|(&peer, state)| state.is_active.then_some((peer, state.cost)))
                .collect(),
        };
        router.database.insert(id, lsa.clone());
        self.schedule_spf(id);
        if triggered {
            self.stats.triggered_updates += 1;
        } else {
            self.stats.periodic_updates += 1;
        }
        for peer in neighbors {
            self.send_message(id, peer, Message::Lsa(lsa.clone()));
        }
    }

    fn receive(&mut self, packet: Packet) {
        let eligible =
            self.nodes
                .get(&packet.from)
                .is_some_and(|router| router.is_running)
                && self.nodes.get(&packet.to).is_some_and(|router| {
                    router.is_running && router.peers.contains_key(&packet.from)
                });
        if !eligible {
            return;
        }
        if self.cfg.pck_loss_perc > 0
            && self.rng.float() < f64::from(self.cfg.pck_loss_perc) / 100.0
        {
            self.stats.pcks_lost += 1;
            self.log(
                EventType::PacketLost,
                packet.to,
                Some(packet.from),
                None,
                None,
                None,
                None,
                None,
                format!(
                    "Router #{}: OSPF packet from #{} lost",
                    packet.to, packet.from
                ),
            );
            return;
        }
        if self.cfg.pck_invalid_perc > 0
            && self.rng.float() < f64::from(self.cfg.pck_invalid_perc) / 100.0
        {
            self.stats.pcks_dropped += 1;
            self.log(
                EventType::PacketDropped,
                packet.to,
                Some(packet.from),
                None,
                None,
                None,
                None,
                None,
                format!(
                    "Router #{} dropped invalid OSPF packet from #{}",
                    packet.to, packet.from
                ),
            );
            return;
        }
        let recovered = if let Some(peer) = self
            .nodes
            .get_mut(&packet.to)
            .and_then(|router| router.peers.get_mut(&packet.from))
        {
            let recovered = !peer.is_active;
            peer.is_active = true;
            peer.last_hello = self.now;
            recovered
        } else {
            false
        };
        if recovered {
            self.stats.adjacency_changes += 1;
            self.log(
                EventType::AdjacencyUp,
                packet.to,
                Some(packet.from),
                None,
                None,
                None,
                None,
                None,
                format!(
                    "Router #{}: adjacency to #{} restored",
                    packet.to, packet.from
                ),
            );
            self.originate(packet.to, true);
            self.send_database(packet.to, packet.from);
            self.send_hello(packet.to);
        }
        match packet.message {
            Message::Hello => {
                self.stats.hello_rcvd += 1;
            }
            Message::Lsa(lsa) => {
                if lsa.origin == 0
                    || lsa.origin > self.cfg.nodes
                    || lsa.sequence == 0
                    || lsa.links.iter().any(|(&peer, &cost)| {
                        peer == lsa.origin
                            || peer == 0
                            || peer > self.cfg.nodes
                            || !(MIN_LINK_COST..=MAX_LINK_COST).contains(&cost)
                    })
                {
                    self.stats.pcks_dropped += 1;
                    self.log(
                        EventType::PacketDropped,
                        packet.to,
                        Some(packet.from),
                        None,
                        None,
                        None,
                        None,
                        None,
                        format!("Router #{} rejected malformed LSA", packet.to),
                    );
                    return;
                }
                self.stats.updates_rcvd += 1;
                let newer = self
                    .nodes
                    .get(&packet.to)
                    .and_then(|router| router.database.get(&lsa.origin))
                    .is_none_or(|current| lsa.sequence > current.sequence);
                if !newer {
                    self.stats.duplicate_lsas += 1;
                    return;
                }
                self.nodes
                    .get_mut(&packet.to)
                    .expect("receiver exists")
                    .database
                    .insert(lsa.origin, lsa.clone());
                self.log(
                    EventType::Receive,
                    packet.to,
                    Some(packet.from),
                    Some(lsa.origin),
                    None,
                    None,
                    None,
                    None,
                    format!(
                        "Router #{} received LSA #{} sequence {} from #{}",
                        packet.to, lsa.origin, lsa.sequence, packet.from
                    ),
                );
                self.schedule_spf(packet.to);
                for peer in self.neighbors(packet.to) {
                    if peer != packet.from {
                        self.send_message(packet.to, peer, Message::Lsa(lsa.clone()));
                    }
                }
            }
        }
    }

    fn calculate_spf(&mut self, id: u16) {
        let Some(router) = self.nodes.get_mut(&id) else {
            return;
        };
        router.spf_due = None;
        if !router.is_running {
            return;
        }
        let mut edges = Vec::new();
        for (&origin, lsa) in &router.database {
            for (&peer, &cost) in &lsa.links {
                if origin >= peer {
                    continue;
                }
                let reciprocal = router
                    .database
                    .get(&peer)
                    .and_then(|other| other.links.get(&origin));
                if reciprocal == Some(&cost) {
                    edges.push(Edge {
                        a: origin,
                        b: peer,
                        cost,
                        can_increase: false,
                        can_decrease: false,
                    });
                }
            }
        }
        let ids = (1..=self.cfg.nodes).collect::<BTreeSet<_>>();
        let computed = shortest_paths(&ids, &edges, id);
        let next: BTreeMap<u16, Route> = computed
            .into_iter()
            .map(|(dest, path)| {
                (
                    dest,
                    Route {
                        dest,
                        nxt_hop: path.next_hop,
                        metric: path.cost,
                    },
                )
            })
            .collect();
        let before = std::mem::replace(&mut router.routes, next.clone());
        self.stats.spf_runs += 1;
        self.log(
            EventType::Spf,
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            format!("Router #{id} recalculated shortest paths from its link-state database"),
        );
        let destinations: BTreeSet<u16> = before.keys().chain(next.keys()).copied().collect();
        for dest in destinations {
            let old = before.get(&dest);
            let new = next.get(&dest);
            if old == new {
                continue;
            }
            self.stats.route_changes += 1;
            let kind = match (old, new) {
                (None, Some(_)) => EventType::Discover,
                (Some(_), None) => EventType::Invalidate,
                _ => EventType::RouteChange,
            };
            self.log(
                kind,
                id,
                None,
                Some(dest),
                old.map(|r| r.metric),
                new.map(|r| r.metric),
                old.map(|r| r.nxt_hop),
                new.map(|r| r.nxt_hop),
                format!(
                    "Router #{id}: route to #{dest} {}",
                    match new {
                        Some(route) => format!("via #{} cost {}", route.nxt_hop, route.metric),
                        None => "withdrawn".into(),
                    }
                ),
            );
        }
    }

    fn timers(&mut self) {
        let ids: Vec<u16> = self.nodes.keys().copied().collect();
        for id in ids {
            if !self.nodes.get(&id).is_some_and(|router| router.is_running) {
                continue;
            }
            let expired: Vec<u16> = self.nodes[&id]
                .peers
                .iter()
                .filter_map(|(&peer, state)| {
                    (state.is_active && self.now - state.last_hello + EPS >= ROUTER_DEAD_INTERVAL)
                        .then_some(peer)
                })
                .collect();
            if !expired.is_empty() {
                for peer in expired {
                    self.nodes
                        .get_mut(&id)
                        .expect("router exists")
                        .peers
                        .get_mut(&peer)
                        .expect("peer exists")
                        .is_active = false;
                    self.stats.adjacency_changes += 1;
                    self.log(
                        EventType::AdjacencyDown,
                        id,
                        Some(peer),
                        None,
                        None,
                        None,
                        None,
                        None,
                        format!("Router #{id}: adjacency with #{peer} timed out"),
                    );
                }
                self.originate(id, true);
            }
            let hello_due = self.nodes[&id].next_hello <= self.now + EPS;
            if hello_due {
                self.nodes.get_mut(&id).expect("router exists").next_hello += HELLO_PERIOD;
                self.send_hello(id);
            }
            let refresh_due = self.nodes[&id].next_refresh <= self.now + EPS;
            if refresh_due {
                self.nodes.get_mut(&id).expect("router exists").next_refresh += LSA_REFRESH_PERIOD;
                self.log(
                    EventType::Periodic,
                    id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    format!("Router #{id}: periodic LSA refresh"),
                );
                self.originate(id, false);
            }
            if self.nodes[&id]
                .spf_due
                .is_some_and(|due| due <= self.now + EPS)
            {
                self.calculate_spf(id);
            }
        }
    }

    pub(super) fn advance(&mut self, delta: f64) {
        if !delta.is_finite() || delta <= 0.0 || self.cfg.is_finished_at(self.now) {
            return;
        }
        let end = self
            .cfg
            .end_time()
            .map_or(self.now + delta.min(MAX_ADVANCE), |limit| {
                (self.now + delta.min(MAX_ADVANCE)).min(limit)
            });
        while self.now + EPS < end {
            let tick_secs = (end - self.now).min(CLOCK_TICK);
            self.now += tick_secs;
            let (due, waiting): (Vec<_>, Vec<_>) = std::mem::take(&mut self.pending)
                .into_iter()
                .partition(|packet| packet.at <= self.now + EPS);
            self.pending = waiting;
            for packet in due {
                self.receive(packet);
            }
            self.timers();
            self.random_faults(tick_secs);
        }
        self.measure();
        if self.cfg.is_finished_at(self.now) {
            self.log(
                EventType::Complete,
                0,
                None,
                None,
                None,
                None,
                None,
                None,
                format!("Simulation reached the {:.0}-second limit", self.now),
            );
        }
    }

    fn random_faults(&mut self, tick_secs: f64) {
        let due: Vec<u16> = self
            .nodes
            .values()
            .filter_map(|router| {
                (!router.is_running
                    && router
                        .auto_restart_at
                        .is_some_and(|at| at <= self.now + EPS))
                .then_some(router.id)
            })
            .collect();
        for id in &due {
            self.start_router(*id, true);
        }
        let candidates: Vec<u16> = self
            .nodes
            .values()
            .filter(|router| router.is_running && !due.contains(&router.id))
            .map(|router| router.id)
            .collect();
        for id in candidates {
            if should_fail(&mut self.rng, self.cfg.rand_fail_perc, tick_secs) {
                let outage = random_outage(&mut self.rng);
                self.stop_router(id, Some(outage));
            }
        }
    }

    fn measure(&mut self) {
        let alive = self
            .nodes
            .values()
            .filter(|router| router.is_running)
            .map(|router| router.id)
            .collect::<BTreeSet<_>>();
        let mut correct = 0usize;
        let mut total = 0usize;
        let mut reachable = 0usize;
        let mut costs = Vec::new();
        for &source in &alive {
            let ideal = shortest_paths(&alive, &self.edges, source);
            for dest in 1..=self.cfg.nodes {
                if dest == source {
                    continue;
                }
                total += 1;
                let actual = self.nodes[&source].routes.get(&dest);
                let expected = ideal.get(&dest);
                if actual.map(|route| route.metric) == expected.map(|path| path.cost) {
                    correct += 1;
                }
                if let Some(path) = expected {
                    costs.push(f64::from(path.cost));
                    if actual.is_some_and(|route| route.metric == path.cost) {
                        reachable += 1;
                    }
                }
            }
        }
        self.stats.accuracy = if total == 0 {
            1.0
        } else {
            correct as f64 / total as f64
        };
        self.stats.reachable_pairs = costs.len();
        self.stats.ospf_reachability = if costs.is_empty() {
            1.0
        } else {
            reachable as f64 / costs.len() as f64
        };
        self.stats.shortest_path_cost_min =
            costs.iter().copied().reduce(f64::min).unwrap_or(0.0) as u16;
        self.stats.shortest_path_cost_max =
            costs.iter().copied().reduce(f64::max).unwrap_or(0.0) as u16;
        self.stats.shortest_path_cost_mean = if costs.is_empty() {
            0.0
        } else {
            costs.iter().sum::<f64>() / costs.len() as f64
        };
        let converged = correct == total;
        if !converged {
            self.stats.convergence_secs = None;
        }
        if converged && !self.stats.is_converged {
            let secs = (self.now - self.topology_changed).max(0.0);
            self.stats.convergence_secs = Some(secs);
            self.log(
                EventType::Converged,
                0,
                None,
                None,
                None,
                None,
                None,
                None,
                format!("OSPF network converged after {secs:.2}s"),
            );
        }
        self.stats.is_converged = converged;
    }

    pub(super) fn take_snapshot(&mut self) -> Snapshot {
        let nodes = self
            .nodes
            .values()
            .map(|router| NodeView {
                id: router.id,
                is_running: router.is_running,
                is_quiescent: router.is_running && router.peers.values().all(|p| !p.is_active),
                auto_restart_in: router.auto_restart_at.map(|at| (at - self.now).max(0.0)),
                active_nbrs: router.peers.values().filter(|p| p.is_active).count(),
                garbage_nbrs: router.peers.values().filter(|p| !p.is_active).count(),
                routes: router.routes.values().cloned().collect(),
                offline_dests: (1..=self.cfg.nodes)
                    .filter(|dest| *dest != router.id && !router.routes.contains_key(dest))
                    .collect(),
                known_lsas: router.database.len(),
            })
            .collect();
        Snapshot {
            time: self.now,
            dur: f64::from(self.cfg.to_secs),
            hist_start: 0.0,
            is_complete: self.cfg.is_finished_at(self.now),
            nodes,
            edges: self
                .edges
                .iter()
                .map(|edge| Edge {
                    a: edge.a,
                    b: edge.b,
                    cost: edge.cost,
                    can_increase: edge.cost < MAX_LINK_COST,
                    can_decrease: edge.cost > MIN_LINK_COST,
                })
                .collect(),
            events: std::mem::take(&mut self.events),
            stats: self.stats.clone(),
        }
    }

    pub(super) fn kill(&mut self, id: u16) -> bool {
        self.stop_router(id, None)
    }

    fn stop_router(&mut self, id: u16, outage: Option<f64>) -> bool {
        let Some(router) = self.nodes.get_mut(&id).filter(|router| router.is_running) else {
            return false;
        };
        if self.cfg.is_finished_at(self.now) {
            return false;
        }
        router.is_running = false;
        router.auto_restart_at = outage.map(|secs| self.now + secs);
        router.routes.clear();
        router.spf_due = None;
        self.pending
            .retain(|packet| packet.from != id && packet.to != id);
        self.stats.fails += 1;
        if outage.is_some() {
            self.stats.rand_fails += 1;
        }
        self.mark_changed();
        self.log(
            if outage.is_some() {
                EventType::AutoFailure
            } else {
                EventType::RouterDown
            },
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            format!("Router #{id}: OSPF process stopped"),
        );
        self.measure();
        true
    }

    pub(super) fn restart(&mut self, id: u16) -> bool {
        self.start_router(id, false)
    }

    fn start_router(&mut self, id: u16, automatic: bool) -> bool {
        let Some(router) = self.nodes.get_mut(&id).filter(|router| !router.is_running) else {
            return false;
        };
        if self.cfg.is_finished_at(self.now) || (automatic && router.auto_restart_at.is_none()) {
            return false;
        }
        router.is_running = true;
        router.auto_restart_at = None;
        router.database.clear();
        router.routes.clear();
        router.spf_due = None;
        router.next_hello = self.now + HELLO_PERIOD;
        router.next_refresh = self.now + LSA_REFRESH_PERIOD;
        for peer in router.peers.values_mut() {
            peer.is_active = false;
            peer.last_hello = self.now;
        }
        self.stats.restarts += 1;
        if automatic {
            self.stats.rand_recovs += 1;
        }
        self.mark_changed();
        self.log(
            if automatic {
                EventType::AutoRecovery
            } else {
                EventType::RouterRestart
            },
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            format!("Router #{id}: OSPF process restarted"),
        );
        self.originate(id, true);
        self.send_hello(id);
        self.measure();
        true
    }

    pub(super) fn set_link_cost(&mut self, a: u16, b: u16, cost: u8) -> bool {
        if self.cfg.is_finished_at(self.now)
            || !(MIN_LINK_COST..=MAX_LINK_COST).contains(&cost)
            || a == b
        {
            return false;
        }
        let Some(edge) = self
            .edges
            .iter_mut()
            .find(|edge| (edge.a == a && edge.b == b) || (edge.a == b && edge.b == a))
        else {
            return false;
        };
        if edge.cost == cost {
            return false;
        }
        let old = edge.cost;
        edge.cost = cost;
        for (id, peer) in [(a, b), (b, a)] {
            if let Some(state) = self
                .nodes
                .get_mut(&id)
                .and_then(|router| router.peers.get_mut(&peer))
            {
                state.cost = cost;
            }
        }
        self.mark_changed();
        self.log(
            EventType::LinkCost,
            0,
            None,
            None,
            Some(u16::from(old)),
            Some(u16::from(cost)),
            None,
            None,
            format!("Link R{a} ↔ R{b} cost changed: {old} → {cost}"),
        );
        for id in [a, b] {
            self.originate(id, true);
        }
        self.measure();
        true
    }

    pub(super) fn set_pck_loss_perc(&mut self, perc: u8) -> bool {
        if self.cfg.is_finished_at(self.now)
            || perc > MAX_PCK_LOSS_PERC
            || perc == self.cfg.pck_loss_perc
        {
            return false;
        }
        self.cfg.pck_loss_perc = perc;
        self.log(
            EventType::PacketLossSetting,
            0,
            None,
            None,
            None,
            None,
            None,
            None,
            format!("OSPF packet loss probability set to {perc}%"),
        );
        true
    }

    pub(super) fn set_pck_invalid_perc(&mut self, perc: u8) -> bool {
        if self.cfg.is_finished_at(self.now)
            || perc > MAX_PCK_INVALID_PERC
            || perc == self.cfg.pck_invalid_perc
        {
            return false;
        }
        self.cfg.pck_invalid_perc = perc;
        self.log(
            EventType::PacketInvalidSetting,
            0,
            None,
            None,
            None,
            None,
            None,
            None,
            format!("Invalid OSPF packet probability set to {perc}%"),
        );
        true
    }
}
