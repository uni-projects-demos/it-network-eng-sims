mod config;
mod faults;
mod history;
mod shortest;
mod topology;

pub(crate) use config::{
    CLOCK_TICK, CONTINUOUS_DUR, Config, DELIVERY_DELAY, EPS, GARBAGE_FACTOR, HIST_TICK, INF,
    LONG_OUTAGE, MAX_ADVANCE, MAX_LINK_COST, MAX_PCK_INVALID_PERC, MAX_PCK_LOSS_PERC,
    MIN_LINK_COST, Rand, SHORT_OUTAGE, TO_FACTOR, Topology, UPDATE_PERIOD,
};
pub(crate) use faults::{random_outage, should_fail};
pub(crate) use history::{History, HistoryState};
pub(crate) use shortest::shortest_paths;
pub(crate) use topology::{default_edges, gen_edges};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Edge {
    pub(crate) a: u16,
    pub(crate) b: u16,
    pub(crate) cost: u8,
    pub(crate) can_increase: bool,
    pub(crate) can_decrease: bool,
}
