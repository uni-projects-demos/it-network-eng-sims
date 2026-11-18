import { runSimOSPF } from "#lib/wasm/engine.ts";
import { RoutingController } from "../routing/controller.ts";
import type { OspfSnapshot } from "./types.ts";

export class OspfController extends RoutingController<OspfSnapshot> {
  constructor(
    onSnapshot: (snapshot: OspfSnapshot) => void,
    onError: (err: string) => void,
    onBranch: (time: number) => void,
    onPlaying: (isPlaying: boolean) => void,
  ) {
    super(runSimOSPF, onSnapshot, onError, onBranch, onPlaying);
  }
}
