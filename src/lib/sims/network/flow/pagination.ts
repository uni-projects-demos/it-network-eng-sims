export const FLOW_ROW_HEIGHT: number = 34;

export function paginateFlowRows(
  len: number,
  height: number,
  page: number,
  headerHeight: number = FLOW_ROW_HEIGHT,
): { page: number; pages: number; size: number; start: number; end: number } {
  const size: number = Math.max(
    1,
    Math.floor(Math.max(0, height - headerHeight) / FLOW_ROW_HEIGHT),
  );
  const pages: number = Math.max(1, Math.ceil(len / size));
  const cur: number = Math.max(0, Math.min(pages - 1, Math.floor(page)));
  const start: number = cur * size;
  return {
    page: cur,
    pages,
    size,
    start,
    end: Math.min(len, start + size),
  };
}
