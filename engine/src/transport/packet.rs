use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Packet {
    pub magic_no: i32,
    pub data_type: i32,
    pub seq_no: i32,
    pub data_len: i32,

    #[serde(skip)]
    pub data: Vec<u8>,
}

impl Packet {
    pub const MAGIC_NO: i32 = 0x497E;
    pub const PTYPE_DATA: i32 = 0;
    pub const PTYPE_ACK: i32 = 1;

    pub fn data(seq_no: i32, data: Vec<u8>) -> Self {
        Self {
            magic_no: Self::MAGIC_NO,
            data_type: Self::PTYPE_DATA,
            seq_no,
            data_len: data.len() as i32,
            data,
        }
    }

    pub fn ack(seq_no: i32) -> Self {
        Self {
            magic_no: Self::MAGIC_NO,
            data_type: Self::PTYPE_ACK,
            seq_no,
            data_len: 0,
            data: Vec::new(),
        }
    }

    pub fn is_magic(&self) -> bool {
        self.magic_no == Self::MAGIC_NO
    }

    pub fn is_sender(&self) -> bool {
        !self.is_magic() || self.data_type != Self::PTYPE_ACK || self.data_len != 0
    }

    pub fn is_receiver(&self) -> bool {
        self.is_magic() && self.data_type == Self::PTYPE_DATA
    }
}

pub trait WirePacket: Clone {
    fn packet(&self) -> &Packet;
}
