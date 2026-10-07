use super::{event::EventType, packet::WirePacket};
use crate::transport::{
    channel::SimpleRng,
    event::{EventLog, ModuleName},
};

pub struct ChannelState {
    pub bit_err_prob: f64,
    pub bit_errs: usize,
}

impl ChannelState {
    pub fn new(bit_err_prob: f64) -> Self {
        Self {
            bit_err_prob,
            bit_errs: 0,
        }
    }
}

pub fn apply_bit_error(
    state: &mut ChannelState,
    wire: &mut WirePacket,
    rng: &mut SimpleRng,
    log: &mut EventLog<EventType>,
) {
    let v: f64 = rng.uniform_4();
    if v >= state.bit_err_prob {
        return;
    }

    state.bit_errs += 1;
    wire.packet.data_len += randint_1_10(rng);

    let event: usize = log.event(
        EventType::BitErr,
        None,
        None,
        ModuleName::Channel,
        "Bit error",
        format!("v={v:.4} < {}", state.bit_err_prob),
        Some(&wire.packet),
        Some(v),
        Some(state.bit_err_prob),
    );

    log.output(
        ModuleName::Channel,
        Some(event),
        format!(
            "A v value of {v} < {}, indicating #{} occurrence of the probability of a bit error event.",
            state.bit_err_prob, state.bit_errs
        ),
    );
}

pub fn log_report(
    state: &ChannelState,
    log: &mut EventLog<EventType>,
    summary_event_id: Option<usize>,
) {
    log.output(
        ModuleName::Channel,
        summary_event_id,
        format!(
            "\nTCP bit-error report:\n - {} bit errors at a probability of {}%.\n",
            state.bit_errs,
            state.bit_err_prob * 100.0
        ),
    );
}

fn randint_1_10(rng: &mut SimpleRng) -> i32 {
    (rng.next_u32() % 10 + 1) as i32
}
