use serde::Deserialize;
pub(super) const INF: u8 = 16;
pub(super) const MIN_LINK_COST: u8 = 1;
pub(super) const MAX_LINK_COST: u8 = 15;
pub(super) const UPDATE_PERIOD: f64 = 4.0;
pub(super) const CONTINUOUS_DUR: u16 = 0;
pub(super) const DEFAULT_DUR: u16 = 60;
pub(super) const EPS: f64 = 0.000_001;
pub(super) const MAX_PCK_LOSS_PERC: u8 = 10;
pub(super) const MAX_PCK_INVALID_PERC: u8 = 10;
pub(super) const SHORT_OUTAGE: (f64, f64) = (8.0, 30.0);
pub(super) const LONG_OUTAGE: (f64, f64) = (48.0, 90.0);
pub(super) const DELIVERY_DELAY: f64 = 0.1;
pub(super) const CLOCK_TICK: f64 = 0.25;
pub(super) const MAX_ADVANCE: f64 = 30.0;
pub(super) const HIST_TICK: f64 = 0.0625;
pub(super) const TO_FACTOR: f64 = 6.0;
pub(super) const GARBAGE_FACTOR: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub(super) enum Topology {
    #[default]
    Default, // assignment presentation
    Rand,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(super) struct Config {
    pub(super) nodes: u16,
    pub(super) seed: u32,
    #[serde(default = "default_dur")]
    pub(super) to_secs: u16,
    #[serde(default)]
    pub(super) topology: Topology,
    #[serde(default)]
    pub(super) rand_fail_perc: u8,
    #[serde(default)]
    pub(super) pck_loss_perc: u8,
    #[serde(default)]
    pub(super) pck_invalid_perc: u8,
}

fn default_dur() -> u16 { DEFAULT_DUR }

impl Config {
    pub(super) fn check(&self) -> Result<(), String> {
        Ok(())
    }

    pub(super) fn is_finished_at(&self, now: f64) -> bool {
        self.to_secs != CONTINUOUS_DUR
            && now + EPS >= f64::from(self.to_secs)
    }

    pub(super) fn end_time(&self) -> Option<f64> {
        (self.to_secs != CONTINUOUS_DUR).then_some(f64::from(self.to_secs))
    }
}

#[derive(Debug, Clone)]
pub(super) struct Rand(pub(super) u32);
impl Rand {
    fn nxt(&mut self) -> u32 {
        let mut x = self.0;
        if x == 0 { x = 0x9e37_79b9; }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    pub(super) fn float(&mut self) -> f64 { f64::from(self.nxt()) / f64::from(u32::MAX) }
    pub(super) fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + (self.nxt() as usize % (hi - lo + 1))
    }
}
