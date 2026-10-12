<script lang="ts">
  import { onMount } from "svelte";
  import RipSettings from "./rip/RipSettings.svelte";
  import RipStatistics from "./rip/RipStats.svelte";
  import RipTopology from "./rip/RipTopology.svelte";
  import RipRoutingTable from "./rip/RipRoutingTable.svelte";
  import RipConsole from "./rip/RipConsole.svelte";
  import { randSeed } from "#lib/sims/shared/random.ts";
  import { RipController } from "#lib/sims/network/controller.ts";
  import { appendConsoleEvents, latestConsoleEvent } from "#lib/sims/network/console.ts";
  import { routingTableRows } from "#lib/sims/network/router.ts";
  import { selectedRoute } from "#lib/sims/network/layout.ts";
  import { RIP_DEF_TO_SECS } from "#lib/sims/network/types.ts";
  import type { RipConfig, RipNode, RipSnapshot } from "#lib/sims/network/types.ts";
  import type { ConsoleEntry } from "#lib/sims/network/console.ts";
  import type { DisplayRoute } from "#lib/sims/network/router.ts";
  import type { PlaybackDir } from "#lib/sims/shared/playback.ts";

  const defaultConfig: () => RipConfig = (): RipConfig => ({
    nodes: 7, seed: 1, topology: "default", to_secs: RIP_DEF_TO_SECS,
    rand_fail_perc: 0, pck_loss_perc: 0, pck_invalid_perc: 0,
  });

  let cfg: RipConfig = defaultConfig();
  let activeTopology: RipConfig["topology"] = "default";
  let controller: RipController;
  let topologyComponent: { focus: () => void } | undefined;
  let snapshot: RipSnapshot | null = null;
  let isPlaying: boolean = false;
  let speed: number = 1;
  let isLoading: boolean = false;
  let err: string = "";
  let entries: ConsoleEntry[] = [];
  let nxtConsoleEventId: number = 0;
  let consoleSession: number = 0;
  let selectedRouter: number = 1;
  let selectedDest: number = 3;
  let curRouter: RipNode | undefined;
  let displayRoutes: DisplayRoute[] = [];
  let availableDests: number[] = [];
  let highlightPath: number[] = [];
  let focusEvent: ConsoleEntry | undefined;

  $: curRouter = snapshot?.nodes.find((node: RipNode): boolean => node.id === selectedRouter);
  $: displayRoutes = routingTableRows(curRouter);
  $: availableDests = displayRoutes.map((route: DisplayRoute): number => route.dest);
  $: if (!availableDests.includes(selectedDest)) {
    selectedDest = availableDests[0] ?? 0;
  }
  $: highlightPath = snapshot ? selectedRoute(snapshot.nodes, selectedRouter, selectedDest): [];
  $: focusEvent = latestConsoleEvent(entries, snapshot?.time ?? 0);

  function consume(nxt: RipSnapshot): void {
    snapshot = nxt;
    if (nxt.events.length > 0) {
      entries = appendConsoleEvents(entries, nxt.events, nxtConsoleEventId);
      nxtConsoleEventId += nxt.events.length;
    }
  }

  async function reset(isNewTopology: boolean): Promise<void> {
    isLoading = true;
    err = "";
    entries = [];
    nxtConsoleEventId = 0;
    ++consoleSession;
    snapshot = null;
    cfg = {
      ...cfg,
      nodes: cfg.topology === "default" ? 7 : Math.max(3, Math.min(15, Math.round(cfg.nodes))),
      seed: isNewTopology ? randSeed(): cfg.seed,
    };
    activeTopology = cfg.topology;
    selectedRouter = 1;
    selectedDest = Math.min(3, cfg.nodes);
    try {
      await controller.load({ ...cfg });
    } catch (cause: unknown) {
      err = String(cause);
    } finally {
      isLoading = false;
    }
  }

  function updateConfig(changes: Partial<RipConfig>): void {
    cfg = { ...cfg, ...changes };
  }
  function changePacketSetting(_type: "loss" | "invalid", percent: number): void {
    cfg = { ...cfg, [_type === "loss" ? "pck_loss_perc" : "pck_invalid_perc"]: percent };
    if (!isLoading) {
      if (_type === "loss") controller.setPacketLoss(percent);
      else controller.setPacketInvalid(percent);
    }
  }
  function selectDest(dest: number): void {
    selectedDest = dest;
    if (typeof window !== "undefined" && window.matchMedia("(max-width: 760px)").matches) {
      topologyComponent?.focus();
    }
  }
  function adjustLinkCost(a: number, b: number, cur: number, delta: -1 | 1): void {
    const nxt: number = Math.max(1, Math.min(15, cur + delta));
    if (nxt !== cur) controller.setLinkCost(a, b, nxt);
  }
  function move(dir: PlaybackDir): void { if (!isLoading) controller.move(dir); }
  function toggle(): void { isPlaying ? controller.pause(): controller.play(speed); }
  function changeSpeed(val: number): void { speed = val; controller.setSpeed(val); }

  onMount((): () => void => {
    controller = new RipController(
      consume,
      (msg: string): void => { err = msg; },
      (time: number): void => { entries = entries.filter((event: ConsoleEntry): boolean => event.time <= time + 0.000001); },
      (nxt: boolean): void => { isPlaying = nxt; },
    );
    void reset(true);
    return (): void => controller.dispose();
  });
</script>

<div class="grid h-full min-h-0 min-w-0 grid-cols-[minmax(225px,.76fr)_minmax(480px,1.85fr)_minmax(275px,1fr)] grid-rows-[minmax(0,1fr)] gap-3 max-[1240px]:h-auto max-[1240px]:grid-cols-[minmax(260px,.8fr)_minmax(0,1.65fr)] max-[1240px]:grid-rows-none max-[760px]:grid-cols-1">
  <aside class="flex h-full min-h-0 min-w-0 flex-col gap-3 max-[1240px]:h-auto max-[760px]:contents">
    <RipSettings {cfg} {isLoading} {err}
      onConfigChange={updateConfig}
      onPacketLoss={(percent) => changePacketSetting("loss", percent)}
      onPacketInvalid={(percent) => changePacketSetting("invalid", percent)}
      onReset={() => { cfg = defaultConfig(); void reset(false); }}
      onGenerate={() => { cfg = { ...cfg, topology: "rand" }; void reset(true); }}/>
    <RipStatistics {snapshot}/>
  </aside>

  <RipTopology bind:this={topologyComponent} {snapshot} cfgTimeout={cfg.to_secs}
    {activeTopology} bind:selectedRouter bind:selectedDest
    {availableDests} {highlightPath} {focusEvent} {isPlaying} {speed} {isLoading}
    onMove={move} onToggle={toggle} onSpeed={changeSpeed}
    onTerminate={(id) => controller.terminate(id)} onAdjustLink={adjustLinkCost}/>

  <aside class="flex h-full min-h-0 min-w-0 flex-col gap-3 max-[1240px]:col-span-2 max-[1240px]:h-auto max-[1240px]:flex-row max-[760px]:contents">
    <RipRoutingTable {snapshot} {curRouter} {displayRoutes} bind:selectedRouter
      {selectedDest}
      onRestart={(id) => controller.restart(id)} onSelectDest={selectDest}/>
    {#key consoleSession}
      <RipConsole {entries} {snapshot}/>
    {/key}
  </aside>
</div>
