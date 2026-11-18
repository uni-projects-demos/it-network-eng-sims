<script lang="ts">
  import { locale, translate } from "#lib/i18n/index.ts";
  import { wsnText } from "#lib/sims/network/wsn/i18n.ts";
  import type { WsnConfig } from "#lib/sims/network/wsn/types.ts";
  import { SIM_PANEL, SIM_HEADING, SIM_SETTINGS_LBL } from "#lib/components/simulation/styles.ts";
  import {
    MIN_DURATION_MINUTES,
    DURATION_STEP_MINUTES,
    CONTINUOUS_DURATION_SLIDER_VALUE,
    CONTINUOUS_DURATION_SECS,
    durationSliderValue,
    durationSeconds,
  } from "#lib/sims/shared/duration.ts";
  import RangeSetting from "#lib/components/simulation/RangeSetting.svelte";
  import SettingsActions from "#lib/components/simulation/SettingsActions.svelte";

  export let cfg: WsnConfig;
  export let isLoading: boolean;
  export let onChange: (changes: Partial<WsnConfig>) => void;
  export let onReset: () => void;
  export let onGenerate: () => void;
  let t: (key: string) => string;
  $: t = (key: string): string => wsnText(key, $locale);
</script>

<section class={`${SIM_PANEL} wsn-settings`}>
  <h2 class={`${SIM_HEADING} mb-[9px]`}>{t('settings')}</h2>
  <div class="mb-3">
    <label class={`${SIM_SETTINGS_LBL} mb-[5px] flex justify-between`} for="wsn-nodes">
      <span>{t('numberOfNodes')}</span><strong>{cfg.nodes}</strong>
    </label>
    <input id="wsn-nodes" class="w-full accent-[var(--primary)]" type="range" min="3" max="15" step="1"
      value={cfg.nodes} disabled={isLoading} aria-label={t('numberOfNodes')}
      oninput={(event: Event): void => onChange({ nodes: Number((event.currentTarget as HTMLInputElement).value) })}/>
  </div>
  <RangeSetting id="wsn-duration" lbl={t('timeout')}
    valLbl={cfg.to_secs === CONTINUOUS_DURATION_SECS ? '∞' : translate('rip.minutesShort', $locale, { count: cfg.to_secs / 60 })}
    accessibleVal={cfg.to_secs === CONTINUOUS_DURATION_SECS ? t('continuous') : translate('rip.minutes', $locale, { count: cfg.to_secs / 60 })}
    val={durationSliderValue(cfg.to_secs)}
    min={MIN_DURATION_MINUTES} max={CONTINUOUS_DURATION_SLIDER_VALUE} step={DURATION_STEP_MINUTES}
    disabled={isLoading} onChange={(value) => onChange({ to_secs: durationSeconds(value) })}/>
  <RangeSetting id="wsn-power" lbl={t('txPower')} valLbl={`${cfg.tx_power_dbm} dBm`}
    val={cfg.tx_power_dbm} min={-20} max={20} step={1} disabled={isLoading}
    onChange={(tx_power_dbm) => onChange({tx_power_dbm})}/>
  <RangeSetting id="wsn-noise" lbl={t('noise')} valLbl={`${cfg.noise_floor_dbm} dBm`}
    val={cfg.noise_floor_dbm} min={-115} max={-75} step={1} disabled={isLoading}
    onChange={(noise_floor_dbm) => onChange({noise_floor_dbm})}/>
  <RangeSetting id="wsn-duty" lbl={t('duty')} valLbl={`${cfg.duty_cycle_pct}%`}
    val={cfg.duty_cycle_pct} min={10} max={100} step={5} disabled={isLoading}
    onChange={(duty_cycle_pct) => onChange({duty_cycle_pct})}/>
  <RangeSetting id="wsn-battery" lbl={t('battery')} valLbl={`${cfg.initial_battery_mj} mJ`}
    val={cfg.initial_battery_mj} min={500} max={10000} step={500} disabled={isLoading}
    onChange={(initial_battery_mj) => onChange({initial_battery_mj})}/>
  <SettingsActions generateLabel={t('generate')} resetLabel={t('reset')} {isLoading} {onGenerate} {onReset}/>
</section>
