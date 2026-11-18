import { type PlaybackDir, PlaybackTimer } from "./playback.ts";

export type TimedSnapshot = {
  time: number;
  hist_start: number;
  is_complete: boolean;
};

export type TimedHandle<TSnapshot extends TimedSnapshot> = {
  snapshot(): TSnapshot;
  advance(secs: number): TSnapshot;
  step_back(): TSnapshot;
  seek_start(): TSnapshot;
  seek_end(): TSnapshot;
  free(): void;
};

export class SimulationController<
  TConfig,
  TSnapshot extends TimedSnapshot,
  THandle extends TimedHandle<TSnapshot>,
> {
  private handle: THandle | null = null;
  private current: TSnapshot | null = null;
  private readonly timer: PlaybackTimer = new PlaybackTimer();
  private generation: number = 0;
  private isRunning: boolean = false;
  private speed: number = 1;
  private readonly create: (config: TConfig) => Promise<THandle>;
  private readonly onSnapshot: (snapshot: TSnapshot) => void;
  private readonly onError: (message: string) => void;
  private readonly onBranch: (time: number) => void;
  private readonly onPlaying: (playing: boolean) => void;

  constructor(
    create: (config: TConfig) => Promise<THandle>,
    onSnapshot: (snapshot: TSnapshot) => void,
    onError: (message: string) => void,
    onBranch: (time: number) => void,
    onPlaying: (playing: boolean) => void,
  ) {
    this.create = create;
    this.onSnapshot = onSnapshot;
    this.onError = onError;
    this.onBranch = onBranch;
    this.onPlaying = onPlaying;
  }

  private publish(snapshot: TSnapshot): void {
    this.current = snapshot;
    this.onSnapshot(snapshot);
    if (snapshot.is_complete) this.pause();
  }

  async load(config: TConfig): Promise<void> {
    const revision: number = ++this.generation;
    this.pause();
    this.handle?.free();
    this.handle = null;
    this.current = null;
    const created: THandle = await this.create(config);
    if (revision !== this.generation) {
      created.free();
      return;
    }
    this.handle = created;
    this.publish(created.snapshot());
  }

  pause(): void {
    this.isRunning = false;
    this.timer.clear();
    this.onPlaying(false);
  }

  play(speed: number = this.speed): void {
    this.speed = speed;
    if (!this.handle || !this.current || this.current.is_complete) return;
    this.pause();
    this.isRunning = true;
    this.onPlaying(true);
    this.scheduleTick();
  }

  setSpeed(speed: number): void {
    this.speed = speed;
    if (this.isRunning) this.scheduleTick();
  }

  private scheduleTick(): void {
    if (!this.isRunning) return;
    this.timer.schedule(250, (): void => {
      if (!this.handle || !this.isRunning) return;
      try {
        this.publish(this.handle.advance(0.25 * this.speed));
        this.scheduleTick();
      } catch (error: unknown) {
        this.onError(String(error));
        this.pause();
      }
    });
  }

  move(dir: PlaybackDir): void {
    if (!this.handle || !this.current) return;
    this.pause();
    try {
      const snapshot: TSnapshot =
        dir === "start"
          ? this.handle.seek_start()
          : dir === "back"
            ? this.handle.step_back()
            : dir === "forward"
              ? this.handle.advance(1)
              : this.handle.seek_end();
      this.publish(snapshot);
    } catch (error: unknown) {
      this.onError(String(error));
    }
  }

  protected allowEditingComplete(): boolean {
    return false;
  }

  protected edit(action: (handle: THandle) => TSnapshot): void {
    if (!this.handle || !this.current || (this.current.is_complete && !this.allowEditingComplete()))
      return;
    try {
      const time: number = this.current.time;
      const snapshot: TSnapshot = action(this.handle);
      this.onBranch(time);
      this.publish(snapshot);
    } catch (error: unknown) {
      this.onError(String(error));
    }
  }

  dispose(): void {
    ++this.generation;
    this.pause();
    this.handle?.free();
    this.handle = null;
    this.current = null;
  }
}
