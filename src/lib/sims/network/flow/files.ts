import type { FlowConfig } from "./types.ts";

export function flowFilename(cfg: FlowConfig): string {
  return `X=${cfg.srcs}Y=${cfg.trans}Z=${cfg.dests}.lp`;
}
