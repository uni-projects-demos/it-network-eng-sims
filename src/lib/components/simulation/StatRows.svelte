<script lang="ts">
  import type { StatRow } from "#lib/sims/shared/stats.ts";
  export let rows: readonly StatRow[];
</script>

<dl class="m-0 grid gap-0 [&>div]:flex [&>div]:justify-between [&>div]:gap-2 [&>div]:border-b [&>div]:border-[var(--divider)] [&>div]:py-1 [&_dt]:text-[9px] [&_dt]:text-[var(--muted-text)] [&_dd]:m-0 [&_dd]:text-right [&_dd]:text-[10px] [&_dd]:font-extrabold [&_dd]:text-[var(--page-text)] [&_dd]:tabular-nums">
  {#each rows as row}
    <div><dt title={row.title}>{row.lbl}</dt><dd dir={row.dir}>{row.val}</dd></div>
  {/each}
</dl>
