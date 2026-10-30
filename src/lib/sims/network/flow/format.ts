const numFormats: Map<string, Intl.NumberFormat> = new Map<string, Intl.NumberFormat>();

function formatter(code: string, style: "number" | "duration" | "percent"): Intl.NumberFormat {
  const key: string = `${code}:${style}`;
  let format: Intl.NumberFormat | undefined = numFormats.get(key);
  if (!format) {
    const options: Intl.NumberFormatOptions = { maximumFractionDigits: 3 };
    if (style === "duration") {
      options.style = "unit";
      options.unit = "millisecond";
      options.unitDisplay = "short";
    } else if (style === "percent") {
      options.style = "percent";
    }
    try {
      format = new Intl.NumberFormat(code, options);
    } catch {
      format = new Intl.NumberFormat("en", options);
    }
    numFormats.set(key, format);
  }
  return format;
}

export function flowNumber(val: number | null | undefined, code: string = "en"): string {
  return val == null ? "—" : formatter(code, "number").format(val);
}

export function flowDuration(val: number, code: string): string {
  return formatter(code, "duration").format(val);
}

export function flowPercent(val: number | null | undefined, code: string): string {
  return val == null ? "—" : formatter(code, "percent").format(val);
}
