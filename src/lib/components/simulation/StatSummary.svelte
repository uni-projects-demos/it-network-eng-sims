<script lang="ts">
  import type { StatRow } from "#lib/sims/shared/stats.ts";
  export let rows: readonly StatRow[];
</script>

<div class="sim-stat-summary mb-3 grid grid-cols-2 gap-2">
  {#each rows as row, idx (row.lbl)}
    <div class="min-w-0 rounded-lg bg-[var(--surface-2)] p-2.5">
      <span class="block text-[10px] text-[var(--muted-text)]">{row.lbl}</span>
      <strong class={`block tabular-nums ${idx === 2 ? 'text-base' : 'text-xl'}`}>{row.val}</strong>
    </div>
  {/each}
</div>
