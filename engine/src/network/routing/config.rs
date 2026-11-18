use serde::Deserialize;
pub(crate) const INF: u8 = 16;
pub(crate) const MIN_LINK_COST: u8 = 1;
pub(crate) const MAX_LINK_COST: u8 = 15;
pub(crate) const UPDATE_PERIOD: f64 = 4.0;
pub(crate) const CONTINUOUS_DUR: u16 = 0;
pub(crate) const DEFAULT_DUR: u16 = 60;
pub(crate) const EPS: f64 = 0.000_001;
pub(crate) const MAX_PCK_LOSS_PERC: u8 = 10;
pub(crate) const MAX_PCK_INVALID_PERC: u8 = 10;
pub(crate) const SHORT_OUTAGE: (f64, f64) = (8.0, 30.0);
pub(crate) const LONG_OUTAGE: (f64, f64) = (48.0, 90.0);
pub(crate) const DELIVERY_DELAY: f64 = 0.1;
pub(crate) const CLOCK_TICK: f64 = 0.25;
pub(crate) const MAX_ADVANCE: f64 = 30.0;
pub(crate) const HIST_TICK: f64 = 0.0625;
pub(crate) const TO_FACTOR: f64 = 6.0;
pub(crate) const GARBAGE_FACTOR: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub(crate) enum Topology {
    #[default]
    Default,
    Rand,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct Config {
    pub(crate) nodes: u16,
    pub(crate) seed: u32,
    #[serde(default = "default_dur")]
    pub(crate) to_secs: u16,
    #[serde(default)]
    pub(crate) topology: Topology,
    #[serde(default)]
    pub(crate) rand_fail_perc: u8,
    #[serde(default)]
    pub(crate) pck_loss_perc: u8,
    #[serde(default)]
    pub(crate) pck_invalid_perc: u8,
}

fn default_dur() -> u16 {
    DEFAULT_DUR
}

impl Config {
    pub(crate) fn check(&self) -> Result<(), String> {
        if (self.topology == Topology::Rand && !(3..=15).contains(&self.nodes))
            || (self.topology == Topology::Default && self.nodes != 7)
            || self.pck_loss_perc > MAX_PCK_LOSS_PERC
            || self.pck_invalid_perc > MAX_PCK_INVALID_PERC
            || self.rand_fail_perc > 10
        {
            return Err("Invalid routing simulation configuration".into());
        }
        Ok(())
    }

    pub(crate) fn is_finished_at(&self, now: f64) -> bool {
        self.to_secs != CONTINUOUS_DUR && now + EPS >= f64::from(self.to_secs)
    }

    pub(crate) fn end_time(&self) -> Option<f64> {
        (self.to_secs != CONTINUOUS_DUR).then_some(f64::from(self.to_secs))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Rand(pub(crate) u32);
impl Rand {
    fn nxt(&mut self) -> u32 {
        let mut x = self.0;
        if x == 0 {
            x = 0x9e37_79b9;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    pub(crate) fn float(&mut self) -> f64 {
        f64::from(self.nxt()) / f64::from(u32::MAX)
    }
    pub(crate) fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + (self.nxt() as usize % (hi - lo + 1))
    }
}
