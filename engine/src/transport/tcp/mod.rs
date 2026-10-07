pub mod channel;
pub mod event;
mod handshake;
pub mod packet;
pub mod sim;

use super::{
    channel::{Direction, SimpleRng},
    event::{EventLog, TransportEventType},
    packet::Packet,
    protocol::TransportProtocol,
};
use event::EventType;
use packet::WirePacket;

pub const HEADER_SIZE: usize = 20;
pub const PTYPE_SYN: i32 = 2;
pub const PTYPE_SYN_ACK: i32 = 3;
pub const PTYPE_HANDSHAKE_ACK: i32 = 4;

pub struct Tcp {
    pub channel: channel::ChannelState,
}

impl Tcp {
    pub fn new(bit_err_prob: f64) -> Self {
        Self {
            channel: channel::ChannelState::new(bit_err_prob),
        }
    }
}

impl TransportProtocol for Tcp {
    type Event = EventType;
    type Wire = WirePacket;

    const NAME: &'static str = "TCP";
    const CHANNEL_DATA_UNIT: &'static str = "chars";
    const DATA_UNIT: &'static str = "char";

    fn make_wire_packet(&self, packet: Packet) -> Self::Wire {
        WirePacket::new(packet)
    }

    fn decode_wire_packet(&self, wire: Self::Wire) -> Packet {
        wire.into_corrected_packet()
    }

    fn packet_event_type(packet: &Packet, direction: Direction) -> (Self::Event, &'static str) {
        match packet.data_type {
            PTYPE_SYN => (EventType::SynSend, "SYN"),
            PTYPE_SYN_ACK => (EventType::SynAckSend, "SYN-ACK"),
            PTYPE_HANDSHAKE_ACK => (EventType::HandshakeAckSend, "ACK"),
            Packet::PTYPE_ACK => (
                EventType::from(TransportEventType::AckSend),
                "Acknowledgement",
            ),
            _ if matches!(direction, Direction::FromSender) => {
                (EventType::from(TransportEventType::DataSend), "Data")
            }
            _ => (
                EventType::from(TransportEventType::AckSend),
                "Acknowledgement",
            ),
        }
    }

    fn record_send_attempt(&self, _sent_count: &mut usize) {}

    fn record_acknowledged_send(&self, sent_count: &mut usize, attempts: usize) {
        *sent_count += attempts;
    }

    fn apply_channel_effects(
        &mut self,
        wire: &mut Self::Wire,
        rng: &mut SimpleRng,
        log: &mut EventLog<Self::Event>,
    ) {
        channel::apply_bit_error(&mut self.channel, wire, rng, log);
    }
}
