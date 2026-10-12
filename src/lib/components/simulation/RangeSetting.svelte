<script lang="ts">
  import { SIM_SETTINGS_LBL } from "./styles.ts";
  export let id: string;
  export let lbl: string;
  export let valLbl: string;
  export let val: number;
  export let min: number;
  export let max: number;
  export let step: number = 1;
  export let disabled: boolean = false;
  export let accessibleVal: string | undefined = undefined;
  export let onChange: (val: number) => void;
</script>

<label class={`${SIM_SETTINGS_LBL} mb-3 block`} for={id}>
  <span class="flex justify-between gap-2"><span>{lbl}</span><strong>{valLbl}</strong></span>
  <input {id} aria-label={lbl} aria-valuetext={accessibleVal} class="w-full accent-[var(--primary)] focus:shadow-[0_0_0_3px_var(--focus-ring)]" type="range"
    {min} {max} {step} value={val} {disabled}
    oninput={(event) => onChange(Number(event.currentTarget.value))} />
</label>
