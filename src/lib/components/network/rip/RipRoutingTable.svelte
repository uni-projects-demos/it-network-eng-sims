<script lang="ts">
  import { locale, translate, type TranslationParams } from "#lib/i18n/index.ts";
  import { SIM_PANEL, SIM_HEADING, SIM_CONTROL_BTN, SIM_MUTED } from "#lib/components/simulation/styles.ts";
  import RouterSelect from "./RipRouterSelect.svelte";
  import { routerBadgeStyle, routeStatus } from "#lib/sims/network/router.ts";
  import type { DisplayRoute } from "#lib/sims/network/router.ts";
  import type { RipNode, RipSnapshot } from "#lib/sims/network/types.ts";
  $: t = (key: string, params: TranslationParams = {}): string => translate(`rip.${key}`, $locale, params);
  export let snapshot: RipSnapshot | null;
  export let curRouter: RipNode | undefined;
  export let displayRoutes: DisplayRoute[];
  export let selectedRouter: number;
  export let selectedDest: number;
  export let onRestart: (router: number) => void;
  export let onSelectDest: (dest: number) => void;
</script>

<section class={`${SIM_PANEL} flex min-h-[200px] min-w-0 flex-[0.85] flex-col overflow-hidden p-[11px] max-[1240px]:h-[380px] max-[1240px]:min-h-0 max-[1240px]:flex-1 max-[760px]:order-4 max-[760px]:h-[300px] max-[760px]:flex-none`}>
      <div class="mb-2 flex flex-wrap items-center justify-between gap-2">
        <h2 class={SIM_HEADING}>{t("routingTable")}</h2>
        <div class="flex items-center gap-2">
          <label for="rip-table-router" class="text-[10px] font-[650] text-[var(--control-text)]">{t("router")}</label>
          <RouterSelect id="rip-table-router" lbl={t("routingTableRouter")} val={selectedRouter}
            routerIds={snapshot?.nodes.map(node => node.id) ?? []}
            onSelect={(routerId) => { selectedRouter = routerId; }} />
          {#if curRouter && !curRouter.is_running && !snapshot?.is_complete}
            <button class={SIM_CONTROL_BTN} onclick={() => onRestart(selectedRouter)}>{t("restart")}</button>
          {/if}
        </div>
      </div>
      <div class={`mb-2 text-[10px] ${SIM_MUTED}`}>{!curRouter?.is_running ? curRouter?.auto_restart_in != null ? t("temporaryInactive", {seconds: curRouter.auto_restart_in.toFixed(1)}): t("terminated"): curRouter.is_quiescent ? t("neighborsAllGarbage"): t("activeNeighbors", {active: curRouter.active_nbrs, garbage: curRouter.garbage_nbrs})}</div>
      <div class="min-h-0 flex-1 overflow-auto rounded-md border border-[var(--divider)]">
        <table class="w-full min-w-max table-auto border-collapse text-left text-[11px] whitespace-nowrap">
          <thead class="sticky top-0 bg-[var(--surface-2)]">
            <tr><th class="px-3 py-1.5">{t("destination")}</th><th class="px-3 py-1.5">{t("nextHop")}</th><th class="px-3 py-1.5">{t("metric")}</th><th class="px-3 py-1.5">{t("status")}</th></tr>
          </thead>
          <tbody>
            {#each displayRoutes as r (r.dest)}
              {@const status = r.isOffline ? 'Offline' : routeStatus(r)}
              <tr class:font-bold={r.dest === selectedDest} class="group relative border-t border-[var(--divider)] hover:bg-[var(--selected-bg)]" style:opacity={r.isOffline ? 0.57 : 1}>
                <td class="px-3 py-1.5">
                  <button type="button"
                    class="absolute inset-0 z-10 cursor-pointer border-0 bg-transparent p-0 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-[var(--primary)]"
                    aria-pressed={r.dest === selectedDest}
                    aria-label={t("viewRoute", {from: selectedRouter, to: r.dest}) + (r.isOffline ? ` (${t("offline")})` : "")}
                    title={t("viewRouteInTopology", {from: selectedRouter, to: r.dest})}
                    onclick={() => onSelectDest(r.dest)}>
                    <span class="sr-only">{t("selectRoute", {to: r.dest})}</span>
                  </button>
                  <span class="inline-block rounded-md border border-black/10 px-1.5 py-0.5 font-semibold whitespace-nowrap"
                    style={routerBadgeStyle(r.dest)} class:line-through={r.isOffline}>R{r.dest}</span>
                </td>
                <td class="px-3 py-1.5">
                  {#if r.isOffline || r.metric >= 16}
                    <span class:line-through={r.isOffline}>—</span>
                  {:else}
                    <span class="inline-block rounded-md border border-black/10 px-1.5 py-0.5 font-semibold" style={routerBadgeStyle(r.nxt_hop)}>R{r.nxt_hop}</span>
                  {/if}
                </td>
                <td class:line-through={r.isOffline} class="px-3 py-1.5">{r.metric === 16 ? '∞ (16)' : r.metric}</td>
                <td class="px-3 py-1.5" class:line-through={r.isOffline}>
                  <span class="inline-flex items-center gap-1" title={t(status === "Offline" ? "offline" : status === "Unreachable (invalid)" ? "unreachableInvalid" : status === "Reachable (direct)" ? "reachableDirect" : "reachableIndirect")}>
                    <span class={`size-1.5 shrink-0 rounded-full ${status === 'Unreachable (invalid)' ? 'bg-[var(--danger)]' : status === 'Offline' ? 'bg-[var(--muted-text)]' : 'bg-[var(--receiver)]'}`}></span>
                    {t(status === "Offline" ? "offline" : status === "Unreachable (invalid)" ? "unreachableInvalid" : status === "Reachable (direct)" ? "reachableDirect" : "reachableIndirect")}
                  </span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
