export type BatteryHealth = "low" | "medium" | "high";

export function batteryHealth(prc: number): BatteryHealth {
  if (!Number.isFinite(prc) || prc <= 20) return "low";
  if (prc <= 50) return "medium";
  return "high";
}

export type BatteryFill = `var(--wsn-battery-${BatteryHealth})`;

export function batteryFill(prc: number): BatteryFill {
  return `var(--wsn-battery-${batteryHealth(prc)})`;
}
