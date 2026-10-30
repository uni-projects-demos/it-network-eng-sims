<script lang="ts">
  import { locale } from "#lib/i18n/index.ts";
  import { flowTranslator } from "#lib/sims/network/flow/i18n.ts";
  import { SIM_PANEL, SIM_HEADING } from "#lib/components/simulation/styles.ts";
  import StatRows from "#lib/components/simulation/StatRows.svelte";
  import { type FlowSnapshot } from "#lib/sims/network/flow/types.ts";
  import { flowDuration, flowNumber, flowPercent } from "#lib/sims/network/flow/format.ts";
  import type { StatRow } from "#lib/sims/shared/stats.ts";
  export let snapshot: FlowSnapshot | null;
  export let computeMS: number;
  $: t = flowTranslator($locale);
  $: number = (val: number | null | undefined): string => flowNumber(val, $locale);
  let rows: readonly StatRow[];
  $: rows = [
    { lbl: t("totalDemand"), val: number(snapshot?.total_demand) },
    { lbl: t("peakLoad"), val: number(snapshot?.obj) },
    { lbl: t("lowerBound"), val: number(snapshot?.status === "infeasible" ? null : snapshot?.lower_bound) },
    { lbl: t("gap"), val: flowPercent(snapshot?.gap, $locale) },
    { lbl: t("nonzeroLinks"), val: snapshot ? `${number(snapshot.nonzero_links)} / ${number(snapshot.config.trans * (snapshot.config.srcs + snapshot.config.dests))}` : "—", dir: "ltr" },
    { lbl: t("highestCapacity"), val: number(snapshot?.max_cap) },
    { lbl: t("computeTime"), val: flowDuration(computeMS, $locale) },
    { lbl: t("workDone"), val: number(snapshot?.work) },
  ];
</script>
<section class={`${SIM_PANEL} shrink-0 p-3.5`}>
  <h2 class={`${SIM_HEADING} mb-2`}>{t("statistics")}</h2>
  <StatRows {rows} />
</section>
