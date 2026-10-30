use serde::{Deserialize, Serialize};
use std::fmt;

pub const PATHS: usize = 3;
pub const MAX_NODES: usize = 32;
const MAX_LP_PATH_VARIABLES: usize = 1_000_000;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FlowConfig {
    pub srcs: usize,
    pub trans: usize,
    pub dests: usize,
    #[serde(default = "default_seed")]
    pub seed: u32,
}

fn default_seed() -> u32 {
    1
}

impl FlowConfig {
    pub fn validate(&self) -> Result<(), String> {
        for (name, count) in [
            ("Sources", self.srcs),
            ("Transits", self.trans),
            ("Destinations", self.dests),
        ] {
            if count == 0 {
                return Err(format!("{name} must be a positive integer."));
            }
        }
        let variables = self
            .srcs
            .checked_mul(self.trans)
            .and_then(|n| n.checked_mul(self.dests));
        if variables.is_none_or(|n| n > MAX_LP_PATH_VARIABLES) {
            return Err(format!(
                "LP export is limited to {MAX_LP_PATH_VARIABLES} path variables."
            ));
        }
        Ok(())
    }

    pub fn validate_solver(&self) -> Result<(), String> {
        self.validate()?;
        if [self.srcs, self.trans, self.dests]
            .iter()
            .any(|&count| count > MAX_NODES)
        {
            return Err(format!(
                "Interactive solving supports at most {MAX_NODES} nodes per tier; larger models can still be exported as LP."
            ));
        }
        Ok(())
    }

    pub fn total_demand(&self) -> u64 {
        let (x, z) = (self.srcs as u64, self.dests as u64);
        x * z * (x + z + 2) / 2
    }

    pub fn lower_bound_scaled(&self) -> u64 {
        (3 * self.total_demand())
            .div_ceil(self.trans as u64)
            .max((self.srcs + self.dests) as u64)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FlowRoute {
    pub src: usize,
    pub dest: usize,
    pub demand: u64,
    pub trans: [usize; PATHS],
    pub flow_per_path: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct FlowLink {
    pub from: String,
    pub to: String,
    pub cap: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowPhase {
    Search,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowStatus {
    Optimal,
    Feasible,
    Infeasible,
}

impl fmt::Display for FlowStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Optimal => "optimal",
            Self::Feasible => "feasible",
            Self::Infeasible => "infeasible",
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FlowSnapshot {
    pub config: FlowConfig,
    pub status: FlowStatus,
    pub phase: FlowPhase,
    pub work: u64,
    pub search_nodes: u64,
    pub improvements: u64,
    pub lower_bound: f64,
    pub obj: Option<f64>,
    pub gap: Option<f64>,
    pub total_demand: u64,
    pub loads: Vec<f64>,
    pub routes: Vec<FlowRoute>,
    pub links: Vec<FlowLink>,
    pub nonzero_links: usize,
    pub max_cap: Option<f64>,
    pub highest_links: Vec<FlowLink>,
}
