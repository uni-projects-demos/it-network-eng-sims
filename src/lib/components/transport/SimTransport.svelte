<script
  lang="ts"
  generics="TEvent extends TransportEvent<string, number>"
>
import { onDestroy, tick, type Snippet } from "svelte";
import PayloadSizeSelect from "#lib/components/transport/PayloadSizeSelect.svelte";
import PlaybackControls from "#lib/components/simulation/PlaybackControls.svelte";
import ConsoleTabs from "#lib/components/simulation/ConsoleTabs.svelte";
import StatRows from "#lib/components/simulation/StatRows.svelte";
import { SIM_PANEL, SIM_HEADING, SIM_DIAGRAM_HEADER, SIM_SETTINGS_LBL, SIM_PRIMARY_BTN } from "#lib/components/simulation/styles.ts";
import type { StatRow } from "#lib/sims/shared/stats.ts";
import { PlaybackTimer, type PlaybackDir } from "#lib/sims/shared/playback.ts";
import {
  isLocaleRTL,
  type Locale,
  locale,
  type TranslationParams,
} from "#lib/i18n/index.ts";
import { DEFAULT_PAYLOAD_SIZE } from "#lib/sims/transport/payload.ts";
import type {
  Endpoint,
  ModuleName,
  TransportEvent,
  TransportResult,
} from "#lib/sims/transport/typesTransport.ts";

type ModuleTab = "all" | ModuleName;

type TranslateFn = (key: string, params?: TranslationParams) => string;

type TextAnchor = "start" | "end";

type ConsoleLine = {
  idx: number;
  event_id: number;
  module: ModuleName;
  text: string;
};

type SpecialEventType = "warning" | "info" | "danger";

type SpecialEventDisplay = {
  type: SpecialEventType;
  lbl: string;
};

export let translateSim: (
  key: string,
  code: Locale,
  params?: TranslationParams,
) => string;

export let runSimulation: (
  msg: string,
  packetLossProb: number,
  payloadSize: number,
) => Promise<TransportResult<TEvent>>;

export let eventText: (event: TEvent, code: Locale) => string;
export let packetName: (event: TEvent, code: Locale) => string;
export let rcvdPacketLbl: (event: TEvent, code: Locale) => string;
export let isAckPacket: (event: TEvent) => boolean;
export let calcStatRows: (events: TEvent[], code: Locale) => readonly StatRow[];

export let specialEventDisplay:
  | ((event: TEvent, code: Locale) => SpecialEventDisplay | undefined)
  | undefined = undefined;

export let settingsBeforePayload: Snippet<[TranslateFn]> | undefined = undefined;
export let settingsAfterPayload: Snippet<[TranslateFn]> | undefined = undefined;
export let payloadHeaderKey: string | undefined = undefined;
export let payloadHeaderSize: number | undefined = undefined;

let curLocale: Locale;
let isRtl: boolean;
let t: TranslateFn;

let msg: string =
  "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
let packetLossProb: number = 0.3;
let payloadSize: number = DEFAULT_PAYLOAD_SIZE;
let res: TransportResult<TEvent> | null = null;
let curIdx: number = -1;
let isPlaying: boolean = false;
let speed: number = 1;
let activeModule: ModuleTab = "all";
let err: string = "";
const playbackTimer: PlaybackTimer = new PlaybackTimer();
let seqScrollEl: HTMLDivElement | undefined;
let consoleEl: HTMLDivElement | undefined;
let seqWidth: number = 800;
let autoFollowIdx: number = -2;

const moduleTabs: readonly ModuleTab[] = ["all", "sender", "channel", "receiver"];
const lanes: readonly Endpoint[] = ["sender", "channel", "receiver"];

let diagramWidth: number;
let laneX: Record<Endpoint, number>;
let laneCardWidth: number;
let packetLblWidth: number;

let playedEvents: TEvent[];
let curEvent: TEvent | null | undefined;
let consoleLines: ConsoleLine[];
let statRows: readonly StatRow[];
let consoleTabs: readonly { val: ModuleTab; lbl: string }[];
let rcvdMsg: string;
let hasEvents: boolean;
let canGoBack: boolean;
let canGoForward: boolean;

$: curLocale = $locale;

$: isRtl = isLocaleRTL(curLocale);

$: t = (key: string, params: TranslationParams = {}): string =>
  translateSim(key, curLocale, params);

$: diagramWidth = Math.max(300, seqWidth || 800);

$: laneX = {
  sender: Math.max(52, diagramWidth * 0.13),
  channel: diagramWidth * 0.5,
  receiver: Math.min(diagramWidth - 52, diagramWidth * 0.87),
};

$: laneCardWidth = Math.min(122, Math.max(82, diagramWidth * 0.26));

$: packetLblWidth = Math.min(150, Math.max(92, diagramWidth * 0.3));

$: playedEvents = res === null ? [] : res.events.slice(0, curIdx + 1);

$: curEvent = res !== null && curIdx >= 0 ? res.events[curIdx] : null;

$: consoleLines = playedEvents
  .map(
    (event: TEvent, idx: number): ConsoleLine => ({
      idx,
      event_id: event.id,
      module: event.module,
      text: eventText(event, curLocale),
    }),
  )
  .filter(
    (line: ConsoleLine): boolean =>
      activeModule === "all" || line.module === activeModule,
  );

$: statRows = calcStatRows(playedEvents, curLocale);
$: consoleTabs = moduleTabs.map((tab: ModuleTab): { val: ModuleTab; lbl: string; } => ({ val: tab, lbl: tab === "all" ? t("all"): moduleLbl(tab, curLocale) }));

$: rcvdMsg = playedEvents
  .filter(
    (event: TEvent): boolean =>
      event.module === "receiver" &&
      event._type === "accepted" &&
      event.data_chunk !== undefined,
  )
  .map((event: TEvent): string => event.data_chunk ?? "")
  .join("");

$: hasEvents = res !== null && res.events.length > 0;

$: canGoBack = hasEvents && curIdx > -1;

$: canGoForward = hasEvents && res !== null && curIdx < res.events.length - 1;

$: if (curIdx !== autoFollowIdx) {
  autoFollowIdx = curIdx;

  if (curIdx >= 0) {
    void followLatest();
  }
}

async function run(): Promise<void> {
  stop();

  err = "";
  res = null;
  curIdx = -1;
  autoFollowIdx = -2;

  packetLossProb = Math.min(0.95, Math.max(0, Number(packetLossProb)));
  payloadSize = Number(payloadSize);

  try {
    res = await runSimulation(msg, packetLossProb, payloadSize);

    curIdx = -1;
    isPlaying = true;
    scheduleNxt();
  } catch (e: unknown) {
    err = e instanceof Error ? e.message : t("errorSimulation");

    res = null;
    curIdx = -1;
  }
}

function scheduleNxt(): void {
  if (!isPlaying || res === null) {
    return;
  }

  const delay: number = Math.max(60, 700 / speed);

  playbackTimer.schedule(delay, (): void => {
    if (res === null) {
      return;
    }

    if (curIdx >= res.events.length - 1) {
      isPlaying = false;
      return;
    }

    curIdx += 1;
    scheduleNxt();
  });
}

function togglePlay(): void {
  if (res === null || res.events.length === 0) {
    return;
  }

  if (curIdx >= res.events.length - 1) {
    curIdx = -1;
  }

  isPlaying = !isPlaying;
  if (isPlaying) {
    scheduleNxt();
  } else {
    playbackTimer.clear();
  }
}

function step(delta: number): void {
  if (res === null) {
    return;
  }

  const nxtIdx: number = Math.max(-1, Math.min(res.events.length - 1, curIdx + delta));
  if (nxtIdx === curIdx) {
    return;
  }

  isPlaying = false;
  playbackTimer.clear();
  curIdx = nxtIdx;
}

function jump(idx: number): void {
  if (res === null) {
    return;
  }

  const nxtIdx: number = Math.max(-1, Math.min(res.events.length - 1, idx));
  if (nxtIdx === curIdx) {
    return;
  }

  isPlaying = false;
  playbackTimer.clear();
  curIdx = nxtIdx;
}

function stop(): void {
  isPlaying = false;
  playbackTimer.clear();
}

function setSpeed(nxt: number): void {
  speed = nxt;

  if (isPlaying) {
    scheduleNxt();
  }
}

async function followLatest(): Promise<void> {
  await tick();

  if (seqScrollEl !== undefined) {
    seqScrollEl.scrollTop = seqScrollEl.scrollHeight;
  }

  if (consoleEl !== undefined) {
    consoleEl.scrollTop = consoleEl.scrollHeight;
  }
}

async function selectModule(tab: ModuleTab): Promise<void> {
  activeModule = tab;

  await tick();

  if (consoleEl !== undefined) {
    consoleEl.scrollTop = consoleEl.scrollHeight;
  }
}

function sideTextAnchor(side: "left" | "right", rtl: boolean): TextAnchor {
  if (side === "left") {
    return rtl ? "start" : "end";
  }

  return rtl ? "end" : "start";
}

function moduleLbl(module: ModuleName, code: Locale = curLocale): string {
  return translateSim(module, code);
}

function packetLbl(event: TEvent, code: Locale = curLocale): string {
  return packetName(event, code) || event._type.replaceAll("-", " ");
}

function getSpecialEventDisplay(event: TEvent): SpecialEventDisplay | undefined {
  return specialEventDisplay?.(event, curLocale);
}

function movePlayback(dir: PlaybackDir): void {
  switch (dir) {
    case "start": jump(-1); break;
    case "back": step(-1); break;
    case "forward": step(1); break;
    case "end": if (res !== null) jump(res.events.length - 1); break;
  }
}

onDestroy((): void => playbackTimer.clear());
</script>

<section
  class="grid h-full min-h-0 grid-cols-[minmax(238px,.78fr)_minmax(440px,1.62fr)_minmax(300px,1fr)] grid-rows-[auto_minmax(0,1fr)] items-stretch gap-3 max-[1180px]:h-auto max-[1180px]:grid-cols-[minmax(250px,.72fr)_minmax(0,1.55fr)] max-[1180px]:grid-rows-[auto_minmax(280px,1fr)_minmax(300px,.72fr)] max-[700px]:grid-cols-[minmax(0,1fr)] max-[700px]:grid-rows-[auto]"
>
  <aside
    class={`${SIM_PANEL} col-start-1 row-start-1 min-h-0 self-start overflow-visible px-3.5 py-[13px]`}
  >
    <h2 class={`${SIM_HEADING} mb-[9px]`}>{t('settings')}</h2>

    <label class={SIM_SETTINGS_LBL}>
      <span>{t('msg')}</span>
      <textarea
        class="h-[100px] min-h-[78px] max-h-[150px] w-full resize-y rounded-[9px] border border-[var(--field-border)] bg-[var(--field-bg)] px-[9px] py-[7px] leading-[1.35] text-[var(--page-text)] outline-none focus:border-[var(--accent-border)] focus:shadow-[0_0_0_3px_var(--focus-ring)]"
        bind:value={msg}
        rows="6"
        maxlength="12000"
      ></textarea>
    </label>

    <label class={SIM_SETTINGS_LBL}>
      <span class="flex items-baseline justify-between gap-2">
        {t('packetLossProb')} <strong>P = {packetLossProb.toFixed(2)}</strong>
      </span>
      <input
        class="w-full accent-[var(--primary)] focus:border-[var(--accent-border)] focus:shadow-[0_0_0_3px_var(--focus-ring)]"
        bind:value={packetLossProb}
        type="range"
        min="0"
        max="0.99"
        step="0.01"
      />
    </label>

    {#if settingsBeforePayload}
      {@render settingsBeforePayload(t)}
    {/if}

    <PayloadSizeSelect
      lbl={t("maxPayload")}
      headerLbl={payloadHeaderKey === undefined ? undefined : t(payloadHeaderKey)}
      headerSize={payloadHeaderSize}
      bind:val={payloadSize}
    />

    {#if settingsAfterPayload}
      {@render settingsAfterPayload(t)}
    {/if}

    <button
      class={`${SIM_PRIMARY_BTN} mt-[11px] w-full`}
      onclick={run}
    >{t('runSim')}</button>
    {#if err}<p class="mt-[7px] mb-0 text-[10px] text-[var(--danger)]">{err}</p>{/if}
  </aside>

  <section
    class={`${SIM_PANEL}
      @container
      col-start-2 row-start-1 row-span-2
      flex min-h-0 flex-col overflow-hidden

      max-[1180px]:min-h-[620px]
      max-[700px]:col-start-1
      max-[700px]:row-start-2
      max-[700px]:row-span-1
      max-[700px]:min-h-0
    `}
  >
    <div class={SIM_DIAGRAM_HEADER}>
      <h2 class={`${SIM_HEADING} min-w-0 overflow-hidden text-ellipsis whitespace-nowrap`}>{t('seqDiagram')}</h2>
      <PlaybackControls {isPlaying} {speed} onMove={movePlayback} onToggle={togglePlay} onSpeed={setSpeed}
        canBack={canGoBack} canForward={canGoForward} canEnd={canGoForward} canPlay={hasEvents}
        playLbl={curIdx >= (res?.events.length ?? 1) - 1 && hasEvents ? t('replay'): t('play')}
        pauseLbl={t('pause')} backLbl={t('stepBack')} forwardLbl={t('stepForward')}
        startTitle={t('start')} endTitle={t('end')}
        controlsLbl={t('sequenceControls')} speedLbl={t('playbackSpeed')}/>
      <div aria-hidden="true" class="@max-[880px]:hidden"></div>
    </div>
    <div
      class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto bg-[var(--diagram-bg)] [scroll-behavior:smooth] max-[700px]:min-h-[330px] max-[700px]:max-h-[540px]"
      bind:this={seqScrollEl}
      bind:clientWidth={seqWidth}
    >
      <svg
        class="seq block h-auto w-full min-w-0"
        viewBox={`0 0 ${diagramWidth} ${Math.max(410, 150 + playedEvents.length * 42)}`}
        role="img"
        aria-label={t("sequenceAria")}
        direction={isRtl ? "rtl" : "ltr"}
        lang={curLocale}
      >
        <defs>
          <marker id="arrow-data" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M 0 0 L 10 5 L 0 10 z" class="data-arrow-head" />
          </marker>
          <marker id="arrow-ack" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M 0 0 L 10 5 L 0 10 z" class="ack-arrow-head" />
          </marker>
        </defs>

        {#each lanes as lane}
          <g>
            <rect
              x={laneX[lane] - laneCardWidth / 2}
              y="26"
              width={laneCardWidth}
              height="42"
              rx="9"
              class={`lane-card ${lane}`}
            />
            <text x={laneX[lane]} y="52" text-anchor="middle" class="lane-title">{t(lane)}</text>
            <line x1={laneX[lane]} x2={laneX[lane]} y1="68" y2={Math.max(390, 132 + playedEvents.length * 42)} class="lifeline" />
          </g>
        {/each}

        {#each playedEvents as event, idx}
          {@const y = 106 + idx * 42}
          {@const fromX = event.from ? laneX[event.from] : laneX.channel}
          {@const toX = event.to ? laneX[event.to] : laneX.channel}
          {@const isAck = isAckPacket(event)}
          {@const specialEvent = getSpecialEventDisplay(event)}
          <g
            class:current={idx === curIdx}
            class:data-transmission={!isAck}
            class:ack-transmission={isAck}
            class="event-row"
            role="button"
            tabindex="0"
            onclick={(): void => jump(idx)}
            onkeydown={(e: KeyboardEvent): void => {
              if (e.key === "Enter" || e.key === " ") {
                jump(idx);
              }
            }}
          >
            {#if event._type === 'packet-loss'}
              <circle cx={laneX.channel} cy={y} r="11" class="loss" />
              <text x={laneX.channel} y={y + 5} text-anchor="middle" class="loss-x">×</text>
              <text
                x={laneX.channel + 18}
                y={y + 5}
                text-anchor={sideTextAnchor("right", isRtl)}
                class="event-label"
              >{t("packetLost")}</text>
            {:else if specialEvent?.type === "warning"}
              <circle cx={laneX.channel} cy={y} r="11" class="err-dot" />
              <text x={laneX.channel} y={y + 5} text-anchor="middle" class="loss-x">+</text>
              <text
                x={laneX.channel + 18}
                y={y + 5}
                text-anchor={sideTextAnchor("right", isRtl)}
                class="event-label"
              >{specialEvent.lbl}</text>
            {:else if event._type === 'timeout'}
              {@const timeoutX = event.module === "receiver" ? laneX.receiver : laneX.sender}
              <path d={`M ${timeoutX - 14} ${y - 10} q -16 10 0 20`} class="timeout-loop" />
              <text
                x={event.module === "receiver" ? timeoutX - 22 : timeoutX + 18}
                y={y + 5}
                text-anchor={sideTextAnchor(event.module === "receiver" ? "left" : "right", isRtl)}
                class="event-label"
              >{t("timeoutRetransmit")}</text>
            {:else if event._type === 'packet-received'}
              <circle cx={toX} cy={y} r="9" class="packet-received-dot" />
              <text
                x={toX + (event.to === 'receiver' ? -16 : 16)}
                y={y + 5}
                text-anchor={sideTextAnchor(event.to === "receiver" ? "left" : "right", isRtl)}
                class="event-label received-label"
              >{rcvdPacketLbl(event, curLocale)}</text>
            {:else if event._type === 'receiver-terminated'}
              <circle cx={laneX.receiver} cy={y} r="10" class="terminated-dot" />
              <text
                x={laneX.receiver - 16}
                y={y + 5}
                text-anchor={sideTextAnchor("left", isRtl)}
                class="event-label terminated-label"
              >{t('receiverTerminated')}</text>
            {:else if event._type === 'receiver-unavailable'}
              <circle cx={laneX.receiver} cy={y} r="11" class="unavailable-dot" />
              <text x={laneX.receiver} y={y + 5} text-anchor="middle" class="loss-x">×</text>
              <text
                x={laneX.receiver - 18}
                y={y + 5}
                text-anchor={sideTextAnchor("left", isRtl)}
                class="event-label"
              >{t('receiverUnavailable')}</text>
            {:else if event._type === 'deadlock'}
              <rect x={diagramWidth * .25} y={y - 16} width={diagramWidth * .5} height="32" rx="8" class="deadlock-banner" />
              <text x={diagramWidth * .5} y={y + 5} text-anchor="middle" class="deadlock-label">{t('deadlock')}</text>
            {:else if specialEvent?.type === "danger"}
              <rect x={diagramWidth * .27} y={y - 14} width={diagramWidth * .46} height="28" rx="8" class="deadlock-banner" />
              <text x={diagramWidth * .5} y={y + 4} text-anchor="middle" class="deadlock-label">{specialEvent.lbl}</text>
            {:else if specialEvent?.type === "info"}
              <rect x={diagramWidth * .27} y={y - 14} width={diagramWidth * .46} height="28" rx="8" class="info-banner" />
              <text x={diagramWidth * .5} y={y + 4} text-anchor="middle" class="info-label">{specialEvent.lbl}</text>
            {:else if event._type === 'complete'}
              <rect x={diagramWidth * .27} y={y - 14} width={diagramWidth * .46} height="28" rx="8" class="complete-banner" />
              <text x={diagramWidth * .5} y={y + 4} text-anchor="middle" class="complete-label">{t('eventComplete')}</text>
            {:else if event.from && event.to}
              <line x1={fromX} x2={toX} y1={y} y2={y} class="packet-line" marker-end={isAck ? 'url(#arrow-ack)' : 'url(#arrow-data)'} />
              <rect x={(fromX + toX) / 2 - packetLblWidth / 2} y={y - 13} width={packetLblWidth} height="25" rx="7" class="packet-label-bg" />
              <text x={(fromX + toX) / 2} y={y + 4} text-anchor="middle" class="packet-label">{packetLbl(event, curLocale)}</text>
              {#if idx === curIdx}
                <circle r="6" cy={y} class="moving-packet">
                  <animate attributeName="cx" from={fromX} to={toX} dur={`${Math.max(0.15, 0.7 / speed)}s`} fill="freeze" />
                </circle>
              {/if}
            {/if}
          </g>
        {/each}
      </svg>
    </div>

    <div class="flex min-h-10 flex-none items-center gap-2 overflow-hidden border-t border-[var(--divider)] px-2.5 py-[7px] text-[10px] text-[var(--muted-text)]">
      {#if curEvent}
        <span class={`module-pill ${curEvent.module} rounded-full px-1.5 py-[3px] text-[8px] font-extrabold uppercase`}>
          {moduleLbl(curEvent.module, curLocale)}
        </span>
        <strong class="whitespace-nowrap text-[var(--page-text)]">{t('event')} {curIdx + 1} / {res?.events.length}</strong>
        <span class="overflow-hidden text-ellipsis whitespace-nowrap">{eventText(curEvent, curLocale)}</span>
      {:else}
        <span>{t('runToGenerate')}</span>
      {/if}
    </div>
  </section>

  <section
    class={`${SIM_PANEL} col-start-1 row-start-2 flex min-h-0 flex-col overflow-hidden p-[11px] max-[700px]:row-start-3 max-[700px]:overflow-visible`}
  >
    <h2 class={`${SIM_HEADING} mb-[7px]`}>{t('statistics')}</h2>
    <StatRows rows={statRows}/>
    <div class="mt-[7px] min-h-0">
      <span class="text-[8px] tracking-[.1em] text-[var(--muted-text)] uppercase">{t('receivedMsg')}</span>
      <p
        class="mt-1 mb-0 max-h-24 overflow-auto whitespace-pre-wrap rounded-[7px] border border-[var(--divider)] bg-[var(--diagram-bg)] p-1.5 text-[9px] leading-[1.35] text-[var(--page-text)]"
      >{rcvdMsg || t('noDataYet')}</p>
    </div>
  </section>

  <section
    class={`${SIM_PANEL} col-start-3 row-start-1 row-span-2 flex min-h-0 min-w-0 flex-col overflow-hidden p-[11px] max-[1180px]:col-start-1 max-[1180px]:col-span-2 max-[1180px]:row-start-3 max-[1180px]:row-span-1 max-[1180px]:min-h-[300px] max-[700px]:col-span-1 max-[700px]:row-start-4 max-[700px]:min-h-[320px]`}
  >
    <div class="mb-[7px] flex flex-none items-center justify-between gap-2.5 max-[480px]:flex-col max-[480px]:items-start">
      <h2 class={SIM_HEADING}>{t('console')}</h2>
      <ConsoleTabs tabs={consoleTabs} selected={activeModule} lbl={t("console")} disabled={!hasEvents}
        onSelect={(tab) => { void selectModule(tab); }}/>

    </div>

    <div
      class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto rounded-lg border border-[var(--divider)] bg-[var(--console-bg)] p-1 [scroll-behavior:smooth] max-[1180px]:max-h-[430px] max-[700px]:max-h-[420px]"
      bind:this={consoleEl}
      aria-live="polite"
    >
      {#if consoleLines.length}
        {#each consoleLines as line}
          <button
            class={[
              'grid w-full grid-cols-[40px_76px_minmax(0,1fr)] gap-1.5 rounded px-[5px] py-[3px] text-start text-[9px] leading-[1.35] text-[var(--control-text)] max-[480px]:grid-cols-[36px_62px_minmax(0,1fr)]',
              line.idx === curIdx
                ? 'bg-[var(--selected-bg)] text-[var(--selected-text)]'
                : 'bg-transparent hover:bg-[var(--selected-bg)] hover:text-[var(--selected-text)]'
            ]}
            onclick={(): void => jump(line.idx)}
          >
            <span class="text-[var(--muted-text)]">[{String(line.idx + 1).padStart(3, '0')}]</span>
            <span class={`module ${line.module} font-extrabold uppercase`}>{moduleLbl(line.module, curLocale)}</span>
            <span>{line.text}</span>
          </button>
        {/each}
      {:else}
        <div class="p-[18px] text-center text-[10px] text-[var(--muted-text)]">{t('noOutputYet')}</div>
      {/if}
    </div>
  </section>
</section>

<style>
  .lane-card {
    fill: var(--lane-bg);
    stroke: var(--control-border);
    stroke-width: 1;
  }

  .lane-card.sender { stroke: var(--sender); }
  .lane-card.channel { stroke: var(--channel); }
  .lane-card.receiver { stroke: var(--receiver); }

  .lane-title {
    fill: var(--lane-text);
    font-size: 12px;
    font-weight: 750;
  }

  .lifeline {
    stroke: var(--lifeline);
    stroke-width: 1;
    stroke-dasharray: 6 7;
  }

  .packet-line { stroke-width: 1.8; }
  .data-arrow-head { fill: var(--sender); }
  .ack-arrow-head { fill: var(--receiver); }

  .packet-label-bg {
    fill: var(--label-bg);
    stroke-width: 1;
  }

  .packet-label,
  .event-label {
    fill: var(--label-text);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 10px;
  }

  .event-row {
    cursor: pointer;
    opacity: .92;
  }

  .event-row.current { opacity: 1; }

  .event-row.data-transmission {
    --packet-color: var(--sender);
    --packet-soft: var(--sender-soft);
    --packet-fill: var(--sender-fill);
  }

  .event-row.ack-transmission {
    --packet-color: var(--receiver);
    --packet-soft: var(--receiver-soft);
    --packet-fill: var(--receiver-fill);
  }

  .event-row .packet-line,
  .event-row .packet-label-bg {
    stroke: var(--packet-color);
  }

  .event-row .packet-label,
  .event-row .received-label {
    fill: var(--packet-soft);
  }

  .event-row .moving-packet { fill: var(--packet-color); }

  .event-row .packet-received-dot {
    fill: var(--packet-fill);
    stroke: var(--packet-color);
  }

  .event-row.current .packet-line { stroke-width: 2.8; }
  .packet-received-dot { stroke-width: 2; }

  .loss {
    fill: var(--danger-fill);
    stroke: var(--danger);
  }

  .loss-x {
    fill: var(--danger-text);
    font-size: 16px;
    font-weight: 900;
  }

  .err-dot {
    fill: var(--warning-fill);
    stroke: var(--channel);
  }

  .terminated-dot {
    fill: var(--terminated-fill);
    stroke: var(--terminated);
    stroke-width: 2;
  }

  .unavailable-dot {
    fill: var(--danger-fill);
    stroke: var(--danger);
    stroke-width: 2;
  }

  .terminated-label { fill: var(--terminated-text); }

  .deadlock-banner {
    fill: var(--danger-fill);
    stroke: var(--danger);
    stroke-width: 1.5;
  }

  .deadlock-label {
    fill: var(--danger-text);
    font-size: 10px;
    font-weight: 900;
    letter-spacing: .04em;
  }

  .info-banner {
    fill: var(--sender-fill);
    stroke: var(--sender);
    stroke-width: 1.5;
  }

  .info-label {
    fill: var(--sender-soft);
    font-size: 10px;
    font-weight: 900;
    letter-spacing: .04em;
  }

  .complete-banner {
    fill: var(--receiver-fill);
    stroke: var(--receiver);
    stroke-width: 1.5;
  }

  .complete-label {
    fill: var(--receiver-soft);
    font-size: 10px;
    font-weight: 900;
    letter-spacing: .035em;
  }

  .timeout-loop {
    fill: none;
    stroke: var(--timeout);
    stroke-width: 2;
  }

  .module-pill.sender {
    background: var(--sender-fill);
    color: var(--sender-soft);
  }

  .module-pill.channel {
    background: var(--channel-fill);
    color: var(--channel-text);
  }

  .module-pill.receiver {
    background: var(--receiver-fill);
    color: var(--receiver-soft);
  }

  .module-pill.simulation {
    background: var(--danger-fill);
    color: var(--danger-text);
  }

  .module.sender { color: var(--sender-soft); }
  .module.channel { color: var(--channel-text); }
  .module.receiver { color: var(--receiver-soft); }
  .module.simulation { color: var(--danger-text); }

  :global(html[data-theme='light']) .seq
    :is(
      .lane-title,
      .packet-label,
      .event-label,
      .received-label,
      .info-label,
      .complete-label
    ) {
    fill: #000;
  }

  @media (max-width: 480px) {
    .packet-label,
    .event-label {
      font-size: 9px;
    }

    .deadlock-label,
    .info-label {
      font-size: 8px;
    }
  }
</style>
