import { runSimRIP } from "#lib/wasm/engine.ts";
import { RoutingController } from "../routing/controller.ts";
import type { RipSnapshot } from "./types.ts";

export class RipController extends RoutingController<RipSnapshot> {
  constructor(
    onSnapshot: (snapshot: RipSnapshot) => void,
    onError: (err: string) => void,
    onBranch: (time: number) => void,
    onPlaying: (isPlaying: boolean) => void,
  ) {
    super(runSimRIP, onSnapshot, onError, onBranch, onPlaying);
  }
}
