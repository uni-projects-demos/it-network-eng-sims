import { type Locale, translate } from "#lib/i18n/index.ts";
import type { StatRow } from "#lib/sims/shared/stats.ts";
import type { OspfStats } from "../ospf/types.ts";
import type { RipStats } from "../rip/types.ts";
import type { RoutingNode, RoutingProtocol, RoutingSnapshot, RoutingStats } from "./types.ts";

function isRipStats(stats: RoutingStats): stats is RipStats {
  return "rip_reachability" in stats && "invalids" in stats && "garbage_collections" in stats;
}

function isOspfStats(stats: RoutingStats): stats is OspfStats {
  return (
    "ospf_reachability" in stats &&
    "spf_runs" in stats &&
    "duplicate_lsas" in stats &&
    "hello_sent" in stats &&
    "hello_rcvd" in stats &&
    "adjacency_changes" in stats
  );
}

export function routingStatRows(
  snapshot: RoutingSnapshot | null,
  protocol: RoutingProtocol,
  locale: Locale,
): readonly StatRow[] {
  const stats: RoutingStats | undefined = snapshot?.stats;
  const rip: RipStats | undefined = stats && isRipStats(stats) ? stats : undefined;
  const ospf: OspfStats | undefined = stats && isOspfStats(stats) ? stats : undefined;
  const common: StatRow[] = [
    {
      lbl: translate("rip.routeAccuracy", locale),
      val: `${((stats?.accuracy ?? 0) * 100).toFixed(0)}%`,
    },
    {
      lbl: translate("rip.averageConvergenceTime", locale),
      val: stats?.convergence_secs == null ? "—" : `${stats.convergence_secs.toFixed(1)} s`,
    },
    {
      lbl: translate("rip.updatesSentReceived", locale),
      val: `${stats?.updates_sent ?? 0} / ${stats?.updates_rcvd ?? 0}`,
    },
    { lbl: translate("rip.routesChanged", locale), val: stats?.route_changes ?? 0 },
    {
      lbl: translate("rip.runningRouters", locale),
      val: `${snapshot?.nodes.filter((node: RoutingNode): boolean => node.is_running).length ?? 0} / ${snapshot?.nodes.length ?? 0}`,
    },
  ];
  const specific: StatRow[] =
    protocol === "rip"
      ? [
          {
            lbl: translate("rip.ripReachability", locale),
            val: `${((rip?.rip_reachability ?? 0) * 100).toFixed(1)}%`,
          },
          { lbl: translate("rip.poisonedRoutes", locale), val: rip?.invalids ?? 0 },
          {
            lbl: translate("rip.garbageCollectedRoutes", locale),
            val: rip?.garbage_collections ?? 0,
          },
        ]
      : [
          {
            lbl: translate("ospf.reachability", locale),
            val: `${((ospf?.ospf_reachability ?? 0) * 100).toFixed(1)}%`,
          },
          { lbl: translate("ospf.spfRuns", locale), val: ospf?.spf_runs ?? 0 },
          { lbl: translate("ospf.adjacencyChanges", locale), val: ospf?.adjacency_changes ?? 0 },
        ];
  const details: StatRow[] = [
    {
      lbl: translate(protocol === "rip" ? "rip.ripPacketsLost" : "ospf.packetsLost", locale),
      val: stats?.pcks_lost ?? 0,
    },
    {
      lbl: translate(
        protocol === "rip" ? "rip.invalidPacketsDropped" : "ospf.invalidPacketsDropped",
        locale,
      ),
      val: stats?.pcks_dropped ?? 0,
    },
    { lbl: translate("rip.triggeredUpdates", locale), val: stats?.triggered_updates ?? 0 },
    { lbl: translate("rip.maxShortestCost", locale), val: stats?.shortest_path_cost_max ?? 0 },
    {
      lbl: translate("rip.meanShortestCost", locale),
      val: (stats?.shortest_path_cost_mean ?? 0).toFixed(2),
    },
    { lbl: translate("rip.minShortestCost", locale), val: stats?.shortest_path_cost_min ?? 0 },
  ];
  return [...common, ...specific, ...details];
}
