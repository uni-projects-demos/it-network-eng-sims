import { type PlaybackDir, PlaybackTimer } from "#lib/sims/shared/playback.ts";
import { runSimRIP } from "#lib/wasm/engine.ts";
import type { RipConfig, RipHandle, RipSnapshot } from "./types.ts";

export class RipController {
  private handle: RipHandle | null = null;
  private cur: RipSnapshot | null = null;
  private timer: PlaybackTimer = new PlaybackTimer();
  private gen: number = 0;
  private is_running: boolean = false;
  private playbackSpeed: number = 1;

  constructor(
    private readonly onSnapshot: (snapshot: RipSnapshot) => void,
    private readonly onError: (err: string) => void,
    private readonly onBranch: (time: number) => void,
    private readonly onPlaying: (isPlaying: boolean) => void,
  ) {}

  private publish(snapshot: RipSnapshot): void {
    this.cur = snapshot;
    this.onSnapshot(snapshot);
    if (snapshot.is_complete) this.pause();
  }

  async load(config: RipConfig): Promise<void> {
    const revision: number = ++this.gen;
    this.pause();
    this.handle?.free();
    this.handle = null;
    this.cur = null;
    const created: RipHandle = await runSimRIP(config);
    if (revision !== this.gen) {
      created.free();
      return;
    }
    this.handle = created;
    this.publish(created.snapshot());
  }

  pause(): void {
    this.is_running = false;
    this.timer.clear();
    this.onPlaying(false);
  }

  play(speed: number = this.playbackSpeed): void {
    this.playbackSpeed = speed;
    if (!this.handle || !this.cur || this.cur.is_complete) return;
    this.pause();
    this.is_running = true;
    this.onPlaying(true);
    this.scheduleTick();
  }

  setSpeed(speed: number): void {
    this.playbackSpeed = speed;
    if (this.is_running) this.scheduleTick();
  }

  private scheduleTick(): void {
    if (!this.is_running) return;
    this.timer.schedule(250, (): void => {
      if (!this.handle || !this.is_running) return;
      try {
        this.publish(this.handle.advance(0.25 * this.playbackSpeed));
        this.scheduleTick();
      } catch (err: unknown) {
        this.onError(String(err));
        this.pause();
      }
    });
  }

  move(dir: PlaybackDir): void {
    if (!this.handle || !this.cur) return;
    this.pause();
    try {
      const nxt: RipSnapshot =
        dir === "start"
          ? this.handle.seek_start()
          : dir === "back"
            ? this.handle.step_back()
            : dir === "forward"
              ? this.handle.advance(1)
              : this.handle.seek_end();
      this.publish(nxt);
    } catch (err: unknown) {
      this.onError(String(err));
    }
  }

  private edit(action: (handle: RipHandle) => RipSnapshot): void {
    if (!this.handle || !this.cur || this.cur.is_complete) return;
    try {
      const nxt: RipSnapshot = action(this.handle);
      this.onBranch(this.cur.time);
      this.publish(nxt);
    } catch (err: unknown) {
      this.onError(String(err));
    }
  }

  terminate(id: number): void {
    this.edit((handle: RipHandle): RipSnapshot => handle.terminate_router(id));
  }
  restart(id: number): void {
    this.edit((handle: RipHandle): RipSnapshot => handle.restart_router(id));
  }
  setLinkCost(a: number, b: number, cost: number): void {
    this.edit((handle: RipHandle): RipSnapshot => handle.set_link_cost(a, b, cost));
  }
  setPacketLoss(percent: number): void {
    this.edit((handle: RipHandle): RipSnapshot => handle.set_pck_loss_perc(percent));
  }
  setPacketInvalid(percent: number): void {
    this.edit((handle: RipHandle): RipSnapshot => handle.set_pck_invalid_perc(percent));
  }

  dispose(): void {
    ++this.gen;
    this.pause();
    this.handle?.free();
    this.handle = null;
    this.cur = null;
  }
}
