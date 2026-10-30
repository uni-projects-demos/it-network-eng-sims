<script lang="ts">
  import { locale } from "#lib/i18n/index.ts";
  import { flowTranslator } from "#lib/sims/network/flow/i18n.ts";
  import { SIM_PANEL, SIM_HEADING, SIM_SETTINGS_LBL, SIM_FIELD, SIM_PRIMARY_BTN } from "#lib/components/simulation/styles.ts";
  import { FLOW_COUNT_FIELDS, FLOW_MIN_COUNTS, FLOW_MAX_NODES, type FlowCountField, type FlowConfig } from "#lib/sims/network/flow/types.ts";

  export let cfg: FlowConfig;
  export let isBusy: boolean;
  export let onChange: (changes: Partial<FlowConfig>) => void;
  export let onRun: () => void;
  export let onReset: () => void;
  export let error: string;

  function commitCount(key: FlowCountField, input: HTMLInputElement): void {
    const parsed: number = input.valueAsNumber;
    const val: number = Math.max(FLOW_MIN_COUNTS[key], Math.min(FLOW_MAX_NODES, Math.trunc(Number.isFinite(parsed) ? parsed : FLOW_MIN_COUNTS[key])));
    input.value = String(val);
    if (val !== cfg[key]) onChange({[key]:val});
  }

  $: t = flowTranslator($locale);
</script>

<section class={`${SIM_PANEL} shrink-0 p-3.5`}>
  <div class="mb-3 flex items-center justify-between gap-2">
    <h2 class={SIM_HEADING}>{t("configuration")}</h2>
    <button class="text-[10px] text-[var(--primary)] underline underline-offset-2" onclick={onReset} disabled={isBusy}>{t("reset")}</button>
  </div>
  <div class="grid grid-cols-3 gap-2">
    {#each FLOW_COUNT_FIELDS as key}
      <label class={`${SIM_SETTINGS_LBL} mt-0 min-w-0`} for={`flow-${key}`}>
        {t(key)}
        <input id={`flow-${key}`} class={`${SIM_FIELD} w-full text-xs`} type="number" min={FLOW_MIN_COUNTS[key]} max={FLOW_MAX_NODES} step="1" required
          value={cfg[key]} disabled={isBusy}
          oninput={(event) => {if (event.currentTarget.validity.valid) onChange({ [key]: event.currentTarget.valueAsNumber });}}
          onblur={(event) => commitCount(key,event.currentTarget)} />
      </label>
    {/each}
  </div>
  <button class={`${SIM_PRIMARY_BTN} mt-3 w-full px-3 text-[11px]`} onclick={onRun} disabled={isBusy}>{isBusy ? t("loading") : t("optimise")}</button>
  {#if error}<p role="alert" class="mb-0 mt-3 break-words text-[11px] text-[var(--danger-text)]">{error}</p>{/if}
</section>
