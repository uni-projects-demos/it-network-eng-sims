import { DEFAULT_DURATION_SECS } from "../../shared/duration.ts";

export type WsnMac = "csma" | "tdma";

export type WsnConfig = {
  nodes: number;
  seed: number;
  mac: WsnMac;
  tx_power_dbm: number;
  noise_floor_dbm: number;
  duty_cycle_pct: number;
  data_interval_secs: number;
  initial_battery_mj: number;
  to_secs: number;
};

export type WsnNode = {
  id: number;
  x: number;
  y: number;
  battery_pct: number;
  battery_mj: number;
  is_active: boolean;
  is_awake: boolean;
  queue_len: number;
  next_hop: number | null;
  route_cost: number | null;
};

export type WsnLink = { a: number; b: number; snr_db: number; success_probability: number };
export type WsnTransmission = {
  from: number;
  to: number;
  outcome: "success" | "collision" | "error" | "sleep";
};
export type WsnEventType =
  | "startup"
  | "sample"
  | "route"
  | "retry"
  | "collision"
  | "error"
  | "deliver"
  | "forward"
  | "drop"
  | "depleted"
  | "topology"
  | "failure"
  | "restart"
  | "config";
export type WsnEvent = {
  time: number;
  _type: WsnEventType;
  node: number;
  peer: number | null;
  msg: string;
};
export type WsnStats = {
  generated: number;
  delivered: number;
  forwarded: number;
  transmissions: number;
  collisions: number;
  errors: number;
  retries: number;
  dropped: number;
  energy_mj: number;
  delivery_pct: number;
  avg_latency_secs: number;
  active_nodes: number;
  sleeping_nodes: number;
  depleted_nodes: number;
  queued_frames: number;
  first_depletion_secs: number | null;
  latency_total_secs: number;
};
export type WsnSnapshot = {
  time: number;
  dur: number;
  hist_start: number;
  is_complete: boolean;
  nodes: WsnNode[];
  links: WsnLink[];
  transmissions: WsnTransmission[];
  events: WsnEvent[];
  stats: WsnStats;
  mac: WsnMac;
};
export type WsnHandle = {
  snapshot(): WsnSnapshot;
  advance(secs: number): WsnSnapshot;
  step_back(): WsnSnapshot;
  seek_start(): WsnSnapshot;
  seek_end(): WsnSnapshot;
  move_node(id: number, x: number, y: number): WsnSnapshot;
  set_node_enabled(id: number, enabled: boolean): WsnSnapshot;
  recharge_node(id: number): WsnSnapshot;
  set_mac(mac: WsnMac): WsnSnapshot;
  set_tx_power(value: number): WsnSnapshot;
  set_noise(value: number): WsnSnapshot;
  set_duty(value: number): WsnSnapshot;
  set_interval(value: number): WsnSnapshot;
  set_nodes(value: number): WsnSnapshot;
  set_battery(value: number): WsnSnapshot;
  set_duration(value: number): WsnSnapshot;
  free(): void;
};

export function defaultWsnConfig(seed: number): WsnConfig {
  return {
    nodes: 9,
    seed,
    mac: "csma",
    tx_power_dbm: 0,
    noise_floor_dbm: -95,
    duty_cycle_pct: 80,
    data_interval_secs: 2,
    initial_battery_mj: 2000,
    to_secs: DEFAULT_DURATION_SECS,
  };
}
