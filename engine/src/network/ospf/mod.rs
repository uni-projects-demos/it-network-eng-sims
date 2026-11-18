mod hist;
mod model;
mod sim;

use super::routing::{
    CLOCK_TICK, Config, EPS, Edge, History, MAX_ADVANCE, MAX_LINK_COST, MAX_PCK_INVALID_PERC,
    MAX_PCK_LOSS_PERC, MIN_LINK_COST, Rand, default_edges, gen_edges, random_outage,
    shortest_paths, should_fail,
};
#[allow(unused_imports)]
pub use hist::SimOSPF;
use model::*;
use std::collections::{BTreeMap, BTreeSet};

const HELLO_PERIOD: f64 = 1.0;
const ROUTER_DEAD_INTERVAL: f64 = 4.0;
const LSA_REFRESH_PERIOD: f64 = 12.0;
const SPF_DELAY: f64 = 0.25;
const DELIVERY_DELAY: f64 = 0.1;
