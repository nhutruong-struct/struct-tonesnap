/** Swaps redaction placeholders such as [EMAIL_1] back to the original values. */
export function restore(text: string, redactions: [string, string][]): string {
  return redactions.reduce((acc, [placeholder, original]) => acc.split(placeholder).join(original), text);
}
