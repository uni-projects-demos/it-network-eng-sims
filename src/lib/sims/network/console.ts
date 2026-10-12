import type { RipEvent, RipEventType } from "./types.ts";

export type ConsoleEntry = RipEvent & Readonly<{ id: number }>;
export const CONSOLE_FILTERS: readonly ["all", "routing", "msgs", "faults"] = [
  "all",
  "routing",
  "msgs",
  "faults",
] as const;
export type ConsoleFilter = (typeof CONSOLE_FILTERS)[number];
export const MAX_STORED_CONSOLE_EVENTS: number = 20_000;
export const MAX_VISIBLE_CONSOLE_EVENTS: number = 1_200;

const ROUTING_EVENTS: ReadonlySet<RipEventType> = new Set([
  "discover",
  "route_change",
  "invalidate",
  "route_garbage",
  "converged",
  "link_cost",
]);
const MSG_EVENTS: ReadonlySet<RipEventType> = new Set([
  "send",
  "receive",
  "periodic",
  "startup",
  "packet_lost",
  "packet_loss_setting",
  "packet_invalid_setting",
  "packet_dropped",
]);
const FAULT_EVENTS: ReadonlySet<RipEventType> = new Set([
  "timeout",
  "neighbor_garbage",
  "all_garbage",
  "router_down",
  "router_restart",
  "auto_failure",
  "auto_recovery",
  "recovered",
  "packet_dropped",
  "complete",
]);
const DANGER_EVENTS: ReadonlySet<RipEventType> = new Set([
  "router_down",
  "auto_failure",
  "timeout",
  "invalidate",
  "all_garbage",
]);
const RECOVERY_EVENTS: ReadonlySet<RipEventType> = new Set([
  "route_change",
  "discover",
  "recovered",
  "converged",
  "router_restart",
  "auto_recovery",
  "link_cost",
]);
const WARNING_EVENTS: ReadonlySet<RipEventType> = new Set(["route_garbage", "neighbor_garbage"]);

export function eventMatchesFilter(_type: RipEventType, filter: ConsoleFilter): boolean {
  switch (filter) {
    case "all":
      return true;
    case "routing":
      return ROUTING_EVENTS.has(_type);
    case "msgs":
      return MSG_EVENTS.has(_type);
    case "faults":
      return FAULT_EVENTS.has(_type);
  }
}

export function eventTone(_type: RipEventType): "danger" | "receiver" | "timeout" | "primary" {
  if (DANGER_EVENTS.has(_type)) return "danger";
  if (RECOVERY_EVENTS.has(_type)) return "receiver";
  if (WARNING_EVENTS.has(_type)) return "timeout";
  return "primary";
}

export function consoleUpperBound(events: readonly ConsoleEntry[], time: number): number {
  let low: number = 0;
  let high: number = events.length;
  while (low < high) {
    const mid: number = low + Math.floor((high - low) / 2);
    if (events[mid].time <= time + 0.000_001) low = mid + 1;
    else high = mid;
  }
  return low;
}

export function latestConsoleEvent(
  events: readonly ConsoleEntry[],
  time: number,
): ConsoleEntry | undefined {
  return events[consoleUpperBound(events, time) - 1];
}

export function visibleConsoleEvents(
  events: readonly ConsoleEntry[],
  time: number,
  filter: ConsoleFilter,
  router: number | null,
  limit: number = MAX_VISIBLE_CONSOLE_EVENTS,
): ConsoleEntry[] {
  const visible: ConsoleEntry[] = [];
  for (
    let idx: number = consoleUpperBound(events, time) - 1;
    idx >= 0 && visible.length < limit;
    idx--
  ) {
    const event: ConsoleEntry = events[idx];
    if (
      event &&
      (router === null || event.router === router) &&
      eventMatchesFilter(event._type, filter)
    ) {
      visible.push(event);
    }
  }
  return visible.reverse();
}

export function appendConsoleEvents(
  existing: readonly ConsoleEntry[],
  added: readonly RipEvent[],
  nxtId: number,
): ConsoleEntry[] {
  if (added.length === 0) return [...existing];
  const appended: ConsoleEntry[] = added.map(
    (event: RipEvent, idx: number): ConsoleEntry => ({ ...event, id: nxtId + idx }),
  );
  return [...existing, ...appended].slice(-MAX_STORED_CONSOLE_EVENTS);
}
