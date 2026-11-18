<script lang="ts">
  import { SIM_FIELD } from "./styles.ts";
  import { nodeBadgeStyle } from "#lib/sims/shared/nodeColors.ts";

  export let id: string;
  export let lbl: string;
  export let val: number;
  export let nodeIds: readonly number[];
  export let onSelect: (nodeId: number) => void;
  export let isIncludeAll: boolean = false;
  export let allValue: number = 0;
  export let allLabel: string = "All";
  export let labelFor: ((id: number) => string) | undefined = undefined;
  export let disabled: boolean = false;

  let trigger: HTMLButtonElement;
  let menu: HTMLDivElement;
  let open: boolean = false;
  $: choices = isIncludeAll ? [allValue, ...nodeIds] : [...nodeIds];

  function optionLbl(nodeId: number): string {
    return labelFor ? labelFor(nodeId) : nodeId === allValue ? allLabel : `R${nodeId}`;
  }

  function focusOption(nodeId: number): void {
    menu?.querySelector<HTMLElement>(`[data-node-id="${nodeId}"]`)?.focus();
  }

  function show(): void {
    if (!menu || !trigger || disabled || choices.length === 0) return;
    if (menu.matches(":popover-open")) { menu.hidePopover(); return; }
    const rect: DOMRect = trigger.getBoundingClientRect();
    const width: number = Math.max(rect.width, 110);
    const desiredHeight: number = Math.min(266, choices.length * 34 + 12);
    const below: number = window.innerHeight - rect.bottom - 8;
    const above: number = rect.top - 8;
    const placeAbove: boolean = below < Math.min(150, desiredHeight) && above > below;
    const maxHeight: number = Math.max(48, Math.min(desiredHeight, placeAbove ? above : below));
    const left: number = Math.max(8, Math.min(rect.left, window.innerWidth - width - 8));
    const top: number = placeAbove ? Math.max(8, rect.top - maxHeight - 4): rect.bottom + 4;
    menu.style.cssText = `position:fixed;inset:auto;margin:0;left:${left}px;top:${top}px;width:${width}px;max-height:${maxHeight}px;`;
    menu.showPopover();
    focusOption(choices.includes(val) ? val : choices[0]);
  }

  function choose(nodeId: number): void {
    onSelect(nodeId);
    if (menu?.matches(":popover-open")) menu.hidePopover();
    trigger?.focus();
  }

  function onTriggerKeydown(event: KeyboardEvent): void {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      show();
    }
  }

  function onMenuKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      menu.hidePopover();
      trigger.focus();
      return;
    }
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp" && event.key !== "Home" && event.key !== "End") return;
    event.preventDefault();
    const focused: number = Number((event.target as HTMLElement).dataset.nodeId);
    const curIdx: number = Math.max(0, choices.indexOf(focused));
    const nxtIdx: number = event.key === "Home" ? 0
      : event.key === "End" ? choices.length - 1
      : (curIdx + (event.key === "ArrowDown" ? 1 : -1) + choices.length) % choices.length;
    focusOption(choices[nxtIdx]);
  }
</script>

<button bind:this={trigger} id={id} type="button" class={`${SIM_FIELD} inline-flex shrink-0 items-center justify-between gap-2 whitespace-nowrap text-left`}
  style={val > 0 ? nodeBadgeStyle(val) : val === 0 && allValue !== 0 ? "background: var(--sender-soft); color: #16202c" : undefined}
  aria-label={lbl} aria-haspopup="listbox" aria-controls={`${id}-listbox`} aria-expanded={open}
  {disabled} onclick={show} onkeydown={onTriggerKeydown}>
  <span class="font-semibold">{optionLbl(val)}</span>
  <svg width="11" height="11" viewBox="0 0 12 12" aria-hidden="true" class="shrink-0">
    <path d="m2.5 4.5 3.5 3.5 3.5-3.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
  </svg>
</button>

<div bind:this={menu} id={`${id}-listbox`} popover="auto" role="listbox" tabindex="-1" aria-label={lbl}
  class="overflow-y-auto rounded-lg border border-[var(--field-border)] bg-[var(--surface-solid)] p-1 shadow-lg"
  ontoggle={(event) => { open = (event.currentTarget as HTMLElement).matches(":popover-open"); }}
  onkeydown={onMenuKeydown}>
  {#each choices as nodeId (nodeId)}
    <button type="button" role="option" aria-selected={val === nodeId} data-node-id={nodeId}
      class="flex min-h-[32px] w-full cursor-pointer items-center rounded-md px-1.5 text-left text-[11px] hover:bg-[var(--selected-bg)] focus-visible:outline-2 focus-visible:outline-[var(--primary)]"
      tabindex={val === nodeId ? 0 : -1} onclick={() => choose(nodeId)}>
      {#if nodeId === allValue}
        <span class="px-1.5 text-[var(--page-text)]">{optionLbl(nodeId)}</span>
      {:else}
        <span class="rounded-md border border-black/10 px-2 py-1 font-semibold" style={nodeId === 0 ? "background: var(--sender-soft); color: #16202c" : nodeBadgeStyle(nodeId)}>{optionLbl(nodeId)}</span>
      {/if}
      {#if val === nodeId}<span class="ms-auto px-1 text-[var(--page-text)]" aria-hidden="true">✓</span>{/if}
    </button>
  {/each}
</div>
