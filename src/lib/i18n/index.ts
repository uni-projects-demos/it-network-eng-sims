import { type Writable, writable } from "svelte/store";
import english from "./locales/en.json";

export type Locale = string;
export type TranslationParams = Record<string, string | number | undefined>;

interface Msgs {
  [key: string]: string | Msgs;
}
type LocaleLoader = () => Promise<Msgs>;

const FB_LOCALE: Locale = "en";
const STORAGE_KEY: string = "network-sim-locale";
const PARAM_PATTERN: RegExp = /\{([A-Za-z0-9_]+)\}/gu;

const fallback: Msgs = english as Msgs;

const modules: Record<string, LocaleLoader> = import.meta.glob<Msgs>(
  ["./locales/*.json", "!./locales/en.json"],
  { import: "default" },
);

const loaders: Map<Locale, LocaleLoader> = new Map<Locale, LocaleLoader>();

for (const [path, load] of Object.entries(modules) as [string, LocaleLoader][]) {
  const code: Locale | undefined = /\/([^/]+)\.json$/u.exec(path)?.[1];

  if (code !== undefined) {
    loaders.set(code, load);
  }
}

export const supportedLocales: Locale[] = [FB_LOCALE, ...loaders.keys()].sort();
export const locale: Writable<Locale> = writable<Locale>(FB_LOCALE);

const dicts: Map<Locale, Msgs> = new Map<Locale, Msgs>([[FB_LOCALE, fallback]]);

const RTL_BASES: ReadonlySet<string> = new Set<string>([
  "ar",
  "arc",
  "ckb",
  "dv",
  "fa",
  "hbo",
  "he",
  "ks",
  "ku",
  "pal",
  "phn",
  "ps",
  "sam",
  "sd",
  "syc",
  "ug",
  "ur",
  "yi",
]);

async function loadLocale(code: Locale): Promise<void> {
  if (dicts.has(code)) {
    return;
  }

  const load: LocaleLoader | undefined = loaders.get(code);
  if (load === undefined) {
    return;
  }

  const msgs: Msgs = await load();
  dicts.set(code, msgs);
}

export function isLocaleRTL(code: Locale): boolean {
  const base: string = code.toLowerCase().split("-", 1)[0];
  return RTL_BASES.has(base);
}

function resMsg(msgs: Msgs | undefined, key: string): string | undefined {
  if (msgs === undefined) {
    return undefined;
  }

  let cur: string | Msgs | undefined = msgs;

  for (const segment of key.split(".")) {
    if (typeof cur === "string") {
      return undefined;
    }

    cur = cur[segment];

    if (cur === undefined) {
      return undefined;
    }
  }

  return typeof cur === "string" ? cur : undefined;
}

export function translate(
  key: string,
  code: Locale = FB_LOCALE,
  params: TranslationParams = {},
): string {
  const template: string = resMsg(dicts.get(code), key) ?? resMsg(fallback, key) ?? key;

  return template.replace(PARAM_PATTERN, (_match: string, name: string): string => {
    const val: string | number | undefined = params[name];
    return val === undefined ? `{${name}}` : String(val);
  });
}

export async function setLocale(code: Locale): Promise<void> {
  const nxt: Locale = code === FB_LOCALE || loaders.has(code) ? code : FB_LOCALE;

  await loadLocale(nxt);
  locale.set(nxt);

  if (typeof document !== "undefined") {
    const root: HTMLElement = document.documentElement;

    root.lang = nxt;
    root.dir = isLocaleRTL(nxt) ? "rtl" : "ltr";
    root.dataset.locale = nxt;
  }

  if (typeof localStorage !== "undefined") {
    try {
      localStorage.setItem(STORAGE_KEY, nxt);
    } catch {}
  }
}

export { localeDisplayName } from "./names.ts";
