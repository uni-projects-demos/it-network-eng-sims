<script lang="ts" generics="T extends string">
  export let tabs: readonly Readonly<{ val: T; lbl: string }>[];
  export let selected: T;
  export let onSelect: (val: T) => void;
  export let lbl: string;
  export let disabled: boolean = false;
</script>

<div class="flex w-max shrink-0 flex-nowrap items-center gap-[3px] rounded-lg border border-[var(--control-border)] bg-[var(--surface-2)] p-[3px]" role="tablist" aria-label={lbl}>
  {#each tabs as tab (tab.val)}
    <button type="button" role="tab" aria-selected={selected === tab.val}
      class={['rounded-[5px] border-0 bg-transparent px-[7px] py-1 text-[9px] text-[var(--muted-text)]', selected === tab.val && 'bg-[var(--selected-bg)] text-[var(--selected-text)] shadow-[inset_0_0_0_1px_var(--accent-border)]']}
      {disabled} onclick={() => onSelect(tab.val)}>{tab.lbl}</button>
  {/each}
</div>
