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
  data_type: TPacketType;
  seq_no: number;
  data_len: number;
}

export interface TransportEvent<
  TEventType extends string = EventType,
  TPacketType extends number = PacketType,
> {
  id: number;
  _type: TEventType;
  module: ModuleName;
  detail: string;
  from?: Endpoint;
  to?: Endpoint;
  packet?: TransportPacket<TPacketType>;
  random_val?: number;
  prob?: number;
  data_chunk?: string;
}

export interface TransportResult<
  TEvent extends TransportEvent<string, number> = TransportEvent<EventType, PacketType>,
> {
  events: TEvent[];
}
