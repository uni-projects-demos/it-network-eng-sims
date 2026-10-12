import type { RipConfig, RipHandle } from "#lib/sims/network/types.ts";
import { randSeed } from "#lib/sims/shared/random.ts";
import type { SimResTCP } from "#lib/sims/transport/typesTCP.ts";
import type { SimResUDP } from "#lib/sims/transport/typesUDP.ts";

type WasmModule = {
  SimRIP: new (options: RipConfig) => RipHandle;
  default: () => Promise<unknown>;
  run_tcp_sim: (
    msg: string,
    packetLossProb: number,
    bitErrProb: number,
    payloadSize: number,
    isHandshake: boolean,
    seed: number,
  ) => SimResTCP;
  run_udp_sim: (
    msg: string,
    packetLossProb: number,
    payloadSize: number,
    seed: number,
  ) => SimResUDP;
};

let wasm: WasmModule | null = null;

export async function initEngine(): Promise<void> {
  if (wasm) return;
  // @ts-expect-error
  const module: WasmModule = (await import("./pkg/it_network_eng.js")) as WasmModule;
  await module.default();
  wasm = module;
}

export async function runSimTCP(
  msg: string,
  packetLossProb: number,
  bitErrProb: number,
  payloadSize: number,
  isHandshake: boolean,
): Promise<SimResTCP> {
  await initEngine();

  if (wasm === null) {
    throw new Error("WASM engine failed to initialize.");
  }

  return wasm.run_tcp_sim(msg, packetLossProb, bitErrProb, payloadSize, isHandshake, randSeed());
}

export async function runSimUDP(
  msg: string,
  packetLossProb: number,
  payloadSize: number,
): Promise<SimResUDP> {
  await initEngine();

  if (wasm === null) {
    throw new Error("WASM engine failed to initialize.");
  }

  return wasm.run_udp_sim(msg, packetLossProb, payloadSize, randSeed());
}

export async function runSimRIP(options: RipConfig): Promise<RipHandle> {
  await initEngine();

  if (wasm === null) {
    throw new Error("WASM engine failed to initialize.");
  }

  return new wasm.SimRIP(options);
}
