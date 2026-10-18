import { describe, expect, test } from "vitest";
import { hasKey, KEYS, resolveLang, translator } from "./i18n";

const placeholders = (text: string) => [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe("dictionaries", () => {
  const vi = translator("vi");
  const en = translator("en");

  test("every string exists in both languages", () => {
    for (const key of KEYS) {
      expect(vi(key).trim(), key).not.toBe("");
      expect(en(key).trim(), key).not.toBe("");
    }
  });

  test("both languages use the same placeholders", () => {
    for (const key of KEYS) {
      expect(placeholders(vi(key)), key).toEqual(placeholders(en(key)));
    }
  });

  test("every tone, trigger and notice the app can show has a label", () => {
    const required = [
      ...["minimal", "concise", "natural", "polite", "formal", "friendly"].map((t) => `tone.${t}`),
      ...["hotkey", "icon", "auto", "off"].map((t) => `trigger.${t}`),
      ...["no_selection", "password_field", "blocked_app", "apply_failed"].map((n) => `notice.${n}`),
      ...["uia", "clipboard"].map((s) => `popup.source.${s}`),
      ...["uia", "ocr", "none"].map((s) => `popup.context.${s}`),
      "settings.issue.invalid_fields",
      "settings.issue.broken",
      "settings.issue.unreadable",
    ];
    for (const key of required) expect(hasKey(key), key).toBe(true);
  });
});

describe("translator", () => {
  test("fills a placeholder", () => {
    expect(translator("en")("notice.blocked_app", { app: "Chrome" })).toBe(
      "ToneSnap is turned off in Chrome (blocked apps in Settings).",
    );
  });

  test("leaves unknown placeholders alone", () => {
    expect(translator("en")("error.generic", {})).toBe("Error {status}");
  });
});

test("resolveLang honors an explicit choice and otherwise follows the browser", () => {
  expect(resolveLang("vi")).toBe("vi");
  expect(resolveLang("en")).toBe("en");
  const expected = navigator.language.toLowerCase().startsWith("vi") ? "vi" : "en";
  expect(resolveLang("auto")).toBe(expected);
  expect(resolveLang(undefined)).toBe(expected);
});
