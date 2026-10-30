<script lang="ts">
  import { locale, translate } from "#lib/i18n/index.ts";
  import StatRows from "#lib/components/simulation/StatRows.svelte";
  import { SIM_PANEL, SIM_HEADING } from "#lib/components/simulation/styles.ts";
  import type { StatRow } from "#lib/sims/shared/stats.ts";
  import type { RipNode, RipSnapshot } from "#lib/sims/network/rip/types.ts";
  export let snapshot: RipSnapshot | null;

  let rows: readonly StatRow[];
  let featuredRows: readonly StatRow[];
  let detailRows: readonly StatRow[];
  $: rows = [
    { lbl: translate("rip.routeAccuracy", $locale), val: `${((snapshot?.stats.accuracy ?? 0) * 100).toFixed(0)}%` },
    { lbl: translate("rip.averageConvergenceTime", $locale), val: snapshot?.stats.convergence_secs == null ? "—" : `${snapshot.stats.convergence_secs.toFixed(1)} s` },
    { lbl: translate("rip.updatesSentReceived", $locale), val: `${snapshot?.stats.updates_sent ?? 0} / ${snapshot?.stats.updates_rcvd ?? 0}` },
    { lbl: translate("rip.routesChanged", $locale), val: snapshot?.stats.route_changes ?? 0 },
    { lbl: translate("rip.runningRouters", $locale), val: `${snapshot?.nodes.filter((n: RipNode): boolean => n.is_running).length ?? 0} / ${snapshot?.nodes.length ?? 0}` },
    { lbl: translate("rip.ripReachability", $locale), val: `${((snapshot?.stats.rip_reachability ?? 0) * 100).toFixed(1)}%` },
    { lbl: translate("rip.poisonedRoutes", $locale), val: snapshot?.stats.invalids ?? 0 },
    { lbl: translate("rip.garbageCollectedRoutes", $locale), val: snapshot?.stats.garbage_collections ?? 0 },
    { lbl: translate("rip.ripPacketsLost", $locale), val: snapshot?.stats.pcks_lost ?? 0 },
    { lbl: translate("rip.invalidPacketsDropped", $locale), val: snapshot?.stats.pcks_dropped ?? 0 },
    { lbl: translate("rip.triggeredUpdates", $locale), val: snapshot?.stats.triggered_updates ?? 0 },
    { lbl: translate("rip.maxShortestCost", $locale), val: snapshot?.stats.shortest_path_cost_max ?? 0 },
    { lbl: translate("rip.meanShortestCost", $locale), val: (snapshot?.stats.shortest_path_cost_mean ?? 0).toFixed(2) },
    { lbl: translate("rip.minShortestCost", $locale), val: snapshot?.stats.shortest_path_cost_min ?? 0 },
  ];
  $: featuredRows = rows.slice(0, 4);
  $: detailRows = rows.slice(4);
</script>

<section class={`${SIM_PANEL} mt-auto shrink-0 p-[11px] max-[760px]:order-3 max-[760px]:mt-0`}>
  <h2 class={`${SIM_HEADING} mb-[7px]`}>{translate("rip.statistics", $locale)}</h2>
  <div class="mb-3 grid grid-cols-2 gap-2">
    {#each featuredRows as row, idx (row.lbl)}
      <div class="min-w-0 rounded-lg bg-[var(--surface-2)] p-2.5">
        <span class="block text-[10px] text-[var(--muted-text)]">{row.lbl}</span>
        <strong class={`block tabular-nums ${idx === 2 ? 'text-base' : 'text-xl'}`}>{row.val}</strong>
      </div>
    {/each}
  </div>
  <StatRows rows={detailRows}/>
</section>
