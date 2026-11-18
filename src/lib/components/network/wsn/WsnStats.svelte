<script lang="ts">
  import { locale } from "#lib/i18n/index.ts";
  import { wsnText } from "#lib/sims/network/wsn/i18n.ts";
  import type { WsnSnapshot } from "#lib/sims/network/wsn/types.ts";
  import type { StatRow } from "#lib/sims/shared/stats.ts";
  import StatRows from "#lib/components/simulation/StatRows.svelte";
  import StatSummary from "#lib/components/simulation/StatSummary.svelte";
  import { SIM_PANEL, SIM_HEADING } from "#lib/components/simulation/styles.ts";

  export let snapshot: WsnSnapshot | null;
  let t: (key: string) => string;
  let rows: StatRow[] = [];
  $: t = (key: string): string => wsnText(key, $locale);
  $: rows = (() : StatRow[] => {
    const stats = snapshot?.stats;
    return [
      { lbl: t('delivery'), val: `${(stats?.delivery_pct ?? 0).toFixed(1)}%` },
      { lbl: t('delivered'), val: stats?.delivered ?? 0 },
      { lbl: t('collisions'), val: stats?.collisions ?? 0 },
      { lbl: t('energyUsed'), val: `${(stats?.energy_mj ?? 0).toFixed(0)} mJ` },
      { lbl: t('generated'), val: stats?.generated ?? 0 },
      { lbl: t('forwarded'), val: stats?.forwarded ?? 0 },
      { lbl: t('transmissions'), val: stats?.transmissions ?? 0 },
      { lbl: t('errors'), val: stats?.errors ?? 0 },
      { lbl: t('retries'), val: stats?.retries ?? 0 },
      { lbl: t('dropped'), val: stats?.dropped ?? 0 },
      { lbl: t('latency'), val: `${(stats?.avg_latency_secs ?? 0).toFixed(2)} s` },
      { lbl: t('depleted'), val: stats?.depleted_nodes ?? 0 },
      { lbl: t('queued'), val: stats?.queued_frames ?? 0 },
    ];
  })();
</script>

<section class={`${SIM_PANEL} sim-stat-panel wsn-stats`}>
  <h2 class={`${SIM_HEADING} mb-[7px]`}>{t('statistics')}</h2>
  <StatSummary rows={rows.slice(0, 4)}/>
  <div class="sim-stat-panel__details wsn-stats__details"><StatRows rows={rows.slice(4)} fill/></div>
</section>
