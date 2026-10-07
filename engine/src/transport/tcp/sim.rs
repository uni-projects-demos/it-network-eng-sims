use serde::Serialize;

use super::{HEADER_SIZE, Tcp, channel, event::EventType, handshake};
use crate::transport::{
    event::SimEvent,
    sim::{
        TransferState, TransportContext, TransportResult, TransportStats, common_stats, finalize,
        run_file_transfer,
    },
};

#[derive(Serialize)]
pub struct StatsTCP {
    #[serde(flatten)]
    pub transport: TransportStats,
    pub bit_errs: usize,
    pub bit_err_prob: f64,
    pub header_size: usize,
}

pub type SimResTCP = TransportResult<EventType, StatsTCP>;

pub fn run(
    msg: &[u8],
    packet_loss_prob: f64,
    bit_err_prob: f64,
    payload_size: usize,
    is_handshake: bool,
    seed: u32,
) -> SimResTCP {
    let mut context: TransportContext<Tcp> = TransportContext::new(Tcp::new(bit_err_prob), seed);

    let mut state: TransferState =
        if is_handshake && !handshake::run(&mut context, packet_loss_prob) {
            let terminal_event_id: Option<usize> = context
                .log
                .events
                .last()
                .map(|event: &SimEvent<EventType>| -> usize { event.id });
            TransferState::incomplete(terminal_event_id)
        } else {
            run_file_transfer(&mut context, msg, packet_loss_prob, payload_size)
        };

    if !state.is_complete && state.terminal_event_id.is_none() {
        state.terminal_event_id = context
            .log
            .events
            .last()
            .map(|event: &SimEvent<EventType>| -> usize { event.id });
    }

    let summary_event_id: Option<usize> = finalize(&mut context, &state, packet_loss_prob);

    channel::log_report(
        &context.protocol.channel,
        &mut context.log,
        summary_event_id,
    );

    let stats: StatsTCP = StatsTCP {
        transport: common_stats(&context, &state, packet_loss_prob, payload_size),
        bit_errs: context.protocol.channel.bit_errs,
        bit_err_prob: context.protocol.channel.bit_err_prob,
        header_size: HEADER_SIZE,
    };

    TransportResult {
        events: context.log.events,
        output: context.log.output,
        stats,
        received_msg: String::from_utf8_lossy(&context.receiver.received_bytes).into_owned(),
    }
}
