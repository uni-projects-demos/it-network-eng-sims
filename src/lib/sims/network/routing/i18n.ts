import { type Locale, type TranslationParams, translate } from "#lib/i18n/index.ts";
import type { RoutingProtocol } from "./types.ts";

const OSPF_KEYS: ReadonlySet<string> = new Set<string>([
  "activeNeighbors",
  "invalidPacketProbability",
  "loadingWasm",
  "networkAria",
  "unreachable",
  "lsa",
]);

export function routingText(
  protocol: RoutingProtocol,
  key: string,
  code: Locale,
  params: TranslationParams = {},
): string {
  const namespace: "rip" | "ospf" = protocol === "ospf" && OSPF_KEYS.has(key) ? "ospf" : "rip";
  return translate(`${namespace}.${key}`, code, params);
}
