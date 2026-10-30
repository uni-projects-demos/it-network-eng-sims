import type { RipConfig, RipEdge, RipNode, RipRoute } from "./types.ts";

export type Pos = { id: number; x: number; y: number };
type XY = { x: number; y: number };
type EdgeEndpoints = Pick<RipEdge, "a" | "b">;

const DEFAULT_WIDTH: number = 960;
const DEFAULT_HEIGHT: number = 585;
const EPSILON: 1e-7 = 1e-7;

function reqPos(points: ReadonlyMap<number, XY>, routerId: number): XY {
  const point: XY | undefined = points.get(routerId);
  if (!point) throw new Error(`Missing layout position for router ${routerId}.`);
  return point;
}

function cross(a: XY, b: XY, c: XY): number {
  return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
}

function segmentsCross(a: XY, b: XY, c: XY, d: XY): boolean {
  const abC: number = cross(a, b, c),
    abD: number = cross(a, b, d);
  const cdA: number = cross(c, d, a),
    cdB: number = cross(c, d, b);
  if (abC * abD < -EPSILON && cdA * cdB < -EPSILON) return true;
  const on: (p: XY, q: XY, r: XY) => boolean = (p: XY, q: XY, r: XY): boolean =>
    Math.abs(cross(p, q, r)) < EPSILON &&
    r.x >= Math.min(p.x, q.x) - EPSILON &&
    r.x <= Math.max(p.x, q.x) + EPSILON &&
    r.y >= Math.min(p.y, q.y) - EPSILON &&
    r.y <= Math.max(p.y, q.y) + EPSILON;
  return on(a, b, c) || on(a, b, d) || on(c, d, a) || on(c, d, b);
}

export function hasEdgeCrossings(positions: Pos[], edges: EdgeEndpoints[]): boolean {
  const points: Map<number, Pos> = new Map(positions.map((p: Pos): [number, Pos] => [p.id, p]));
  for (let i: number = 0; i < edges.length; i++) {
    const first: EdgeEndpoints = edges[i];
    const a: Pos | undefined = points.get(first.a),
      b: Pos | undefined = points.get(first.b);
    if (!a || !b) continue;
    for (let j: number = i + 1; j < edges.length; j++) {
      const second: EdgeEndpoints = edges[j];
      if (
        first.a === second.a ||
        first.a === second.b ||
        first.b === second.a ||
        first.b === second.b
      )
        continue;
      const c: Pos | undefined = points.get(second.a),
        d: Pos | undefined = points.get(second.b);
      if (c && d && segmentsCross(a, b, c, d)) return true;
    }
  }
  return false;
}

function pointToSegment(p: XY, a: XY, b: XY): number {
  const dx: number = b.x - a.x,
    dy: number = b.y - a.y;
  const t: number = Math.max(
    0,
    Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / Math.max(1, dx * dx + dy * dy)),
  );
  return Math.hypot(p.x - a.x - t * dx, p.y - a.y - t * dy);
}

function layoutPenalty(
  points: Map<number, XY>,
  edges: RipEdge[],
  width: number,
  height: number,
  nodeRadius: number,
): number {
  const vals: XY[] = [...points.values()];
  const areaPerNode: number = Math.sqrt(
    Math.max(1, (width - 120) * (height - 120)) / Math.max(1, vals.length),
  );
  const preferredSpacing: number = Math.max(nodeRadius * 2 + 15, Math.min(125, areaPerNode * 0.9));
  let penalty: number = 0;

  const inset: number = nodeRadius + 27;
  const xs: number[] = vals.map((p: XY): number => p.x),
    ys: number[] = vals.map((p: XY): number => p.y);
  const emptyMargins: number[] = [
    Math.min(...xs) - inset,
    width - inset - Math.max(...xs),
    Math.min(...ys) - inset,
    height - inset - Math.max(...ys),
  ];
  for (const margin of emptyMargins) {
    if (margin > 18) penalty += (margin - 18) ** 2 * 0.9;
  }

  for (let i: number = 0; i < vals.length; i++) {
    let nearest: number = Infinity;
    for (let j: number = 0; j < vals.length; j++) {
      if (i === j) continue;
      nearest = Math.min(nearest, Math.hypot(vals[i].x - vals[j].x, vals[i].y - vals[j].y));
    }
    if (nearest < preferredSpacing) penalty += (preferredSpacing - nearest) ** 2 * 5;
  }

  for (const edge of edges) {
    const a: XY = reqPos(points, edge.a),
      b: XY = reqPos(points, edge.b);
    const dx: number = b.x - a.x,
      dy: number = b.y - a.y;
    const length: number = Math.max(1, Math.hypot(dx, dy));
    const lblExtent: number = (30 * Math.abs(dx) + 17 * Math.abs(dy)) / length;
    const endpointGap: number = length / 2 - nodeRadius - lblExtent;
    if (endpointGap < 18) penalty += (18 - endpointGap) ** 2 * 12;
    for (const [id, p] of points) {
      if (id === edge.a || id === edge.b) continue;
      const gap: number = pointToSegment(p, a, b);
      const desired: number = nodeRadius + 27;
      if (gap < desired) penalty += (desired - gap) ** 2 * 7;
    }
  }

  const lbls: { x: number; y: number; edge: RipEdge }[] = edges.map(
    (e: RipEdge): { x: number; y: number; edge: RipEdge } => {
      const a: XY = reqPos(points, e.a),
        b: XY = reqPos(points, e.b);
      return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2, edge: e };
    },
  );
  for (let i: number = 0; i < lbls.length; i++) {
    const lbl: { x: number; y: number; edge: RipEdge } = lbls[i];
    for (let j: number = i + 1; j < lbls.length; j++) {
      const gap: number = Math.hypot(lbl.x - lbls[j].x, lbl.y - lbls[j].y);
      if (gap < 47) penalty += (47 - gap) ** 2 * 4;
    }
    for (const [id, p] of points) {
      if (id === lbl.edge.a || id === lbl.edge.b) continue;
      const gap: number = Math.hypot(lbl.x - p.x, lbl.y - p.y);
      if (gap < nodeRadius + 29) penalty += (nodeRadius + 29 - gap) ** 2 * 5;
    }
  }
  return penalty;
}

export function assignmentPositions(w: number, h: number, nodeRadius: number): Pos[] {
  const reference: Pos[] = [
    { id: 1, x: 0.18, y: 0.15 },
    { id: 2, x: 0.72, y: 0.06 },
    { id: 3, x: 0.91, y: 0.34 },
    { id: 4, x: 0.88, y: 0.76 },
    { id: 5, x: 0.52, y: 0.95 },
    { id: 6, x: 0.09, y: 0.74 },
    { id: 7, x: 0.49, y: 0.47 },
  ];
  const margin: number = nodeRadius + 30;
  const availableWidth: number = Math.max(1, w - 2 * margin);
  const availableHeight: number = Math.max(1, h - 2 * margin);
  return reference.map(({ id, x, y }: Pos): { id: number; x: number; y: number } => ({
    id,
    x: margin + x * availableWidth,
    y: margin + y * availableHeight,
  }));
}

export function nodePositions(
  nodes: RipNode[],
  edges: RipEdge[],
  width: number = DEFAULT_WIDTH,
  height: number = DEFAULT_HEIGHT,
  nodeRadius: number = 31,
  topology: RipConfig["topology"] = "rand",
): Pos[] {
  if (!nodes.length) return [];
  if (topology === "default") {
    if (nodes.length !== 7) throw new Error("The default layout requires seven routers.");
    return assignmentPositions(Math.max(200, width), Math.max(300, height), nodeRadius);
  }
  width = Math.max(200, width);
  height = Math.max(300, height);
  const adjacency: Map<number, number[]> = new Map<number, number[]>(
    nodes.map((node: RipNode): [number, number[]] => [node.id, []]),
  );
  for (const edge of edges) {
    adjacency.get(edge.a)?.push(edge.b);
    adjacency.get(edge.b)?.push(edge.a);
  }
  for (const neighbors of adjacency.values())
    neighbors.sort((a: number, b: number): number => a - b);

  const degrees: Map<number, number> = new Map(
    [...adjacency].map(([id, neighbors]: [number, number[]]): [number, number] => [
      id,
      neighbors.length,
    ]),
  );
  const queue: number[] = [...degrees]
    .filter(([, degree]: [number, number]): boolean => degree <= 1)
    .map(([id]: [number, number]): number => id);
  const cycle: Set<number> = new Set(adjacency.keys());
  for (let i: number = 0; i < queue.length; i++) {
    const id: number = queue[i];
    cycle.delete(id);
    for (const nxt of adjacency.get(id) ?? []) {
      const degree: number = (degrees.get(nxt) ?? 0) - 1;
      degrees.set(nxt, degree);
      if (degree === 1) queue.push(nxt);
    }
  }
  if (cycle.size < 3) {
    const padding: number = nodeRadius + 25;
    return nodes.map((node: RipNode, i: number): { id: number; x: number; y: number } => ({
      id: node.id,
      x: width / 2 + Math.cos((i * 2 * Math.PI) / nodes.length) * (width / 2 - padding),
      y: height / 2 + Math.sin((i * 2 * Math.PI) / nodes.length) * (height / 2 - padding),
    }));
  }

  const start: number = Math.min(...cycle);
  const order: number[] = [start];
  if (edges.length > nodes.length) {
    const visited: Set<number> = new Set([start]);
    function findCycle(id: number): boolean {
      if (order.length === cycle.size) return (adjacency.get(id) ?? []).includes(start);
      for (const nxt of adjacency.get(id) ?? []) {
        if (!cycle.has(nxt) || visited.has(nxt)) continue;
        visited.add(nxt);
        order.push(nxt);
        if (findCycle(nxt)) return true;
        order.pop();
        visited.delete(nxt);
      }
      return false;
    }
    if (!findCycle(start)) throw new Error("Unable to find RIP topology cycle.");
  } else {
    let prev: number = -1,
      cur: number = start;
    do {
      const nxt: number | undefined = (adjacency.get(cur) ?? []).find(
        (id: number): boolean => cycle.has(id) && id !== prev,
      );
      if (nxt == null || nxt === start) break;
      order.push(nxt);
      prev = cur;
      cur = nxt;
    } while (order.length < cycle.size);
  }

  const children: Map<number, number[]> = new Map<number, number[]>();
  const weight: Map<number, number> = new Map<number, number>();
  function buildTree(id: number, parent: number): number {
    const descendants: number[] = (adjacency.get(id) ?? []).filter(
      (n: number): boolean => n !== parent && !cycle.has(n),
    );
    children.set(id, descendants);
    const total: number = descendants.reduce(
      (sum: number, child: number): number => sum + buildTree(child, id),
      0,
    );
    const result: number = Math.max(1, total);
    weight.set(id, result);
    return result;
  }
  for (const id of order) buildTree(id, -1);

  const count: number = order.length;
  const sector: number = (2 * Math.PI) / count;
  const branchPadding: number = nodeRadius + 27;
  const radialStep: number = 92;
  const cycleRadius: number = 90 + count * 13;
  const rotations: number = 72;
  let best: { points: Map<number, XY>; score: number } | undefined;

  for (let trial: number = 0; trial < rotations; trial++) {
    const rotation: number = -Math.PI / 2 + (trial * 2 * Math.PI) / rotations;
    const raw: Map<number, XY> = new Map<number, XY>();
    function placeBranch(id: number, depth: number, low: number, high: number): void {
      const subs: number[] = children.get(id) ?? [];
      const total: number = subs.reduce(
        (sum: number, child: number): number => sum + (weight.get(child) ?? 1),
        0,
      );
      let angle: number = low;
      for (const child of subs) {
        const span: number = ((high - low) * (weight.get(child) ?? 1)) / total;
        const mid: number = angle + span / 2;
        const radius: number = cycleRadius + (depth + 1) * radialStep;
        raw.set(child, { x: Math.cos(mid) * radius, y: Math.sin(mid) * radius });
        placeBranch(child, depth + 1, angle, angle + span);
        angle += span;
      }
    }
    order.forEach((id: number, i: number): void => {
      const angle: number = rotation + i * sector;
      raw.set(id, { x: Math.cos(angle) * cycleRadius, y: Math.sin(angle) * cycleRadius });
      placeBranch(id, 0, angle - sector * 0.46, angle + sector * 0.46);
    });

    const vals: XY[] = [...raw.values()];
    const xMin: number = Math.min(...vals.map((p: XY): number => p.x)),
      xMax: number = Math.max(...vals.map((p: XY): number => p.x));
    const yMin: number = Math.min(...vals.map((p: XY): number => p.y)),
      yMax: number = Math.max(...vals.map((p: XY): number => p.y));
    const pad: number = Math.max(nodeRadius + 19, Math.min(70, branchPadding));
    const scaleX: number = (width - pad * 2) / Math.max(1, xMax - xMin);
    const scaleY: number = (height - pad * 2) / Math.max(1, yMax - yMin);
    const points: Map<number, { x: number; y: number }> = new Map(
      [...raw].map(([id, p]: [number, XY]): [number, { x: number; y: number }] => [
        id,
        {
          x: pad + (p.x - xMin) * scaleX,
          y: pad + (p.y - yMin) * scaleY,
        },
      ]),
    );
    const positioned: { x: number; y: number; id: number }[] = nodes.map(
      (node: RipNode): { x: number; y: number; id: number } => ({
        id: node.id,
        ...reqPos(points, node.id),
      }),
    );
    if (hasEdgeCrossings(positioned, edges)) continue;
    const score: number = layoutPenalty(points, edges, width, height, nodeRadius);
    if (!best || score < best.score) best = { points, score };
  }

  if (best) {
    const bestLayout: { points: Map<number, XY>; score: number } = best;
    const fixed: Set<number> = new Set(order);
    const cycleCenter: { x: number; y: number } = {
      x:
        order.reduce(
          (total: number, id: number): number => total + reqPos(bestLayout.points, id).x,
          0,
        ) / count,
      y:
        order.reduce(
          (total: number, id: number): number => total + reqPos(bestLayout.points, id).y,
          0,
        ) / count,
    };
    const root: Map<number, number> = new Map<number, number>();
    function markRoot(id: number, anchor: number): void {
      root.set(id, anchor);
      for (const child of children.get(id) ?? []) markRoot(child, anchor);
    }
    for (const id of order) markRoot(id, id);
    const points: Map<number, XY> = best.points;
    const padding: number = nodeRadius + 20;
    let curPenalty: number = best.score;
    const dirs: { x: number; y: number }[] = Array.from(
      { length: 12 },
      (_: unknown, i: number): { x: number; y: number } => ({
        x: Math.cos((i * Math.PI) / 6),
        y: Math.sin((i * Math.PI) / 6),
      }),
    );
    for (const step of [45, 26, 14, 7, 3]) {
      for (let sweep: number = 0; sweep < 2; sweep++) {
        for (const node of nodes) {
          if (fixed.has(node.id)) continue;
          const original: XY = reqPos(points, node.id);
          const anchorId: number | undefined = root.get(node.id);
          if (anchorId === undefined) throw new Error(`Missing branch root for router ${node.id}.`);
          const anchor: XY = reqPos(points, anchorId);
          const away: { x: number; y: number } = {
            x: anchor.x - cycleCenter.x,
            y: anchor.y - cycleCenter.y,
          };
          let choice: XY = original;
          let choicePenalty: number = curPenalty;
          for (const dir of dirs) {
            const proposed: { x: number; y: number } = {
              x: original.x + dir.x * step,
              y: original.y + dir.y * step,
            };
            if (
              proposed.x < padding ||
              proposed.x > width - padding ||
              proposed.y < padding ||
              proposed.y > height - padding
            )
              continue;
            if ((proposed.x - anchor.x) * away.x + (proposed.y - anchor.y) * away.y < -EPSILON)
              continue;
            points.set(node.id, proposed);
            if (
              hasEdgeCrossings(
                nodes.map((n: RipNode): { x: number; y: number; id: number } => ({
                  id: n.id,
                  ...reqPos(points, n.id),
                })),
                edges,
              )
            )
              continue;
            const score: number = layoutPenalty(points, edges, width, height, nodeRadius);
            if (score + 1e-5 < choicePenalty) {
              choice = proposed;
              choicePenalty = score;
            }
          }
          points.set(node.id, choice);
          curPenalty = choicePenalty;
        }
      }
    }
  }

  if (!best) {
    throw new Error("RIP topology must not have crossing links.");
  }
  return nodes.map((node: RipNode): { x: number; y: number; id: number } => ({
    id: node.id,
    ...reqPos(best.points, node.id),
  }));
}

export function selectedRoute(nodes: RipNode[], from: number, to: number): number[] {
  if (from === to) return [from];
  const visited: Set<number> = new Set<number>();
  const hops: number[] = [from];
  let cur: number = from;
  for (let i: number = 0; i < nodes.length; i++) {
    const node: RipNode | undefined = nodes.find((r: RipNode): boolean => r.id === cur);
    if (!node?.is_running || visited.has(cur)) return [];
    visited.add(cur);
    const nxt: RipRoute | undefined = node.routes.find((r: RipRoute): boolean => r.dest === to);
    if (!nxt || nxt.metric >= 16) return [];
    cur = nxt.nxt_hop;
    if (cur === to) return [...hops, to];
    hops.push(cur);
  }
  return [];
}

export function isHighlighted(edge: RipEdge, path: number[]): boolean {
  for (let i: number = 1; i < path.length; i++) {
    if (
      (edge.a === path[i - 1] && edge.b === path[i]) ||
      (edge.b === path[i - 1] && edge.a === path[i])
    )
      return true;
  }
  return false;
}

export function formatDur(secs: number): string {
  const time: number = Math.floor(secs);
  return `${String(Math.floor(time / 60)).padStart(2, "0")}:${String(time % 60).padStart(2, "0")}`;
}
