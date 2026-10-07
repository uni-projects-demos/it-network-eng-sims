pub mod event;
pub mod packet;
pub mod sim;

use super::{
    channel::Direction, event::TransportEventType, packet::Packet, protocol::TransportProtocol,
};
use event::EventType;
use packet::WirePacket;

pub struct Udp;

impl TransportProtocol for Udp {
    type Event = EventType;
    type Wire = WirePacket;

    const NAME: &'static str = "UDP";
    const CHANNEL_DATA_UNIT: &'static str = "bytes";
    const DATA_UNIT: &'static str = "byte";

    fn make_wire_packet(&self, packet: Packet) -> Self::Wire {
        WirePacket::new(packet)
    }

    fn decode_wire_packet(&self, wire: Self::Wire) -> Packet {
        wire.into_packet()
    }

    fn packet_event_type(packet: &Packet, direction: Direction) -> (Self::Event, &'static str) {
        match packet.data_type {
            Packet::PTYPE_ACK => (TransportEventType::AckSend, "Acknowledgement"),
            _ if matches!(direction, Direction::FromSender) => {
                (TransportEventType::DataSend, "Data")
            }
            _ => (TransportEventType::AckSend, "Acknowledgement"),
        }
    }

    fn record_send_attempt(&self, sent_count: &mut usize) {
        *sent_count += 1;
    }

    fn record_acknowledged_send(&self, _sent_count: &mut usize, _attempts: usize) {}
}
