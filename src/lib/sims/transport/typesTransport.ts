export type Endpoint = "sender" | "channel" | "receiver";
export type ModuleName = Endpoint | "simulation";

export type EventType =
  | "data-send"
  | "ack-send"
  | "deliver"
  | "packet-received"
  | "packet-loss"
  | "timeout"
  | "accepted"
  | "duplicate"
  | "receiver-terminated"
  | "receiver-unavailable"
  | "deadlock"
  | "complete";

export type PacketType = 0 | 1;

export interface TransportPacket<TPacketType extends number = PacketType> {
  magic_no: number;
  data_type: TPacketType;
  seq_no: number;
  data_len: number;
  data_preview: string;
}

export interface TransportEvent<
  TEventType extends string = EventType,
  TPacketType extends number = PacketType,
> {
  id: number;
  tick: number;
  _type: TEventType;
  module: ModuleName;
  label: string;
  detail: string;
  from?: Endpoint;
  to?: Endpoint;
  packet?: TransportPacket<TPacketType>;
  random_val?: number;
  prob?: number;
  output_line_ids: number[];
  data_chunk?: string;
}

export interface OutputLine {
  id: number;
  module: ModuleName;
  event_id?: number;
  txt: string;
}

export interface TransportStats {
  sender_packets: number;
  sender_transmissions_sent: number;
  sender_transmissions_received: number;
  receiver_packets: number;
  receiver_transmissions_sent: number;
  receiver_transmissions_received: number;
  channel_packets: number;
  channel_transmissions_sent: number;
  channel_transmissions_received: number;
  packets_lost: number;
  packet_loss_prob: number;
  payload_size: number;
  is_complete: boolean;
  is_deadlock: boolean;
}

export interface TransportResult<
  TEvent extends TransportEvent<string, number> = TransportEvent<EventType, PacketType>,
  TStats extends TransportStats = TransportStats,
> {
  events: TEvent[];
  output: OutputLine[];
  stats: TStats;
  received_msg: string;
}
