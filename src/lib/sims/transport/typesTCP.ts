import type {
  TransportEvent,
  EventType as TransportEventType,
  TransportPacket,
  PacketType as TransportPacketType,
  TransportResult,
} from "./typesTransport.ts";

export type {
  Endpoint,
  ModuleName,
} from "./typesTransport.ts";

export type EventType =
  | TransportEventType
  | "syn-send"
  | "syn-ack-send"
  | "handshake-ack-send"
  | "handshake-complete"
  | "handshake-failed"
  | "bit-err";

export type PacketType = TransportPacketType | 2 | 3 | 4;

export type PacketS = TransportPacket<PacketType>;

export type SimEvent = TransportEvent<EventType, PacketType>;

export type SimResTCP = TransportResult<SimEvent>;
