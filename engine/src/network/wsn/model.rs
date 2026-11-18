use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Mac {
    Csma,
    Tdma,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct Config {
    pub(super) nodes: u16,
    pub(super) seed: u32,
    pub(super) mac: Mac,
    pub(super) tx_power_dbm: i16,
    pub(super) noise_floor_dbm: i16,
    pub(super) duty_cycle_pct: u8,
    pub(super) data_interval_secs: f64,
    pub(super) initial_battery_mj: f64,
    pub(super) to_secs: u16,
}

impl Config {
    pub(super) fn check(&self) -> Result<(), String> {
        if !(3..=15).contains(&self.nodes)
            || !(-20..=20).contains(&self.tx_power_dbm)
            || !(-115..=-75).contains(&self.noise_floor_dbm)
            || !(10..=100).contains(&self.duty_cycle_pct)
            || !self.data_interval_secs.is_finite()
            || !(0.5..=30.0).contains(&self.data_interval_secs)
            || !self.initial_battery_mj.is_finite()
            || !(500.0..=50_000.0).contains(&self.initial_battery_mj)
            || (self.to_secs != 0 && !(30..=3600).contains(&self.to_secs))
        {
            return Err("Invalid wireless sensor network configuration".into());
        }
        Ok(())
    }
    pub(super) fn end_time(&self) -> Option<f64> {
        (self.to_secs != 0).then_some(f64::from(self.to_secs))
    }
}

#[derive(Clone, Debug)]
pub(super) struct Frame {
    pub(super) source: u16,
    pub(super) born: f64,
    pub(super) retries: u8,
}

#[derive(Clone, Debug)]
pub(super) struct Node {
    pub(super) id: u16,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) battery_mj: f64,
    pub(super) enabled: bool,
    pub(super) queue: VecDeque<Frame>,
    pub(super) next_sample: f64,
    pub(super) backoff: u8,
    pub(super) next_hop: Option<u16>,
    pub(super) route_cost: Option<f64>,
    pub(super) awake: bool,
}

impl Node {
    pub(super) fn alive(&self) -> bool {
        self.enabled && (self.id == 0 || self.battery_mj > 0.0)
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct LinkView {
    pub(super) a: u16,
    pub(super) b: u16,
    pub(super) snr_db: f64,
    pub(super) success_probability: f64,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct NodeView {
    pub(super) id: u16,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) battery_pct: f64,
    pub(super) battery_mj: f64,
    pub(super) is_active: bool,
    pub(super) is_awake: bool,
    pub(super) queue_len: usize,
    pub(super) next_hop: Option<u16>,
    pub(super) route_cost: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct Transmission {
    pub(super) from: u16,
    pub(super) to: u16,
    pub(super) outcome: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct Event {
    pub(super) time: f64,
    pub(super) _type: String,
    pub(super) node: u16,
    pub(super) peer: Option<u16>,
    pub(super) msg: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct Stats {
    pub(super) generated: u64,
    pub(super) delivered: u64,
    pub(super) forwarded: u64,
    pub(super) transmissions: u64,
    pub(super) collisions: u64,
    pub(super) errors: u64,
    pub(super) retries: u64,
    pub(super) dropped: u64,
    pub(super) energy_mj: f64,
    pub(super) delivery_pct: f64,
    pub(super) avg_latency_secs: f64,
    pub(super) active_nodes: usize,
    pub(super) sleeping_nodes: usize,
    pub(super) depleted_nodes: usize,
    pub(super) queued_frames: usize,
    pub(super) first_depletion_secs: Option<f64>,
    pub(super) latency_total_secs: f64,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct Snapshot {
    pub(super) time: f64,
    pub(super) dur: f64,
    pub(super) hist_start: f64,
    pub(super) is_complete: bool,
    pub(super) nodes: Vec<NodeView>,
    pub(super) links: Vec<LinkView>,
    pub(super) transmissions: Vec<Transmission>,
    pub(super) events: Vec<Event>,
    pub(super) stats: Stats,
    pub(super) mac: Mac,
}
