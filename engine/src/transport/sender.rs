use super::{
    event::{Endpoint, EventKind, EventLog, ModuleName, TransportEventType},
    packet::{Packet, WirePacket},
    protocol::TransportProtocol,
};

pub struct Sender {
    pub nxt: i32,
    pub packet_count: usize,
    pub received_count: usize,
    pub sent_count: usize,
}

impl Default for Sender {
    fn default() -> Self {
        Self::new()
    }
}

impl Sender {
    pub fn new() -> Self {
        Self {
            nxt: 0,
            packet_count: 1,
            received_count: 0,
            sent_count: 0,
        }
    }

    pub fn make_data_packet<P: TransportProtocol>(&self, protocol: &P, data: Vec<u8>) -> P::Wire {
        protocol.make_wire_packet(Packet::data(self.nxt, data))
    }

    pub fn timeout<E: EventKind>(&self, packet: &Packet, log: &mut EventLog<E>) {
        log.event(
            E::from(TransportEventType::Timeout),
            None,
            None,
            ModuleName::Sender,
            "Timeout / retry",
            format!(
                "No valid ACK received for DATA seq={}; retransmitting",
                packet.seq_no
            ),
            Some(packet),
            None,
            None,
        );
    }

    pub fn receive_ack<P: TransportProtocol>(
        &mut self,
        protocol: &P,
        wire: P::Wire,
        raw_data: &[u8],
        attempts: usize,
        log: &mut EventLog<P::Event>,
    ) -> bool {
        let received_event: usize = log.event(
            P::Event::from(TransportEventType::PacketReceived),
            Some(Endpoint::Channel),
            Some(Endpoint::Sender),
            ModuleName::Sender,
            format!("ACK seq={} received", wire.packet().seq_no),
            format!(
                "ACK packet received at sender (len={})",
                wire.packet().data_len
            ),
            Some(wire.packet()),
            None,
            None,
        );

        log.output(
            ModuleName::Sender,
            Some(received_event),
            format!(
                "ACK packet received from channel: seq={}, len={}.",
                wire.packet().seq_no,
                wire.packet().data_len
            ),
        );

        let packet: Packet = protocol.decode_wire_packet(wire);
        self.received_count += 1;

        if packet.is_sender() {
            let event: usize = log.event(
                P::Event::from(TransportEventType::PacketLoss),
                Some(Endpoint::Channel),
                Some(Endpoint::Sender),
                ModuleName::Sender,
                "Invalid ACK",
                "Received packet from channel is invalid. Packet dropped.",
                Some(&packet),
                None,
                None,
            );

            log.output(
                ModuleName::Sender,
                Some(event),
                "\nReceived packet from channel is invalid. Packet dropped.",
            );

            return false;
        }

        if packet.seq_no != self.nxt {
            return false;
        }

        let event: usize = log.event(
            P::Event::from(TransportEventType::Accepted),
            Some(Endpoint::Channel),
            Some(Endpoint::Sender),
            ModuleName::Sender,
            format!("ACK seq={} accepted", packet.seq_no),
            format!(
                "DATA seq={} acknowledged after {} attempt(s)",
                packet.seq_no, attempts
            ),
            Some(&packet),
            None,
            None,
        );

        log.output(
            ModuleName::Sender,
            Some(event),
            format!(
                "\nData packet #{} transmission success after {} attempt(s). {}-{} data packet contents:\n",
                self.packet_count,
                attempts,
                raw_data.len(),
                P::DATA_UNIT
            ),
        );

        let body: String = if raw_data.is_empty() {
            "<empty packet>".into()
        } else {
            String::from_utf8_lossy(raw_data).into_owned()
        };

        log.output(ModuleName::Sender, Some(event), body);

        protocol.record_acknowledged_send(&mut self.sent_count, attempts);

        self.nxt = 1 - self.nxt;

        if !raw_data.is_empty() {
            self.packet_count += 1;
        }

        true
    }
}
