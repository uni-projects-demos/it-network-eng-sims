export interface FlowConfig {
  srcs: number;
  trans: number;
  dests: number;
  seed: number;
}

export const FLOW_COUNT_FIELDS = ["srcs", "trans", "dests"] as const;
export type FlowCountField = (typeof FLOW_COUNT_FIELDS)[number];
export const FLOW_MAX_NODES: number = 32;
export const FLOW_MIN_COUNTS: Readonly<Record<FlowCountField, number>> = {
  srcs: 1,
  trans: 3,
  dests: 1,
};

export interface FlowRoute {
  src: number;
  dest: number;
  demand: number;
  trans: [number, number, number];
  flow_per_path: number;
}

export interface FlowLink {
  from: string;
  to: string;
  cap: number;
}

export interface FlowSnapshot {
  config: FlowConfig;
  status: "optimal" | "feasible" | "infeasible";
  phase: "search" | "complete";
  work: number;
  search_nodes: number;
  improvements: number;
  lower_bound: number;
  obj: number | null | undefined;
  gap: number | null | undefined;
  total_demand: number;
  loads: number[];
  routes: FlowRoute[];
  links: FlowLink[];
  nonzero_links: number;
  max_cap: number | null | undefined;
  highest_links: FlowLink[];
}

export interface FlowHandle {
  snapshot: () => FlowSnapshot;
  advance: (work: number) => FlowSnapshot;
  free: () => void;
}

export interface FlowBenchmark {
  res: FlowSnapshot;
  rust_ms: number;
}

export const FLOW_WORK_LIMIT: number = 100_000_000;

export const defaultFlowConfig = (): FlowConfig => ({
  srcs: 7,
  trans: 7,
  dests: 7,
  seed: 1,
});

export class FlowConfigErr extends Error {
  constructor(
    msg: string,
    public readonly field?: FlowCountField,
  ) {
    super(msg);
    this.name = "FlowConfigError";
  }
}

function validateCount(val: unknown, field: FlowCountField): number {
  if (typeof val !== "number" || !Number.isInteger(val) || val < 1 || val > FLOW_MAX_NODES) {
    throw new FlowConfigErr(`${field} must be an integer from 1 to ${FLOW_MAX_NODES}.`, field);
  }
  return val;
}

export function validateFlowConfig(val: unknown): FlowConfig {
  if (
    !val ||
    typeof val !== "object" ||
    Array.isArray(val) ||
    !("srcs" in val) ||
    !("trans" in val) ||
    !("dests" in val)
  ) {
    throw new FlowConfigErr("Expected a JSON object with srcs, trans, dests and optional seed.");
  }
  const srcs: number = validateCount(val.srcs, "srcs");
  const trans: number = validateCount(val.trans, "trans");
  const dests: number = validateCount(val.dests, "dests");
  const seed: unknown = "seed" in val ? (val.seed ?? 1) : 1;
  if (typeof seed !== "number" || !Number.isInteger(seed) || seed < 0 || seed > 0xffffffff) {
    throw new Error("seed must be an integer from 0 to 4294967295.");
  }
  return { srcs, trans, dests, seed };
}
