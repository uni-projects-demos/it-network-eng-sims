#[cfg(not(target_arch = "wasm32"))]
pub mod cplex;
mod lp;
mod model;
mod solver;

pub use lp::{gen_lp, lp_filename};
pub use model::{FlowConfig, FlowPhase, FlowSnapshot, FlowStatus};
pub use solver::FlowSolver;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const FLOW_TYPES: &str =
    "import type { FlowConfig, FlowSnapshot } from '../../sims/network/flow/types';";

#[wasm_bindgen]
pub struct SimFlow {
    solver: FlowSolver,
}

#[wasm_bindgen]
impl SimFlow {
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(unchecked_param_type = "FlowConfig")] options: JsValue,
    ) -> Result<SimFlow, JsValue> {
        let cfg: FlowConfig = crate::deserialize(options)?;
        Ok(Self {
            solver: FlowSolver::new(cfg).map_err(|err| JsValue::from_str(&err))?,
        })
    }

    #[wasm_bindgen(unchecked_return_type = "FlowSnapshot")]
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        crate::serialize(&self.solver.snapshot())
    }

    #[wasm_bindgen(unchecked_return_type = "FlowSnapshot")]
    pub fn advance(&mut self, work: u32) -> Result<JsValue, JsValue> {
        self.solver.advance(work.min(16_384));
        self.snapshot()
    }
}

#[wasm_bindgen]
pub fn gen_flow_lp(
    #[wasm_bindgen(unchecked_param_type = "FlowConfig")] options: JsValue,
) -> Result<String, JsValue> {
    let cfg: FlowConfig = crate::deserialize(options)?;
    gen_lp(&cfg).map_err(|err| JsValue::from_str(&err))
}
