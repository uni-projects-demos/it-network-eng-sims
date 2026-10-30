import { type TranslationParams, translate } from "#lib/i18n/index.ts";
import type english from "#lib/i18n/locales/en.json";
import { FlowConfigErr } from "./types.ts";

export type FlowMsgKey = keyof typeof english.flow;
export type FlowTranslator = (key: FlowMsgKey, params?: TranslationParams) => string;

export function flowTranslator(code: string): FlowTranslator {
  return (key, params): string => translate(`flow.${key}`, code, params);
}

export function flowErr(err: unknown, code: string): string {
  if (!err) return "";
  if (err instanceof FlowConfigErr && err.field) {
    return translate("flow.invalidCount", code, { field: translate(`flow.${err.field}`, code) });
  }
  return translate("flow.errorSimulation", code);
}
