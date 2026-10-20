import { expect, test } from "vitest";
import { translator } from "./i18n";
import { TONE_IDS, toneDescription, toneLabel } from "./tones";

const en = translator("en");

test("the tones are listed in the order the popup shows them", () => {
  expect(TONE_IDS).toEqual(["concise", "natural", "formal"]);
});

test("every tone has a label and a description in both languages", () => {
  for (const lang of ["en", "vi"] as const) {
    const t = translator(lang);
    for (const id of TONE_IDS) {
      expect(toneLabel(t, id), `${lang} ${id}`).not.toBe(id);
      expect(toneDescription(t, id), `${lang} ${id}`).not.toBe("");
    }
  }
  expect(toneLabel(en, "concise")).toBe("Concise");
});

test("a tone the dictionary does not know shows as written", () => {
  expect(toneLabel(en, "pirate")).toBe("pirate");
  expect(toneLabel(en, undefined)).toBe("");
});
