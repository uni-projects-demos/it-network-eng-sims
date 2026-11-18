export function graphNodeRadius(count: number, width: number, height: number): number {
  const usable: number = Math.max(1, (width - 110) * (height - 110));
  return Math.min(
    count >= 14 ? 25 : 31,
    Math.max(16, Math.floor(0.3 * Math.sqrt(usable / Math.max(1, count)))),
  );
}
