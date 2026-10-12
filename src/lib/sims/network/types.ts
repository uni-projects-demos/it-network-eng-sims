export const RIP_MIN_TO_MINS: number = 1;
export const RIP_TO_STEP_MINS: number = 1;
export const RIP_CONT_SLIDER_VAL: number = 6;
export const RIP_CONT_TO_SECS: number = 0;
export const RIP_DEF_TO_SECS: number = RIP_MIN_TO_MINS * 60;
export function timeOutSliderVal(secs: number): number {
  return secs === RIP_CONT_TO_SECS ? RIP_CONT_SLIDER_VAL : secs / 60;
}
export function sliderTimeOutSecs(val: number): number {
  return val === RIP_CONT_SLIDER_VAL ? RIP_CONT_TO_SECS : val * 60;
}

export type RipEventType =
  | "startup"
  | "send"
  | "receive"
  | "periodic"
  | "discover"
  | "route_change"
  | "invalidate"
  | "route_garbage"
  | "timeout"
  | "neighbor_garbage"
  | "all_garbage"
  | "recovered"
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

export type RipConfig = {
  nodes: number;
  seed: number;
  topology: "default" | "rand";
  rand_fail_perc: number;
  pck_loss_perc: number;
  pck_invalid_perc: number;
  to_secs: number;
};
export type RipRoute = { dest: number; nxt_hop: number; metric: number };
export type RipNode = {
  id: number;
  is_running: boolean;
  is_quiescent: boolean;
  auto_restart_in: number | null;
  active_nbrs: number;
  garbage_nbrs: number;
  routes: RipRoute[];
  offline_dests: number[];
};
export type RipEdge = {
  a: number;
  b: number;
  cost: number;
};
export type RipEvent = {
  time: number;
  _type: RipEventType;
  router: number;
  peer: number | null;
  dest: number | null;
  old_metric: number | null;
  new_metric: number | null;
  old_hop: number | null;
  new_hop: number | null;
  msg: string;
};
export type RipStats = {
  updates_sent: number;
  updates_rcvd: number;
  pcks_dropped: number;
  pcks_lost: number;
  periodic_updates: number;
  triggered_updates: number;
  route_changes: number;
  invalids: number;
  garbage_collections: number;
  is_converged: boolean;
  accuracy: number;
  rip_reachability: number;
  shortest_path_cost_mean: number;
  shortest_path_cost_min: number;
  shortest_path_cost_max: number;
  convergence_secs: number | null;
};
export type RipSnapshot = {
  time: number;
  dur: number;
  hist_start: number;
  is_complete: boolean;
  nodes: RipNode[];
  edges: RipEdge[];
  events: RipEvent[];
  stats: RipStats;
};

export type RipHandle = {
  snapshot(): RipSnapshot;
  advance(secs: number): RipSnapshot;
  step_back(): RipSnapshot;
  seek_start(): RipSnapshot;
  seek_end(): RipSnapshot;
  terminate_router(id: number): RipSnapshot;
  restart_router(id: number): RipSnapshot;
  set_link_cost(a: number, b: number, cost: number): RipSnapshot;
  set_pck_loss_perc(perc: number): RipSnapshot;
  set_pck_invalid_perc(perc: number): RipSnapshot;
  free(): void;
};
