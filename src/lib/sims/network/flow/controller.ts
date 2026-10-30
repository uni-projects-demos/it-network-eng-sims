import { initEngine, runSimFlow } from "#lib/wasm/engine.ts";
import {
  FLOW_WORK_LIMIT,
  type FlowBenchmark,
  type FlowConfig,
  type FlowHandle,
  type FlowSnapshot,
} from "./types.ts";

export class FlowController {
  private gen: number = 0;

  constructor(private readonly onSnapshot: (snapshot: FlowSnapshot, computeMs: number) => void) {}

  async solve(config: FlowConfig): Promise<FlowBenchmark | null> {
    const gen: number = ++this.gen;
    await initEngine();
    if (gen !== this.gen) return null;
    const started: number = performance.now();
    const handle: FlowHandle = await runSimFlow(config);
    try {
      if (gen !== this.gen) return null;
      let res: FlowSnapshot = handle.snapshot();
      let computeMS: number = performance.now() - started;
      this.onSnapshot(res, computeMS);
      while (res.status === "feasible" && res.work < FLOW_WORK_LIMIT) {
        await new Promise<void>((resolve) => setTimeout(resolve, 0));
        if (gen !== this.gen) return null;
        const stepStarted: number = performance.now();
        res = handle.advance(Math.min(4096, FLOW_WORK_LIMIT - res.work));
        computeMS += performance.now() - stepStarted;
        this.onSnapshot(res, computeMS);
      }
      return gen === this.gen ? { res, rust_ms: computeMS } : null;
    } finally {
      handle.free();
    }
  }

  cancel(): void {
    ++this.gen;
  }
}
