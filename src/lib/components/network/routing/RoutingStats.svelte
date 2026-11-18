<script lang="ts">
  import { locale, translate } from "#lib/i18n/index.ts";
  import StatRows from "#lib/components/simulation/StatRows.svelte";
  import StatSummary from "#lib/components/simulation/StatSummary.svelte";
  import { SIM_PANEL, SIM_HEADING } from "#lib/components/simulation/styles.ts";
  import { routingStatRows } from "#lib/sims/network/routing/stats.ts";
  import type { StatRow } from "#lib/sims/shared/stats.ts";
  import type { RoutingSnapshot, RoutingProtocol } from "#lib/sims/network/routing/types.ts";
  export let snapshot: RoutingSnapshot | null;
  export let protocol: RoutingProtocol;

  let rows: readonly StatRow[];
  let featuredRows: readonly StatRow[];
  let detailRows: readonly StatRow[];
  $: rows = routingStatRows(snapshot, protocol, $locale);
  $: featuredRows = rows.slice(0, 4);
  $: detailRows = rows.slice(4);
</script>

<section class={`${SIM_PANEL} sim-stat-panel routing-stats max-[760px]:order-3`}>
  <h2 class={`${SIM_HEADING} mb-[7px]`}>{translate("rip.statistics", $locale)}</h2>
  <StatSummary rows={featuredRows}/>
  <div class="sim-stat-panel__details"><StatRows rows={detailRows} fill/></div>
</section>
