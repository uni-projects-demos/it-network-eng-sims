export const PLAYBACK_SPEEDS: readonly [0.25, 0.5, 1, 2, 4, 8] = [0.25, 0.5, 1, 2, 4, 8] as const;
export type PlaybackDir = "start" | "back" | "forward" | "end";

export class PlaybackTimer {
  private timer: ReturnType<typeof setTimeout> | undefined;

  schedule(delayMs: number, callback: () => void): void {
    this.clear();
    this.timer = setTimeout((): void => {
      this.timer = undefined;
      callback();
    }, delayMs);
  }

  clear(): void {
    if (this.timer !== undefined) clearTimeout(this.timer);
    this.timer = undefined;
  }
}
