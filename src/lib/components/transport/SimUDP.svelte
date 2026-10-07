<script lang="ts">
import SimTransport from "#lib/components/transport/SimTransport.svelte";
import {
  type Locale,
  type TranslationParams,
  translate,
} from "#lib/i18n/index.ts";
import type { Endpoint, SimEvent, SimResUDP } from "#lib/sims/transport/typesUDP.ts";
import { runSimUDP } from "#lib/wasm/engine.ts";

type TranslateFn = (key: string, params?: TranslationParams) => string;

type StatRow = {
  label: string;
  value: number;
};

function translateUDP(
  key: string,
  code: Locale,
  params: TranslationParams = {},
): string {
  return translate(`udp.${key}`, code, params);
}

function endpointName(endpoint: Endpoint | undefined, code: Locale): string {
  if (endpoint === undefined) {
    return "";
  }

  return translateUDP(endpoint, code);
}

function packetName(event: SimEvent, code: Locale): string {
  const packet: SimEvent["packet"] = event.packet;
  if (packet === undefined) {
    return "";
  }

  if (packet.data_type === 0) {
    return translateUDP("packetData", code, {
      seq: packet.seq_no,
      len: packet.data_len,
    });
  }

  return translateUDP("packetAck", code, {
    seq: packet.seq_no,
    len: packet.data_len,
  });
}

function eventText(event: SimEvent, code: Locale): string {
  const tr: TranslateFn = (key: string, params: TranslationParams = {}): string =>
    translateUDP(key, code, params);

  const packet: string = packetName(event, code);
  const seq: number = event.packet?.seq_no ?? 0;
  const len: number = event.packet?.data_len ?? 0;

  switch (event._type) {
    case "data-send":
    case "ack-send":
      if (event.module === "channel" && event.to === "channel") {
        return tr("eventChannelReceived", {
          packet,
          from: endpointName(event.from, code),
        });
      }

      if (event.from === "channel") {
        return tr("eventForwarded", {
          packet,
          to: endpointName(event.to, code),
        });
      }

      return tr("eventSent", {
        packet,
        from: endpointName(event.from, code),
        to: endpointName(event.to, code),
      });

    case "deliver":
      return tr("eventForwarded", {
        packet,
        to: endpointName(event.to, code),
      });

    case "packet-received":
      return tr("eventPacketReceived", {
        packet,
        to: endpointName(event.to, code),
      });

    case "packet-loss":
      if (event.random_val !== undefined && event.prob !== undefined) {
        return tr("eventPacketLost", {
          packet,
          random: event.random_val.toFixed(4),
          probability: event.prob.toFixed(2),
        });
      }

      return tr("eventInvalidPacket", { packet });

    case "timeout":
      return tr("eventTimeout", { packet });

    case "accepted":
      return event.module === "receiver"
        ? tr("eventDataAccepted", { seq, len })
        : tr("eventAckAccepted", { seq, len });

    case "duplicate":
      return tr("eventDuplicate", { seq });

    case "receiver-terminated":
      return tr("eventReceiverTerminated");

    case "receiver-unavailable":
      return tr("eventReceiverUnavailable");

    case "deadlock":
      return tr("eventDeadlock");

    case "complete":
      return tr("eventComplete");

    default:
      return event.detail || tr("eventGeneric", { detail: event.detail });
  }
}

function receivedPacketLabel(event: SimEvent, code: Locale): string {
  return event.packet?.data_type === 1
    ? translateUDP("ackReceived", code)
    : translateUDP("dataReceived", code);
}

function isAckPacket(event: SimEvent): boolean {
  return event.packet?.data_type === 1;
}

function calcStatRows(events: SimEvent[], code: Locale): readonly StatRow[] {
  const dataTransmissionAttempts: number = events.filter(
    (event: SimEvent): boolean =>
      event.to === "channel" && event._type === "data-send" && event.packet?.data_type === 0,
  ).length;

  const ackTransmissionAttempts: number = events.filter(
    (event: SimEvent): boolean => event.to === "channel" && event._type === "ack-send",
  ).length;

  const dataPacketsReceived: number = events.filter(
    (event: SimEvent): boolean =>
      event._type === "packet-received" &&
      event.module === "receiver" &&
      event.packet?.data_type === 0 &&
      (event.packet?.data_len ?? 0) > 0,
  ).length;

  const packetLosses: number = events.filter(
    (event: SimEvent): boolean =>
      event.module === "channel" &&
      event._type === "packet-loss" &&
      event.random_val !== undefined,
  ).length;

  const totalTransmissions: number = events.filter(
    (event: SimEvent): boolean =>
      event.to === "channel" && ["data-send", "ack-send"].includes(event._type),
  ).length;

  return [
    { label: translateUDP("dataPacketsReceived", code), value: dataPacketsReceived },
    { label: translateUDP("dataTransmissionAttempts", code), value: dataTransmissionAttempts },
    { label: translateUDP("ackTransmissionAttempts", code), value: ackTransmissionAttempts },
    { label: translateUDP("totalTransmissions", code), value: totalTransmissions },
    { label: translateUDP("packetLosses", code), value: packetLosses },
  ];
}

async function runSimulation(
  msg: string,
  packetLossProb: number,
  payloadSize: number,
): Promise<SimResUDP> {
  return runSimUDP(msg, packetLossProb, payloadSize);
}
</script>

<SimTransport
  translateSim={translateUDP}
  {runSimulation}
  {eventText}
  {packetName}
  {receivedPacketLabel}
  {isAckPacket}
  {calcStatRows}
/>
