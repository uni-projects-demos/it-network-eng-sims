<script lang="ts">
  import { locale } from "#lib/i18n/index.ts";
  import { SIM_PANEL, SIM_HEADING, SIM_MUTED, SIM_CONTROL_BTN } from "#lib/components/simulation/styles.ts";
  import { wsnText } from "#lib/sims/network/wsn/i18n.ts";
  import { nodeColor } from "#lib/sims/shared/nodeColors.ts";
  import { batteryFill, batteryHealth } from "#lib/sims/network/wsn/battery.ts";
  import NodeSelect from "#lib/components/simulation/NodeSelect.svelte";
  import type { WsnNode, WsnSnapshot } from "#lib/sims/network/wsn/types.ts";

  export let snapshot: WsnSnapshot | null;
  export let selectedNode: number;
  export let onSelect: (id: number) => void;
  export let onEnabled: (id: number, enabled: boolean) => void;
  export let onRecharge: (id: number) => void;
  let t: (key: string) => string;
  let selected: WsnNode | undefined;
  $: t = (key: string): string => wsnText(key, $locale);
  $: selected = snapshot?.nodes.find((node: WsnNode): boolean => node.id === selectedNode);
</script>

<section class={`${SIM_PANEL} wsn-node-panel`}>
  <div class="mb-2 flex flex-wrap items-center justify-between gap-2">
    <h2 class={SIM_HEADING}>{t('nodeStatus')}</h2>
    <div class="flex items-center gap-2">
      <label for="wsn-status-node" class="text-[10px] font-[650] text-[var(--control-text)]">{t('node')}</label>
      <NodeSelect id="wsn-status-node" lbl={t('node')} val={selectedNode}
        nodeIds={snapshot?.nodes.map((node) => node.id) ?? []}
        labelFor={(id: number): string => id === 0 ? t('gateway') : `${t('node')} ${id}`}
        onSelect={onSelect}/>
    </div>
  </div>
  {#if selected}
    <div class="wsn-node-panel__summary">
      <div class="wsn-node-panel__summary-cell">
        <span class={SIM_MUTED}>{t('batteryLeft')}</span>
        <span class="wsn-node-panel__value tabular-nums">{selected.id === 0 ? '∞' : `${selected.battery_pct.toFixed(1)}%`}</span>
      </div>
      <div class="wsn-node-panel__summary-cell">
        <span class={SIM_MUTED}>{t('queue')}</span>
        <span class="wsn-node-panel__value tabular-nums">{selected.queue_len}</span>
      </div>
      <div class="wsn-node-panel__summary-cell">
        <span class={SIM_MUTED}>{t('nextHop')}</span>
        <span class="wsn-node-panel__value">{selected.id === 0 ? '—' : selected.next_hop === null ? t('none') : selected.next_hop === 0 ? t('gateway') : String(selected.next_hop)}</span>
      </div>
      <div class="wsn-node-panel__summary-cell">
        <span class={SIM_MUTED}>{t('state')}</span>
        <span class="wsn-node-panel__value">{!selected.is_active ? t('off') : selected.is_awake ? t('awake') : t('asleep')}</span>
      </div>
    </div>
    {#if selected.id > 0 && selected.battery_mj > 0}
      <button type="button" class={`${SIM_CONTROL_BTN} w-full`}
        onclick={() => onEnabled(selectedNode, !selected.is_active)}>
        {selected.is_active ? t('stopNode') : t('startNode')}
      </button>
    {:else if selected.id > 0 && selected.battery_mj <= 0}
      <button type="button" class={`${SIM_CONTROL_BTN} w-full`}
        onclick={() => onRecharge(selectedNode)}>{t('rechargeNode')}</button>
    {/if}
  {/if}
  <div class="wsn-node-panel__list">
    {#each snapshot?.nodes.slice(1) ?? [] as node (node.id)}
      <button type="button" class={selectedNode === node.id ? "wsn-node-panel__row wsn-node-panel__selected" : "wsn-node-panel__row"} onclick={() => onSelect(node.id)} aria-pressed={selectedNode === node.id}>
        <span class="wsn-node-panel__id" style={`--node-fill: ${nodeColor(node.id)?.fill ?? 'var(--sender-fill)'}`}>{node.id}</span>
        <svg class="wsn-battery" viewBox="0 0 22 12" aria-hidden="true">
          <rect x=".75" y=".75" width="18" height="10.5" rx="2" fill="none" stroke="var(--wsn-battery-border)" stroke-width="1" />
          <rect x="20" y="3.5" width="2" height="5" rx=".8" fill="var(--wsn-battery-border)" />
          <rect x="3" y="3" width={Math.max(0, 14 * node.battery_pct / 100)} height="6" rx=".7"
            fill={batteryFill(node.battery_pct)}/>
        </svg>
        <div class="wsn-node-panel__meter" role="progressbar" aria-label={`${t('node')} ${node.id}: ${t('batteryLeft')}`}
          aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(node.battery_pct)}>
          <div class={`wsn-node-panel__charge wsn-node-panel__${batteryHealth(node.battery_pct)}`}
            style={`width: ${Math.max(0, Math.min(100, node.battery_pct))}%`}></div>
        </div>
        <span class="wsn-node-panel__percent">{node.battery_pct.toFixed(0)}%</span>
      </button>
    {/each}
  </div>
</section>
