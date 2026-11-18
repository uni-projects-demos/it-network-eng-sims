import { SimulationController } from "#lib/sims/shared/controller.ts";
import { runSimWSN } from "#lib/wasm/engine.ts";
import type { WsnConfig, WsnHandle, WsnMac, WsnSnapshot } from "./types.ts";

export class WsnController extends SimulationController<WsnConfig, WsnSnapshot, WsnHandle> {
  constructor(
    onSnapshot: (snapshot: WsnSnapshot) => void,
    onError: (message: string) => void,
    onBranch: (time: number) => void,
    onPlaying: (playing: boolean) => void,
  ) {
    super(runSimWSN, onSnapshot, onError, onBranch, onPlaying);
  }

  protected override allowEditingComplete(): boolean {
    return true;
  }

  moveNode(id: number, x: number, y: number): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.move_node(id, x, y));
  }
  setNodeEnabled(id: number, enabled: boolean): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.set_node_enabled(id, enabled));
  }
  rechargeNode(id: number): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.recharge_node(id));
  }
  setMac(mac: WsnMac): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.set_mac(mac));
  }
  setTxPower(value: number): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.set_tx_power(value));
  }
  setNoise(value: number): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.set_noise(value));
  }
  setDuty(value: number): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.set_duty(value));
  }
  setInterval(value: number): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.set_interval(value));
  }
  setNodes(value: number): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.set_nodes(value));
  }
  setBattery(value: number): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.set_battery(value));
  }
  setDuration(value: number): void {
    this.edit((handle: WsnHandle): WsnSnapshot => handle.set_duration(value));
  }
}
