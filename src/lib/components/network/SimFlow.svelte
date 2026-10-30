<script lang="ts">
  import { onMount } from "svelte";
  import { locale } from "#lib/i18n/index.ts";
  import FlowSettings from "./flow/FlowSettings.svelte";
  import FlowStats from "./flow/FlowStats.svelte";
  import FlowTopology from "./flow/FlowTopology.svelte";
  import FlowResults from "./flow/FlowResults.svelte";
  import FlowBenchmarkPanel from "./flow/FlowBenchmark.svelte";
  import FlowLP from "./flow/FlowLP.svelte";
  import { flowErr } from "#lib/sims/network/flow/i18n.ts";
  import { FlowController } from "#lib/sims/network/flow/controller.ts";
  import { defaultFlowConfig, validateFlowConfig, type FlowConfig, type FlowSnapshot, type FlowBenchmark } from "#lib/sims/network/flow/types.ts";
  import { genFlowLP } from "#lib/wasm/engine.ts";

  let cfg: FlowConfig = defaultFlowConfig();
  let snapshot: FlowSnapshot | null = null;
  let controller: FlowController;
  let isLoading: boolean = true;
  let isRunning: boolean = false;
  let cause: unknown = null;
  $: error = flowErr(cause, $locale);
  let computeMS: number = 0;
  let selectedSrc: number = 1;
  let selectedDest: number = 1;
  let benchmarks: FlowBenchmark[] = [];
  let isBenchmarking: boolean = false;
  let isDisposed: boolean = false;
  let revision: number = 0;
  let isMounted: boolean = false;
  let lpRevision: number = 0;
  let lpTxt: string = "";
  let lpUrl: string = "";
  $: if (isMounted) void prepLP(cfg);

  async function prepLP(options: FlowConfig): Promise<void> {
    const token: number = ++lpRevision;
    lpTxt = "";
    if (lpUrl) URL.revokeObjectURL(lpUrl);
    lpUrl = "";
    try {
      const generated = await genFlowLP(validateFlowConfig(options));
      if (!isDisposed && token === lpRevision) {
        lpTxt = generated;
        lpUrl = URL.createObjectURL(new Blob([generated], {type:"application/octet-stream"}));
      }
    } catch {}
  }

  function selectConfig(config: FlowConfig): void {
    selectedSrc = Math.min(selectedSrc, config.srcs);
    selectedDest = Math.min(selectedDest, config.dests);
  }

  async function run(): Promise<void> {
    const cur: number = ++revision;
    isLoading = true;
    isRunning = true;
    cause = null;
    try {
      const config = validateFlowConfig(cfg);
      selectConfig(config);
      await controller.solve(config);
    } catch (err: unknown) {
      if (!isDisposed && cur === revision) cause = err;
    } finally {
      if (!isDisposed && cur === revision) {isLoading = false;isRunning = false;}
    }
  }

  async function runBenchmarks(): Promise<void> {
    const cur: number = ++revision;
    isBenchmarking = true;
    isRunning = true;
    benchmarks = [];
    cause = null;
    try {
      for (let y = 3; y <= 7; y++) {
        cfg = {...defaultFlowConfig(), trans: y};
        selectConfig(cfg);
        const row = await controller.solve(cfg);
        if (isDisposed || cur !== revision || !row) return;
        benchmarks = [...benchmarks, row];
      }
    } catch (err: unknown) {
      if (!isDisposed && cur === revision) cause = err;
    } finally {
      if (!isDisposed && cur === revision) {
        isBenchmarking = false;
        isRunning = false;
        isLoading = false;
      }
    }
  }

  function cancelBenchmark(): void {
    ++revision;
    controller.cancel();
    isBenchmarking = false;
    isRunning = false;
    isLoading = false;
  }

  onMount(() => {
    isMounted = true;
    controller = new FlowController((next, ms) => {
      snapshot = next;
      computeMS = ms;
      isLoading = false;
    });
    void run();
    return () => {
      isDisposed = true;
      isMounted = false;
      ++lpRevision;
      ++revision;
      controller.cancel();
      if(lpUrl)URL.revokeObjectURL(lpUrl);
    };
  });
</script>

<div class="flow-layout grid h-full min-h-0 min-w-0 grid-cols-[minmax(230px,.78fr)_minmax(460px,1.8fr)_minmax(310px,1.05fr)] grid-rows-[minmax(0,1fr)] gap-3">
  <aside class="flow-sidebar flex min-h-0 min-w-0 flex-col gap-3 overflow-y-auto">
    <div class="flow-controls flex shrink-0 flex-col gap-3">
      <div class="flow-settings flex min-w-0 flex-col gap-3">
        <FlowSettings {cfg} isBusy={isLoading || isBenchmarking} {error}
          onChange={(changes)=>{cfg={...cfg,...changes};}}
          onRun={()=>{void run();}} onReset={()=>{cfg=defaultFlowConfig();void run();}} />
      </div>
      <div class="flow-statistics min-w-0"><FlowStats {snapshot} {computeMS} /></div>
    </div>
    <div class="flow-lp flex min-h-0 min-w-0 flex-1 flex-col">
      <FlowLP {cfg} {lpTxt} {lpUrl} isBusy={isLoading || isBenchmarking} />
    </div>
  </aside>
  <div class="flow-topology min-h-0 min-w-0">
    <FlowTopology {snapshot} {cfg} isRunning={isRunning || isBenchmarking} bind:selectedSrc bind:selectedDest />
  </div>
  <aside class="flow-results-column flex min-h-0 min-w-0 flex-col gap-3">
    <div class="flow-solution flex min-h-0 min-w-0 flex-1 flex-col">
      <FlowResults {snapshot} bind:selectedSrc bind:selectedDest />
    </div>
    <div class="flow-benchmark min-h-0 min-w-0 shrink-0">
      <FlowBenchmarkPanel rows={benchmarks} isRunning={isBenchmarking} {isLoading}
        onRun={()=>{void runBenchmarks();}} onCancel={cancelBenchmark} />
    </div>
  </aside>
</div>

<style>
  @media (max-width:1240px) {
    .flow-layout { height:auto; grid-template-columns:minmax(250px,.8fr) minmax(0,1.6fr); grid-template-rows:auto minmax(0,1fr) 390px; grid-template-areas:"controls topology" "benchmark topology" "solution lp"; }
    .flow-sidebar, .flow-results-column { display:contents; }
    .flow-controls { grid-area:controls; }
    .flow-topology { grid-area:topology; }
    .flow-benchmark { grid-area:benchmark; }
    .flow-solution { grid-area:solution; }
    .flow-lp { grid-area:lp; }
  }
  @media (max-width:800px) {
    .flow-layout { grid-template-columns:minmax(0,1fr); grid-template-rows:auto auto auto 390px 390px auto; grid-template-areas:"settings" "topology" "statistics" "lp" "solution" "benchmark"; }
    .flow-controls { display:contents; }
    .flow-settings { grid-area:settings; }
    .flow-statistics { grid-area:statistics; }
  }
</style>
