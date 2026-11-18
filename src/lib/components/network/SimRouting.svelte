<script lang="ts">
  import { onMount } from "svelte";
  import RoutingSettings from "./routing/RoutingSettings.svelte";
  import RoutingStatistics from "./routing/RoutingStats.svelte";
  import RoutingTopology from "./routing/RoutingTopology.svelte";
  import RoutingTable from "./routing/RoutingTable.svelte";
  import RoutingConsole from "./routing/RoutingConsole.svelte";
  import { randSeed } from "#lib/sims/shared/random.ts";
  import { RoutingController } from "#lib/sims/network/routing/controller.ts";
  import { RipController } from "#lib/sims/network/rip/controller.ts";
  import { OspfController } from "#lib/sims/network/ospf/controller.ts";
  import { appendConsoleEvents, latestConsoleEvent } from "#lib/sims/network/routing/console.ts";
  import { routingTableRows } from "#lib/sims/network/routing/router.ts";
  import { selectedRoute } from "#lib/sims/network/routing/layout.ts";
  import { ROUTING_DEF_TO_SECS } from "#lib/sims/network/routing/types.ts";
  import type { RoutingConfig, RoutingNode, RoutingSnapshot } from "#lib/sims/network/routing/types.ts";
  import type { ConsoleEntry } from "#lib/sims/network/routing/console.ts";
  import type { DisplayRoute } from "#lib/sims/network/routing/router.ts";
  import type { PlaybackDir } from "#lib/sims/shared/playback.ts";

  import type { RoutingProtocol } from "#lib/sims/network/routing/types.ts";
  export let protocol: RoutingProtocol;

  type RoutingActions = Pick<RoutingController<RoutingSnapshot>,
    "load" | "pause" | "play" | "setSpeed" | "move" | "terminate" | "restart" |
    "setLinkCost" | "setPacketLoss" | "setPacketInvalid" | "dispose">;

  const defaultConfig: () => RoutingConfig = (): RoutingConfig => ({
    nodes: 7, seed: 1, topology: "default", to_secs: ROUTING_DEF_TO_SECS,
    rand_fail_perc: 0, pck_loss_perc: 0, pck_invalid_perc: 0,
  });

  let cfg: RoutingConfig = defaultConfig();
  let activeTopology: RoutingConfig["topology"] = "default";
  let controller: RoutingActions;
  let topologyComponent: { focus: () => void } | undefined;
  let snapshot: RoutingSnapshot | null = null;
  let isPlaying: boolean = false;
  let speed: number = 1;
  let isLoading: boolean = false;
  let err: string = "";
  let entries: ConsoleEntry[] = [];
  let nxtConsoleEventId: number = 0;
  let consoleSession: number = 0;
  let selectedRouter: number = 1;
  let selectedDest: number = 3;
  let curRouter: RoutingNode | undefined;
  let displayRoutes: DisplayRoute[] = [];
  let availableDests: number[] = [];
  let highlightPath: number[] = [];
  let focusEvent: ConsoleEntry | undefined;

  $: curRouter = snapshot?.nodes.find((node: RoutingNode): boolean => node.id === selectedRouter);
  $: displayRoutes = routingTableRows(curRouter, protocol);
  $: availableDests = displayRoutes.map((route: DisplayRoute): number => route.dest);
  $: if (!availableDests.includes(selectedDest)) {
    selectedDest = availableDests[0] ?? 0;
  }
  $: highlightPath = snapshot ? selectedRoute(snapshot.nodes, selectedRouter, selectedDest, protocol): [];
  $: focusEvent = latestConsoleEvent(entries, snapshot?.time ?? 0);

  function consume(nxt: RoutingSnapshot): void {
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

  function updateConfig(changes: Partial<RoutingConfig>): void {
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
    const callback = (
      onSnapshot: (snapshot: RoutingSnapshot) => void,
      onError: (message: string) => void,
      onBranch: (time: number) => void,
      onPlaying: (isPlaying: boolean) => void,
    ): RoutingActions => protocol === "rip"
      ? new RipController(onSnapshot, onError, onBranch, onPlaying)
      : new OspfController(onSnapshot, onError, onBranch, onPlaying);
    controller = callback(
      consume,
      (msg: string): void => { err = msg; },
      (time: number): void => { entries = entries.filter((event: ConsoleEntry): boolean => event.time <= time + 0.000001); },
      (nxt: boolean): void => { isPlaying = nxt; },
    );
    void reset(true);
    return (): void => controller.dispose();
  });
</script>

<div class="routing-layout grid h-full min-h-0 min-w-0 grid-cols-[minmax(225px,.76fr)_minmax(480px,1.85fr)_minmax(275px,1fr)] grid-rows-[minmax(0,1fr)] gap-3 max-[1240px]:h-auto max-[1240px]:grid-cols-[minmax(260px,.8fr)_minmax(0,1.65fr)] max-[1240px]:grid-rows-none max-[760px]:grid-cols-1">
  <aside class="routing-layout__settings flex h-full min-h-0 min-w-0 flex-col gap-3 max-[1240px]:h-auto max-[760px]:contents">
    <RoutingSettings {protocol} {cfg} {isLoading} {err}
      onConfigChange={updateConfig}
      onPacketLoss={(percent) => changePacketSetting("loss", percent)}
      onPacketInvalid={(percent) => changePacketSetting("invalid", percent)}
      onReset={() => { cfg = defaultConfig(); void reset(false); }}
      onGenerate={() => { cfg = { ...cfg, topology: "rand" }; void reset(true); }}/>
    <RoutingStatistics {protocol} {snapshot}/>
  </aside>

  <RoutingTopology {protocol} bind:this={topologyComponent} {snapshot} cfgTimeout={cfg.to_secs}
    {activeTopology} bind:selectedRouter bind:selectedDest
    {availableDests} {highlightPath} {focusEvent} {isPlaying} {speed} {isLoading}
    onMove={move} onToggle={toggle} onSpeed={changeSpeed}
    onTerminate={(id) => controller.terminate(id)} onAdjustLink={adjustLinkCost}/>

  <aside class="flex h-full min-h-0 min-w-0 flex-col gap-3 max-[1240px]:col-span-2 max-[1240px]:h-auto max-[1240px]:flex-row max-[760px]:contents">
    <RoutingTable {protocol} {snapshot} {curRouter} {displayRoutes} bind:selectedRouter
      {selectedDest}
      onRestart={(id) => controller.restart(id)} onSelectDest={selectDest}/>
    {#key consoleSession}
      <RoutingConsole {protocol} {entries} {snapshot}/>
    {/key}
  </aside>
</div>
