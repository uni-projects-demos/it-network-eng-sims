import type {
  RoutingConfig,
  RoutingEdge,
  RoutingEvent,
  RoutingEventType,
  RoutingHandle,
  RoutingNode,
  RoutingRoute,
  RoutingSnapshot,
  RoutingStats,
} from "../routing/types.ts";

export { sliderTimeOutSecs, timeOutSliderVal } from "../routing/types.ts";
export type RipEventType = Exclude<RoutingEventType, "spf" | "adjacency_down" | "adjacency_up">;
export type RipConfig = RoutingConfig;
export type RipRoute = RoutingRoute;
export type RipNode = RoutingNode;
export type RipEdge = RoutingEdge;
export type RipEvent = RoutingEvent & { _type: RipEventType };
export type RipStats = RoutingStats & {
  invalids: number;
  garbage_collections: number;
  rip_reachability: number;
};
export type RipSnapshot = RoutingSnapshot<RipStats, RipEvent>;
export type RipHandle = RoutingHandle<RipSnapshot>;
