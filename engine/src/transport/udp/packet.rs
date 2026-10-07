use crate::transport::packet::{Packet, WirePacket as TransportWirePacket};

#[derive(Clone, Debug)]
pub struct WirePacket {
    pub packet: Packet,
}

impl WirePacket {
    pub fn new(packet: Packet) -> Self {
        Self { packet }
    }

    pub fn into_packet(self) -> Packet {
        self.packet
    }
}

impl TransportWirePacket for WirePacket {
    fn packet(&self) -> &Packet {
        &self.packet
    }
}
