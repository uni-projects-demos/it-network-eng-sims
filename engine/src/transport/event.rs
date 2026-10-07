use serde::Serialize;
use std::fmt::Debug;

use super::packet::Packet;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransportEventType {
    DataSend,
    AckSend,
    Deliver,
    PacketReceived,
    PacketLoss,
    Timeout,
    Accepted,
    Duplicate,
    ReceiverTerminated,
    ReceiverUnavailable,
    Deadlock,
    Complete,
}

pub trait EventKind: Clone + Copy + Debug + Serialize + From<TransportEventType> {}

impl<T> EventKind for T where T: Clone + Copy + Debug + Serialize + From<TransportEventType> {}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Endpoint {
    Sender,
    Channel,
    Receiver,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleName {
    Sender,
    Channel,
    Receiver,
    Simulation,
}

#[derive(Clone, Debug, Serialize)]
pub struct PacketS {
    pub magic_no: i32,
    pub data_type: i32,
    pub seq_no: i32,
    pub data_len: i32,
    pub data_preview: String,
}

impl From<&Packet> for PacketS {
    fn from(packet: &Packet) -> Self {
        let preview: String = if packet.data.is_empty() {
            "<empty packet>".into()
        } else {
            String::from_utf8_lossy(&packet.data)
                .chars()
                .take(64)
                .collect()
        };

        Self {
            magic_no: packet.magic_no,
            data_type: packet.data_type,
            seq_no: packet.seq_no,
            data_len: packet.data_len,
            data_preview: preview,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SimEvent<E> {
    pub id: usize,
    pub tick: usize,
    pub _type: E,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Endpoint>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Endpoint>,

    pub module: ModuleName,
    pub label: String,
    pub detail: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet: Option<PacketS>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub random_val: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prob: Option<f64>,

    pub output_line_ids: Vec<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_chunk: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutputLine {
    pub id: usize,
    pub module: ModuleName,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<usize>,

    pub txt: String,
}

pub struct EventLog<E> {
    pub events: Vec<SimEvent<E>>,
    pub output: Vec<OutputLine>,
}

#[allow(clippy::derivable_impls)]
impl<E: EventKind> Default for EventLog<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: EventKind> EventLog<E> {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            output: Vec::new(),
        }
    }

    fn event_mut(&mut self, id: usize) -> Option<&mut SimEvent<E>> {
        self.events.iter_mut().find(|event| event.id == id)
    }

    pub fn output(
        &mut self,
        module: ModuleName,
        event_id: Option<usize>,
        txt: impl Into<String>,
    ) -> usize {
        let id: usize = self.output.len();
        let txt: String = txt.into();

        self.output.push(OutputLine {
            id,
            module,
            event_id,
            txt,
        });

        if let Some(event_id) = event_id
            && let Some(event) = self.event_mut(event_id)
        {
            event.output_line_ids.push(id);
        }

        id
    }

    pub fn attach_data_chunk(&mut self, event_id: usize, data: &[u8]) {
        if let Some(event) = self.event_mut(event_id) {
            let data_chunk: String = String::from_utf8_lossy(data).into_owned();
            event.data_chunk = Some(data_chunk);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn event(
        &mut self,
        _type: E,
        from: Option<Endpoint>,
        to: Option<Endpoint>,
        module: ModuleName,
        label: impl Into<String>,
        detail: impl Into<String>,
        packet: Option<&Packet>,
        random_val: Option<f64>,
        prob: Option<f64>,
    ) -> usize {
        let id: usize = self.events.len();
        let label: String = label.into();
        let detail: String = detail.into();
        let packet: Option<PacketS> = packet.map(PacketS::from);

        self.events.push(SimEvent {
            id,
            tick: id,
            _type,
            from,
            to,
            module,
            label,
            detail,
            packet,
            random_val,
            prob,
            output_line_ids: Vec::new(),
            data_chunk: None,
        });

        id
    }
}
