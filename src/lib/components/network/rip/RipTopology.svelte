<script lang="ts">
  import { locale, translate, type TranslationParams } from "#lib/i18n/index.ts";
  import { onMount } from "svelte";
  import { SIM_PANEL, SIM_HEADING, SIM_DIAGRAM_HEADER, SIM_MUTED } from "#lib/components/simulation/styles.ts";
  import RouterSelect from "./RipRouterSelect.svelte";
  import PlaybackControls from "#lib/components/simulation/PlaybackControls.svelte";
  import { nodePositions, isHighlighted, formatDur } from "#lib/sims/network/rip/layout.ts";
  import { routerCol, routerBadgeStyle, selectedRouteMetric } from "#lib/sims/network/rip/router.ts";
  import type { Pos } from "#lib/sims/network/rip/layout.ts";
  import type { ConsoleEntry } from "#lib/sims/network/rip/console.ts";
  import type { RipNode, RipEdge, RipSnapshot, RipConfig } from "#lib/sims/network/rip/types.ts";
  import type { PlaybackDir } from "#lib/sims/shared/playback.ts";

  $: t = (key: string, params: TranslationParams = {}): string => translate(`rip.${key}`, $locale, params);
  export let snapshot: RipSnapshot | null;
  export let cfgTimeout: number;
  export let activeTopology: RipConfig["topology"];
  export let selectedRouter: number;
  export let selectedDest: number;
  export let availableDests: number[];
  export let highlightPath: number[];
  export let focusEvent: ConsoleEntry | undefined;
  export let isPlaying: boolean;
  export let speed: number;
  export let isLoading: boolean;
  export let onMove: (dir: PlaybackDir) => void;
  export let onToggle: () => void;
  export let onSpeed: (speed: number) => void;
  export let onTerminate: (router: number) => void;
  export let onAdjustLink: (a: number, b: number, cost: number, delta: -1 | 1) => void;

  let sectionEl: HTMLElement;
  let graphEl: HTMLDivElement;
  let graphWidth: number = 960;
  let graphHeight: number = 585;
  let positions: Pos[] = [];
  let posMap: Map<number, Pos> = new Map();
  let layoutSignature: string = "";
  let nodeRadius: number = 31;
  let curRouter: RipNode | undefined;
  let routeMetric: string = "—";

  function radiusFor(count: number, w: number, h: number): number {
    const usable: number = Math.max(1, (w - 110) * (h - 110));
    return Math.min(count >= 14 ? 25 : 31,
      Math.max(16, Math.floor(0.30 * Math.sqrt(usable / Math.max(1, count)))));
  }

  $: curRouter = snapshot?.nodes.find((node: RipNode): boolean => node.id === selectedRouter);
  $: routeMetric = selectedRouteMetric(curRouter, selectedDest);
  $: nodeRadius = radiusFor(snapshot?.nodes.length ?? 7, graphWidth, graphHeight);
  $: {
    const topology: string = snapshot?.edges.map((edge: RipEdge): string => `${edge.a}-${edge.b}`).join(",") ?? "";
    const key: string = `${activeTopology}:${snapshot?.nodes.length ?? 0}:${topology}:${graphWidth}:${graphHeight}`;
    if (key !== layoutSignature) {
      layoutSignature = key;
      positions = snapshot ? nodePositions(snapshot.nodes, snapshot.edges, graphWidth, graphHeight, nodeRadius, activeTopology): [];
      posMap = new Map<number, Pos>(positions.map((pos: Pos): readonly [number, Pos] => [pos.id, pos] as const));
    }
  }

  function updateGraphSize(): void {
    if (!graphEl) return;
    const { width, height }: DOMRect = graphEl.getBoundingClientRect();
    if (width < 1 || height < 1) return;
    graphWidth = Math.round(width);
    graphHeight = Math.round(height);
  }

  export function focus(): void {
    sectionEl?.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  function stateCol(node: RipNode): string {
    return !node.is_running ? "var(--danger)" : node.is_quiescent ? "var(--timeout)" : "var(--receiver)";
  }

  onMount((): () => void => {
    const obs: ResizeObserver = new ResizeObserver(updateGraphSize);
    obs.observe(graphEl);
    updateGraphSize();
    return (): void => obs.disconnect();
  });
</script>

  <section bind:this={sectionEl} class={`${SIM_PANEL} @container flex h-full min-h-0 min-w-0 flex-col overflow-hidden max-[1240px]:min-h-[640px] max-[760px]:order-2 max-[760px]:min-h-[500px]`}>
    <div class={SIM_DIAGRAM_HEADER}>
      <h2 class={`${SIM_HEADING} min-w-0`}>{t("networkTopology")}</h2>
      <PlaybackControls {isPlaying} {speed}
        canBack={!isLoading && !!snapshot && snapshot.time > snapshot.hist_start + 0.000001}
        canForward={!isLoading && !!snapshot && !snapshot.is_complete}
        canPlay={!isLoading && !!snapshot && (!snapshot.is_complete || isPlaying)}
        canEnd={!isLoading && !!snapshot && !snapshot.is_complete && snapshot.dur !== 0}
        startTitle={snapshot?.dur === 0 ? t("backToEarliest"): t("backToStartTime")}
        endTitle={snapshot?.dur === 0 ? t("continuousNoEnd"): t("forwardToTime", {time: formatDur(snapshot?.dur ?? cfgTimeout)})}
        backLbl={t("stepBack")} forwardLbl={t("stepForward")} playLbl={t("play")} pauseLbl={t("pause")}
        controlsLbl={t("playbackControls")} speedLbl={t("playbackSpeed")} {onMove} {onToggle} {onSpeed}/>
      <div aria-hidden="true" class="@max-[880px]:hidden"></div>
    </div>
    <div class="flex items-center gap-2 border-b border-[var(--divider)] px-3 py-2 text-[11px]">
      <strong class="font-mono tabular-nums">{formatDur(snapshot?.time ?? 0)} / {(snapshot?.dur ?? cfgTimeout) === 0 ? '∞' : formatDur(snapshot?.dur ?? cfgTimeout)}</strong>
      {#if (snapshot?.dur ?? cfgTimeout) > 0}
        <div class="h-1.5 flex-1 overflow-hidden rounded-full bg-[var(--control-bg)]"><div class="h-full bg-[var(--primary)] transition-all" style={`width:${((snapshot?.time ?? 0) / (snapshot?.dur ?? cfgTimeout)) * 100}%`}></div></div>
      {:else}
        <div class="h-1.5 flex-1 rounded-full bg-[var(--control-bg)]" aria-label={t("continuousNoEndpoint")}></div>
      {/if}
      <span class={snapshot?.stats.is_converged ? 'text-[var(--receiver)]' : 'text-[var(--timeout)]'}>{snapshot?.is_complete ? t("finished"): snapshot?.dur === 0 ? t("continuous"): snapshot?.stats.is_converged ? t("converged"): t("converging")}</span>
    </div>
    <div bind:this={graphEl} class="relative min-h-[340px] flex-1 overflow-hidden bg-[var(--diagram-bg)]">
      {#if snapshot}
        <svg role="img" aria-label={t("networkAria")} viewBox={`0 0 ${graphWidth} ${graphHeight}`} class="absolute inset-0 h-full w-full" preserveAspectRatio="xMidYMid meet">
          <defs>
            <pattern id="rip-grid" width="26" height="26" patternUnits="userSpaceOnUse"><circle cx="1" cy="1" r="1" fill="var(--divider)" opacity=".24" /></pattern>
            <filter id="rip-glow"><feGaussianBlur stdDeviation="4" /></filter>
          </defs>
          <rect width={graphWidth} height={graphHeight} fill="url(#rip-grid)" />
          {#each snapshot.edges as edge (`${edge.a}-${edge.b}`)}
            {@const from = posMap.get(edge.a)}
            {@const to = posMap.get(edge.b)}
            {@const onPath = isHighlighted(edge, highlightPath)}
            {@const down = !snapshot.nodes.find(n => n.id === edge.a)?.is_running || !snapshot.nodes.find(n => n.id === edge.b)?.is_running}
            {#if from && to}
              {@const midX = (from.x + to.x) / 2}
              {@const midY = (from.y + to.y) / 2}
              {#if onPath}<line x1={from.x} y1={from.y} x2={to.x} y2={to.y} stroke="var(--primary)" stroke-width="11" opacity=".1" filter="url(#rip-glow)" />{/if}
              <line x1={from.x} y1={from.y} x2={to.x} y2={to.y} stroke={down ? 'var(--danger)' : onPath ? 'var(--primary)' : 'var(--border)'} stroke-width={onPath ? 4 : 1.8} stroke-dasharray={down ? '7 6' : undefined} opacity={down ? .45 : onPath ? 1 : .7}/>
              <g transform={`translate(${midX},${midY})`}>
                <rect x="-29" y="-16" width="35" height="32" rx="7" fill="var(--surface-solid)" stroke={onPath ? 'var(--primary)' : 'var(--divider)'} stroke-width="1" />
                <text x="-11.5" y="5" fill={onPath ? 'var(--primary)' : 'var(--page-text)'} font-size="13" font-weight="700" text-anchor="middle" pointer-events="none">{edge.cost}</text>
                {#if !snapshot.is_complete}
                  <g role="button" tabindex="0" aria-label={t("increaseLinkCost", {a: edge.a, b: edge.b})} class="cursor-pointer" onclick={(event) => { event.stopPropagation(); onAdjustLink(edge.a, edge.b, edge.cost, 1); }} onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); onAdjustLink(edge.a, edge.b, edge.cost, 1); } }}>
                    <title>{t("increaseCostTitle")}</title>
                    <rect x="7" y="-16" width="20" height="15" rx="4" fill="var(--control-bg)" stroke="var(--control-border)" />
                    <path d="m13 -6 4 -4 4 4" fill="none" stroke="var(--page-text)" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" pointer-events="none" />
                  </g>
                  <g role="button" tabindex="0" aria-label={t("decreaseLinkCost", {a: edge.a, b: edge.b})} class="cursor-pointer" onclick={(event) => { event.stopPropagation(); onAdjustLink(edge.a, edge.b, edge.cost, -1); }} onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); onAdjustLink(edge.a, edge.b, edge.cost, -1); } }}>
                    <title>{t("decreaseCostTitle")}</title>
                    <rect x="7" y="1" width="20" height="15" rx="4" fill="var(--control-bg)" stroke="var(--control-border)" />
                    <path d="m13 6 4 4 4 -4" fill="none" stroke="var(--page-text)" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" pointer-events="none" />
                  </g>
                {/if}
              </g>
            {/if}
          {/each}
          {#each snapshot.nodes as node (node.id)}
            {@const pos = posMap.get(node.id)}
            {#if pos}
              <g transform={`translate(${pos.x},${pos.y})`}>
                {#if node.id === selectedRouter}<circle r={nodeRadius + 12} fill="none" stroke="var(--primary)" stroke-width="2.5" stroke-dasharray="6 3" />{/if}
                <circle r={nodeRadius} fill="var(--surface-solid)" stroke={stateCol(node)} stroke-width="3.5" opacity={node.is_running ? 1 : .55}/>
                <circle r={nodeRadius - 8} fill={routerCol(node.id)?.fill ?? 'var(--sender-fill)'} opacity="1" />
                <text x="0" y="6" fill="#16202c" font-size="19" font-weight="800" text-anchor="middle" pointer-events="none">{node.id}</text>
                {#if node.auto_restart_in != null}
                  <title>{t("routerAutoRestart", {id: node.id, seconds: node.auto_restart_in.toFixed(1)})}</title>
                {/if}
                <circle r={nodeRadius} fill="transparent" class="cursor-pointer" role="button" tabindex="0" aria-label={t("inspectRouter", {id: node.id})} onclick={() => selectedRouter = node.id} onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); selectedRouter = node.id; } }} />
                {#if node.is_running && !snapshot.is_complete}
                  <g role="button" tabindex="0" aria-label={t("terminateRouter", {id: node.id})} class="cursor-pointer" onclick={(event) => { event.stopPropagation(); onTerminate(node.id); }} onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); onTerminate(node.id); } }}>
                    <circle cx="27" cy="-27" r="13" fill="var(--danger-fill)" stroke="var(--danger)" stroke-width="1.8" />
                    <path d="M22 -32 32 -22m0-10-10 10" fill="none" stroke="var(--danger-text)" stroke-width="2.3" stroke-linecap="round" />
                  </g>
                {/if}
              </g>
            {/if}
          {/each}
        </svg>
        {#if focusEvent && ['send', 'receive'].includes(focusEvent._type) && focusEvent.peer}
          <div class="pointer-events-none absolute right-3 bottom-3 rounded-lg border border-[var(--divider)] bg-[var(--surface-solid)] px-3 py-2 text-[10px] shadow-md">
            <span class="font-semibold text-[var(--primary)]">{t("ripResponse")}</span> · R{focusEvent._type === 'send' ? focusEvent.router : focusEvent.peer} → R{focusEvent._type === 'send' ? focusEvent.peer : focusEvent.router}
          </div>
        {/if}
      {:else}
        <div class={`grid h-full place-content-center text-center text-sm ${SIM_MUTED}`}>{t("loadingWasm")}</div>
      {/if}
    </div>
    <div class="flex flex-wrap items-center gap-x-5 gap-y-1 border-t border-[var(--divider)] px-3 py-2 text-[10px]">
      <span><span class="me-1 text-[var(--receiver)]">●</span>{t("active")}</span>
      <span><span class="me-1 text-[var(--timeout)]">●</span>{t("isolated")}</span>
      <span><span class="me-1 text-[var(--danger)]">●</span>{t("terminated")}</span>
      <span><span class="me-1 text-[var(--primary)]">━</span>{t("selectedShortestPath")}</span>
    </div>
    <div class="flex min-w-0 items-center gap-3 border-t border-[var(--divider)] bg-[var(--surface-2)] px-3 py-3 text-xs max-[600px]:flex-wrap">
      <div class="flex shrink-0 items-center gap-1.5 font-semibold">
        <label for="rip-from">{t("from")}</label>
        <RouterSelect id="rip-from" lbl={t("sourceRouter")} val={selectedRouter}
          routerIds={snapshot?.nodes.map(node => node.id) ?? []}
          onSelect={(routerId) => { selectedRouter = routerId; }} />
      </div>
      <div class="flex min-w-0 flex-1 items-center justify-evenly gap-1 overflow-x-auto py-1" aria-label={t("selectedRoute")}>
        {#if highlightPath.length > 0}
          {#each highlightPath as routerId, idx}
            {#if idx > 0}<span class="shrink-0 text-base font-bold text-[var(--primary)]" aria-hidden="true">→</span>{/if}
            <span class="min-w-[31px] shrink-0 rounded-lg border border-black/10 px-2 py-1 text-center font-semibold" style={routerBadgeStyle(routerId)}>R{routerId}</span>
          {/each}
        {:else}
          <span class={`whitespace-nowrap text-[11px] ${SIM_MUTED}`}>{t("noRoute")}</span>
        {/if}
      </div>
      <div class="flex shrink-0 items-center gap-1.5 font-semibold">
        <label for="rip-to">{t("to")}</label>
        <RouterSelect id="rip-to" lbl={t("destinationRouter")} val={selectedDest}
          routerIds={availableDests} disabled={availableDests.length === 0}
          onSelect={(routerId) => { selectedDest = routerId; }} />
      </div>
      <span class="min-w-10 shrink-0 rounded-md bg-[var(--label-bg)] px-2 py-1.5 text-center font-mono font-bold" title={t("curRouteMetric")}>{routeMetric}</span>
    </div>
  </section>
