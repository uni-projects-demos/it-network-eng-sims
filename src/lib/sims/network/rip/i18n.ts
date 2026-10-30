import { type Locale, type TranslationParams, translate } from "#lib/i18n/index.ts";
import type { RipEvent } from "./types.ts";

export function ripText(key: string, code: Locale, params: TranslationParams = {}): string {
  return translate(`rip.${key}`, code, params);
}

export function ripEventText(event: RipEvent, code: Locale): string {
  const t: (key: string, params?: TranslationParams) => string = (
    key: string,
    params: TranslationParams = {},
  ): string => ripText(key, code, params);
  const params: TranslationParams = {
    router: event.router,
    peer: event.peer ?? undefined,
    destination: event.dest ?? undefined,
    oldMetric: event.old_metric ?? undefined,
    newMetric: event.new_metric ?? undefined,
    oldHop: event.old_hop ?? undefined,
    newHop: event.new_hop ?? undefined,
  };

  switch (event._type) {
    case "startup":
      return t("eventStartup", params);
    case "packet_loss_setting":
    case "packet_invalid_setting": {
      const match: RegExpExecArray | null = /\b(\d+)%$/u.exec(event.msg);
      if (!match) return event.msg;
      return t(
        event._type === "packet_loss_setting"
          ? "eventPacketLossSetting"
          : "eventPacketInvalidSetting",
        { percent: match[1] },
      );
    }
    case "auto_failure": {
      const match: RegExpExecArray | null = /for (\d+(?:\.\d+)?)s;/u.exec(event.msg);
      return match ? t("eventAutoFailure", { ...params, seconds: match[1] }) : event.msg;
    }
    case "router_down":
      return t("eventRouterDown", params);
    case "auto_recovery":
      return t("eventAutoRecovery", params);
    case "router_restart":
      return t("eventRouterRestart", params);
    case "link_cost": {
      const match: RegExpExecArray | null = /Link R(\d+) ↔ R(\d+) cost changed:/u.exec(event.msg);
      return match ? t("eventLinkCost", { ...params, a: match[1], b: match[2] }) : event.msg;
    }
    case "timeout": {
      const match: RegExpExecArray | null = /after (\d+(?:\.\d+)?)s/u.exec(event.msg);
      return match ? t("eventTimeout", { ...params, seconds: match[1] }) : event.msg;
    }
    case "all_garbage":
      return t("eventAllGarbage", params);
    case "neighbor_garbage":
      return t("eventNeighborGarbage", params);
    case "route_garbage":
      return t("eventRouteGarbage", params);
    case "periodic":
      return t("eventPeriodic", params);
    case "packet_lost":
      return t("eventPacketLost", params);
    case "complete": {
      const match: RegExpExecArray | null =
        /the (\d+(?:\.\d+)?)-minute \((\d+(?:\.\d+)?)-second\) limit/u.exec(event.msg);
      return match ? t("eventComplete", { minutes: match[1], seconds: match[2] }) : event.msg;
    }
    case "send": {
      const match: RegExpExecArray | null = /with (\d+) entr(?:y|ies) \((\d+) bytes\)/u.exec(
        event.msg,
      );
      if (!match) return event.msg;
      const sent: string = t("eventSend", { ...params, entries: match[1], bytes: match[2] });
      return event.msg.endsWith("(triggered)") ? `${sent} ${t("triggeredSuffix")}` : sent;
    }
    case "packet_dropped":
      return t("eventPacketDropped", params);
    case "recovered":
      return t("eventRecovered", params);
    case "receive": {
      const match: RegExpExecArray | null = /\((\d+) entries\)$/u.exec(event.msg);
      return match ? t("eventReceive", { ...params, entries: match[1] }) : event.msg;
    }
    case "invalidate":
      return t("eventInvalidate", params);
    case "discover":
      return t("eventDiscover", params);
    case "route_change":
      return t("eventRouteChange", params);
    case "converged": {
      const match: RegExpExecArray | null = /after (\d+(?:\.\d+)?)s/u.exec(event.msg);
      return match ? t("eventConverged", { seconds: match[1] }) : event.msg;
    }
  }
}
