import { type Locale, type TranslationParams, translate } from "#lib/i18n/index.ts";
import { ripEventText } from "../rip/i18n.ts";
import type { RoutingEvent } from "../routing/types.ts";

export function ospfText(key: string, code: Locale, params: TranslationParams = {}): string {
  return translate(`ospf.${key}`, code, params);
}

export function ospfEventText(event: RoutingEvent, code: Locale): string {
  const t: (key: string, params?: TranslationParams) => string = (
    key: string,
    params: TranslationParams = {},
  ): string => ospfText(key, code, params);
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
    case "send":
    case "receive": {
      const match: RegExpExecArray | null = /LSA #(\d+) sequence (\d+)/u.exec(event.msg);
      if (!match) return event.msg;
      return t(event._type === "send" ? "eventSend" : "eventReceive", {
        ...params,
        origin: match[1],
        sequence: match[2],
      });
    }
    case "spf":
      return t("eventSpf", params);
    case "adjacency_up":
      return t("eventAdjacencyUp", params);
    case "adjacency_down":
      return t("eventAdjacencyDown", params);
    case "periodic":
      return t("eventPeriodic", params);
    case "packet_lost":
      return t("eventPacketLost", params);
    case "packet_dropped":
      return t(
        event.msg.includes("malformed LSA") ? "eventMalformedLsa" : "eventPacketDropped",
        params,
      );
    case "complete": {
      const match: RegExpExecArray | null = /the (\d+(?:\.\d+)?)-second limit/u.exec(event.msg);
      return match ? t("eventComplete", { seconds: match[1] }) : event.msg;
    }
    case "router_down":
      return t("eventRouterDown", params);
    case "auto_failure":
      return t("eventAutoFailure", params);
    case "auto_recovery":
      return t("eventAutoRecovery", params);
    case "router_restart":
      return t("eventRouterRestart", params);
    case "discover":
      return t("eventDiscover", params);
    case "invalidate":
      return t("eventInvalidate", params);
    case "route_change":
      return t("eventRouteChange", params);
    case "link_cost":
    case "converged":
      return ripEventText(event, code);
    case "packet_loss_setting":
    case "packet_invalid_setting":
      return ripEventText(event, code).replaceAll("RIP", "OSPF");
    default:
      return event.msg;
  }
}
