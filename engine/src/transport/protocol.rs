use super::{
    channel::{Direction, SimpleRng},
    event::{EventKind, EventLog},
    packet::{Packet, WirePacket},
};

pub trait TransportProtocol {
    type Event: EventKind;
    type Wire: WirePacket;

    const NAME: &'static str;
    const CHANNEL_DATA_UNIT: &'static str;
    const DATA_UNIT: &'static str;

    fn make_wire_packet(&self, packet: Packet) -> Self::Wire;

    fn decode_wire_packet(&self, wire: Self::Wire) -> Packet;

    fn packet_event_type(packet: &Packet, direction: Direction) -> (Self::Event, &'static str);

    fn record_send_attempt(&self, sent_count: &mut usize);

    fn record_acknowledged_send(&self, sent_count: &mut usize, attempts: usize);

    fn apply_channel_effects(
        &mut self,
        _wire: &mut Self::Wire,
        _rng: &mut SimpleRng,
        _log: &mut EventLog<Self::Event>,
    ) {
    }
}
