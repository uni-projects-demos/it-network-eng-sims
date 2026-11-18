<script lang="ts">
  import { locale, translate, type TranslationParams } from "#lib/i18n/index.ts";
  import { routingText } from "#lib/sims/network/routing/i18n.ts";
  import { SIM_PANEL, SIM_HEADING, SIM_SETTINGS_LBL } from "#lib/components/simulation/styles.ts";
  import RangeSetting from "#lib/components/simulation/RangeSetting.svelte";
  import SettingsActions from "#lib/components/simulation/SettingsActions.svelte";
  import {
    ROUTING_MIN_TO_MINS,
    ROUTING_TO_STEP_MINS,
    ROUTING_CONT_SLIDER_VAL,
    ROUTING_CONT_TO_SECS,
    timeOutSliderVal,
    sliderTimeOutSecs,
  } from "#lib/sims/network/routing/types.ts";
  import type { RoutingConfig, RoutingProtocol } from "#lib/sims/network/routing/types.ts";

  $: t = (key: string, params: TranslationParams = {}): string => routingText(protocol, key, $locale, params);

  export let protocol: RoutingProtocol;
  export let cfg: RoutingConfig;
  export let isLoading: boolean;
  export let err: string;
  export let onConfigChange: (changes: Partial<RoutingConfig>) => void;
  export let onPacketLoss: (percent: number) => void;
  export let onPacketInvalid: (percent: number) => void;
  export let onReset: () => void;
  export let onGenerate: () => void;
</script>

<section class={`${SIM_PANEL} routing-settings shrink-0 px-3.5 py-[13px] max-[760px]:order-1`}>
  <h2 class={`${SIM_HEADING} mb-[9px]`}>{t("settings")}</h2>
  <div class="mb-3">
    <label class={`${SIM_SETTINGS_LBL} mb-[5px] flex justify-between`} for="routing-router-count">
      <span>{t("numberOfRouters")}</span><strong>{cfg.nodes}</strong>
    </label>
    <input id="routing-router-count" aria-label={t("numberOfRouters")}
      class="w-full accent-[var(--primary)]" type="range" min="3" max="15" step="1"
      value={cfg.nodes} disabled={isLoading}
      oninput={(event) => onConfigChange({ nodes: Number(event.currentTarget.value), topology: "rand" })} />
  </div>
  <RangeSetting id="routing-simulation-timeout" lbl={t("simulationTimeout")}
    val={timeOutSliderVal(cfg.to_secs)}
    valLbl={cfg.to_secs === ROUTING_CONT_TO_SECS ? '∞' : t("minutesShort", {count: cfg.to_secs / 60})}
    accessibleVal={cfg.to_secs === 0 ? t("continuousDaemon"): t("minutes", {count: cfg.to_secs / 60})}
    min={ROUTING_MIN_TO_MINS} max={ROUTING_CONT_SLIDER_VAL} step={ROUTING_TO_STEP_MINS}
    disabled={isLoading} onChange={(val) => onConfigChange({ to_secs: sliderTimeOutSecs(val) })} />
  <RangeSetting id="routing-random-failure" lbl={t("randomRouterInactivity")}
    val={cfg.rand_fail_perc} valLbl={cfg.rand_fail_perc === 0 ? t("off"): t("perMinutePerRouter", {percent: cfg.rand_fail_perc})}
    min={0} max={10} disabled={isLoading} onChange={(val) => onConfigChange({ rand_fail_perc: val })} />
  <RangeSetting id="routing-packet-loss" lbl={t("packetLossProbability")}
    val={cfg.pck_loss_perc} valLbl={cfg.pck_loss_perc === 0 ? t("off"): t("perPacket", {percent: cfg.pck_loss_perc})}
    min={0} max={10} disabled={isLoading} onChange={onPacketLoss} />
  <RangeSetting id="routing-packet-invalid" lbl={t("invalidPacketProbability")}
    val={cfg.pck_invalid_perc} valLbl={cfg.pck_invalid_perc === 0 ? t("off"): t("perPacket", {percent: cfg.pck_invalid_perc})}
    min={0} max={10} disabled={isLoading} onChange={onPacketInvalid} />
  <SettingsActions
    generateLabel={translate("wsn.generate", $locale)}
    resetLabel={t("reset")} {isLoading} {onGenerate} {onReset}/>
  {#if err}<p role="alert" class="mt-2 text-xs text-[var(--danger)]">{err}</p>{/if}
</section>
