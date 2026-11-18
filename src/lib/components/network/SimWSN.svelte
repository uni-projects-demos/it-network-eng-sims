<script lang="ts">
  import { onMount } from "svelte";
  import WsnSettings from "./wsn/WsnSettings.svelte";
  import WsnStats from "./wsn/WsnStats.svelte";
  import WsnTopology from "./wsn/WsnTopology.svelte";
  import WsnNodes from "./wsn/WsnNodes.svelte";
  import WsnConsole from "./wsn/WsnConsole.svelte";
  import { randSeed } from "#lib/sims/shared/random.ts";
  import { WsnController } from "#lib/sims/network/wsn/controller.ts";
  import { defaultWsnConfig, type WsnConfig, type WsnMac, type WsnSnapshot } from "#lib/sims/network/wsn/types.ts";
  import { locale } from "#lib/i18n/index.ts";
  import { wsnText } from "#lib/sims/network/wsn/i18n.ts";
  import type { WsnEvent } from "#lib/sims/network/wsn/types.ts";
  import type { PlaybackDir } from "#lib/sims/shared/playback.ts";

  type ConsoleEntry = WsnEvent & { id: number };
  let cfg: WsnConfig = defaultWsnConfig(1);
  let controller: WsnController;
  let snapshot: WsnSnapshot | null = null;
  let entries: ConsoleEntry[] = [];
  let nextEventId: number = 0;
  let session: number = 0;
  let isPlaying: boolean = false;
  let isLoading: boolean = true;
  let speed: number = 1;
  let error: string = '';
  let selectedNode: number = 1;
  let t: (key: string) => string;
  $: t = (key: string): string => wsnText(key, $locale);

  function consume(next: WsnSnapshot): void {
    snapshot = next;
    if (next.events.length > 0) {
      entries = [...entries, ...next.events.map((event: WsnEvent, index: number): ConsoleEntry => ({
        ...event, id: nextEventId + index,
      }))].slice(-1200);
      nextEventId += next.events.length;
    }
    if (!next.nodes.some((node) => node.id === selectedNode)) selectedNode = 1;
  }

  async function reset(newTopology: boolean): Promise<void> {
    if (newTopology) cfg = { ...cfg, seed: randSeed() };
    isLoading = true;
    error = '';
    entries = [];
    nextEventId = 0;
    ++session;
    snapshot = null;
    selectedNode = 1;
    try {
      await controller.load({ ...cfg });
    } catch (cause: unknown) {
      error = String(cause);
    } finally {
      isLoading = false;
    }
  }

  function change(changes: Partial<WsnConfig>): void {
    cfg = { ...cfg, ...changes };
    if (isLoading) return;
    if (changes.nodes !== undefined) controller.setNodes(changes.nodes);
    if (changes.mac !== undefined) controller.setMac(changes.mac);
    if (changes.tx_power_dbm !== undefined) controller.setTxPower(changes.tx_power_dbm);
    if (changes.noise_floor_dbm !== undefined) controller.setNoise(changes.noise_floor_dbm);
    if (changes.duty_cycle_pct !== undefined) controller.setDuty(changes.duty_cycle_pct);
    if (changes.data_interval_secs !== undefined) controller.setInterval(changes.data_interval_secs);
    if (changes.initial_battery_mj !== undefined) controller.setBattery(changes.initial_battery_mj);
    if (changes.to_secs !== undefined) controller.setDuration(changes.to_secs);
  }

  function toggle(): void { isPlaying ? controller.pause() : controller.play(speed); }
  function changeSpeed(next: number): void { speed = next; controller.setSpeed(next); }
  function move(dir: PlaybackDir): void { if (!isLoading) controller.move(dir); }

  onMount((): () => void => {
    controller = new WsnController(
      consume,
      (message: string): void => { error = message; },
      (time: number): void => { entries = entries.filter((event) => event.time <= time + 0.000001); },
      (playing: boolean): void => { isPlaying = playing; },
    );
    void reset(false);
    return (): void => controller.dispose();
  });
</script>

<div class="wsn-layout">
  <aside class="wsn-layout__settings">
    <WsnSettings {cfg} {isLoading} onChange={change}
      onReset={() => { cfg = defaultWsnConfig(randSeed()); void reset(false); }}
      onGenerate={() => { void reset(true); }}/>
    <WsnStats {snapshot}/>
  </aside>

  <WsnTopology {snapshot} {selectedNode} {isPlaying} {speed} cfgDuration={cfg.to_secs}
    onSelect={(id) => selectedNode = id}
    onMoveNode={(id, x, y) => controller.moveNode(id, x, y)}
    onMove={move} onToggle={toggle} onSpeed={changeSpeed}
    onMac={(mac: WsnMac): void => change({ mac })}/>

  <aside class="wsn-layout__status">
    <WsnNodes {snapshot} {selectedNode} onSelect={(id) => selectedNode = id}
      onEnabled={(id, enabled) => controller.setNodeEnabled(id, enabled)}
      onRecharge={(id) => controller.rechargeNode(id)}/>
    {#key session}
      <WsnConsole {entries} now={snapshot?.time ?? 0}/>
    {/key}
  </aside>
</div>
{#if error}
  <p class="mt-2 rounded-lg border border-[var(--danger)] px-3 py-2 text-xs" role="alert">{t('error')}: {error}</p>
{/if}
