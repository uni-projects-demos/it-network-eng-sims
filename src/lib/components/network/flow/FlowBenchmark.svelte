<script lang="ts">
  import { locale } from "#lib/i18n/index.ts";
  import { flowTranslator } from "#lib/sims/network/flow/i18n.ts";
  import { SIM_PANEL, SIM_HEADING, SIM_CONTROL_BTN } from "#lib/components/simulation/styles.ts";
  import { type FlowBenchmark } from "#lib/sims/network/flow/types.ts";
  import { flowDuration, flowNumber } from "#lib/sims/network/flow/format.ts";
  export let rows: FlowBenchmark[];
  export let isRunning: boolean;
  export let isLoading: boolean;
  export let onRun: () => void;
  export let onCancel: () => void;
  $: t = flowTranslator($locale);
  $: number = (val: number | null | undefined): string => flowNumber(val, $locale);
</script>

<section class={`${SIM_PANEL} flex max-h-[330px] min-h-0 shrink-0 flex-col overflow-hidden`}>
  <div class="flex flex-wrap items-center justify-between gap-2 border-b border-[var(--divider)] px-3 py-2.5">
    <h2 class={SIM_HEADING}>{t("benchmark")}</h2>
    <bdi dir="ltr" class="flex-1 text-center text-[11px] text-[var(--muted-text)]">
      <math display="inline" aria-label={"X = Z = 7, Y ∈ {3, …, 7}"}>
        <mrow><mi>X</mi><mo>=</mo><mi>Z</mi><mo>=</mo><mn>7</mn><mo>,</mo><mspace width="0.4em"></mspace><mi>Y</mi><mo>∈</mo><mo>{'{'}</mo><mn>3</mn><mo>,</mo><mo>…</mo><mo>,</mo><mn>7</mn><mo>{'}'}</mo></mrow>
      </math>
    </bdi>
    <button class={SIM_CONTROL_BTN} onclick={isRunning ? onCancel : onRun} disabled={isLoading}>{isRunning ? t("cancel") : t("runBenchmark")}</button>
  </div>
  <div class="min-h-0 overflow-auto px-3 pb-3">
    {#each rows as {res, rust_ms}}
      <details class="border-b border-[var(--divider)] py-2 text-[10px]">
        <summary class="cursor-pointer"><bdi class="me-2 font-bold">Y={res.config.trans}</bdi><span class="text-[var(--receiver)]">{t(res.status === "feasible" ? "limitReached" : res.status)}</span> · <bdi>r={number(res.obj)}</bdi> · {flowDuration(rust_ms, $locale)}</summary>
        <dl class="mb-1 grid grid-cols-[1fr_auto] gap-1.5 pt-2 text-[9px]">
          <dt class="text-[var(--muted-text)]">{t("nonzeroLinks")}</dt><dd class="m-0 text-end">{number(res.nonzero_links)}</dd>
          <dt class="text-[var(--muted-text)]">{t("highestCapacity")}</dt><dd class="m-0 text-end">{number(res.max_cap)}</dd>
          <dt class="text-[var(--muted-text)]">{t("loads")}</dt><dd class="m-0 max-w-44 text-end">{res.loads.map(number).join(" · ")}</dd>
        </dl>
      </details>
    {/each}
  </div>
</section>
