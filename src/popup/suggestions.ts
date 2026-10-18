import { Allow, parse } from "partial-json";
import type { Suggestions, ToneId } from "../lib/api";
import { restore } from "../lib/redaction";

/** Reads the model output while it streams in; incomplete JSON yields what is there so far. */
export function parseSuggestions(raw: string): Suggestions {
  if (!raw) return {};
  try {
    return parse(raw, Allow.ALL) as Suggestions;
  } catch {
    return {};
  }
}

export interface Variant {
  tone: ToneId;
  /** With redacted values restored; empty until the model starts this tone. */
  text: string;
  note: string;
  /** Final, so it can be applied or copied. */
  ready: boolean;
}

/**
 * One entry per requested tone, in order. A variant is final once the model has moved on to the
 * next one (or to the issues), or the request has finished.
 */
export function toVariants(
  tones: ToneId[],
  suggestions: Suggestions,
  finished: boolean,
  redactions: [string, string][],
): Variant[] {
  const raw = tones.map((tone) => suggestions.variants?.[tone]);
  return tones.map((tone, i) => {
    const complete = finished || raw[i + 1] !== undefined || suggestions.issues !== undefined;
    const text = restore(raw[i]?.text ?? "", redactions);
    return { tone, text, note: restore(raw[i]?.note ?? "", redactions), ready: complete && !!text };
  });
}
