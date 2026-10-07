<script lang="ts">
import SimTransport from "#lib/components/transport/SimTransport.svelte";
import {
  type Locale,
  type TranslationParams,
  translate,
} from "#lib/i18n/index.ts";
import type { Endpoint, SimEvent, SimResTCP } from "#lib/sims/transport/typesTCP.ts";
import { runSimTCP } from "#lib/wasm/engine.ts";

type TranslateFn = (key: string, params?: TranslationParams) => string;

type StatRow = {
  label: string;
  value: number;
};

type SpecialEventDisplay = {
  type: "warning" | "info" | "danger";
  label: string;
};

let bitErrProb: number = 0.1;
let isHandshake: boolean = false;

const headerSize: number = 20;

function translateTCP(
  key: string,
  code: Locale,
  params: TranslationParams = {},
): string {
  return translate(`tcp.${key}`, code, params);
}

function endpointName(endpoint: Endpoint | undefined, code: Locale): string {
  if (endpoint === undefined) {
    return "";
  }

  return translateTCP(endpoint, code);
}

function packetName(event: SimEvent, code: Locale): string {
  const packet: SimEvent["packet"] = event.packet;
  if (packet === undefined) {
    return "";
  }

  if (packet.data_type === 0) {
    return translateTCP("packetData", code, {
      seq: packet.seq_no,
      len: packet.data_len,
    });
  }

  if (packet.data_type === 2) {
    return translateTCP("packetSyn", code, {
      seq: packet.seq_no,
      len: packet.data_len,
    });
  }

  if (packet.data_type === 3) {
    return translateTCP("packetSynAck", code, {
      seq: packet.seq_no,
      len: packet.data_len,
    });
  }

  return translateTCP("packetAck", code, {
    seq: packet.seq_no,
    len: packet.data_len,
  });
}

function eventText(event: SimEvent, code: Locale): string {
  const tr: TranslateFn = (key: string, params: TranslationParams = {}): string =>
    translateTCP(key, code, params);

  const packet: string = packetName(event, code);
  const seq: number = event.packet?.seq_no ?? 0;
  const len: number = event.packet?.data_len ?? 0;

  switch (event._type) {
    case "data-send":
    case "ack-send":
    case "syn-send":
    case "syn-ack-send":
    case "handshake-ack-send":
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

    case "bit-err":
      return tr("eventBitError", {
        packet,
        random: event.random_val?.toFixed(4) ?? "—",
        probability: event.prob?.toFixed(2) ?? "—",
      });

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

    case "handshake-complete":
      return tr("eventHandshakeComplete");

    case "handshake-failed":
      return tr("eventHandshakeFailed");

    case "complete":
      return tr("eventComplete");

    default:
      return event.detail || tr("eventGeneric", { detail: event.detail });
  }
}

function receivedPacketLabel(event: SimEvent, code: Locale): string {
  const type: number | undefined = event.packet?.data_type;

  if (type === 2) {
    return translateTCP("synReceived", code);
  }

  if (type === 3) {
    return translateTCP("synAckReceived", code);
  }

  if (type === 1 || type === 4) {
    return translateTCP("ackReceived", code);
  }

  return translateTCP("dataReceived", code);
}

function isAckPacket(event: SimEvent): boolean {
  const dataType: number = event.packet?.data_type ?? -1;
  return [1, 3, 4].includes(dataType);
}

function specialEventDisplay(
  event: SimEvent,
  code: Locale,
): SpecialEventDisplay | undefined {
  if (event._type === "bit-err") {
    return {
      type: "warning",
      label: translateTCP("bitErrInjected", code),
    };
  }

  if (event._type === "handshake-complete") {
    return {
      type: "info",
      label: translateTCP("handshakeComplete", code),
    };
  }

  if (event._type === "handshake-failed") {
    return {
      type: "danger",
      label: translateTCP("handshakeFailed", code),
    };
  }

  return undefined;
}

function calcStatRows(events: SimEvent[], code: Locale): readonly StatRow[] {
  const dataTransmissionAttempts: number = events.filter(
    (event: SimEvent): boolean =>
      event.to === "channel" && event._type === "data-send" && event.packet?.data_type === 0,
  ).length;

  const ackTransmissionAttempts: number = events.filter(
    (event: SimEvent): boolean =>
      event.to === "channel" &&
      ["ack-send", "syn-ack-send", "handshake-ack-send"].includes(event._type),
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

  const bitErrs: number = events.filter(
    (event: SimEvent): boolean => event.module === "channel" && event._type === "bit-err",
  ).length;

  const totalTransmissions: number = events.filter(
    (event: SimEvent): boolean =>
      event.to === "channel" &&
      ["data-send", "ack-send", "syn-send", "syn-ack-send", "handshake-ack-send"].includes(
        event._type,
      ),
  ).length;

  return [
    { label: translateTCP("dataPacketsReceived", code), value: dataPacketsReceived },
    { label: translateTCP("dataTransmissionAttempts", code), value: dataTransmissionAttempts },
    { label: translateTCP("ackTransmissionAttempts", code), value: ackTransmissionAttempts },
    { label: translateTCP("totalTransmissions", code), value: totalTransmissions },
    { label: translateTCP("packetLosses", code), value: packetLosses },
    { label: translateTCP("bitErrs", code), value: bitErrs },
  ];
}

async function runSimulation(
  msg: string,
  packetLossProb: number,
  payloadSize: number,
): Promise<SimResTCP> {
  bitErrProb = Math.min(0.95, Math.max(0, Number(bitErrProb)));

  return runSimTCP(
    msg,
    packetLossProb,
    bitErrProb,
    payloadSize,
    isHandshake,
  );
}
</script>

{#snippet settingsBeforePayload(t: TranslateFn)}
  <label class="mt-[9px] grid gap-[5px] text-[10px] font-[650] text-[var(--control-text)]">
    <span class="flex items-baseline justify-between gap-2">
      {t('bitErrProb')} <strong>N = {bitErrProb.toFixed(2)}</strong>
    </span>
    <input
      class="w-full accent-[var(--primary)] focus:border-[var(--accent-border)] focus:shadow-[0_0_0_3px_var(--focus-ring)]"
      bind:value={bitErrProb}
      type="range"
      min="0"
      max="0.99"
      step="0.01"
    />
  </label>
{/snippet}

{#snippet settingsAfterPayload(t: TranslateFn)}
  <div
    class="mt-2.5 flex min-h-[38px] items-center justify-between gap-3 rounded-[9px] border border-[var(--field-border)] bg-[var(--field-bg)] px-[9px] py-1.5 text-[10px] font-[650] text-[var(--control-text)]"
  >
    <span>{t('handshake')}</span>
    <button
      type="button"
      class="binary-switch flex h-[26px] w-[46px] items-center justify-start rounded-full border border-[var(--control-border)] bg-[var(--control-bg)] p-[3px] transition-colors duration-150"
      class:on={isHandshake}
      role="switch"
      aria-checked={isHandshake}
      aria-label={t('includeHandshake')}
      onclick={(): void => {
        isHandshake = !isHandshake;
      }}
    >
      <span class="size-[18px] rounded-full bg-[var(--page-text)] shadow-[0_1px_4px_rgba(0,0,0,0.25)]" aria-hidden="true"></span>
    </button>
  </div>
{/snippet}

<SimTransport
  translateSim={translateTCP}
  {runSimulation}
  {eventText}
  {packetName}
  {receivedPacketLabel}
  {isAckPacket}
  {calcStatRows}
  {specialEventDisplay}
  {settingsBeforePayload}
  {settingsAfterPayload}
  payloadHeaderKey="header"
  payloadHeaderSize={headerSize}
/>

<style>
  .binary-switch.on {
    justify-content: flex-end;
    border-color: var(--primary);
    background: color-mix(in srgb, var(--primary) 34%, var(--control-bg));
  }
</style>
