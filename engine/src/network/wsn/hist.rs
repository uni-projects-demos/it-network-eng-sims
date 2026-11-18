use super::model::{Config, Event, Mac, Snapshot};
use super::{EPS, History, HistoryState, Sim};
use wasm_bindgen::prelude::*;

impl HistoryState for Sim {
    type Event = Event;
    fn now(&self) -> f64 {
        self.now
    }
    fn end_time(&self) -> Option<f64> {
        self.cfg.end_time()
    }
    fn duration(&self) -> u16 {
        self.cfg.to_secs
    }
    fn advance(&mut self, delta: f64) {
        Sim::advance(self, delta);
    }
    fn events_mut(&mut self) -> &mut Vec<Event> {
        &mut self.events
    }
}

#[wasm_bindgen(typescript_custom_section)]
const WSN_TYPES: &str =
    "import type { WsnConfig, WsnSnapshot } from '../../sims/network/wsn/types';";

#[wasm_bindgen]
pub struct SimWSN {
    history: History<Sim>,
}

#[wasm_bindgen]
impl SimWSN {
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(unchecked_param_type = "WsnConfig")] config: JsValue,
    ) -> Result<SimWSN, JsValue> {
        let cfg: Config = crate::deserialize(config)?;
        Sim::new(cfg)
            .map(|sim| Self {
                history: History::new(sim),
            })
            .map_err(|error| JsValue::from_str(&error))
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn snapshot(&mut self) -> Result<JsValue, JsValue> {
        let mut snapshot: Snapshot = self.history.state.take_snapshot();
        snapshot.hist_start = self.history.earliest_time();
        crate::serialize(&snapshot)
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn advance(&mut self, secs: f64) -> Result<JsValue, JsValue> {
        if secs.is_finite() && secs > EPS {
            self.history
                .advance_to(self.history.state.now + secs.min(30.0));
        }
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn step_back(&mut self) -> Result<JsValue, JsValue> {
        self.history.step_back();
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn seek_start(&mut self) -> Result<JsValue, JsValue> {
        self.history.seek_start();
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn seek_end(&mut self) -> Result<JsValue, JsValue> {
        self.history.seek_end();
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn move_node(&mut self, id: u16, x: f64, y: f64) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.move_node(id, x, y));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn set_node_enabled(&mut self, id: u16, enabled: bool) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_enabled(id, enabled));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn recharge_node(&mut self, id: u16) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.recharge_node(id));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn set_mac(&mut self, mac: &str) -> Result<JsValue, JsValue> {
        let value = match mac {
            "csma" => Mac::Csma,
            "tdma" => Mac::Tdma,
            _ => return Err(JsValue::from_str("Unknown MAC protocol")),
        };
        self.history.edit(|sim| sim.set_mac(value));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn set_tx_power(&mut self, value: i16) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_tx_power(value));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn set_noise(&mut self, value: i16) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_noise(value));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn set_duty(&mut self, value: u8) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_duty(value));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn set_nodes(&mut self, value: u16) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_nodes(value));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn set_battery(&mut self, value: f64) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_battery(value));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn set_duration(&mut self, value: u16) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_duration(value));
        self.snapshot()
    }

    #[wasm_bindgen(unchecked_return_type = "WsnSnapshot")]
    pub fn set_interval(&mut self, value: f64) -> Result<JsValue, JsValue> {
        self.history.edit(|sim| sim.set_interval(value));
        self.snapshot()
    }
}
