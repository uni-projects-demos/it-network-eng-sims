import type { FlowConfig, FlowHandle } from "#lib/sims/network/flow/types.ts";
import type { RipConfig, RipHandle } from "#lib/sims/network/rip/types.ts";
import { randSeed } from "#lib/sims/shared/random.ts";
import type { SimResTCP } from "#lib/sims/transport/typesTCP.ts";
import type { SimResUDP } from "#lib/sims/transport/typesUDP.ts";

// @ts-expect-error
type WasmModule = typeof import("./pkg/it_network_eng.js");

let wasm: WasmModule | null = null;
let initializing: Promise<WasmModule> | null = null;

async function loadEngine(): Promise<WasmModule> {
  if (wasm) return wasm;
  if (!initializing) {
    initializing = (async (): Promise<WasmModule> => {
      // @ts-expect-error
      const module: WasmModule = await import("./pkg/it_network_eng.js");
      await module.default();
      wasm = module;
      return module;
    })().catch((error: unknown): never => {
      initializing = null;
      throw error;
    });
  }
  return initializing;
}

export async function initEngine(): Promise<void> {
  await loadEngine();
}

export async function runSimTCP(
  msg: string,
  packetLossProb: number,
  bitErrProb: number,
  payloadSize: number,
  isHandshake: boolean,
): Promise<SimResTCP> {
  const module: WasmModule = await loadEngine();
  return module.run_tcp_sim(msg, packetLossProb, bitErrProb, payloadSize, isHandshake, randSeed());
}

export async function runSimUDP(
  msg: string,
  packetLossProb: number,
  payloadSize: number,
): Promise<SimResUDP> {
  const module: WasmModule = await loadEngine();
  return module.run_udp_sim(msg, packetLossProb, payloadSize, randSeed());
}

export async function runSimRIP(options: RipConfig): Promise<RipHandle> {
  const module: WasmModule = await loadEngine();
  return new module.SimRIP(options);
}

export async function runSimFlow(options: FlowConfig): Promise<FlowHandle> {
  const module: WasmModule = await loadEngine();
  return new module.SimFlow(options);
}

export async function genFlowLP(options: FlowConfig): Promise<string> {
  const module: WasmModule = await loadEngine();
  return module.gen_flow_lp(options);
}
