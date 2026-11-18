export { routeInfinity } from "./types.ts";

import {
  type RoutingNode,
  type RoutingProtocol,
  type RoutingRoute,
  routeInfinity,
} from "./types.ts";

export type RouteStatus =
  | "Offline"
  | "Unreachable (invalid)"
  | "Reachable (direct)"
  | "Reachable (indirect)";

export function routeStatus(route: RoutingRoute, protocol: RoutingProtocol): RouteStatus {
  if (route.metric >= routeInfinity(protocol)) return "Unreachable (invalid)";
  return route.nxt_hop === route.dest ? "Reachable (direct)" : "Reachable (indirect)";
}

export type DisplayRoute = RoutingRoute & Readonly<{ isOffline: boolean }>;

export function routingTableRows(
  router: RoutingNode | undefined,
  protocol: RoutingProtocol,
): DisplayRoute[] {
  if (!router) return [];
  const activeDests: Set<number> = new Set(
    router.routes.map((route: RoutingRoute): number => route.dest),
  );
  return [
    ...router.routes.map((route: RoutingRoute): DisplayRoute => ({ ...route, isOffline: false })),
    ...router.offline_dests
      .filter((dest: number): boolean => !activeDests.has(dest))
      .map(
        (dest: number): DisplayRoute => ({
          dest,
          nxt_hop: 0,
          metric: routeInfinity(protocol),
          isOffline: true,
        }),
      ),
  ].sort(
    (a: DisplayRoute, b: DisplayRoute): number =>
      Number(a.isOffline) - Number(b.isOffline) || a.dest - b.dest,
  );
}

export function selectedRouteMetric(
  router: RoutingNode | undefined,
  dest: number,
  protocol: RoutingProtocol,
): string {
  if (!router?.is_running || dest < 1) return "—";
  if (router.id === dest) return "0";
  const route: RoutingRoute | undefined = router.routes.find(
    (candidate: RoutingRoute): boolean => candidate.dest === dest,
  );
  return route && route.metric < routeInfinity(protocol) ? String(route.metric) : "∞";
}
