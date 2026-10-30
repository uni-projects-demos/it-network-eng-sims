<script lang="ts">
  import { locale } from "#lib/i18n/index.ts";
  import { flowTranslator } from "#lib/sims/network/flow/i18n.ts";
  import { SIM_PANEL, SIM_HEADING, SIM_CONTROL_BTN } from "#lib/components/simulation/styles.ts";
  import type { FlowConfig } from "#lib/sims/network/flow/types.ts";
  import { flowFilename } from "#lib/sims/network/flow/files.ts";
  export let cfg: FlowConfig;
  export let lpTxt: string;
  export let lpUrl: string;
  export let isBusy: boolean;
  $: t = flowTranslator($locale);
</script>

<section class={`${SIM_PANEL} flex min-h-[180px] flex-1 flex-col overflow-hidden max-[1240px]:min-h-[300px]`}>
  <div class="flex shrink-0 flex-wrap items-center justify-between gap-2 border-b border-[var(--divider)] px-3 py-2.5">
    <h2 class={SIM_HEADING}>{t("previewLp")}</h2>
    <div class="ms-auto flex shrink-0 items-center gap-1.5">
      <a class={`${SIM_CONTROL_BTN} inline-flex items-center justify-center no-underline ${isBusy || !lpUrl ? "pointer-events-none opacity-40" : ""}`}
        href={isBusy || !lpUrl ? undefined : lpUrl} download={flowFilename(cfg)}
        aria-disabled={isBusy || !lpUrl} tabindex={isBusy || !lpUrl ? -1 : 0}>{t("downloadLp")}</a>
    </div>
  </div>
  <div class="min-h-0 flex-1 p-3">
    <textarea readonly dir="ltr" lang="en" spellcheck="false" value={lpTxt} aria-label={t("generatedLp")} rows="12" class="block h-full w-full resize-none rounded border border-[var(--field-border)] bg-[var(--field-bg)] p-2 font-mono text-[9px] text-[var(--page-text)]"></textarea>
  </div>
</section>
