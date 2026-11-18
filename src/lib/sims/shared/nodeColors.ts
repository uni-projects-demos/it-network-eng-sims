export type NodeColor = Readonly<{
  fill: string;
  light: string;
  dark: string;
}>;

export const NODE_COLORS: readonly NodeColor[] = [
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

export function nodeColor(nodeId: number): NodeColor | undefined {
  return Number.isInteger(nodeId) && nodeId >= 1 && nodeId <= NODE_COLORS.length
    ? NODE_COLORS[nodeId - 1]
    : undefined;
}

export function nodeBadgeStyle(nodeId: number): string {
  const fill: string | undefined = nodeColor(nodeId)?.fill;
  return fill
    ? `background-color: ${fill}; color: #16202c`
    : "background-color: var(--selected-bg); color: var(--selected-text)";
}
