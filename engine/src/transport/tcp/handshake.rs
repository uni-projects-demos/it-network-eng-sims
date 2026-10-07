use super::{
    PTYPE_HANDSHAKE_ACK, PTYPE_SYN, PTYPE_SYN_ACK, Tcp, event::EventType, packet::WirePacket,
};
use crate::transport::{
    channel::Direction,
    event::{Endpoint, EventLog, ModuleName, TransportEventType},
    packet::{Packet, WirePacket as TransportWirePacket},
    protocol::TransportProtocol,
    sim::TransportContext,
};

const MAX_HANDSHAKE_ATTEMPTS: usize = 1_000;

pub fn run(context: &mut TransportContext<Tcp>, packet_loss_prob: f64) -> bool {
    context
        .log
        .output(ModuleName::Sender, None, "TCP three-way handshake enabled.");

    let syn: WirePacket = control_packet(PTYPE_SYN);

    for syn_attempt in 1..=MAX_HANDSHAKE_ATTEMPTS {
        let Some(syn_received): Option<WirePacket> =
            context.transit(syn.clone(), Direction::FromSender, packet_loss_prob, false)
        else {
            handshake_timeout(
                ModuleName::Sender,
                syn.packet(),
                &mut context.log,
                format!(
                    "SYN attempt #{syn_attempt} was not delivered; sender retries the handshake"
                ),
            );
            continue;
        };

        let syn_received: Packet = context.protocol.decode_wire_packet(syn_received);
        log_control_received(&syn_received, Endpoint::Receiver, &mut context.log);

        if !is_control_packet(&syn_received, PTYPE_SYN) {
            handshake_timeout(
                ModuleName::Sender,
                syn.packet(),
                &mut context.log,
                "Invalid SYN received; sender retries the handshake",
            );
            continue;
        }

        let syn_ack: WirePacket = control_packet(PTYPE_SYN_ACK);
        let Some(syn_ack_received): Option<WirePacket> = context.transit(
            syn_ack.clone(),
            Direction::FromReceiver,
            packet_loss_prob,
            false,
        ) else {
            handshake_timeout(
                ModuleName::Sender,
                syn_ack.packet(),
                &mut context.log,
                "SYN-ACK was not delivered; sender times out and retransmits SYN",
            );
            continue;
        };

        let syn_ack_received: Packet = context.protocol.decode_wire_packet(syn_ack_received);
        log_control_received(&syn_ack_received, Endpoint::Sender, &mut context.log);

        if !is_control_packet(&syn_ack_received, PTYPE_SYN_ACK) {
            handshake_timeout(
                ModuleName::Sender,
                syn_ack.packet(),
                &mut context.log,
                "Invalid SYN-ACK received; sender retries the handshake",
            );
            continue;
        }

        let ack: WirePacket = control_packet(PTYPE_HANDSHAKE_ACK);

        for ack_attempt in 1..=MAX_HANDSHAKE_ATTEMPTS {
            let Some(ack_received): Option<WirePacket> =
                context.transit(ack.clone(), Direction::FromSender, packet_loss_prob, false)
            else {
                handshake_timeout(
                    ModuleName::Receiver,
                    ack.packet(),
                    &mut context.log,
                    format!(
                        "Handshake ACK attempt #{ack_attempt} was not delivered; receiver retransmits SYN-ACK"
                    ),
                );

                let Some(repeat_syn_ack): Option<WirePacket> = context.transit(
                    syn_ack.clone(),
                    Direction::FromReceiver,
                    packet_loss_prob,
                    false,
                ) else {
                    continue;
                };

                let repeat_syn_ack: Packet = context.protocol.decode_wire_packet(repeat_syn_ack);
                log_control_received(&repeat_syn_ack, Endpoint::Sender, &mut context.log);
                continue;
            };

            let ack_received: Packet = context.protocol.decode_wire_packet(ack_received);
            log_control_received(&ack_received, Endpoint::Receiver, &mut context.log);

            if !is_control_packet(&ack_received, PTYPE_HANDSHAKE_ACK) {
                continue;
            }

            let complete_event: usize = context.log.event(
                EventType::HandshakeComplete,
                Some(Endpoint::Sender),
                Some(Endpoint::Receiver),
                ModuleName::Simulation,
                "Handshake complete",
                "SYN, SYN-ACK and ACK completed; file transfer can begin",
                None,
                None,
                None,
            );

            context.log.output(
                ModuleName::Sender,
                Some(complete_event),
                "Three-way handshake complete. TCP file transfer begins.",
            );

            return true;
        }
    }

    let failed_event: usize = context.log.event(
        EventType::HandshakeFailed,
        Some(Endpoint::Sender),
        Some(Endpoint::Receiver),
        ModuleName::Simulation,
        "Handshake failed",
        "Handshake did not complete within the bounded simulation attempt limit",
        None,
        None,
        None,
    );

    context.log.output(
        ModuleName::Sender,
        Some(failed_event),
        "Three-way handshake did not complete within the simulation attempt limit.",
    );

    false
}

fn control_packet(data_type: i32) -> WirePacket {
    WirePacket::new(Packet {
        magic_no: Packet::MAGIC_NO,
        data_type,
        seq_no: 0,
        data_len: 0,
        data: Vec::new(),
    })
}

fn is_control_packet(packet: &Packet, data_type: i32) -> bool {
    packet.is_magic() && packet.data_type == data_type
}

fn handshake_timeout(
    module: ModuleName,
    packet: &Packet,
    log: &mut EventLog<EventType>,
    detail: impl Into<String>,
) {
    let detail: String = detail.into();
    let event: usize = log.event(
        EventType::from(TransportEventType::Timeout),
        None,
        None,
        module,
        "Handshake timeout",
        detail.as_str(),
        Some(packet),
        None,
        None,
    );

    log.output(module, Some(event), detail);
}

fn log_control_received(packet: &Packet, endpoint: Endpoint, log: &mut EventLog<EventType>) {
    let (module, name): (ModuleName, &'static str) = match endpoint {
        Endpoint::Sender => (ModuleName::Sender, "sender"),
        Endpoint::Receiver => (ModuleName::Receiver, "receiver"),
        Endpoint::Channel => (ModuleName::Channel, "channel"),
    };

    let packet_name: &'static str = match packet.data_type {
        PTYPE_SYN => "SYN",
        PTYPE_SYN_ACK => "SYN-ACK",
        PTYPE_HANDSHAKE_ACK | Packet::PTYPE_ACK => "ACK",
        _ => "DATA",
    };

    let event: usize = log.event(
        EventType::from(TransportEventType::PacketReceived),
        Some(Endpoint::Channel),
        Some(endpoint),
        module,
        format!("{packet_name} received"),
        format!(
            "{packet_name} packet received at {name} (len={})",
            packet.data_len
        ),
        Some(packet),
        None,
        None,
    );

    log.output(
        module,
        Some(event),
        format!(
            "{packet_name} packet received from channel: len={}.",
            packet.data_len
        ),
    );
}
