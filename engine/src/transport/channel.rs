use super::{
    event::{Endpoint, EventKind, EventLog, ModuleName, TransportEventType},
    packet::{Packet, WirePacket},
    protocol::TransportProtocol,
};

#[derive(Default)]
pub struct ChannelStats {
    pub p_cnt: usize,
    pub r_cnt: usize,
    pub sent: usize,
    pub loss: usize,
}

#[derive(Clone, Copy)]
pub enum Direction {
    FromSender,
    FromReceiver,
}

pub struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    pub(crate) fn next_u32(&mut self) -> u32 {
        let mut x: u32 = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    pub fn uniform_4(&mut self) -> f64 {
        let val: f64 = self.next_u32() as f64 / u32::MAX as f64;
        (val * 10_000.0).round() / 10_000.0
    }
}

pub struct TransitParams<'a, E: EventKind> {
    pub packet_loss_prob: f64,
    pub stats: &'a mut ChannelStats,
    pub rng: &'a mut SimpleRng,
    pub log: &'a mut EventLog<E>,
    pub is_final_data_seen: bool,
}

pub fn transit<P: TransportProtocol>(
    protocol: &mut P,
    mut wire: P::Wire,
    direction: Direction,
    params: TransitParams<'_, P::Event>,
) -> Option<P::Wire> {
    let TransitParams {
        packet_loss_prob,
        stats,
        rng,
        log,
        is_final_data_seen,
    } = params;

    let (from, to): (Endpoint, Endpoint) = match direction {
        Direction::FromSender => (Endpoint::Sender, Endpoint::Receiver),
        Direction::FromReceiver => (Endpoint::Receiver, Endpoint::Sender),
    };

    let (event_type, type_name): (P::Event, &'static str) =
        P::packet_event_type(wire.packet(), direction);

    stats.r_cnt += 1;

    let msg: String = format!(
        "{} packet containing {} {} transmitted from {}",
        type_name,
        wire.packet().data_len,
        P::CHANNEL_DATA_UNIT,
        endpoint_name(from)
    );

    let receive_event: usize = log.event(
        event_type,
        Some(from),
        Some(Endpoint::Channel),
        ModuleName::Channel,
        format!("{} received by channel", type_name),
        &msg,
        Some(wire.packet()),
        None,
        None,
    );

    log.output(ModuleName::Channel, Some(receive_event), msg);

    if !wire.packet().is_magic() {
        stats.sent += 1;

        let msg: &'static str = "Received packet invalid. Packet dropped.";
        let event: usize = log.event(
            P::Event::from(TransportEventType::PacketLoss),
            None,
            None,
            ModuleName::Channel,
            "Invalid packet dropped",
            msg,
            Some(wire.packet()),
            None,
            None,
        );

        log.output(ModuleName::Channel, Some(event), msg);
        return None;
    }

    let u: f64 = rng.uniform_4();
    if u < packet_loss_prob {
        stats.loss += 1;
        stats.sent += 1;

        let event: usize = log.event(
            P::Event::from(TransportEventType::PacketLoss),
            None,
            None,
            ModuleName::Channel,
            "Packet dropped",
            format!("μ={u:.4} < P={packet_loss_prob}"),
            Some(wire.packet()),
            Some(u),
            Some(packet_loss_prob),
        );

        log.output(
            ModuleName::Channel,
            Some(event),
            format!(
                "A μ value of {u} < {packet_loss_prob}, indicating #{} occurrence of the probability of a packet loss event.",
                stats.loss
            ),
        );

        if is_final_data_seen && matches!(direction, Direction::FromReceiver) {
            log.output(
                ModuleName::Channel,
                Some(event),
                format!(
                    "\nProtocol deadlock: final acknowledgement packet is lost with probability {packet_loss_prob}.\n"
                ),
            );
        }

        return None;
    }

    protocol.apply_channel_effects(&mut wire, rng, log);

    let msg: String = format!(
        "{} packet containing {} {} transmitted to {}",
        type_name,
        wire.packet().data_len,
        P::CHANNEL_DATA_UNIT,
        endpoint_name(to)
    );

    let send_event: usize = log.event(
        if wire.packet().data_type == Packet::PTYPE_DATA {
            P::Event::from(TransportEventType::Deliver)
        } else {
            event_type
        },
        Some(Endpoint::Channel),
        Some(to),
        ModuleName::Channel,
        format!("{} forwarded", type_name),
        &msg,
        Some(wire.packet()),
        None,
        None,
    );

    log.output(ModuleName::Channel, Some(send_event), msg);

    stats.sent += 1;

    if matches!(direction, Direction::FromReceiver) {
        stats.p_cnt += 2;
    }

    Some(wire)
}

fn endpoint_name(endpoint: Endpoint) -> &'static str {
    match endpoint {
        Endpoint::Sender => "sender",
        Endpoint::Channel => "channel",
        Endpoint::Receiver => "receiver",
    }
}
