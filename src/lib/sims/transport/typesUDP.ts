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

export type EventType = TransportEventType;

export type PacketType = TransportPacketType;

export type PacketS = TransportPacket<PacketType>;

export type SimEvent = TransportEvent<EventType, PacketType>;

export type SimResUDP = TransportResult<SimEvent>;
