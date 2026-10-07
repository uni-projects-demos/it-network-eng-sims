import type { SimResTCP } from "#lib/sims/transport/typesTCP.ts";
import type { SimResUDP } from "#lib/sims/transport/typesUDP.ts";

type WasmModule = {
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
  const module = (await import("./pkg/it_network_eng.js")) as WasmModule;
  await module.default();
  wasm = module;
}

function randomSeed(): number {
  if (typeof globalThis.crypto?.getRandomValues === "function") {
    const vals: Uint32Array<ArrayBuffer> = new Uint32Array(1);
    globalThis.crypto.getRandomValues(vals);
    return vals[0] || 1;
  }

  return Math.floor(Math.random() * 0xffff_ffff) >>> 0 || 1;
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
    throw new Error("WASM engine failed to initialize");
  }

  return wasm.run_tcp_sim(msg, packetLossProb, bitErrProb, payloadSize, isHandshake, randomSeed());
}

export async function runSimUDP(
  msg: string,
  packetLossProb: number,
  payloadSize: number,
): Promise<SimResUDP> {
  await initEngine();

  if (wasm === null) {
    throw new Error("WASM engine failed to initialize");
  }

  return wasm.run_udp_sim(msg, packetLossProb, payloadSize, randomSeed());
}
