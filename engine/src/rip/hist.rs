use super::{Sim, Config, CONTINUOUS_DUR, EPS, HIST_TICK};

pub(super) const CONT_HIST_LIMIT: usize = 1_201;
const CONT_CHECKPOINT_PERIOD: f64 = 0.25;
const HIST_TRIM_BATCH: usize = 64;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SimRIP {
    pub(super) inner: Sim,
    pub(super) checkpoints: Vec<Sim>,
    pub(super) curs: usize,
}

impl SimRIP {
    fn checkpoint(&mut self) -> Sim {
        let events = std::mem::take(&mut self.inner.events);
        let checkpoint = self.inner.clone();
        self.inner.events = events;
        checkpoint
    }

    pub(super) fn record(&mut self) {
        self.checkpoints.truncate(self.curs + 1);
        let checkpoint = self.checkpoint();
        self.checkpoints.push(checkpoint);
        self.curs += 1;
        if self.inner.cfg.to_secs == CONTINUOUS_DUR
            && self.checkpoints.len() > CONT_HIST_LIMIT {
            self.checkpoints.drain(..HIST_TRIM_BATCH);
            self.curs -= HIST_TRIM_BATCH;
        }
    }

    pub(super) fn restore(&mut self, idx: usize) {
        self.curs = idx;
        self.inner = self.checkpoints[idx].clone();
        self.inner.events.clear();
    }

    fn branch(&mut self) {
        self.checkpoints.truncate(self.curs + 1);
    }

    pub(super) fn edit(&mut self, f: impl FnOnce(&mut Sim) -> bool) {
        self.branch();
        if f(&mut self.inner) {
            if self.inner.now > self.checkpoints[self.curs].now + EPS {
                self.record();
            } else {
                let checkpoint = self.checkpoint();
                self.checkpoints[self.curs] = checkpoint;
            }
        }
    }

    pub(super) fn advance_to(&mut self, time: f64) {
        let target = self.inner.cfg.end_time().map_or(time, |limit| time.min(limit));
        while self.inner.now + EPS < target {
            let replaying_hist = self.curs + 1 < self.checkpoints.len();
            if replaying_hist && self.checkpoints[self.curs + 1].now <= target + EPS {
                self.restore(self.curs + 1);
                continue;
            }
            let delta = (target - self.inner.now).min(HIST_TICK);
            let prior_events = std::mem::take(&mut self.inner.events);
            self.inner.advance(delta);
            let new_events = std::mem::take(&mut self.inner.events);
            let since_checkpoint = self.inner.now - self.checkpoints[self.curs].now;
            if !replaying_hist && (self.inner.cfg.to_secs != CONTINUOUS_DUR
                || since_checkpoint + EPS >= CONT_CHECKPOINT_PERIOD) {
                self.record();
            }
            self.inner.events = prior_events;
            self.inner.events.extend(new_events);
        }
    }
}

#[wasm_bindgen]
impl SimRIP {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<SimRIP, JsValue> {
        let cfg: Config = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Sim::new(cfg).map(|mut inner| {
            let events = std::mem::take(&mut inner.events);
            let checkpoint = inner.clone();
            inner.events = events;
            Self { inner, checkpoints: vec![checkpoint], curs: 0 }
        }).map_err(|e| JsValue::from_str(&e))
    }
    pub fn snapshot(&mut self) -> Result<JsValue, JsValue> {
        let mut view = self.inner.take_snapshot();
        view.hist_start = self.checkpoints.first().map_or(0.0, |s| s.now);
        serde_wasm_bindgen::to_value(&view)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn advance(&mut self, secs: f64) -> Result<JsValue, JsValue> {
        if secs.is_finite() && secs > 0.0 { self.advance_to(self.inner.now + secs); }
        self.snapshot()
    }
    pub fn step_back(&mut self) -> Result<JsValue, JsValue> {
        let target = (self.inner.now - 1.0).max(0.0);
        let idx = self.checkpoints.iter().rposition(|s| s.now <= target + EPS).unwrap_or(0);
        self.restore(idx);
        self.snapshot()
    }
    pub fn seek_start(&mut self) -> Result<JsValue, JsValue> {
        self.restore(0);
        self.snapshot()
    }
    pub fn seek_end(&mut self) -> Result<JsValue, JsValue> {
        if let Some(limit) = self.inner.cfg.end_time() { self.advance_to(limit); }
        self.snapshot()
    }
    pub fn terminate_router(&mut self, id: u16) -> Result<JsValue, JsValue> {
        self.edit(|sim| sim.kill(id));
        self.snapshot()
    }
    pub fn restart_router(&mut self, id: u16) -> Result<JsValue, JsValue> {
        self.edit(|sim| sim.restart(id));
        self.snapshot()
    }
    pub fn set_link_cost(&mut self, a: u16, b: u16, cost: u8) -> Result<JsValue, JsValue> {
        self.edit(|sim| sim.set_link_cost(a, b, cost));
        self.snapshot()
    }
    pub fn set_pck_invalid_perc(&mut self, perc: u8) -> Result<JsValue, JsValue> {
        self.edit(|sim| sim.set_pck_invalid_perc(perc));
        self.snapshot()
    }
    pub fn set_pck_loss_perc(&mut self, perc: u8) -> Result<JsValue, JsValue> {
        self.edit(|sim| sim.set_pck_loss_perc(perc));
        self.snapshot()
    }
}
