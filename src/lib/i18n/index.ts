import { type Writable, writable } from "svelte/store";
import english from "./locales/en.json";

export type Locale = string;
export type TranslationParams = Record<string, string | number | undefined>;

interface Messages {
  [key: string]: string | Messages;
}
type LocaleLoader = () => Promise<Messages>;

const FB_LOCALE: Locale = "en";
const STORAGE_KEY: string = "network-sim-locale";
const PARAM_PATTERN: RegExp = /\{([A-Za-z0-9_]+)\}/gu;

const fallback: Messages = english as Messages;

const modules: Record<string, LocaleLoader> = import.meta.glob<Messages>(
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

export const supportedLocales: Locale[] = [...loaders.keys()].sort();
export const locale: Writable<Locale> = writable<Locale>(FB_LOCALE);

const dicts: Map<Locale, Messages> = new Map<Locale, Messages>([[FB_LOCALE, fallback]]);

const displayNames: Map<Locale, Intl.DisplayNames> = new Map<Locale, Intl.DisplayNames>();

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

  const messages: Messages = await load();
  dicts.set(code, messages);
}

export function isLocaleRTL(code: Locale): boolean {
  const base: string = code.toLowerCase().split("-", 1)[0];
  return RTL_BASES.has(base);
}

function resolveMessage(messages: Messages | undefined, key: string): string | undefined {
  if (messages === undefined) {
    return undefined;
  }

  let current: string | Messages | undefined = messages;

  for (const segment of key.split(".")) {
    if (typeof current === "string") {
      return undefined;
    }

    current = current[segment];

    if (current === undefined) {
      return undefined;
    }
  }

  return typeof current === "string" ? current : undefined;
}

export function translate(
  key: string,
  code: Locale = FB_LOCALE,
  params: TranslationParams = {},
): string {
  const template: string =
    resolveMessage(dicts.get(code), key) ?? resolveMessage(fallback, key) ?? key;

  return template.replace(PARAM_PATTERN, (_match: string, name: string): string => {
    const value: string | number | undefined = params[name];
    return value === undefined ? `{${name}}` : String(value);
  });
}

export async function setLocale(code: Locale): Promise<void> {
  const next: Locale = loaders.has(code) ? code : FB_LOCALE;

  await loadLocale(next);
  locale.set(next);

  if (typeof document !== "undefined") {
    const root: HTMLElement = document.documentElement;

    root.lang = next;
    root.dir = isLocaleRTL(next) ? "rtl" : "ltr";
    root.dataset.locale = next;
  }

  if (typeof localStorage !== "undefined") {
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {}
  }
}

export function localeDisplayName(code: Locale, displayLocale: Locale): string {
  try {
    let names: Intl.DisplayNames | undefined = displayNames.get(displayLocale);

    if (names === undefined) {
      names = new Intl.DisplayNames([displayLocale], { type: "language" });
      displayNames.set(displayLocale, names);
    }

    const name: string | undefined = names.of(code);

    if (name !== undefined && name.toLowerCase() !== code.toLowerCase()) {
      return name;
    }
  } catch {}

  return code;
}
