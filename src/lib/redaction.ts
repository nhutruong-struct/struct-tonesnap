/**
 * A redaction placeholder such as [EMAIL_1], also the way models tend to rewrite it, in lower or
 * mixed case, with a space or hyphen instead of the underscore, or with spaces inside the brackets.
 */
const PLACEHOLDER = /\[\s*([A-Za-z]+)[\s_-]*(\d+)\s*\]/g;

/** The placeholder `match` stands for, spelled the way Rust creates it. */
function canonical(kind: string, n: string): string {
  return `[${kind.toUpperCase()}_${n}]`;
}

/** Swaps redaction placeholders such as [EMAIL_1] back to the original values. */
export function restore(text: string, redactions: [string, string][]): string {
  const originals = new Map(redactions);
  return text.replace(PLACEHOLDER, (match, kind: string, n: string) => originals.get(canonical(kind, n)) ?? match);
}

/**
 * The original values whose placeholders appear in `source` (the redacted text sent to the model)
 * but not in `output`: the model dropped or reworded them, so restoring cannot bring them back.
 */
export function missingRedactions(source: string, output: string, redactions: [string, string][]): string[] {
  const found = (text: string) => new Set([...text.matchAll(PLACEHOLDER)].map(([, kind, n]) => canonical(kind, n)));
  const inSource = found(source);
  const inOutput = found(output);
  return redactions
    .filter(([placeholder]) => inSource.has(placeholder) && !inOutput.has(placeholder))
    .map(([, original]) => original);
}
