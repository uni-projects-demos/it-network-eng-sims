export function randSeed(): number {
  if (typeof globalThis.crypto?.getRandomValues === "function") {
    const vals: Uint32Array<ArrayBuffer> = new Uint32Array(1);
    globalThis.crypto.getRandomValues(vals);
    return vals[0] || 1;
  }
  return Math.floor(Math.random() * 0x1_0000_0000) >>> 0 || 1;
}
