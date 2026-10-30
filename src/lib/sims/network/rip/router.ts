import type { RipNode, RipRoute } from "./types.ts";

export type RouterCol = Readonly<{
  fill: string;
  light: string;
  dark: string;
}>;

export const ROUTER_COLS: readonly RouterCol[] = [
  { fill: "#D0F1E6", light: "#1C614A", dark: "#A1E3CD" }, // seafoam
  { fill: "#F8EABA", light: "#715A0C", dark: "#F4DD90" }, // butter
  { fill: "#DBC7F9", light: "#340C71", dark: "#B890F3" }, // violet
  { fill: "#F8C2BB", light: "#71170C", dark: "#F49B90" }, // coral
  { fill: "#C0E6F6", light: "#10526D", dark: "#94D5F0" }, // sky
  { fill: "#D7F4CA", light: "#2F6715", dark: "#B2EA9A" }, // sage
  { fill: "#F7BBDA", light: "#700D40", dark: "#F291C3" }, // rose
  { fill: "#B8F7DA", light: "#0D7042", dark: "#91F3C6" }, // mint
  { fill: "#DCDFF6", light: "#1A2263", dark: "#9EA6E6" }, // periwinkle
  { fill: "#F3DECD", light: "#653A18", dark: "#E7BE9C" }, // apricot
  { fill: "#BCD0F8", light: "#0C2E71", dark: "#90B1F3" }, // blue
  { fill: "#F7BBF6", light: "#700D6E", dark: "#F291F1" }, // orchid
  { fill: "#EEF4DA", light: "#51611C", dark: "#D4E3A1" }, // celery
  { fill: "#FAD9F2", light: "#6E0F57", dark: "#F193DA" }, // pink
  { fill: "#B8F7B9", light: "#0D700E", dark: "#91F393" }, // spring green
];

export function routerCol(routerId: number): RouterCol | undefined {
  return Number.isInteger(routerId) && routerId >= 1 && routerId <= ROUTER_COLS.length
    ? ROUTER_COLS[routerId - 1]
    : undefined;
}

export function routerBadgeStyle(routerId: number): string {
  const fill: string | undefined = routerCol(routerId)?.fill;
  return fill
    ? `background-color: ${fill}; color: #16202c`
    : "background-color: var(--selected-bg); color: var(--selected-text)";
}

export type RouteStatus =
  | "Offline"
  | "Unreachable (invalid)"
  | "Reachable (direct)"
  | "Reachable (indirect)";

export function routeStatus(route: RipRoute): RouteStatus {
  if (route.metric >= 16) return "Unreachable (invalid)";
  return route.nxt_hop === route.dest ? "Reachable (direct)" : "Reachable (indirect)";
}

export type DisplayRoute = RipRoute & Readonly<{ isOffline: boolean }>;

export function routingTableRows(router: RipNode | undefined): DisplayRoute[] {
  if (!router) return [];
  const activeDests: Set<number> = new Set(
    router.routes.map((route: RipRoute): number => route.dest),
  );
  return [
    ...router.routes.map((route: RipRoute): DisplayRoute => ({ ...route, isOffline: false })),
    ...router.offline_dests
      .filter((dest: number): boolean => !activeDests.has(dest))
      .map((dest: number): DisplayRoute => ({ dest, nxt_hop: 0, metric: 16, isOffline: true })),
  ].sort(
    (a: DisplayRoute, b: DisplayRoute): number =>
      Number(a.isOffline) - Number(b.isOffline) || a.dest - b.dest,
  );
}

export function selectedRouteMetric(router: RipNode | undefined, dest: number): string {
  if (!router?.is_running || dest < 1) return "—";
  if (router.id === dest) return "0";
  const route: RipRoute | undefined = router.routes.find(
    (candidate: RipRoute): boolean => candidate.dest === dest,
  );
  return route && route.metric < 16 ? String(route.metric) : "∞";
}
