export const MIN_DURATION_MINUTES: number = 1;
export const DURATION_STEP_MINUTES: number = 1;
export const CONTINUOUS_DURATION_SLIDER_VALUE: number = 6;
export const CONTINUOUS_DURATION_SECS: number = 0;
export const DEFAULT_DURATION_SECS: number = MIN_DURATION_MINUTES * 60;

export function durationSliderValue(seconds: number): number {
  return seconds === CONTINUOUS_DURATION_SECS ? CONTINUOUS_DURATION_SLIDER_VALUE : seconds / 60;
}

export function durationSeconds(value: number): number {
  return value === CONTINUOUS_DURATION_SLIDER_VALUE ? CONTINUOUS_DURATION_SECS : value * 60;
}
