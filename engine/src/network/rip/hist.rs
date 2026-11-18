use super::{Config, Sim, Snapshot, EPS};
use super::super::routing::{History, HistoryState};
use super::model::Event;
use wasm_bindgen::prelude::*;

impl HistoryState for Sim {
    type Event = Event;
    fn now(&self) -> f64 { self.now }
    fn end_time(&self) -> Option<f64> { self.cfg.end_time() }
    fn duration(&self) -> u16 { self.cfg.to_secs }
    fn advance(&mut self, delta: f64) { Sim::advance(self, delta); }
    fn events_mut(&mut self) -> &mut Vec<Event> { &mut self.events }
}

#[wasm_bindgen(typescript_custom_section)]
const RIP_TYPES: &str = "import type { RipConfig, RipSnapshot } from '../../sims/network/rip/types';";

#[wasm_bindgen]
pub struct SimRIP {
    history: History<Sim>,
}

#[wasm_bindgen]
impl SimRIP {
    #[wasm_bindgen(constructor)]
    pub fn new(#[wasm_bindgen(unchecked_param_type = "RipConfig")] config: JsValue) -> Result<SimRIP, JsValue> {
        let cfg: Config = crate::deserialize(config)?;
        Sim::new(cfg).map(|sim| Self { history: History::new(sim) })
            .map_err(|err| JsValue::from_str(&err))
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn snapshot(&mut self) -> Result<JsValue, JsValue> {
        let mut view: Snapshot = self.history.state.take_snapshot();
        view.hist_start = self.history.earliest_time();
        crate::serialize(&view)
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn advance(&mut self, secs: f64) -> Result<JsValue, JsValue> {
        if secs.is_finite() && secs > EPS {
            self.history.advance_to(self.history.state.now + secs);
        }
        self.snapshot()
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn step_back(&mut self) -> Result<JsValue, JsValue> {
        self.history.step_back();
        self.snapshot()
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn seek_start(&mut self) -> Result<JsValue, JsValue> {
        self.history.seek_start();
        self.snapshot()
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn seek_end(&mut self) -> Result<JsValue, JsValue> {
        self.history.seek_end();
        self.snapshot()
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn terminate_router(&mut self, id: u16) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.kill(id));
        self.snapshot()
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn restart_router(&mut self, id: u16) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.restart(id));
        self.snapshot()
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn set_link_cost(&mut self, a: u16, b: u16, cost: u8) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_link_cost(a, b, cost));
        self.snapshot()
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn set_pck_invalid_perc(&mut self, perc: u8) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_pck_invalid_perc(perc));
        self.snapshot()
    }
    #[wasm_bindgen(unchecked_return_type = "RipSnapshot")]
    pub fn set_pck_loss_perc(&mut self, perc: u8) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_pck_loss_perc(perc));
        self.snapshot()
    }
}
