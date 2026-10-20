import { describe, expect, test } from "vitest";
import { translator } from "../lib/i18n";
import { describeCommandError, describeSettingsIssue } from "./useSettingsForm";

const en = translator("en");

describe("describeCommandError", () => {
  test("names a taken hotkey the way users see it", () => {
    const error = { kind: "hotkey_taken", hotkey: "ctrl+alt+e", detail: "in use" } as const;
    expect(describeCommandError(en, error)).toBe(
      "Ctrl+Alt+E is already used by another app, so it is not active. Choose a different combination and save.",
    );
  });

  test("an invalid hotkey carries the reason", () => {
    const error = { kind: "hotkey_invalid", hotkey: "ctrl+alt+e", detail: "bad key" } as const;
    expect(describeCommandError(en, error)).toBe("Ctrl+Alt+E is not a valid hotkey: bad key");
  });

  test("a failed write carries the raw message", () => {
    const error = { kind: "save_failed", hotkey: "", detail: "disk full" } as const;
    expect(describeCommandError(en, error)).toBe("Could not save settings: disk full");
  });

  test("is translated into the other language too", () => {
    const vi = translator("vi");
    const error = { kind: "save_failed", hotkey: "", detail: "disk full" } as const;
    expect(describeCommandError(vi, error)).toBe(vi("settings.saveFailed", { detail: "disk full" }));
  });
});

describe("describeSettingsIssue", () => {
  test("a damaged file says where the old copy was kept", () => {
    const text = describeSettingsIssue(en, { kind: "broken", backup: "C:\\data\\settings.broken.json" });
    expect(text).toContain("C:\\data\\settings.broken.json");
    expect(text).toContain("damaged");
  });

  test("an unreadable file says that saving replaces it", () => {
    expect(describeSettingsIssue(en, { kind: "unreadable" })).toMatch(/Saving Settings will replace that file/);
  });

  test("fields that went back to their defaults are named by their labels", () => {
    const text = describeSettingsIssue(en, { kind: "invalid_fields", fields: ["updateMode", "mode", "unknownField"] });
    expect(text).toContain("Updates; Default mode; unknownField");
  });
});
