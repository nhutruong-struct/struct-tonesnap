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

  test("the macOS wording keeps every placeholder and names no Windows feature", () => {
    for (const lang of ["vi", "en"] as const) {
      const base = translator(lang, false);
      const mac = translator(lang, true);
      const changed = KEYS.filter((key) => mac(key) !== base(key));
      expect(changed.length).toBeGreaterThan(5);
      for (const key of changed) {
        expect(placeholders(mac(key)), key).toEqual(placeholders(base(key)));
        expect(mac(key), key).not.toMatch(/Windows|Ctrl|Credential/);
      }
    }
  });

  test("every tone, trigger and notice the app can show has a label", () => {
    const required = [
      ...["concise", "natural", "formal"].map((t) => `tone.${t}`),
      ...["vi", "en", "ja"].map((l) => `target.${l}`),
      ...["translate", "rewrite"].flatMap((m) => [`mode.${m}`, `mode.${m}.hint`]),
      ...["hotkey", "icon", "auto", "off"].map((t) => `trigger.${t}`),
      ...["no_selection", "password_field", "blocked_app", "apply_failed", "accessibility_denied"].map(
        (n) => `notice.${n}`,
      ),
      ...["uia", "clipboard"].map((s) => `popup.source.${s}`),
      ...["uia", "ocr", "none"].map((s) => `popup.context.${s}`),
      ...["no_api_key", "authentication_error", "permission_error", "not_found_error", "rate_limit_error"].map(
        (e) => `error.${e}`,
      ),
      ...["overloaded_error", "refusal", "max_tokens", "network"].map((e) => `error.${e}`),
      "error.generic",
      ...["gemini", "groq", "openrouter", "claude"].flatMap((p) => [
        `provider.${p}`,
        `provider.${p}.hint`,
        `apiKey.get.${p}`,
      ]),
      ...["auto", "manual"].flatMap((m) => [`updateMode.${m}`, `updateMode.${m}.hint`, `updates.status.ready.${m}`]),
      ...["idle", "checking", "up_to_date", "available", "downloading", "installing"].map((s) => `updates.status.${s}`),
      ...["check", "download", "install"].map((s) => `updates.status.failed.${s}`),
      "settings.issue.invalid_fields",
      "settings.issue.broken",
      "settings.issue.unreadable",
    ];
    for (const key of required) expect(hasKey(key), key).toBe(true);
  });

  test("button labels are in title case", () => {
    const buttons = [
      "settings.save",
      "apiKey.getKey",
      "apiKey.save",
      "apiKey.remove",
      "popup.close",
      "popup.retry",
      "popup.openSettings",
      "preview.send",
      "preview.cancel",
      "updates.check",
      "updates.install",
    ] as const;
    // Joining words stay lower case: "for", "and", "&" and the short ones of other languages.
    const small = ["for", "and"];
    for (const [name, t] of [
      ["vi", vi],
      ["en", en],
    ] as const) {
      for (const key of buttons) {
        const words = t(key)
          .split(" ")
          .filter((w) => !w.startsWith("(") && w.length > 2 && !small.includes(w));
        for (const word of words) {
          expect(word[0], `${name} ${key}: "${t(key)}"`).toBe(word[0].toUpperCase());
        }
      }
    }
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
