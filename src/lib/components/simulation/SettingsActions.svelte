<script lang="ts">
  import { SIM_PRIMARY_BTN } from "./styles.ts";

  export let generateLabel: string;
  export let resetLabel: string;
  export let isLoading: boolean;
  export let onGenerate: () => void;
  export let onReset: () => void;
</script>

<div class="sim-settings-actions">
  <button type="button" class={`${SIM_PRIMARY_BTN} sim-settings-actions__button`}
    disabled={isLoading} onclick={onGenerate}>{generateLabel}</button>
  <button type="button" class="sim-settings-actions__button sim-settings-actions__reset"
    disabled={isLoading} onclick={onReset}>{resetLabel}</button>
</div>
