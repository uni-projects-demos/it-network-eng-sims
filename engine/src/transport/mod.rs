pub mod channel;
pub mod event;
pub mod packet;
pub mod protocol;
pub mod receiver;
pub mod sender;
pub mod sim;
pub mod tcp;
pub mod udp;

pub const BYTE_FILE_PAYLOAD_SIZES: &[usize] = &[32, 64, 128, 256, 512, 1024, 1460, 4096];
