import type {
  RoutingConfig,
  RoutingEvent,
  RoutingEventType,
  RoutingHandle,
  RoutingNode,
  RoutingSnapshot,
  RoutingStats,
} from "../routing/types.ts";

export type OspfConfig = RoutingConfig;
export type OspfEventType = Exclude<
  RoutingEventType,
  "route_garbage" | "timeout" | "neighbor_garbage" | "all_garbage" | "recovered"
>;
export type OspfEvent = RoutingEvent & { _type: OspfEventType };
export type OspfNode = RoutingNode & { known_lsas: number };
export type OspfStats = RoutingStats & {
  ospf_reachability: number;
  spf_runs: number;
  duplicate_lsas: number;
  hello_sent: number;
  hello_rcvd: number;
  adjacency_changes: number;
};
export type OspfSnapshot = RoutingSnapshot<OspfStats, OspfEvent, OspfNode>;
export type OspfHandle = RoutingHandle<OspfSnapshot>;
