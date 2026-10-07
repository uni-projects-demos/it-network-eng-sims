mod transport;

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_tcp_sim(
    msg: &str,
    packet_loss_prob: f64,
    bit_err_prob: f64,
    payload_size: usize,
    is_handshake: bool,
    seed: u32,
) -> Result<JsValue, JsValue> {
    validate_prob(packet_loss_prob, "P must satisfy 0.0 <= P < 1.0")?;
    validate_prob(
        bit_err_prob,
        "Bit-error probability must satisfy 0.0 <= probability < 1.0",
    )?;
    validate_payload_size(payload_size)?;

    let res: transport::tcp::sim::SimResTCP = transport::tcp::sim::run(
        msg.as_bytes(),
        packet_loss_prob,
        bit_err_prob,
        payload_size,
        is_handshake,
        seed,
    );

    serialize(&res)
}

#[wasm_bindgen]
pub fn run_udp_sim(
    msg: &str,
    packet_loss_prob: f64,
    payload_size: usize,
    seed: u32,
) -> Result<JsValue, JsValue> {
    validate_prob(packet_loss_prob, "P must satisfy 0.0 <= P < 1.0")?;
    validate_payload_size(payload_size)?;

    let res: transport::udp::sim::SimResUDP =
        transport::udp::sim::run(msg.as_bytes(), packet_loss_prob, payload_size, seed);

    serialize(&res)
}

fn validate_prob(prob: f64, err: &str) -> Result<(), JsValue> {
    if (0.0..1.0).contains(&prob) {
        Ok(())
    } else {
        Err(JsValue::from_str(err))
    }
}

fn validate_payload_size(payload_size: usize) -> Result<(), JsValue> {
    if transport::BYTE_FILE_PAYLOAD_SIZES.contains(&payload_size) {
        Ok(())
    } else {
        Err(JsValue::from_str(
            "Payload size must be one of 32, 64, 128, 256, 512, 1024, 1460, or 4096 bytes",
        ))
    }
}

fn serialize<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|err: serde_wasm_bindgen::Error| -> JsValue {
        JsValue::from_str(&err.to_string())
    })
}
