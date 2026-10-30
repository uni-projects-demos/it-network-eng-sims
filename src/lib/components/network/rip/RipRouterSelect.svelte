<script lang="ts">
  import { locale, translate } from "#lib/i18n/index.ts";
  import { SIM_FIELD } from "#lib/components/simulation/styles.ts";
  import { routerBadgeStyle } from "#lib/sims/network/rip/router.ts";

  export let id: string;
  export let lbl: string;
  export let val: number;
  export let routerIds: readonly number[];
  export let onSelect: (routerId: number) => void;
  export let isIncludeAll: boolean = false;
  export let disabled: boolean = false;

  let trigger: HTMLButtonElement;
  let menu: HTMLDivElement;
  let open: boolean = false;
  $: choices = isIncludeAll ? [0, ...routerIds] : [...routerIds];

  function optionLbl(routerId: number): string {
    return routerId === 0 ? translate("rip.allRouters", $locale) : `R${routerId}`;
  }

  function focusOption(routerId: number): void {
    menu?.querySelector<HTMLElement>(`[data-router-id="${routerId}"]`)?.focus();
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

  function choose(routerId: number): void {
    onSelect(routerId);
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
    const focused: number = Number((event.target as HTMLElement).dataset.routerId);
    const curIdx: number = Math.max(0, choices.indexOf(focused));
    const nxtIdx: number = event.key === "Home" ? 0
      : event.key === "End" ? choices.length - 1
      : (curIdx + (event.key === "ArrowDown" ? 1 : -1) + choices.length) % choices.length;
    focusOption(choices[nxtIdx]);
  }
</script>

<button bind:this={trigger} id={id} type="button" class={`${SIM_FIELD} inline-flex shrink-0 items-center justify-between gap-2 whitespace-nowrap text-left`}
  style={val > 0 ? routerBadgeStyle(val): undefined}
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
  {#each choices as routerId (routerId)}
    <button type="button" role="option" aria-selected={val === routerId} data-router-id={routerId}
      class="flex min-h-[32px] w-full cursor-pointer items-center rounded-md px-1.5 text-left text-[11px] hover:bg-[var(--selected-bg)] focus-visible:outline-2 focus-visible:outline-[var(--primary)]"
      tabindex={val === routerId ? 0 : -1} onclick={() => choose(routerId)}>
      {#if routerId === 0}
        <span class="px-1.5 text-[var(--page-text)]">{translate("rip.allRouters", $locale)}</span>
      {:else}
        <span class="rounded-md border border-black/10 px-2 py-1 font-semibold" style={routerBadgeStyle(routerId)}>{optionLbl(routerId)}</span>
      {/if}
      {#if val === routerId}<span class="ms-auto px-1 text-[var(--page-text)]" aria-hidden="true">✓</span>{/if}
    </button>
  {/each}
</div>
