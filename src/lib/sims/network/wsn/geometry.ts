export type Pnt = Readonly<{ x: number; y: number }>;
export type Bounds = Readonly<{ width: number; height: number }>;

const MODEL = { left: 28, right: 392, top: 30, bottom: 270 } as const;

function padding(view: Bounds): number {
  return Math.min(64, Math.max(52, Math.min(view.width, view.height) * 0.095));
}

export function networkToScreen(pnt: Pnt, view: Bounds): Pnt {
  const inset: number = padding(view);
  return {
    x:
      inset +
      ((pnt.x - MODEL.left) / (MODEL.right - MODEL.left)) * Math.max(1, view.width - 2 * inset),
    y:
      inset +
      ((pnt.y - MODEL.top) / (MODEL.bottom - MODEL.top)) * Math.max(1, view.height - 2 * inset),
  };
}

export function screenToNetwork(pnt: Pnt, view: Bounds): Pnt {
  const inset: number = padding(view);
  return {
    x: Math.max(
      MODEL.left,
      Math.min(
        MODEL.right,
        MODEL.left +
          ((pnt.x - inset) / Math.max(1, view.width - 2 * inset)) * (MODEL.right - MODEL.left),
      ),
    ),
    y: Math.max(
      MODEL.top,
      Math.min(
        MODEL.bottom,
        MODEL.top +
          ((pnt.y - inset) / Math.max(1, view.height - 2 * inset)) * (MODEL.bottom - MODEL.top),
      ),
    ),
  };
}

export function networkEdgeId(a: number, b: number): string {
  return `${Math.min(a, b)}:${Math.max(a, b)}`;
}

export function pathToGateway<T extends { id: number; next_hop: number | null }>(
  nodes: readonly T[],
  src: number,
): number[] {
  const byId = new Map<number, T>(nodes.map((node: T): [number, T] => [node.id, node]));
  const visited = new Set<number>();
  const route: number[] = [];
  let cur: number | null = src;
  while (cur !== null && !visited.has(cur) && byId.has(cur)) {
    route.push(cur);
    if (cur === 0) return route;
    visited.add(cur);
    cur = byId.get(cur)?.next_hop ?? null;
  }
  return [];
}
