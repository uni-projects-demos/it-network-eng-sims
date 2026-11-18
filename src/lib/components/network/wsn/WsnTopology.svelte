<script lang="ts">
  import { locale } from "#lib/i18n/index.ts";
  import PlaybackControls from "#lib/components/simulation/PlaybackControls.svelte";
  import { SIM_PANEL, SIM_HEADING } from "#lib/components/simulation/styles.ts";
  import { nodeColor, nodeBadgeStyle } from "#lib/sims/shared/nodeColors.ts";
  import { batteryFill } from "#lib/sims/network/wsn/battery.ts";
  import { graphNodeRadius } from "#lib/sims/shared/graph.ts";
  import NodeSelect from "#lib/components/simulation/NodeSelect.svelte";
  import { networkEdgeId, networkToScreen, pathToGateway, screenToNetwork } from "#lib/sims/network/wsn/geometry.ts";
  import type { Pnt } from "#lib/sims/network/wsn/geometry.ts";
  import { onMount } from "svelte";
  import { formatDur } from "#lib/sims/network/routing/layout.ts";
  import { wsnText } from "#lib/sims/network/wsn/i18n.ts";
  import type { WsnLink, WsnMac, WsnNode, WsnSnapshot, WsnTransmission } from "#lib/sims/network/wsn/types.ts";
  import type { PlaybackDir } from "#lib/sims/shared/playback.ts";

  export let snapshot: WsnSnapshot | null;
  export let selectedNode: number;
  export let isPlaying: boolean;
  export let speed: number;
  export let onSelect: (id: number) => void;
  export let onMoveNode: (id: number, x: number, y: number) => void;
  export let onMove: (dir: PlaybackDir) => void;
  export let onToggle: () => void;
  export let onSpeed: (value: number) => void;
  export let onMac: (value: WsnMac) => void;
  export let cfgDuration: number;

  let svg: SVGSVGElement | undefined;
  let graph: HTMLDivElement | undefined;
  let graphWidth: number = 420;
  let graphHeight: number = 300;
  let dragging: number | null = null;
  let t: (key: string) => string;
  let nodes: WsnNode[] = [];
  let transmissions: WsnTransmission[] = [];
  let links: WsnLink[] = [];
  let displayNodes: (WsnNode & Pnt)[] = [];
  let routeNodes: number[] = [];
  let canBack: boolean = false;
  let canForward: boolean = false;
  let pathEdges: Set<string> = new Set<string>();
  let currentMac: WsnMac = "csma";
  let nodeRadius: number = 31;
  let duration: number = 0;
  let routeStart: number = 1;
  let routeMetric: string = "—";
  let routeCost: number | null = null;
  $: t = (key: string): string => wsnText(key, $locale);
  $: nodes = snapshot?.nodes ?? [];
  $: transmissions = snapshot?.transmissions ?? [];
  $: links = snapshot?.links ?? [];
  $: displayNodes = nodes.map((node: WsnNode): WsnNode & Pnt => ({...node,
    ...networkToScreen(node, {width: graphWidth, height: graphHeight})}));
  $: routeStart = selectedNode === 0 ? nodes.find((node: WsnNode): boolean => node.id > 0)?.id ?? 1 : selectedNode;
  $: routeNodes = pathToGateway(nodes, routeStart);
  $: routeCost = nodes.find((node: WsnNode): boolean => node.id === routeStart)?.route_cost ?? null;
  $: routeMetric = routeNodes.length > 0 && routeCost !== null && Number.isFinite(routeCost)
    ? routeCost.toFixed(2) : "∞";
  $: canBack = (snapshot?.time ?? 0) > (snapshot?.hist_start ?? 0) + 0.000001;
  $: canForward = snapshot !== null && !snapshot.is_complete;
  $: pathEdges = new Set(routeNodes.slice(1).map((id, index) => networkEdgeId(routeNodes[index], id)));
  $: currentMac = snapshot?.mac ?? "csma";
  $: nodeRadius = graphNodeRadius(nodes.length, graphWidth, graphHeight);
  $: duration = snapshot?.dur ?? cfgDuration;

  function transmissionFor(a: number, b: number): WsnTransmission | undefined {
    return transmissions.find((packet: WsnTransmission): boolean =>
      networkEdgeId(packet.from, packet.to) === networkEdgeId(a, b));
  }

  function position(event: PointerEvent): {x: number; y: number} | null {
    if (!svg) return null;
    const matrix = svg.getScreenCTM();
    if (!matrix) return null;
    const pt = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse());
    return screenToNetwork({x: pt.x, y: pt.y}, {width: graphWidth, height: graphHeight});
  }

  function startDrag(id: number, event: PointerEvent): void {
    if (id === 0) return;
    dragging = id;
    onSelect(id);
    event.preventDefault();
  }

  function drag(event: PointerEvent): void {
    if (dragging === null) return;
    const pt = position(event);
    if (pt) onMoveNode(dragging, pt.x, pt.y);
  }

  function stopDrag(): void { dragging = null; }

  onMount((): (() => void) => {
    const observer = new ResizeObserver((entries: ResizeObserverEntry[]): void => {
      const bounds: DOMRectReadOnly | undefined = entries[0]?.contentRect;
      if (!bounds || bounds.width < 1 || bounds.height < 1) return;
      graphWidth = Math.round(bounds.width);
      graphHeight = Math.round(bounds.height);
    });
    if (graph) observer.observe(graph);
    return (): void => observer.disconnect();
  });
</script>

<svelte:window onpointermove={drag} onpointerup={stopDrag} onpointercancel={stopDrag}/>

<section class={`${SIM_PANEL} wsn-topology`}>
  <div class="wsn-topology__header">
    <h2 class={SIM_HEADING}>{t('topology')}</h2>
    <PlaybackControls {isPlaying} {speed} {canBack} {canForward} canPlay={canForward} canEnd={canForward && duration > 0}
      playLbl={t('play')} pauseLbl={t('pause')} backLbl={t('back')} forwardLbl={t('forward')}
      startTitle={t('start')} endTitle={t('end')} controlsLbl={t('topology')} speedLbl={t('speed')}
      {onMove} {onToggle} {onSpeed}/>
    <div class="wsn-mac-toggle" role="group" aria-label={t('mac')}>
      <button type="button" class={currentMac === 'csma' ? 'wsn-mac-toggle__active' : ''} aria-pressed={currentMac === 'csma'}
        title={t('csma')} onclick={() => onMac('csma')}>CSMA/CA</button>
      <button type="button" class={currentMac === 'tdma' ? 'wsn-mac-toggle__active' : ''} aria-pressed={currentMac === 'tdma'}
        title={t('tdma')} onclick={() => onMac('tdma')}>TDMA</button>
    </div>
  </div>
  <div class="wsn-topology__timebar">
    <strong>{formatDur(snapshot?.time ?? 0)} / {duration === 0 ? '∞' : formatDur(duration)}</strong>
    {#if duration > 0}
      <div class="wsn-topology__track" role="progressbar" aria-label={t('time')} aria-valuemin="0"
        aria-valuemax={duration} aria-valuenow={Math.min(snapshot?.time ?? 0, duration)}>
        <div class="wsn-topology__progress" style={`width: ${Math.min(100, 100 * (snapshot?.time ?? 0) / duration)}%`}></div>
      </div>
    {:else}
      <div class="wsn-topology__track" aria-label={`${t('time')}: ${formatDur(snapshot?.time ?? 0)} / ∞`}></div>
    {/if}
  </div>
  <div bind:this={graph} class="wsn-topology__graph">
    <svg bind:this={svg} viewBox={`0 0 ${graphWidth} ${graphHeight}`} class="wsn-topology__svg"
      preserveAspectRatio="xMidYMid meet" role="group" aria-label={t('topology')}>
      <defs>
        <pattern id="wsn-grid" width="26" height="26" patternUnits="userSpaceOnUse">
          <circle cx="1" cy="1" r="1" fill="var(--divider)" opacity=".24"/>
        </pattern>
        <filter id="wsn-glow"><feGaussianBlur stdDeviation="4"/></filter>
      </defs>
      <rect width={graphWidth} height={graphHeight} fill="url(#wsn-grid)"/>
      {#each links as link (networkEdgeId(link.a, link.b))}
        {@const start = displayNodes[link.a]}
        {@const end = displayNodes[link.b]}
        {@const onPath = pathEdges.has(networkEdgeId(link.a, link.b))}
        {@const transmission = transmissionFor(link.a, link.b)}
        {#if start && end}
          {#if onPath}<line x1={start.x} y1={start.y} x2={end.x} y2={end.y}
            stroke="var(--primary)" stroke-width="11" opacity=".1" filter="url(#wsn-glow)"/>{/if}
          <line x1={start.x} y1={start.y} x2={end.x} y2={end.y}
            stroke={transmission?.outcome === 'success' ? 'var(--receiver)' : transmission ? 'var(--danger)' : onPath ? 'var(--primary)' : 'var(--border)'}
            stroke-width={onPath ? 4 : transmission ? 2.8 : 1.8}
            stroke-dasharray={transmission || onPath ? undefined : link.success_probability < .75 ? '3 6' : '7 5'}
            opacity={transmission || onPath ? 1 : .35 + .35 * link.success_probability}>
            <title>{`${link.a === 0 ? t('gateway') : `${t('node')} ${link.a}`} — ${link.b === 0 ? t('gateway') : `${t('node')} ${link.b}`}: ${link.snr_db.toFixed(1)} dB SNR · ${(100 * link.success_probability).toFixed(1)}%`}</title>
          </line>
        {/if}
      {/each}
      {#each displayNodes as node (node.id)}
        {@const radius = node.id === 0 ? nodeRadius + 9 : nodeRadius}
        <g role="button" tabindex="0" aria-label={node.id === 0 ? t('gateway') : `${t('node')} ${node.id}`}
          aria-pressed={selectedNode === node.id}
          onpointerdown={(event: PointerEvent): void => startDrag(node.id, event)}
          onclick={(): void => onSelect(node.id)}
          onkeydown={(event: KeyboardEvent): void => {
            if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); onSelect(node.id); }
          }}>
          {#if selectedNode === node.id}
            <circle cx={node.x} cy={node.y} r={radius + 12} fill="none" stroke="var(--primary)" stroke-width="2.5" stroke-dasharray="6 3"/>
          {/if}
          <circle cx={node.x} cy={node.y} r={radius} fill="var(--surface-solid)"
            stroke={node.id === 0 ? 'var(--wsn-gateway-ring)' : !node.is_active ? 'var(--danger)' : !node.is_awake ? 'var(--timeout)' : 'var(--receiver)'}
            stroke-width={node.id === 0 ? 4.5 : 3.5} opacity={node.is_active ? 1 : .55}/>
          <circle cx={node.x} cy={node.y} r={radius - 8} fill={node.id === 0 ? 'var(--sender-soft)' : nodeColor(node.id)?.fill ?? 'var(--sender-fill)'}/>
          <text x={node.x} y={node.y + 6} text-anchor="middle" font-size="19" font-weight="800"
            fill="#16202c" pointer-events="none">{node.id === 0 ? 'G' : node.id}</text>
          {#if node.id > 0}
            <g transform={`translate(${node.x - 26}, ${node.y + radius + 8})`} aria-label={`${t('node')} ${node.id}: ${node.battery_pct.toFixed(0)}%`}>
              <title>{`${t('battery')}: ${node.battery_pct.toFixed(0)}%`}</title>
              <rect x="0.75" y="0.75" width="12" height="9" rx="1.5" fill="none" stroke="var(--wsn-battery-border)" stroke-width="1"/>
              <rect x="13.5" y="3.2" width="2" height="4" rx="0.6" fill="var(--wsn-battery-border)"/>
              <rect x="3" y="3" width={Math.max(0, 8 * node.battery_pct / 100)} height="4.5" rx="0.5" fill={batteryFill(node.battery_pct)}/>
              <rect x="18" y="1.5" width="36" height="7" rx="3.5" fill="var(--wsn-battery-track)"/>
              <rect x="19" y="2.5" width={Math.max(0, 34 * node.battery_pct / 100)} height="5" rx="2.5" fill={batteryFill(node.battery_pct)}/>
              <rect x="18.5" y="2" width="35" height="6" rx="3" fill="none" stroke="var(--wsn-battery-border)" stroke-width="1"/>
            </g>
          {/if}
        </g>
      {/each}
    </svg>
  </div>
  <div class="wsn-topology__legend">
    <span><span class="wsn-topology__legend-dot" style="--legend-color: var(--receiver)"></span>{t('awake')}</span>
    <span><span class="wsn-topology__legend-dot" style="--legend-color: var(--timeout)"></span>{t('asleep')}</span>
    <span><span class="wsn-topology__legend-dot" style="--legend-color: var(--danger)"></span>{t('off')}</span>
    <span><span class="wsn-topology__legend-radio" aria-hidden="true">┄┄</span>{t('radioLink')}</span>
    <span><span class="wsn-topology__legend-path">━</span>{t('nextHop')}</span>
  </div>
  <div class="wsn-topology__route">
    <label for="wsn-route-from">{t('node')}</label>
    <NodeSelect id="wsn-route-from" lbl={t('node')} val={routeStart}
      nodeIds={nodes.filter((node) => node.id !== 0).map((node) => node.id)}
      labelFor={(id: number): string => `${t('node')} ${id}`} onSelect={onSelect}/>
    <div class="wsn-topology__hops" aria-label={t('nextHop')}>
      {#if routeNodes.length > 0}
        {#each routeNodes as id, index (id)}
          {#if index > 0}<span class="wsn-topology__arrow" aria-hidden="true">→</span>{/if}
          <span class={id === 0 ? 'wsn-topology__badge wsn-topology__badge--gateway' : 'wsn-topology__badge'}
            style={id === 0 ? undefined : nodeBadgeStyle(id)}>{id === 0 ? t('gateway') : `${t('node')} ${id}`}</span>
        {/each}
      {:else}
        <span class="sim-muted">{t('noRoute')}</span>
      {/if}
    </div>
    <div class="wsn-topology__destination">
      <span>{t('to')}</span>
      <strong class="wsn-topology__badge wsn-topology__badge--gateway">{t('gateway')}</strong>
    </div>
    <span class="wsn-topology__metric" title={t('routeCost')} aria-label={`${t('routeCost')}: ${routeMetric}`}>
      {routeMetric}
    </span>
  </div>
</section>
