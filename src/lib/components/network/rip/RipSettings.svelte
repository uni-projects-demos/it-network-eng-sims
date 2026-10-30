<script lang="ts">
  import { locale, translate, type TranslationParams } from "#lib/i18n/index.ts";
  import { SIM_PANEL, SIM_HEADING, SIM_SETTINGS_LBL, SIM_PRIMARY_BTN } from "#lib/components/simulation/styles.ts";
  import RangeSetting from "#lib/components/simulation/RangeSetting.svelte";
  import {
    RIP_MIN_TO_MINS,
    RIP_TO_STEP_MINS,
    RIP_CONT_SLIDER_VAL,
    RIP_CONT_TO_SECS,
    timeOutSliderVal,
    sliderTimeOutSecs,
  } from "#lib/sims/network/rip/types.ts";
  import type { RipConfig } from "#lib/sims/network/rip/types.ts";

  $: t = (key: string, params: TranslationParams = {}): string => translate(`rip.${key}`, $locale, params);

  export let cfg: RipConfig;
  export let isLoading: boolean;
  export let err: string;
  export let onConfigChange: (changes: Partial<RipConfig>) => void;
  export let onPacketLoss: (percent: number) => void;
  export let onPacketInvalid: (percent: number) => void;
  export let onReset: () => void;
  export let onGenerate: () => void;
</script>

<section class={`${SIM_PANEL} shrink-0 px-3.5 py-[13px] max-[760px]:order-1`}>
  <h2 class={`${SIM_HEADING} mb-[9px]`}>{t("settings")}</h2>
  <div class="mb-3">
    <label class={`${SIM_SETTINGS_LBL} mb-[5px] flex justify-between`} for="rip-router-count">
      <span>{t("numberOfRouters")}</span><strong>{cfg.nodes}</strong>
    </label>
    <div class="flex items-center gap-3">
      <input id="rip-router-count" aria-label={t("numberOfRouters")} class="min-w-0 flex-1 accent-[var(--primary)]"
        type="range" min="3" max="15" step="1" value={cfg.nodes} disabled={isLoading}
        oninput={(event) => onConfigChange({ nodes: Number(event.currentTarget.value), topology: "rand" })} />
      <button type="button" class="shrink-0 text-[10px] font-medium text-[var(--primary)] underline underline-offset-2 hover:opacity-80 disabled:opacity-40"
        onclick={onReset} disabled={isLoading} title={t("resetTitle")}>
        <span aria-hidden="true" class="me-1 inline-block">↺</span>{t("reset")}
      </button>
    </div>
  </div>
  <RangeSetting id="rip-simulation-timeout" lbl={t("simulationTimeout")}
    val={timeOutSliderVal(cfg.to_secs)}
    valLbl={cfg.to_secs === RIP_CONT_TO_SECS ? '∞' : t("minutesShort", {count: cfg.to_secs / 60})}
    accessibleVal={cfg.to_secs === 0 ? t("continuousDaemon"): t("minutes", {count: cfg.to_secs / 60})}
    min={RIP_MIN_TO_MINS} max={RIP_CONT_SLIDER_VAL} step={RIP_TO_STEP_MINS}
    disabled={isLoading} onChange={(val) => onConfigChange({ to_secs: sliderTimeOutSecs(val) })} />
  <RangeSetting id="rip-random-failure" lbl={t("randomRouterInactivity")}
    val={cfg.rand_fail_perc} valLbl={cfg.rand_fail_perc === 0 ? t("off"): t("perMinutePerRouter", {percent: cfg.rand_fail_perc})}
    min={0} max={10} disabled={isLoading} onChange={(val) => onConfigChange({ rand_fail_perc: val })} />
  <RangeSetting id="rip-packet-loss" lbl={t("packetLossProbability")}
    val={cfg.pck_loss_perc} valLbl={cfg.pck_loss_perc === 0 ? t("off"): t("perPacket", {percent: cfg.pck_loss_perc})}
    min={0} max={10} disabled={isLoading} onChange={onPacketLoss} />
  <RangeSetting id="rip-packet-invalid" lbl={t("invalidPacketProbability")}
    val={cfg.pck_invalid_perc} valLbl={cfg.pck_invalid_perc === 0 ? t("off"): t("perPacket", {percent: cfg.pck_invalid_perc})}
    min={0} max={10} disabled={isLoading} onChange={onPacketInvalid} />
  <button type="button" class={`${SIM_PRIMARY_BTN} mt-[11px] w-full px-3 text-[10px]`} onclick={onGenerate} disabled={isLoading}>
    {isLoading ? t("buildingTopology"): t("generateRandomTopology")}
  </button>
  {#if err}<p role="alert" class="mt-2 text-xs text-[var(--danger)]">{err}</p>{/if}
</section>
