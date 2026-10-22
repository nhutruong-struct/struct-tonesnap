import { Allow, parse } from "partial-json";
import type { Suggestions, ToneId } from "../lib/api";
import { missingRedactions, restore } from "../lib/redaction";

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
  /** Final, so it can be applied or copied. */
  ready: boolean;
  /**
   * Masked values of the selection that this final variant lost (the model dropped or reworded
   * their placeholders). Such a variant is offered for Copy only, never pasted straight in.
   */
  missing: string[];
}

/**
 * One entry per requested tone, in order. A variant is final once the model has moved on to the
 * next one, or the request has finished. `selection` is the redacted selection the model got.
 */
export function toVariants(
  tones: ToneId[],
  suggestions: Suggestions,
  finished: boolean,
  redactions: [string, string][],
  selection: string,
): Variant[] {
  const raw = tones.map((tone) => suggestions.variants?.[tone]);
  return tones.map((tone, i) => {
    const complete = finished || raw[i + 1] !== undefined;
    const output = raw[i]?.text ?? "";
    const text = restore(output, redactions);
    const ready = complete && !!text;
    const missing = ready ? missingRedactions(selection, output, redactions) : [];
    return { tone, text, ready, missing };
  });
}
