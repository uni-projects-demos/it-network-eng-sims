use super::model::{
    Config, Event, Frame, LinkView, Mac, Node, NodeView, Snapshot, Stats, Transmission,
};
use super::radio::{packet_failure_prob, snr_db};
use super::{EPS, Rand};
use std::collections::VecDeque;

const SLOT_SECS: f64 = 0.1;
const DUTY_SLOTS: u64 = 20;
const MAX_QUEUE: usize = 12;
const MAX_RETRIES: u8 = 3;
const MIN_SNR_DB: f64 = 5.0;
const TX_MW: f64 = 75.0;
const RX_MW: f64 = 45.0;
const IDLE_MW: f64 = 12.0;
const SLEEP_MW: f64 = 0.5;

#[derive(Clone)]
pub(super) struct Sim {
    pub(super) cfg: Config,
    pub(super) now: f64,
    pub(super) rng: Rand,
    pub(super) nodes: Vec<Node>,
    pub(super) events: Vec<Event>,
    pub(super) stats: Stats,
    pub(super) transmissions: Vec<Transmission>,
    next_slot: f64,
    slots: u64,
}

impl Sim {
    pub(super) fn new(cfg: Config) -> Result<Self, String> {
        cfg.check()?;
        let mut rng = Rand(cfg.seed);
        let mut nodes = Vec::with_capacity(usize::from(cfg.nodes) + 1);
        nodes.push(Node {
            id: 0,
            x: 210.0,
            y: 150.0,
            battery_mj: cfg.initial_battery_mj,
            enabled: true,
            queue: VecDeque::new(),
            next_sample: f64::INFINITY,
            backoff: 0,
            next_hop: None,
            route_cost: Some(0.0),
            awake: true,
        });
        for id in 1..=cfg.nodes {
            let sensor = Self::new_sensor(&cfg, &mut rng, &nodes, id, 0.0);
            nodes.push(sensor);
        }
        let mut sim = Self {
            cfg,
            now: 0.0,
            rng,
            nodes,
            events: Vec::new(),
            stats: Stats::default(),
            transmissions: Vec::new(),
            next_slot: SLOT_SECS,
            slots: 0,
        };
        sim.update_routes();
        sim.log("startup", 0, None, "Wireless sensor network started");
        Ok(sim)
    }

    fn new_sensor(cfg: &Config, rng: &mut Rand, existing: &[Node], id: u16, now: f64) -> Node {
        let (mut x, mut y, mut best) = (210.0, 150.0, f64::NEG_INFINITY);
        for _ in 0..320 {
            let cx = 45.0 + rng.float() * 330.0;
            let cy = 40.0 + rng.float() * 220.0;
            let closest = existing
                .iter()
                .map(|node| (node.x - cx).hypot(node.y - cy))
                .fold(f64::INFINITY, f64::min);
            let border = (cx - 45.0).min(375.0 - cx).min(cy - 40.0).min(260.0 - cy);
            let score = closest + 0.04 * border + rng.float() * 3.0;
            if score > best {
                (x, y, best) = (cx, cy, score);
            }
        }
        Node {
            id,
            x,
            y,
            battery_mj: cfg.initial_battery_mj,
            enabled: true,
            queue: VecDeque::new(),
            next_sample: now + 0.5 + rng.float() * cfg.data_interval_secs,
            backoff: 0,
            next_hop: None,
            route_cost: None,
            awake: true,
        }
    }

    fn log(&mut self, kind: &str, node: u16, peer: Option<u16>, message: &str) {
        self.events.push(Event {
            time: self.now,
            _type: kind.to_owned(),
            node,
            peer,
            msg: message.to_owned(),
        });
    }

    fn snr(&self, a: usize, b: usize) -> f64 {
        let x = self.nodes[a].x - self.nodes[b].x;
        let y = self.nodes[a].y - self.nodes[b].y;
        snr_db(self.cfg.tx_power_dbm, self.cfg.noise_floor_dbm, x.hypot(y))
    }

    fn connected(&self, a: usize, b: usize) -> bool {
        a != b && self.nodes[a].alive() && self.nodes[b].alive() && self.snr(a, b) >= MIN_SNR_DB
    }

    fn update_routes(&mut self) {
        let count = self.nodes.len();
        let mut distance = vec![f64::INFINITY; count];
        let mut visited = vec![false; count];
        let mut parent = vec![None; count];
        distance[0] = 0.0;
        for _ in 0..count {
            let selected = (0..count)
                .filter(|&idx| !visited[idx])
                .min_by(|&a, &b| distance[a].total_cmp(&distance[b]));
            let Some(current) = selected else { break };
            if !distance[current].is_finite() {
                break;
            }
            visited[current] = true;
            for neighbor in 1..count {
                if visited[neighbor] || !self.connected(current, neighbor) {
                    continue;
                }
                let cost = 1.0 + (12.0 - self.snr(current, neighbor)).max(0.0) * 0.15;
                if distance[current] + cost + EPS < distance[neighbor] {
                    distance[neighbor] = distance[current] + cost;
                    parent[neighbor] = Some(current as u16);
                }
            }
        }
        for id in 1..count {
            let next_hop = if self.nodes[id].alive() {
                parent[id]
            } else {
                None
            };
            self.nodes[id].route_cost = if next_hop.is_some() {
                Some(distance[id])
            } else {
                None
            };
            if self.nodes[id].next_hop != next_hop {
                self.nodes[id].next_hop = next_hop;
                self.log("route", id as u16, next_hop, "Next hop updated");
            }
        }
    }

    fn is_awake(&self, idx: usize) -> bool {
        if idx == 0 {
            return true;
        }
        let phase = (self.slots + idx as u64 * 7) % DUTY_SLOTS;
        phase < (DUTY_SLOTS * u64::from(self.cfg.duty_cycle_pct) / 100).max(1)
    }

    fn expend(&mut self, idx: usize, mw: f64) {
        if idx == 0 || !self.nodes[idx].alive() {
            return;
        }
        let amount = mw * SLOT_SECS;
        let consumed = amount.min(self.nodes[idx].battery_mj);
        self.nodes[idx].battery_mj -= consumed;
        self.stats.energy_mj += consumed;
        if self.nodes[idx].battery_mj <= EPS {
            self.nodes[idx].battery_mj = 0.0;
            self.stats.dropped += self.nodes[idx].queue.len() as u64;
            self.nodes[idx].queue.clear();
            if self.stats.first_depletion_secs.is_none() {
                self.stats.first_depletion_secs = Some(self.now);
            }
            self.log("depleted", idx as u16, None, "Battery depleted");
        }
    }

    fn generate(&mut self) {
        for id in 1..self.nodes.len() {
            if !self.nodes[id].alive() {
                continue;
            }
            if self.now + EPS < self.nodes[id].next_sample {
                continue;
            }
            self.nodes[id].next_sample += self.cfg.data_interval_secs;
            self.stats.generated += 1;
            if self.nodes[id].queue.len() < MAX_QUEUE {
                self.nodes[id].queue.push_back(Frame {
                    source: id as u16,
                    born: self.now,
                    retries: 0,
                });
                self.log("sample", id as u16, None, "Sensor sample queued");
            } else {
                self.stats.dropped += 1;
                self.log("drop", id as u16, None, "Queue full");
            }
        }
    }

    fn retry(&mut self, sender: usize, receiver: u16, outcome: &str) {
        let exceeded = self.nodes[sender]
            .queue
            .front()
            .is_some_and(|frame| frame.retries >= MAX_RETRIES);
        if exceeded {
            self.nodes[sender].queue.pop_front();
            self.stats.dropped += 1;
            self.log(
                "drop",
                sender as u16,
                Some(receiver),
                "Retry limit exceeded",
            );
        } else {
            let backoff = if self.cfg.mac == Mac::Csma {
                self.rng.range(1, 4) as u8
            } else {
                0
            };
            if let Some(frame) = self.nodes[sender].queue.front_mut() {
                frame.retries += 1;
                self.stats.retries += 1;
                self.nodes[sender].backoff = backoff;
            }
        }
        self.transmissions.push(Transmission {
            from: sender as u16,
            to: receiver,
            outcome: outcome.to_owned(),
        });
    }

    fn transmit(&mut self, senders: &[usize], awake: &[bool], radio: &mut [f64]) {
        for &sender in senders {
            let Some(receiver) = self.nodes[sender].next_hop.map(usize::from) else {
                continue;
            };
            self.stats.transmissions += 1;
            radio[sender] = TX_MW;
            if awake[receiver] && self.nodes[receiver].alive() {
                radio[receiver] = radio[receiver].max(RX_MW);
            }
        }
        for &sender in senders {
            let Some(receiver) = self.nodes[sender].next_hop.map(usize::from) else {
                continue;
            };
            let receiver_id = receiver as u16;
            if !awake[receiver] || !self.nodes[receiver].alive() {
                self.log(
                    "retry",
                    sender as u16,
                    Some(receiver_id),
                    "Receiver sleeping",
                );
                self.retry(sender, receiver_id, "sleep");
                continue;
            }
            let collision = senders
                .iter()
                .any(|&other| other != sender && self.snr(other, receiver) >= MIN_SNR_DB);
            if collision {
                self.stats.collisions += 1;
                self.log(
                    "collision",
                    sender as u16,
                    Some(receiver_id),
                    "Frame collision",
                );
                self.retry(sender, receiver_id, "collision");
                continue;
            }
            if self.rng.float() < packet_failure_prob(self.snr(sender, receiver)) {
                self.stats.errors += 1;
                self.log(
                    "error",
                    sender as u16,
                    Some(receiver_id),
                    "Radio packet error",
                );
                self.retry(sender, receiver_id, "error");
                continue;
            }
            let Some(frame) = self.nodes[sender].queue.pop_front() else {
                continue;
            };
            if receiver == 0 {
                self.stats.delivered += 1;
                self.stats.latency_total_secs += (self.now - frame.born).max(0.0);
                self.log(
                    "deliver",
                    frame.source,
                    Some(0),
                    "Sample delivered to gateway",
                );
            } else if self.nodes[receiver].queue.len() >= MAX_QUEUE {
                self.stats.dropped += 1;
                self.log("drop", receiver_id, None, "Relay queue full");
            } else {
                self.nodes[receiver].queue.push_back(Frame {
                    retries: 0,
                    ..frame
                });
                self.stats.forwarded += 1;
                self.log(
                    "forward",
                    sender as u16,
                    Some(receiver_id),
                    "Frame forwarded",
                );
            }
            self.transmissions.push(Transmission {
                from: sender as u16,
                to: receiver_id,
                outcome: "success".to_owned(),
            });
        }
    }

    fn slot(&mut self) {
        self.slots += 1;
        self.transmissions.clear();
        self.update_routes();
        self.generate();
        let mut awake = vec![false; self.nodes.len()];
        let mut radio = vec![0.0; self.nodes.len()];
        for id in 0..self.nodes.len() {
            awake[id] = self.nodes[id].alive() && self.is_awake(id);
            self.nodes[id].awake = awake[id];
            if id > 0 && self.nodes[id].alive() {
                radio[id] = if awake[id] { IDLE_MW } else { SLEEP_MW };
            }
        }
        let mut ready: Vec<usize> = (1..self.nodes.len())
            .filter(|&id| {
                awake[id] && self.nodes[id].next_hop.is_some() && !self.nodes[id].queue.is_empty()
            })
            .collect();
        let senders = match self.cfg.mac {
            Mac::Tdma => {
                let slot_owner = 1 + (self.slots as usize - 1) % (self.nodes.len() - 1);
                ready.retain(|&id| id == slot_owner);
                ready
            }
            Mac::Csma => {
                ready.retain(|&id| {
                    if self.nodes[id].backoff > 0 {
                        self.nodes[id].backoff -= 1;
                        false
                    } else {
                        true
                    }
                });
                for id in (1..ready.len()).rev() {
                    let other = self.rng.range(0, id);
                    ready.swap(id, other);
                }
                let mut winners: Vec<usize> = Vec::new();
                for id in ready {
                    if winners.iter().any(|&other| self.connected(other, id)) {
                        self.nodes[id].backoff = self.rng.range(1, 3) as u8;
                    } else {
                        winners.push(id);
                    }
                }
                winners
            }
        };
        self.transmit(&senders, &awake, &mut radio);
        for (id, power) in radio.into_iter().enumerate().skip(1) {
            self.expend(id, power);
        }
        self.update_routes();
    }

    pub(super) fn advance(&mut self, delta: f64) {
        let target = self
            .cfg
            .end_time()
            .map_or(self.now + delta, |end| (self.now + delta).min(end));
        if !target.is_finite() || target <= self.now {
            return;
        }
        while self.next_slot <= target + EPS {
            self.now = self.next_slot;
            self.slot();
            self.next_slot += SLOT_SECS;
        }
        self.now = target;
    }

    pub(super) fn take_snapshot(&mut self) -> Snapshot {
        let mut links = Vec::new();
        for a in 0..self.nodes.len() {
            for b in a + 1..self.nodes.len() {
                if self.connected(a, b) {
                    let snr = self.snr(a, b);
                    links.push(LinkView {
                        a: a as u16,
                        b: b as u16,
                        snr_db: snr,
                        success_probability: 1.0 - packet_failure_prob(snr),
                    });
                }
            }
        }
        let nodes = self
            .nodes
            .iter()
            .map(|n| NodeView {
                id: n.id,
                x: n.x,
                y: n.y,
                battery_pct: if n.id == 0 {
                    100.0
                } else {
                    100.0 * n.battery_mj / self.cfg.initial_battery_mj
                },
                battery_mj: n.battery_mj,
                is_active: n.alive(),
                is_awake: n.awake,
                queue_len: n.queue.len(),
                next_hop: n.next_hop,
                route_cost: n.route_cost,
            })
            .collect();
        let mut stats = self.stats.clone();
        stats.delivery_pct = if stats.generated == 0 {
            0.0
        } else {
            100.0 * stats.delivered as f64 / stats.generated as f64
        };
        stats.avg_latency_secs = if stats.delivered == 0 {
            0.0
        } else {
            stats.latency_total_secs / stats.delivered as f64
        };
        stats.active_nodes = self.nodes.iter().skip(1).filter(|n| n.alive()).count();
        stats.sleeping_nodes = self
            .nodes
            .iter()
            .skip(1)
            .filter(|n| n.alive() && !n.awake)
            .count();
        stats.depleted_nodes = self
            .nodes
            .iter()
            .skip(1)
            .filter(|n| n.battery_mj <= EPS)
            .count();
        stats.queued_frames = self.nodes.iter().skip(1).map(|n| n.queue.len()).sum();
        Snapshot {
            time: self.now,
            dur: f64::from(self.cfg.to_secs),
            hist_start: 0.0,
            is_complete: self.cfg.end_time().is_some_and(|end| self.now + EPS >= end),
            nodes,
            links,
            transmissions: self.transmissions.clone(),
            events: std::mem::take(&mut self.events),
            stats,
            mac: self.cfg.mac,
        }
    }

    pub(super) fn move_node(&mut self, id: u16, x: f64, y: f64) -> bool {
        let Some(node) = self.nodes.get_mut(usize::from(id)) else {
            return false;
        };
        if id == 0 || !x.is_finite() || !y.is_finite() {
            return false;
        }
        let next_x = x.clamp(28.0, 392.0);
        let next_y = y.clamp(30.0, 270.0);
        if (node.x - next_x).abs() < EPS && (node.y - next_y).abs() < EPS {
            return false;
        }
        node.x = next_x;
        node.y = next_y;
        self.update_routes();
        self.log("topology", id, None, "Sensor moved");
        true
    }

    pub(super) fn set_enabled(&mut self, id: u16, enabled: bool) -> bool {
        let Some(node) = self.nodes.get_mut(usize::from(id)) else {
            return false;
        };
        if id == 0 || node.enabled == enabled || (enabled && node.battery_mj <= EPS) {
            return false;
        }
        node.enabled = enabled;
        if !enabled {
            self.stats.dropped += node.queue.len() as u64;
            node.queue.clear();
        }
        self.update_routes();
        self.log(
            if enabled { "restart" } else { "failure" },
            id,
            None,
            if enabled {
                "Sensor restarted"
            } else {
                "Sensor stopped"
            },
        );
        true
    }

    pub(super) fn recharge_node(&mut self, id: u16) -> bool {
        let Some(node) = self.nodes.get_mut(usize::from(id)) else {
            return false;
        };
        if id == 0 || node.battery_mj > EPS {
            return false;
        }
        node.battery_mj = self.cfg.initial_battery_mj;
        node.enabled = true;
        node.next_sample = self.now + self.cfg.data_interval_secs;
        self.update_routes();
        self.log("restart", id, None, "Battery replaced; sensor restarted");
        true
    }

    pub(super) fn set_mac(&mut self, mac: Mac) -> bool {
        if self.cfg.mac == mac {
            return false;
        }
        self.cfg.mac = mac;
        self.log("config", 0, None, "MAC protocol changed");
        true
    }
    pub(super) fn set_tx_power(&mut self, value: i16) -> bool {
        if !(-20..=20).contains(&value) || self.cfg.tx_power_dbm == value {
            return false;
        }
        self.cfg.tx_power_dbm = value;
        self.update_routes();
        self.log("config", 0, None, "Transmit power changed");
        true
    }
    pub(super) fn set_noise(&mut self, value: i16) -> bool {
        if !(-115..=-75).contains(&value) || self.cfg.noise_floor_dbm == value {
            return false;
        }
        self.cfg.noise_floor_dbm = value;
        self.update_routes();
        self.log("config", 0, None, "Noise floor changed");
        true
    }
    pub(super) fn set_duty(&mut self, value: u8) -> bool {
        if !(10..=100).contains(&value) || self.cfg.duty_cycle_pct == value {
            return false;
        }
        self.cfg.duty_cycle_pct = value;
        self.log("config", 0, None, "Duty cycle changed");
        true
    }
    pub(super) fn set_nodes(&mut self, count: u16) -> bool {
        if !(3..=15).contains(&count) || self.cfg.nodes == count {
            return false;
        }
        let current = self.cfg.nodes;
        if count < current {
            for node in self.nodes.drain(usize::from(count) + 1..) {
                self.stats.dropped += node.queue.len() as u64;
            }
        } else {
            for id in current + 1..=count {
                let sensor = Self::new_sensor(&self.cfg, &mut self.rng, &self.nodes, id, self.now);
                self.nodes.push(sensor);
            }
        }
        self.cfg.nodes = count;
        self.update_routes();
        self.log("config", 0, None, "Sensor count changed");
        true
    }

    pub(super) fn set_battery(&mut self, value: f64) -> bool {
        if !value.is_finite()
            || !(500.0..=50_000.0).contains(&value)
            || (self.cfg.initial_battery_mj - value).abs() < EPS
        {
            return false;
        }
        let ratio = value / self.cfg.initial_battery_mj;
        for node in self.nodes.iter_mut().skip(1) {
            node.battery_mj = (node.battery_mj * ratio).min(value);
        }
        self.cfg.initial_battery_mj = value;
        self.log("config", 0, None, "Battery capacity changed");
        true
    }

    pub(super) fn set_duration(&mut self, value: u16) -> bool {
        if (value != 0 && !(30..=3600).contains(&value)) || self.cfg.to_secs == value {
            return false;
        }
        self.cfg.to_secs = value;
        self.log("config", 0, None, "Simulation duration changed");
        true
    }

    pub(super) fn set_interval(&mut self, value: f64) -> bool {
        if !value.is_finite()
            || !(0.5..=30.0).contains(&value)
            || (self.cfg.data_interval_secs - value).abs() < EPS
        {
            return false;
        }
        self.cfg.data_interval_secs = value;
        for node in self.nodes.iter_mut().skip(1) {
            node.next_sample = self.now + value;
        }
        self.log("config", 0, None, "Sample interval changed");
        true
    }
}
