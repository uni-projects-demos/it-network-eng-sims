<script lang="ts">
  import { onMount } from "svelte";
  import { locale } from "#lib/i18n/index.ts";
  import { flowTranslator } from "#lib/sims/network/flow/i18n.ts";
  import { SIM_PANEL, SIM_HEADING } from "#lib/components/simulation/styles.ts";
  import ConsoleTabs from "#lib/components/simulation/ConsoleTabs.svelte";
  import { type FlowSnapshot } from "#lib/sims/network/flow/types.ts";
  import { paginateFlowRows } from "#lib/sims/network/flow/pagination.ts";
  import { flowNumber } from "#lib/sims/network/flow/format.ts";
  export let snapshot: FlowSnapshot | null;
  export let selectedSrc: number;
  export let selectedDest: number;
  const tabIds = ["loads", "routes", "links"] as const;
  let selected: (typeof tabIds)[number] = "loads";
  let isNonZeroOnly: boolean = false;
  let viewport: HTMLDivElement;
  let tableHead: HTMLTableSectionElement;
  let viewportHeight: number = 320;
  let headerHeight: number = 34;
  let page: number = 0;
  let lastView: string = "";
  $: t = flowTranslator($locale);
  $: number = (val: number | null | undefined): string => flowNumber(val, $locale);
  $: tabs = tabIds.map(val => ({val,lbl:t(val)}));
  $: filteredLinks = (snapshot?.links ?? []).filter(link => !isNonZeroOnly || link.cap > 0);
  $: length = selected === "routes" ? snapshot?.routes.length ?? 0 : selected === "links" ? filteredLinks.length : snapshot?.loads.length ?? 0;
  $: range = paginateFlowRows(length, viewportHeight, page, headerHeight);
  $: pageLabel = (target: number): string => `${t("results")} · ${number(target + 1)} / ${number(range.pages)}`;
  $: highestLinks = snapshot?.highest_links.map(link=>`${link.from}→${link.to}`).join(", ") ?? "";
  $: view = `${selected}:${isNonZeroOnly}:${JSON.stringify(snapshot?.config)}`;
  $: if (view !== lastView) {
    lastView = view;
    page = 0;
  }

  onMount(() => {
    const resize: () => void = (): void => {
      const height: number = viewport.clientHeight;
      const head: number = tableHead.getBoundingClientRect().height;
      page = Math.floor(range.start / paginateFlowRows(length, height, page, head).size);
      viewportHeight = height;
      headerHeight = head;
    };
    const obs: ResizeObserver = new ResizeObserver(resize);
    obs.observe(viewport);
    obs.observe(tableHead);
    resize();
    return () => obs.disconnect();
  });
</script>

<section aria-labelledby="flow-results-heading" class={`${SIM_PANEL} flex min-h-0 min-w-0 flex-1 flex-col overflow-clip max-[1240px]:h-[390px] max-[1240px]:flex-none`}>
  <div class="flex min-w-0 shrink-0 items-center justify-between gap-2 border-b border-[var(--divider)] px-3 py-2.5">
    <h2 id="flow-results-heading" class={`${SIM_HEADING} shrink-0`}>{t("results")}</h2>
    <div class="flow-result-tabs min-w-0">
      <ConsoleTabs {tabs} {selected} onSelect={(val) => {selected=val;}} lbl={t("resultTabs")} />
    </div>
  </div>
  {#if selected === "links"}
    <label class="m-2 flex shrink-0 items-center gap-2 text-[10px] text-[var(--muted-text)]"><input type="checkbox" bind:checked={isNonZeroOnly} />{t("nonzeroOnly")}</label>
  {/if}
  <div bind:this={viewport} class="flow-results-body min-h-0 flex-1 overflow-clip">
    <table id="flow-results-table" class="flow-table" aria-label={t("results")} aria-rowcount={length+1}>
      <colgroup>
        {#if selected === "loads"}<col style="width:20%" /><col style="width:28%" /><col style="width:52%" />
        {:else if selected === "routes"}<col style="width:36%" /><col style="width:42%" /><col style="width:22%" />
        {:else}<col style="width:65%" /><col style="width:35%" />{/if}
      </colgroup>
      <thead bind:this={tableHead}>
        <tr aria-rowindex="1">
          {#if selected === "loads"}<th>{t("transit")}</th><th>{t("load")}</th><th>{t("relativeLoad")}</th>
          {:else if selected === "routes"}<th>{t("demand")}</th><th>{t("threePathsShort")}</th><th>{t("share")}</th>
          {:else}<th>{t("link")}</th><th>{t("capacity")}</th>{/if}
        </tr>
      </thead>
      <tbody>
        {#if selected === "loads"}
          {#each (snapshot?.loads ?? []).slice(range.start,range.end) as load, k}
            <tr aria-rowindex={range.start + k + 2}><td><bdi>T{range.start + k + 1}</bdi></td><td class="font-bold tabular-nums">{number(load)}</td><td><div class="h-1.5 rounded bg-[var(--control-bg)]"><div class="h-1.5 rounded bg-[var(--channel)]" style={`width:${100*load/Math.max(1,snapshot?.obj ?? 1)}%`}></div></div></td></tr>
          {/each}
        {:else if selected === "routes"}
          {#each (snapshot?.routes ?? []).slice(range.start,range.end) as route, index}
            <tr aria-rowindex={range.start + index + 2} class={route.src === selectedSrc && route.dest === selectedDest ? "bg-[var(--selected-bg)]" : ""}>
              <td><button class="text-[var(--primary)] underline underline-offset-2" onclick={() => {selectedSrc=route.src; selectedDest=route.dest;}} title={t("inspectDemand")}><bdi dir="ltr">S{route.src}→D{route.dest}</bdi></button><span class="ms-1 text-[var(--muted-text)]">({number(route.demand)})</span></td>
              <td class="font-mono"><bdi dir="ltr">{route.trans.map(k => `T${k}`).join(" · ")}</bdi></td><td class="tabular-nums">{number(route.flow_per_path)}</td>
            </tr>
          {/each}
        {:else}
          {#each filteredLinks.slice(range.start,range.end) as link, index}<tr aria-rowindex={range.start + index + 2}><td class="font-mono"><bdi dir="ltr">{link.from}→{link.to}</bdi></td><td class="tabular-nums">{number(link.cap)}</td></tr>{/each}
        {/if}
      </tbody>
    </table>
    {#if !length}<p class="m-3 text-[11px] text-[var(--muted-text)]">{t("noRoutes")}</p>{/if}
  </div>
  {#if selected === "loads" && highestLinks}<p class="mx-3 my-2 shrink-0 truncate text-[10px] text-[var(--muted-text)]" title={`${t("highestLinks")}: ${highestLinks}`}>{t("highestLinks")}: <bdi dir="ltr" class="font-mono text-[var(--page-text)]">{highestLinks}</bdi></p>{/if}
  <div class="flex shrink-0 items-center justify-center border-t border-[var(--divider)] px-3 py-2 text-[10px] text-[var(--muted-text)]">
    <nav class="flex items-center gap-1" aria-label={t("results")}>
      <button class="flow-page-button" aria-controls="flow-results-table" aria-label={pageLabel(0)} title={pageLabel(0)} disabled={!range.page} onclick={() => {page=0;}}><span aria-hidden="true">«</span></button>
      <button class="flow-page-button" aria-controls="flow-results-table" aria-label={pageLabel(Math.max(0, range.page - 1))} title={pageLabel(Math.max(0, range.page - 1))} disabled={!range.page} onclick={() => {page=range.page - 1;}}><span aria-hidden="true">‹</span></button>
      <span class="min-w-10 text-center tabular-nums" aria-live="polite"><bdi dir="ltr">{number(range.page + 1)} / {number(range.pages)}</bdi></span>
      <button class="flow-page-button" aria-controls="flow-results-table" aria-label={pageLabel(Math.min(range.pages - 1,range.page + 1))} title={pageLabel(Math.min(range.pages - 1,range.page + 1))} disabled={range.page===range.pages - 1} onclick={() => {page=range.page + 1;}}><span aria-hidden="true">›</span></button>
      <button class="flow-page-button" aria-controls="flow-results-table" aria-label={pageLabel(range.pages - 1)} title={pageLabel(range.pages - 1)} disabled={range.page===range.pages - 1} onclick={() => {page=range.pages - 1;}}><span aria-hidden="true">»</span></button>
    </nav>
  </div>
</section>

<style>
  .flow-result-tabs :global([role="tablist"]) { width:auto; flex-wrap:wrap; justify-content:flex-end; }
  .flow-result-tabs :global(button) { white-space:normal; overflow-wrap:anywhere; }
  .flow-table { width:100%; table-layout:fixed; border-collapse:collapse; text-align:start; font-size:10px; line-height:14px; }
  .flow-table th, .flow-table td { box-sizing:border-box; height:34px; padding:9px 8px; border-bottom:1px solid var(--divider); text-align:start; }
  .flow-table th { background:var(--surface-2); color:var(--muted-text); font-weight:600; overflow-wrap:anywhere; }
  .flow-table td { white-space:nowrap; }
  .flow-page-button { display:grid; width:26px; height:26px; place-items:center; border:1px solid var(--control-border); border-radius:6px; background:var(--control-bg); color:var(--control-text); font-size:16px; }
  .flow-page-button:disabled { opacity:0.4; }
  .flow-page-button:focus-visible { outline:2px solid var(--accent-border); outline-offset:2px; }
  :global([dir="rtl"]) .flow-page-button { transform:scaleX(-1); }
</style>
