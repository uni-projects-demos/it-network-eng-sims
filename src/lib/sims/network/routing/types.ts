export {
  CONTINUOUS_DURATION_SECS as ROUTING_CONT_TO_SECS,
  CONTINUOUS_DURATION_SLIDER_VALUE as ROUTING_CONT_SLIDER_VAL,
  DEFAULT_DURATION_SECS as ROUTING_DEF_TO_SECS,
  DURATION_STEP_MINUTES as ROUTING_TO_STEP_MINS,
  durationSeconds as sliderTimeOutSecs,
  durationSliderValue as timeOutSliderVal,
  MIN_DURATION_MINUTES as ROUTING_MIN_TO_MINS,
} from "../../shared/duration.ts";

export type RoutingProtocol = "rip" | "ospf";
export type RoutingConfig = {
  nodes: number;
  seed: number;
  topology: "default" | "rand";
  rand_fail_perc: number;
  pck_loss_perc: number;
  pck_invalid_perc: number;
  to_secs: number;
};
export type RoutingRoute = { dest: number; nxt_hop: number; metric: number };
export type RoutingNode = {
  id: number;
  is_running: boolean;
  is_quiescent: boolean;
  auto_restart_in: number | null | undefined;
  active_nbrs: number;
  garbage_nbrs: number;
  routes: RoutingRoute[];
  offline_dests: number[];
};
export type RoutingEdge = {
  a: number;
  b: number;
  cost: number;
  can_increase: boolean;
  can_decrease: boolean;
};
export type RoutingEventType =
  | "startup"
  | "send"
  | "receive"
  | "periodic"
  | "spf"
  | "discover"
  | "route_change"
  | "invalidate"
  | "route_garbage"
  | "timeout"
  | "neighbor_garbage"
  | "all_garbage"
  | "recovered"
  | "adjacency_down"
  | "adjacency_up"
  | "router_down"
  | "router_restart"
  | "auto_failure"
  | "auto_recovery"
  | "link_cost"
  | "converged"
  | "packet_dropped"
  | "packet_lost"
  | "packet_loss_setting"
  | "packet_invalid_setting"
  | "complete";
export type RoutingEvent = {
  time: number;
  _type: RoutingEventType;
  router: number;
  peer: number | null | undefined;
  dest: number | null | undefined;
  old_metric: number | null | undefined;
  new_metric: number | null | undefined;
  old_hop: number | null | undefined;
  new_hop: number | null | undefined;
  msg: string;
};
export type RoutingStats = {
  updates_sent: number;
  updates_rcvd: number;
  pcks_dropped: number;
  pcks_lost: number;
  periodic_updates: number;
  triggered_updates: number;
  route_changes: number;
  fails: number;
  restarts: number;
  rand_fails: number;
  rand_recovs: number;
  is_converged: boolean;
  accuracy: number;
  reachable_pairs: number;
  shortest_path_cost_mean: number;
  shortest_path_cost_min: number;
  shortest_path_cost_max: number;
  convergence_secs: number | null | undefined;
};
export type RoutingSnapshot<
  TStats extends RoutingStats = RoutingStats,
  TEvent extends RoutingEvent = RoutingEvent,
  TNode extends RoutingNode = RoutingNode,
> = {
  time: number;
  dur: number;
  hist_start: number;
  is_complete: boolean;
  nodes: TNode[];
  edges: RoutingEdge[];
  events: TEvent[];
  stats: TStats;
};
export type RoutingHandle<TSnapshot extends RoutingSnapshot = RoutingSnapshot> = {
  snapshot(): TSnapshot;
  advance(secs: number): TSnapshot;
  step_back(): TSnapshot;
  seek_start(): TSnapshot;
  seek_end(): TSnapshot;
  terminate_router(id: number): TSnapshot;
  restart_router(id: number): TSnapshot;
  set_link_cost(a: number, b: number, cost: number): TSnapshot;
  set_pck_loss_perc(perc: number): TSnapshot;
  set_pck_invalid_perc(perc: number): TSnapshot;
  free(): void;
};

export function routeInfinity(protocol: RoutingProtocol): number {
  return protocol === "rip" ? 16 : Number.POSITIVE_INFINITY;
}
