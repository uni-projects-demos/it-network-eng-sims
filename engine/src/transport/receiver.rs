use super::{
    event::{Endpoint, EventLog, ModuleName, TransportEventType},
    packet::{Packet, WirePacket},
    protocol::TransportProtocol,
};

pub struct Receiver {
    pub expected: i32,
    pub sent_count: usize,
    pub received_count: usize,
    pub packet_count: usize,
    pub received_bytes: Vec<u8>,
    pub is_terminated: bool,
}

impl Default for Receiver {
    fn default() -> Self {
        Self::new()
    }
}

impl Receiver {
    pub fn new() -> Self {
        Self {
            expected: 0,
            sent_count: 0,
            received_count: 0,
            packet_count: 1,
            received_bytes: Vec::new(),
            is_terminated: false,
        }
    }

    pub fn receive_data<P: TransportProtocol>(
        &mut self,
        protocol: &P,
        wire: P::Wire,
        log: &mut EventLog<P::Event>,
    ) -> Option<(P::Wire, bool)> {
        if self.is_terminated {
            let event: usize = log.event(
                P::Event::from(TransportEventType::ReceiverUnavailable),
                Some(Endpoint::Channel),
                Some(Endpoint::Receiver),
                ModuleName::Receiver,
                "Receiver unavailable",
                "Receiver has already terminated after the EOF packet; no ACK can be generated",
                Some(wire.packet()),
                None,
                None,
            );

            log.output(
                ModuleName::Receiver,
                Some(event),
                "Receiver has already terminated. Retransmitted final DATA packet cannot be acknowledged.",
            );

            return None;
        }

        let received_event: usize = log.event(
            P::Event::from(TransportEventType::PacketReceived),
            Some(Endpoint::Channel),
            Some(Endpoint::Receiver),
            ModuleName::Receiver,
            format!("DATA seq={} received", wire.packet().seq_no),
            format!(
                "DATA packet received at receiver (len={})",
                wire.packet().data_len
            ),
            Some(wire.packet()),
            None,
            None,
        );

        log.output(
            ModuleName::Receiver,
            Some(received_event),
            format!(
                "DATA packet received from channel: seq={}, len={}.",
                wire.packet().seq_no,
                wire.packet().data_len
            ),
        );

        let packet: Packet = protocol.decode_wire_packet(wire);
        self.received_count += 1;

        if !packet.is_receiver() {
            let event: usize = log.event(
                P::Event::from(TransportEventType::PacketLoss),
                Some(Endpoint::Channel),
                Some(Endpoint::Receiver),
                ModuleName::Receiver,
                "Invalid data",
                "Received packet from channel is invalid. Packet dropped.",
                Some(&packet),
                None,
                None,
            );

            log.output(
                ModuleName::Receiver,
                Some(event),
                "\nReceived packet from channel is invalid. Packet dropped.",
            );

            return None;
        }

        let is_expected: bool = packet.seq_no == self.expected;

        if is_expected {
            let event: usize = log.event(
                P::Event::from(TransportEventType::Accepted),
                Some(Endpoint::Channel),
                Some(Endpoint::Receiver),
                ModuleName::Receiver,
                format!("DATA seq={} accepted", packet.seq_no),
                format!("Receiver accepted {} bytes", packet.data.len()),
                Some(&packet),
                None,
                None,
            );

            log.attach_data_chunk(event, &packet.data);

            log.output(
                ModuleName::Receiver,
                Some(event),
                format!(
                    "\nData packet #{} transmission success. {}-{} data packet contents:\n",
                    self.packet_count,
                    packet.data.len(),
                    P::DATA_UNIT
                ),
            );

            let body: String = if packet.data.is_empty() {
                "<empty packet>".into()
            } else {
                String::from_utf8_lossy(&packet.data).into_owned()
            };

            log.output(ModuleName::Receiver, Some(event), body);

            if !packet.data.is_empty() {
                self.received_bytes.extend_from_slice(&packet.data);
                self.packet_count += 1;
            }
        } else {
            log.event(
                P::Event::from(TransportEventType::Duplicate),
                Some(Endpoint::Channel),
                Some(Endpoint::Receiver),
                ModuleName::Receiver,
                format!("Duplicate DATA seq={}", packet.seq_no),
                "Receiver re-sends acknowledgement for duplicate packet",
                Some(&packet),
                None,
                None,
            );
        }

        let ack: P::Wire = protocol.make_wire_packet(Packet::ack(packet.seq_no));
        self.sent_count += 1;

        let is_final_packet: bool = packet.data_len == 0 && is_expected;

        if is_expected {
            self.expected = 1 - self.expected;
        }

        if is_final_packet {
            self.is_terminated = true;

            let event: usize = log.event(
                P::Event::from(TransportEventType::ReceiverTerminated),
                Some(Endpoint::Receiver),
                None,
                ModuleName::Receiver,
                "Receiver terminated",
                "Receiver accepted the empty EOF packet, sent its final ACK, and terminated",
                Some(&packet),
                None,
                None,
            );

            log.output(
                ModuleName::Receiver,
                Some(event),
                "Receiver accepted the final empty DATA packet and terminated after sending its ACK.",
            );
        }

        Some((ack, is_final_packet))
    }
}
