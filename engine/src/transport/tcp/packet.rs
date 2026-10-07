use crate::transport::packet::{Packet, WirePacket as TransportWirePacket};

#[derive(Clone, Debug)]
pub struct WirePacket {
    pub packet: Packet,
    checksum: i32,
}

impl WirePacket {
    pub fn new(packet: Packet) -> Self {
        let checksum: i32 = checksum(&packet);
        Self { packet, checksum }
    }

    pub fn into_corrected_packet(mut self) -> Packet {
        let current_checksum: i32 = checksum(&self.packet);

        if self.checksum != current_checksum {
            self.packet.data_len =
                self.checksum - (self.packet.magic_no + self.packet.data_type + self.packet.seq_no);
        }

        self.packet
    }
}

impl TransportWirePacket for WirePacket {
    fn packet(&self) -> &Packet {
        &self.packet
    }
}

fn checksum(packet: &Packet) -> i32 {
    packet.magic_no + packet.data_type + packet.seq_no + packet.data_len
}
