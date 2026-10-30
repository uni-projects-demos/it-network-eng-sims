<script lang="ts">
  import { locale, translate, type TranslationParams } from "#lib/i18n/index.ts";
  import { ripEventText } from "#lib/sims/network/rip/i18n.ts";
  import { tick } from "svelte";
  import RouterSelect from "./RipRouterSelect.svelte";
  import ConsoleTabs from "#lib/components/simulation/ConsoleTabs.svelte";
  import { SIM_PANEL, SIM_HEADING, SIM_MUTED } from "#lib/components/simulation/styles.ts";
  import { formatDur } from "#lib/sims/network/rip/layout.ts";
  import { routerCol } from "#lib/sims/network/rip/router.ts";
  import { CONSOLE_FILTERS, eventTone, visibleConsoleEvents } from "#lib/sims/network/rip/console.ts";
  import type { ConsoleEntry, ConsoleFilter } from "#lib/sims/network/rip/console.ts";
  import type { RipNode, RipSnapshot } from "#lib/sims/network/rip/types.ts";

  $: t = (key: string, params: TranslationParams = {}): string => translate(`rip.${key}`, $locale, params);
  export let entries: ConsoleEntry[];
  export let snapshot: RipSnapshot | null;

  let consoleFilter: ConsoleFilter = "all";
  let consoleRouter: number = 0;
  let consoleEl: HTMLDivElement | undefined;
  let lastEventId: number = -1;
  let filtered: ConsoleEntry[] = [];
  $: consoleTabs = CONSOLE_FILTERS.map((filter: ConsoleFilter): { val: ConsoleFilter; lbl: string } => ({
    val: filter,
    lbl: t(filter === "all" ? "all" : filter === "routing" ? "routes" : filter === "msgs" ? "packets" : "faults"),
  }));

  $: if (consoleRouter !== 0 && !snapshot?.nodes.some((node: RipNode): boolean => node.id === consoleRouter)) {
    consoleRouter = 0;
  }
  $: filtered = visibleConsoleEvents(entries, snapshot?.time ?? 0, consoleFilter,
    consoleRouter === 0 ? null : consoleRouter);
  $: if ((entries.at(-1)?.id ?? -1) !== lastEventId) {
    lastEventId = entries.at(-1)?.id ?? -1;
    void tick().then((): void => { if (consoleEl) consoleEl.scrollTop = consoleEl.scrollHeight; });
  }
</script>

    <section class={`${SIM_PANEL} flex min-h-[260px] min-w-0 flex-[1.15] flex-col overflow-hidden p-[11px] max-[1240px]:h-[380px] max-[1240px]:min-h-0 max-[1240px]:flex-1 max-[760px]:order-5 max-[760px]:h-[540px] max-[760px]:flex-none`}>
      <div class="mb-[7px] flex min-w-0 items-center gap-2">
        <h2 class={`${SIM_HEADING} shrink-0`}>{t("console")}</h2>
        <div class="ms-auto min-w-0 max-w-full overflow-x-auto">
          <div class="ms-auto flex w-max items-center gap-2">
            <RouterSelect id="rip-console-router" lbl={t("routerConsole")} val={consoleRouter}
              routerIds={snapshot?.nodes.map(node => node.id) ?? []} isIncludeAll
              onSelect={(routerId) => { consoleRouter = routerId; }} />
            <ConsoleTabs tabs={consoleTabs} selected={consoleFilter} lbl={t("consoleCategories")}
              onSelect={(val) => { consoleFilter = val; }}/>
          </div>
        </div>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto rounded-lg border border-[var(--divider)] bg-[var(--console-bg)] p-1 font-mono text-[10px] leading-5" bind:this={consoleEl} role="log" aria-live="off">
        {#each filtered as event (event.id)}
          {@const col = routerCol(event.router)}
          <div class="flex items-start gap-2 px-1.5 py-[1px]">
            <time class="w-11 shrink-0 text-[var(--muted-text)]">{formatDur(event.time)}</time>
            <span class="mt-[7px] size-1.5 shrink-0 rounded-full" style={`background:var(--${eventTone(event._type)})`}></span>
            <span class:router-console-text={col !== undefined}
              style={col ? `--rip-console-light:${col.light};--rip-console-dark:${col.dark}` : undefined}>{ripEventText(event, $locale)}</span>
          </div>
        {/each}
        {#if !filtered.length}<span class={SIM_MUTED}>{t("noEvents")}</span>{/if}
      </div>
      <div class={`mt-1 border-t border-[var(--divider)] py-1.5 text-[10px] ${SIM_MUTED}`}>{t("consoleLatest", {count: filtered.length, limit: "1,200"})}</div>
    </section>

<style>
  .router-console-text { color: var(--rip-console-dark); }
  :global(html[data-theme="light"]) .router-console-text { color: var(--rip-console-light); }
</style>
