<script lang="ts">
import { BITE_FILE_PAYLOAD_SIZE_OPTIONS } from "#lib/sims/transport/payload.ts";

export let label: string;
export let value: number;
export let headerLabel: string | undefined = undefined;
export let headerSize: number | undefined = undefined;
</script>

<label class="mt-[9px] grid gap-[5px] text-[10px] font-[650] text-[var(--control-text)]">
  <span class="flex items-baseline justify-between gap-2">
    {label}
    <strong>
      {value} B
      {#if headerSize !== undefined && headerLabel !== undefined}
        <small class="text-[9px] font-semibold text-[var(--muted-text)]">(+ {headerSize} B {headerLabel})</small>
      {/if}
    </strong>
  </span>
  <select
    class="min-h-[34px] w-full rounded-[9px] border border-[var(--field-border)] bg-[var(--field-bg)] px-[9px] py-[7px] text-[var(--page-text)] outline-none focus:border-[var(--accent-border)] focus:shadow-[0_0_0_3px_var(--focus-ring)]"
    bind:value
  >
    {#each BITE_FILE_PAYLOAD_SIZE_OPTIONS as size}
      <option value={size}>{size} B</option>
    {/each}
  </select>
</label>
