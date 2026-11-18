import type { Locale } from "#lib/i18n/index.ts";
import { ospfEventText } from "../ospf/i18n.ts";
import { ripEventText } from "../rip/i18n.ts";
import type { RoutingEvent, RoutingProtocol } from "./types.ts";

export function routingEventText(
  event: RoutingEvent,
  protocol: RoutingProtocol,
  locale: Locale,
): string {
  return protocol === "rip" ? ripEventText(event, locale) : ospfEventText(event, locale);
}
