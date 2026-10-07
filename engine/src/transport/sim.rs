use serde::Serialize;

use super::{
    channel::{self, ChannelStats, Direction, SimpleRng, TransitParams},
    event::{Endpoint, EventLog, ModuleName, OutputLine, SimEvent, TransportEventType},
    packet::WirePacket,
    protocol::TransportProtocol,
    receiver::Receiver,
    sender::Sender,
};

pub const MAX_ATTEMPTS_PER_PACKET: usize = 10_000;
pub const DEADLOCK_VISIBLE_RETRIES: usize = 3;

pub struct TransportContext<P: TransportProtocol> {
    pub protocol: P,
    pub log: EventLog<P::Event>,
    pub sender: Sender,
    pub receiver: Receiver,
    pub channel_stats: ChannelStats,
    pub rng: SimpleRng,
}

impl<P: TransportProtocol> TransportContext<P> {
    pub fn new(protocol: P, seed: u32) -> Self {
        Self {
            protocol,
            log: EventLog::new(),
            sender: Sender::new(),
            receiver: Receiver::new(),
            channel_stats: ChannelStats::default(),
            rng: SimpleRng::new(seed),
        }
    }

    pub fn transit(
        &mut self,
        wire: P::Wire,
        direction: Direction,
        packet_loss_prob: f64,
        is_final_data_seen: bool,
    ) -> Option<P::Wire> {
        channel::transit(
            &mut self.protocol,
            wire,
            direction,
            TransitParams {
                packet_loss_prob,
                stats: &mut self.channel_stats,
                rng: &mut self.rng,
                log: &mut self.log,
                is_final_data_seen,
            },
        )
    }
}

pub struct TransferState {
    pub is_complete: bool,
    pub is_deadlock: bool,
    pub terminal_event_id: Option<usize>,
}

impl TransferState {
    pub fn complete() -> Self {
        Self {
            is_complete: true,
            is_deadlock: false,
            terminal_event_id: None,
        }
    }

    pub fn incomplete(terminal_event_id: Option<usize>) -> Self {
        Self {
            is_complete: false,
            is_deadlock: false,
            terminal_event_id,
        }
    }
}

#[derive(Serialize)]
pub struct TransportStats {
    pub sender_packets: usize,
    pub sender_transmissions_sent: usize,
    pub sender_transmissions_received: usize,
    pub receiver_packets: usize,
    pub receiver_transmissions_sent: usize,
    pub receiver_transmissions_received: usize,
    pub channel_packets: usize,
    pub channel_transmissions_sent: usize,
    pub channel_transmissions_received: usize,
    pub packets_lost: usize,
    pub packet_loss_prob: f64,
    pub payload_size: usize,
    pub is_complete: bool,
    pub is_deadlock: bool,
}

#[derive(Serialize)]
pub struct TransportResult<E, S> {
    pub events: Vec<SimEvent<E>>,
    pub output: Vec<OutputLine>,
    pub stats: S,
    pub received_msg: String,
}

pub fn run_file_transfer<P: TransportProtocol>(
    context: &mut TransportContext<P>,
    msg: &[u8],
    packet_loss_prob: f64,
    payload_size: usize,
) -> TransferState {
    let mut state: TransferState = TransferState::complete();

    let chunks = msg
        .chunks(payload_size)
        .map(|chunk: &[u8]| -> Vec<u8> { chunk.to_vec() })
        .chain(std::iter::once(Vec::<u8>::new()));

    'packets: for data in chunks {
        let data_packet: P::Wire = context
            .sender
            .make_data_packet(&context.protocol, data.clone());
        let mut is_acknowledge: bool = false;

        for attempt in 1..=MAX_ATTEMPTS_PER_PACKET {
            context
                .protocol
                .record_send_attempt(&mut context.sender.sent_count);

            let Some(to_receiver): Option<P::Wire> = context.transit(
                data_packet.clone(),
                Direction::FromSender,
                packet_loss_prob,
                data.is_empty(),
            ) else {
                context
                    .sender
                    .timeout(data_packet.packet(), &mut context.log);
                continue;
            };

            let Some((ack, final_packet)): Option<(P::Wire, bool)> =
                context
                    .receiver
                    .receive_data(&context.protocol, to_receiver, &mut context.log)
            else {
                context
                    .sender
                    .timeout(data_packet.packet(), &mut context.log);
                continue;
            };

            let ack_result: Option<P::Wire> = context.transit(
                ack,
                Direction::FromReceiver,
                packet_loss_prob,
                data.is_empty() || final_packet,
            );

            let Some(to_sender): Option<P::Wire> = ack_result else {
                context
                    .sender
                    .timeout(data_packet.packet(), &mut context.log);

                if final_packet && context.receiver.is_terminated {
                    state.is_deadlock = true;
                    state.is_complete = false;

                    for retry in 1..=DEADLOCK_VISIBLE_RETRIES {
                        let retry_number: usize = attempt + retry;

                        context
                            .protocol
                            .record_send_attempt(&mut context.sender.sent_count);

                        let retry_result: Option<P::Wire> = context.transit(
                            data_packet.clone(),
                            Direction::FromSender,
                            packet_loss_prob,
                            true,
                        );

                        if let Some(retry_packet) = retry_result {
                            let _: Option<(P::Wire, bool)> = context.receiver.receive_data(
                                &context.protocol,
                                retry_packet,
                                &mut context.log,
                            );
                        }

                        context
                            .sender
                            .timeout(data_packet.packet(), &mut context.log);

                        let timeout_event_id: Option<usize> = context
                            .log
                            .events
                            .last()
                            .map(|event: &SimEvent<P::Event>| -> usize { event.id });

                        context.log.output(
                            ModuleName::Sender,
                            timeout_event_id,
                            format!(
                                "Final EOF retransmission attempt #{retry_number} cannot complete because the receiver has terminated."
                            ),
                        );
                    }

                    let deadlock_event: usize = context.log.event(
                        P::Event::from(TransportEventType::Deadlock),
                        Some(Endpoint::Sender),
                        Some(Endpoint::Receiver),
                        ModuleName::Simulation,
                        "Protocol deadlock",
                        "Final ACK was lost after the receiver terminated; the sender can no longer complete the protocol",
                        Some(data_packet.packet()),
                        None,
                        Some(packet_loss_prob),
                    );

                    context.log.output(
                        ModuleName::Channel,
                        Some(deadlock_event),
                        format!(
                            "\nDeadlock event: final acknowledgement packet is lost with probability P={packet_loss_prob}. The receiver has terminated; subsequent EOF retransmissions cannot be acknowledged.\n"
                        ),
                    );

                    context.log.output(
                        ModuleName::Sender,
                        Some(deadlock_event),
                        "Sender remains unable to receive the final acknowledgement. Browser playback stops here to represent the original deadlock without hanging indefinitely.",
                    );

                    state.terminal_event_id = Some(deadlock_event);
                    break 'packets;
                }

                continue;
            };

            if context.sender.receive_ack(
                &context.protocol,
                to_sender,
                &data,
                attempt,
                &mut context.log,
            ) {
                is_acknowledge = true;
                break;
            }

            context
                .sender
                .timeout(data_packet.packet(), &mut context.log);
        }

        if !is_acknowledge {
            state.is_complete = false;

            let stopped_event_id: Option<usize> = context
                .log
                .events
                .last()
                .map(|event: &SimEvent<P::Event>| -> usize { event.id });

            context.log.output(
                ModuleName::Sender,
                stopped_event_id,
                format!(
                    "Simulation stopped after {MAX_ATTEMPTS_PER_PACKET} attempts for one packet."
                ),
            );

            state.terminal_event_id = stopped_event_id;
            break;
        }
    }

    state
}

pub fn finalize<P: TransportProtocol>(
    context: &mut TransportContext<P>,
    state: &TransferState,
    packet_loss_prob: f64,
) -> Option<usize> {
    let summary_event_id: Option<usize> = if state.is_complete {
        let end_event: usize = context.log.event(
            P::Event::from(TransportEventType::Complete),
            None,
            None,
            ModuleName::Simulation,
            "Complete",
            format!("All {} transmissions are complete", P::NAME),
            None,
            None,
            None,
        );

        context.log.output(
            ModuleName::Channel,
            Some(end_event),
            format!("All {} transmissions complete.", P::NAME),
        );

        Some(end_event)
    } else {
        state.terminal_event_id.or_else(|| {
            context
                .log
                .events
                .last()
                .map(|event: &SimEvent<P::Event>| -> usize { event.id })
        })
    };

    let sender_packets: usize = context.sender.packet_count;
    let receiver_packets: usize = context.receiver.packet_count;
    let channel_packets: usize = context.channel_stats.p_cnt;

    if state.is_complete {
        context.log.output(
            ModuleName::Sender,
            summary_event_id,
            format!(
                "\nSuccessful transmission of {sender_packets} packets:\n - {} transmissions sent\n - {} transmissions received.\n",
                context.sender.sent_count, context.sender.received_count
            ),
        );

        context.log.output(
            ModuleName::Receiver,
            summary_event_id,
            format!(
                "\nSuccessful transmission of {receiver_packets} packets:\n - {} transmissions sent\n - {} transmissions received.\n",
                context.receiver.sent_count, context.receiver.received_count
            ),
        );
    } else if state.is_deadlock {
        context.log.output(
            ModuleName::Sender,
            summary_event_id,
            format!(
                "\nTransmission incomplete due to deadlock:\n - {} data transmissions counted before playback stopped\n - {} acknowledgements received.\n",
                context.sender.sent_count,
                context.sender.received_count
            ),
        );

        context.log.output(
            ModuleName::Receiver,
            summary_event_id,
            format!(
                "\nReceiver terminated after the EOF packet:\n - {} acknowledgements sent\n - {} data transmissions received.\n",
                context.receiver.sent_count, context.receiver.received_count
            ),
        );
    }

    context.log.output(
        ModuleName::Channel,
        summary_event_id,
        format!(
            "\nChannel transmission report for {channel_packets} completed packet-pairs:\n - {} transmissions sent\n - {} transmissions received\n - {} packets lost at a probability of {}%.\n",
            context.channel_stats.sent,
            context.channel_stats.r_cnt,
            context.channel_stats.loss,
            packet_loss_prob * 100.0
        ),
    );

    summary_event_id
}

pub fn common_stats<P: TransportProtocol>(
    context: &TransportContext<P>,
    state: &TransferState,
    packet_loss_prob: f64,
    payload_size: usize,
) -> TransportStats {
    TransportStats {
        sender_packets: context.sender.packet_count,
        sender_transmissions_sent: context.sender.sent_count,
        sender_transmissions_received: context.sender.received_count,
        receiver_packets: context.receiver.packet_count,
        receiver_transmissions_sent: context.receiver.sent_count,
        receiver_transmissions_received: context.receiver.received_count,
        channel_packets: context.channel_stats.p_cnt,
        channel_transmissions_sent: context.channel_stats.sent,
        channel_transmissions_received: context.channel_stats.r_cnt,
        packets_lost: context.channel_stats.loss,
        packet_loss_prob,
        payload_size,
        is_complete: state.is_complete,
        is_deadlock: state.is_deadlock,
    }
}
