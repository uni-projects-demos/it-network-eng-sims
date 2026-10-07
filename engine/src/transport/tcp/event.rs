use serde::{Serialize, Serializer};

use crate::transport::event::TransportEventType;

#[derive(Clone, Copy, Debug)]
pub enum EventType {
    Transport(TransportEventType),
    SynSend,
    SynAckSend,
    HandshakeAckSend,
    HandshakeComplete,
    HandshakeFailed,
    BitErr,
}

impl From<TransportEventType> for EventType {
    fn from(event: TransportEventType) -> Self {
        Self::Transport(event)
    }
}

impl Serialize for EventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Transport(event) => event.serialize(serializer),
            Self::SynSend => serializer.serialize_str("syn-send"),
            Self::SynAckSend => serializer.serialize_str("syn-ack-send"),
            Self::HandshakeAckSend => serializer.serialize_str("handshake-ack-send"),
            Self::HandshakeComplete => serializer.serialize_str("handshake-complete"),
            Self::HandshakeFailed => serializer.serialize_str("handshake-failed"),
            Self::BitErr => serializer.serialize_str("bit-err"),
        }
    }
}
