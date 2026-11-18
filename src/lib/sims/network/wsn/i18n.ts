import { type Locale, type TranslationParams, translate } from "#lib/i18n/index.ts";
import type { WsnEvent, WsnEventType } from "./types.ts";

export function wsnText(key: string, code: Locale, params: TranslationParams = {}): string {
  if (key === "settings" || key === "statistics") return translate(`rip.${key}`, code, params);
  if (key === "timeout") return translate("rip.simulationTimeout", code, params);
  if (key === "to" || key === "routeCost")
    return translate(`rip.${key === "to" ? "to" : "currentRouteMetric"}`, code, params);
  return translate(`wsn.${key}`, code, params);
}

export function wsnEventText(event: WsnEvent, code: Locale): string {
  const lbl: string = wsnText(`events.${event._type}`, code);
  const router: string =
    event.node === 0 ? wsnText("gateway", code) : `${wsnText("node", code)} ${event.node}`;
  const peer: string =
    event.peer === null
      ? ""
      : event.peer === 0
        ? wsnText("gateway", code)
        : `${wsnText("node", code)} ${event.peer}`;
  return `${router}${peer ? ` → ${peer}` : ""}: ${lbl}`;
}

export type WsnConsoleFilter = "all" | "traffic" | "network" | "energy";
export function eventCategory(type: WsnEventType): WsnConsoleFilter {
  if (type === "depleted") return "energy";
  if (
    type === "route" ||
    type === "topology" ||
    type === "config" ||
    type === "failure" ||
    type === "restart" ||
    type === "startup"
  )
    return "network";
  return "traffic";
}
