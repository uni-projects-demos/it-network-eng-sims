<script lang="ts">
  import { onMount } from "svelte";
  import { locale } from "#lib/i18n/index.ts";
  import { flowTranslator, type FlowMsgKey } from "#lib/sims/network/flow/i18n.ts";
  import { SIM_PANEL, SIM_HEADING, SIM_FIELD } from "#lib/components/simulation/styles.ts";
  import { FLOW_WORK_LIMIT, type FlowSnapshot, type FlowRoute, type FlowConfig } from "#lib/sims/network/flow/types.ts";
  import { flowNumber } from "#lib/sims/network/flow/format.ts";
  export let snapshot: FlowSnapshot | null;
  export let cfg: FlowConfig;
  export let isRunning: boolean;
  export let selectedSrc: number;
  export let selectedDest: number;

  let isLbls: boolean = false;
  let graphElement: HTMLDivElement;
  let graphWidth: number = 760;
  let graphHeight: number = 440;
  let isStacked: boolean = false;
  let status: FlowMsgKey;
  type TierPrefix = "S" | "T" | "D";
  type Tier = {prefix: TierPrefix; size: number; col: string; fill: string; lbl: string};
  let tiers: readonly Tier[];
  $: t = flowTranslator($locale);
  $: number = (val: number | null | undefined): string => flowNumber(val, $locale);
  $: active = snapshot?.config ?? cfg;
  $: viewWidth = Math.min(760, Math.max(280, graphWidth));
  $: sideMargin = Math.max(44, viewWidth / 8);
  $: tierWidth = Math.min(viewWidth / 3 - 12, 2 * (sideMargin - 8));
  $: contentHeight = Math.max(440, Math.max(active.srcs, active.trans, active.dests) * 68 + 100);
  $: height = isStacked ? contentHeight : Math.max(contentHeight, viewWidth * graphHeight / Math.max(1, graphWidth));
  $: route = snapshot?.routes.find((r: FlowRoute) => r.src === selectedSrc && r.dest === selectedDest);
  $: status = snapshot?.status === "feasible" && !isRunning ? snapshot.work >= FLOW_WORK_LIMIT ? "limitReached" : "pausedFeasible" : snapshot?.status ?? "ready";
  $: tiers = [
    {prefix: "S", size:active.srcs,col: "var(--sender)", fill: "var(--sender-fill)", lbl: t("srcs")},
    {prefix: "T", size:active.trans,col: "var(--channel)", fill: "var(--channel-fill)", lbl: t("trans")},
    {prefix: "D", size:active.dests,col: "var(--receiver)", fill: "var(--receiver-fill)", lbl: t("dests")},
  ];
  const y = (idx: number, count: number, h: number): number => 72 + (count === 1 ? (h - 120) / 2 : (idx-1) * (h - 140) / (count - 1));
  $: x = (id: string): number => id.startsWith("S") ? sideMargin : id.startsWith("T") ? viewWidth / 2 : viewWidth - sideMargin;
  const count = (id: string, config: FlowConfig): number => id.startsWith("S") ? config.srcs : id.startsWith("T") ? config.trans : config.dests;
  function highlighted(from: string, to: string, path: FlowRoute | undefined): boolean {
    if (!path) return false;
    if (from.startsWith("S")) return from === `S${path.src}` && path.trans.includes(Number(to.slice(1)));
    return to === `D${path.dest}` && path.trans.includes(Number(from.slice(1)));
  }
  function selectNode(prefix: TierPrefix, id: number): void {
    if (prefix === "S") selectedSrc = id;
    else if (prefix === "D") selectedDest = id;
  }
  onMount(() => {
    const layout: MediaQueryList = window.matchMedia("(max-width:800px)");
    const resize: () => void = (): void => {
      graphWidth = graphElement.clientWidth;
      graphHeight = graphElement.clientHeight;
      isStacked = layout.matches;
    };
    const obs: ResizeObserver = new ResizeObserver(resize);
    obs.observe(graphElement);
    layout.addEventListener("change", resize);
    resize();
    return () => {
      obs.disconnect();
      layout.removeEventListener("change", resize);
    };
  });
</script>

<section class={`${SIM_PANEL} flow-topology-panel @container flex h-full min-h-0 min-w-0 flex-col overflow-hidden max-[1240px]:min-h-[640px]`}>
  <div class="flex flex-wrap items-center justify-between gap-2 border-b border-[var(--divider)] px-3 py-2.5">
    <h2 class={SIM_HEADING}>{t("networkTopology")}</h2>
    <span role="status" class={`text-[10px] ${snapshot?.status === "optimal" ? "font-bold text-[var(--receiver)]" : "text-[var(--timeout)]"}`}>{t(status)}</span>
  </div>
  <div bind:this={graphElement} class="flow-graph relative min-h-[320px] flex-1 overflow-x-hidden overflow-y-auto bg-[var(--diagram-bg)]">
    <svg role="group" aria-label={t("graphAria")} viewBox={`0 0 ${viewWidth} ${height}`} class="absolute inset-0 h-full w-full min-w-0" style={`--flow-svg-height: ${height * graphWidth / viewWidth}px`}>
      <defs>
        <pattern id="flow-grid" width="26" height="26" patternUnits="userSpaceOnUse"><circle cx="1" cy="1" r="1" fill="var(--divider)" opacity=".24" /></pattern>
        <marker id="flow-arrow" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto-start-reverse"><path d="M0 0 6 3 0 6" fill="var(--primary)" /></marker>
      </defs>
      <rect width={viewWidth} {height} fill="url(#flow-grid)" />
      {#each snapshot?.links ?? [] as link (`${link.from}-${link.to}`)}
        {@const selected = highlighted(link.from, link.to, route)}
        <line x1={x(link.from)+24} y1={y(Number(link.from.slice(1)),count(link.from,active),height)} x2={x(link.to)-26} y2={y(Number(link.to.slice(1)),count(link.to,active),height)}
          stroke={selected ? "var(--primary)" : "var(--lifeline)"} stroke-width={selected ? 2.7 : 1}
          opacity={selected ? 1 : link.cap > 0 ? .17 : .05} marker-end={selected ? "url(#flow-arrow)" : undefined}
          class:flow-active={selected && isRunning}>
          <title>{link.from} → {link.to}: {number(link.cap)}</title>
        </line>
        {#if isLbls && link.cap > 0 && (selected || active.srcs * active.trans + active.trans * active.dests < 35)}
          <text x={(x(link.from)+x(link.to))/2} y={(y(Number(link.from.slice(1)),count(link.from,active),height)+y(Number(link.to.slice(1)),count(link.to,active),height))/2-5}
            text-anchor="middle" fill="var(--label-text)" font-size="10" paint-order="stroke" stroke="var(--diagram-bg)" stroke-width="4">{number(link.cap)}</text>
        {/if}
      {/each}
      {#each tiers as tier}
        <foreignObject x = {x(tier.prefix) - tierWidth / 2} y = "8" width = {tierWidth} height = "40">
          <div class="text-center text-[10px] leading-3 font-semibold text-[var(--muted-text)] [overflow-wrap:anywhere]">{tier.lbl}</div>
        </foreignObject>
        {#each Array.from({length:tier.size},(_,i) => i+1) as id}
          {@const node = `${tier.prefix}${id}`}
          {@const onPath = tier.prefix === "S" ? id === selectedSrc : tier.prefix === "D" ? id === selectedDest : route?.trans.includes(id)}
          <g>
            <circle cx={x(node)} cy={y(id,tier.size,height)} r="24" fill={tier.fill} stroke={tier.col} stroke-width={onPath ? 2.5 : 1.1} opacity={onPath ? 1 : .65} />
            <text x={x(node)} y={y(id,tier.size,height)+4} text-anchor="middle" fill="var(--page-text)" font-size="12" font-weight="700">{node}</text>
            {#if tier.prefix === "T"}<text x={x(node)} y={y(id,tier.size,height)+39} text-anchor="middle" fill="var(--muted-text)" font-size="10">{number(snapshot?.loads[id-1])}</text>{/if}
            {#if tier.prefix !== "T"}
              <circle cx={x(node)} cy={y(id,tier.size,height)} r="24" fill="transparent"
                class="cursor-pointer outline-none focus:stroke-[var(--primary)] focus:stroke-[4px]" role="button" tabindex="0"
                aria-label={t(tier.prefix === "S" ? "selectSource" : "selectDestination", {id})} aria-pressed={onPath}
                onclick={() => selectNode(tier.prefix, id)}
                onkeydown={(event) => { if (event.key === "Enter" || event.key === " ") { event.preventDefault(); selectNode(tier.prefix, id); } }} />
            {/if}
          </g>
        {/each}
      {/each}
    </svg>
  </div>
  <div class="flex min-w-0 shrink-0 flex-wrap items-center gap-x-3 gap-y-2 border-t border-[var(--divider)] bg-[var(--surface-2)] px-3 py-3 text-[10px]">
    <div class="flex min-w-0 flex-1 basis-[280px] items-center gap-3">
      <div class="flex shrink-0 items-center gap-1.5 font-semibold @max-[540px]:max-w-[25%] @max-[540px]:flex-col @max-[540px]:items-start @max-[540px]:[overflow-wrap:anywhere]">
        <label for="flow-source-view">{t("source")}</label>
        <select id="flow-source-view" class={SIM_FIELD} bind:value={selectedSrc}>{#each Array.from({length:active.srcs},(_,i)=>i+1) as id}<option value={id}>S{id}</option>{/each}</select>
      </div>
      <div class="flex min-w-0 flex-1 flex-wrap items-center justify-evenly gap-1 py-1 text-[var(--primary)]" aria-label={t("threePathsShort")}>
        {#if route}
          {#each route.trans as transit, idx}
            {#if idx > 0}<span class="shrink-0" aria-hidden="true">·</span>{/if}
            <strong class="shrink-0">T{transit}</strong>
          {/each}
        {:else}<span class="text-[var(--muted-text)]">{t("noRoutes")}</span>{/if}
      </div>
      <div class="flex shrink-0 items-center gap-1.5 font-semibold @max-[540px]:max-w-[25%] @max-[540px]:flex-col @max-[540px]:items-start @max-[540px]:[overflow-wrap:anywhere]">
        <label for="flow-destination-view">{t("destination")}</label>
        <select id="flow-destination-view" class={SIM_FIELD} bind:value={selectedDest}>{#each Array.from({length:active.dests},(_,i)=>i+1) as id}<option value={id}>D{id}</option>{/each}</select>
      </div>
    </div>
      <label class="ms-auto flex min-w-0 max-w-full items-center gap-1.5 text-[var(--muted-text)]"><input type="checkbox" bind:checked={isLbls} class="shrink-0 accent-[var(--primary)]" /><span class="[overflow-wrap:anywhere]">{t("showCapacities")}</span></label>
  </div>
</section>

<style>
  .flow-graph svg { min-height:var(--flow-svg-height); }
  @media (max-width:800px) {
    .flow-topology-panel { height:auto; min-height:0; }
    .flow-graph { flex:none; min-height:0; overflow:clip; }
    .flow-graph svg { position:static; display:block; height:var(--flow-svg-height); min-height:0; }
  }
  .flow-active { stroke-dasharray: 7 5; animation: flow-travel 1s linear infinite; }
  @keyframes flow-travel { to { stroke-dashoffset: -24; } }
  @media (prefers-reduced-motion: reduce) { .flow-active { animation: none; } }
</style>
