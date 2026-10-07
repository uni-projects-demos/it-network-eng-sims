use serde::Serialize;

use super::{Udp, event::EventType};
use crate::transport::sim::{
    TransferState, TransportContext, TransportResult, TransportStats, common_stats, finalize,
    run_file_transfer,
};

#[derive(Serialize)]
pub struct StatsUDP {
    #[serde(flatten)]
    pub transport: TransportStats,
}

pub type SimResUDP = TransportResult<EventType, StatsUDP>;

pub fn run(msg: &[u8], packet_loss_prob: f64, payload_size: usize, seed: u32) -> SimResUDP {
    let mut context: TransportContext<Udp> = TransportContext::new(Udp, seed);
    let state: TransferState = run_file_transfer(&mut context, msg, packet_loss_prob, payload_size);

    let _: Option<usize> = finalize(&mut context, &state, packet_loss_prob);

    let stats: StatsUDP = StatsUDP {
        transport: common_stats(&context, &state, packet_loss_prob, payload_size),
    };

    TransportResult {
        events: context.log.events,
        output: context.log.output,
        stats,
        received_msg: String::from_utf8_lossy(&context.receiver.received_bytes).into_owned(),
    }
}
