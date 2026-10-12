<script lang="ts">
import { SIM_SETTINGS_LBL } from "#lib/components/simulation/styles.ts";
import { BITE_FILE_PAYLOAD_SIZE_OPTIONS } from "#lib/sims/transport/payload.ts";

export let lbl: string;
export let val: number;
export let headerLbl: string | undefined = undefined;
export let headerSize: number | undefined = undefined;
</script>

<label class={SIM_SETTINGS_LBL}>
  <span class="flex items-baseline justify-between gap-2">
    {lbl}
    <strong>
      {val} B
      {#if headerSize !== undefined && headerLbl !== undefined}
        <small class="text-[9px] font-semibold text-[var(--muted-text)]">(+ {headerSize} B {headerLbl})</small>
      {/if}
    </strong>
  </span>
  <select
    class="min-h-[34px] w-full rounded-[9px] border border-[var(--field-border)] bg-[var(--field-bg)] px-[9px] py-[7px] text-[var(--page-text)] outline-none focus:border-[var(--accent-border)] focus:shadow-[0_0_0_3px_var(--focus-ring)]"
    bind:value={val}
  >
    {#each BITE_FILE_PAYLOAD_SIZE_OPTIONS as size}
      <option value={size}>{size} B</option>
    {/each}
  </select>
</label>
