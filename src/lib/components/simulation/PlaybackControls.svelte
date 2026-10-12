<script lang="ts">
  import { SIM_CONTROL_BTN, SIM_PRIMARY_BTN } from "./styles.ts";
  import { PLAYBACK_SPEEDS } from "#lib/sims/shared/playback.ts";
  import type { PlaybackDir } from "#lib/sims/shared/playback.ts";

  export let isPlaying: boolean;
  export let speed: number;
  export let canBack: boolean;
  export let canForward: boolean;
  export let canPlay: boolean;
  export let canEnd: boolean;
  export let playLbl: string = "Play";
  export let pauseLbl: string = "Pause";
  export let backLbl: string = "← Step";
  export let forwardLbl: string = "Step →";
  export let startTitle: string = "Back to start";
  export let endTitle: string = "Forward to end";
  export let controlsLbl: string = "Simulation playback controls";
  export let speedLbl: string = "Playback speed";
  export let onMove: (dir: PlaybackDir) => void;
  export let onToggle: () => void;
  export let onSpeed: (val: number) => void;

  const btnClass: string = `${SIM_CONTROL_BTN} max-w-full @max-[470px]:min-w-0 @max-[470px]:px-1.5`;
</script>

<div class="flex w-full min-w-0 flex-wrap items-center justify-center gap-x-2 gap-y-1.5 whitespace-nowrap @max-[470px]:gap-x-1" aria-label={controlsLbl}>
  <div class="flex shrink-0 items-center justify-center gap-1 @max-[470px]:gap-0.5">
    <button type="button" class={btnClass} title={startTitle} aria-label={startTitle} onclick={() => onMove("start")} disabled={!canBack}>⏮</button>
    <button type="button" class={btnClass} aria-label={backLbl} title={backLbl} onclick={() => onMove("back")} disabled={!canBack}>
      <span class="@max-[470px]:hidden">{backLbl}</span><span class="hidden @max-[470px]:inline" aria-hidden="true">◀</span>
    </button>
    <button type="button" class={`${SIM_PRIMARY_BTN} min-w-[62px] px-2 text-[10px] @max-[470px]:min-w-[43px] @max-[470px]:px-1`} onclick={onToggle} disabled={!canPlay}>{isPlaying ? pauseLbl : playLbl}</button>
    <button type="button" class={btnClass} aria-label={forwardLbl} title={forwardLbl} onclick={() => onMove("forward")} disabled={!canForward}>
      <span class="@max-[470px]:hidden">{forwardLbl}</span><span class="hidden @max-[470px]:inline" aria-hidden="true">▶</span>
    </button>
    <button type="button" class={btnClass} title={endTitle} aria-label={endTitle} onclick={() => onMove("end")} disabled={!canEnd}>⏭</button>
  </div>
  <div class="flex w-fit shrink-0 items-center gap-[3px] rounded-lg border border-[var(--control-border)] bg-[var(--surface-2)] p-[3px] @max-[470px]:gap-0.5 @max-[470px]:p-[2px]" role="group" aria-label={speedLbl}>
    {#each PLAYBACK_SPEEDS as option}
      <button type="button" class={['rounded-[5px] border-0 bg-transparent px-[7px] py-1 text-[10px] text-[var(--muted-text)] @max-[470px]:px-1', speed === option && 'bg-[var(--selected-bg)] text-[var(--selected-text)] shadow-[inset_0_0_0_1px_var(--accent-border)]']} onclick={() => onSpeed(option)} aria-pressed={speed === option}>{option}×</button>
    {/each}
  </div>
</div>
