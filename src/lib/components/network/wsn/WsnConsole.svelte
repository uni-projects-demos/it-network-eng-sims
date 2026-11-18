<script lang="ts">
  import { tick } from "svelte";
  import { locale } from "#lib/i18n/index.ts";
  import ConsoleTabs from "#lib/components/simulation/ConsoleTabs.svelte";
  import NodeSelect from "#lib/components/simulation/NodeSelect.svelte";
  import { SIM_PANEL, SIM_HEADING, SIM_MUTED } from "#lib/components/simulation/styles.ts";
  import { eventCategory, wsnEventText, wsnText, type WsnConsoleFilter } from "#lib/sims/network/wsn/i18n.ts";
  import { nodeColor } from "#lib/sims/shared/nodeColors.ts";
  import type { WsnEvent } from "#lib/sims/network/wsn/types.ts";

  type ConsoleEntry = WsnEvent & { id: number };
  export let entries: ConsoleEntry[];
  export let now: number;
  let selectedFilter: WsnConsoleFilter = 'all';
  let selectedNode: number = -1;
  let consoleEl: HTMLElement;
  let lastId: number = -1;
  let t: (key: string) => string;
  let tabs: {val: WsnConsoleFilter; lbl: string}[] = [];
  let visible: ConsoleEntry[] = [];
  $: t = (key: string): string => wsnText(key, $locale);
  $: tabs = (['all','traffic','network','energy'] as const).map((value: WsnConsoleFilter) => ({val: value, lbl: t(value)}));
  $: visible = entries.filter((event: ConsoleEntry): boolean => event.time <= now + 0.000001
    && (selectedFilter === 'all' || eventCategory(event._type) === selectedFilter)
    && (selectedNode < 0 || event.node === selectedNode || event.peer === selectedNode));
  $: if ((entries.at(-1)?.id ?? -1) !== lastId) {
    lastId = entries.at(-1)?.id ?? -1;
    void tick().then((): void => { if (consoleEl) consoleEl.scrollTop = consoleEl.scrollHeight; });
  }
</script>

<section class={`${SIM_PANEL} wsn-console`}>
  <div class="mb-[7px] flex min-w-0 flex-wrap items-center gap-2">
    <h2 class={SIM_HEADING}>{t('console')}</h2>
    <div class="ms-auto flex items-center gap-1.5">
      <NodeSelect id="wsn-console-node" lbl={t('node')} val={selectedNode}
        nodeIds={Array.from(new Set(entries.flatMap((event) => event.peer === null ? [event.node] : [event.node, event.peer]))).sort((a, b) => a - b)}
        isIncludeAll allValue={-1}
        labelFor={(id: number): string => id === -1 ? t('allNodes') : id === 0 ? t('gateway') : `${t('node')} ${id}`}
        onSelect={(id: number): void => { selectedNode = id; }}/>
      <ConsoleTabs {tabs} selected={selectedFilter} lbl={t('console')}
        onSelect={(value: WsnConsoleFilter) => selectedFilter = value}/>
    </div>
  </div>
  <div class="wsn-console__entries" bind:this={consoleEl} role="log" aria-live="off">
    {#each visible as event (event.id)}
      <div class="wsn-console__entry">
        <time class="w-11 shrink-0 text-[var(--muted-text)] tabular-nums">{event.time.toFixed(1)}s</time>
        <span class="wsn-console__marker"
          style={`background: ${event._type === 'collision' || event._type === 'drop' || event._type === 'depleted' ? 'var(--danger)' : event._type === 'deliver' ? 'var(--receiver)' : 'var(--primary)'}`}></span>
        <span class="wsn-console__message" style={event.node > 0 && nodeColor(event.node)
          ? `--routing-console-light: ${nodeColor(event.node)?.light}; --routing-console-dark: ${nodeColor(event.node)?.dark}` : undefined}>
          {wsnEventText(event, $locale)}
        </span>
      </div>
    {/each}
    {#if visible.length === 0}<span class={SIM_MUTED}>{t('noEvents')}</span>{/if}
  </div>
  <div class={`mt-1 border-t border-[var(--divider)] py-1.5 text-[10px] ${SIM_MUTED}`}>{visible.length} / 1,200</div>
</section>
