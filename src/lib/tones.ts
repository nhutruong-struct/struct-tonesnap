import type { ToneId } from "./api";
import { hasKey, type Translate } from "./i18n";

/** Display order; labels and descriptions live in the i18n dictionaries. */
export const TONE_IDS: ToneId[] = ["minimal", "concise", "natural", "polite", "formal", "friendly"];

export function toneLabel(t: Translate, id: string | undefined): string {
  const key = `tone.${id}`;
  return hasKey(key) ? t(key) : (id ?? "");
}

export function toneDescription(t: Translate, id: ToneId): string {
  const key = `tone.${id}.desc`;
  return hasKey(key) ? t(key) : "";
}
