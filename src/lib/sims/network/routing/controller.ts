import { SimulationController } from "#lib/sims/shared/controller.ts";
import type { RoutingConfig, RoutingHandle, RoutingSnapshot } from "./types.ts";

export class RoutingController<TSnapshot extends RoutingSnapshot> extends SimulationController<
  RoutingConfig,
  TSnapshot,
  RoutingHandle<TSnapshot>
> {
  constructor(
    create: (config: RoutingConfig) => Promise<RoutingHandle<TSnapshot>>,
    onSnapshot: (snapshot: TSnapshot) => void,
    onError: (message: string) => void,
    onBranch: (time: number) => void,
    onPlaying: (playing: boolean) => void,
  ) {
    super(create, onSnapshot, onError, onBranch, onPlaying);
  }

  terminate(id: number): void {
    this.edit((handle: RoutingHandle<TSnapshot>): TSnapshot => handle.terminate_router(id));
  }
  restart(id: number): void {
    this.edit((handle: RoutingHandle<TSnapshot>): TSnapshot => handle.restart_router(id));
  }
  setLinkCost(a: number, b: number, cost: number): void {
    this.edit((handle: RoutingHandle<TSnapshot>): TSnapshot => handle.set_link_cost(a, b, cost));
  }
  setPacketLoss(percent: number): void {
    this.edit((handle: RoutingHandle<TSnapshot>): TSnapshot => handle.set_pck_loss_perc(percent));
  }
  setPacketInvalid(percent: number): void {
    this.edit(
      (handle: RoutingHandle<TSnapshot>): TSnapshot => handle.set_pck_invalid_perc(percent),
    );
  }
}
