import { expect, test } from "vitest";
import { formatHotkey, hotkeyConflict, normalizeHotkey } from "./hotkey";

test("normalizes aliases, key codes and modifier order", () => {
  expect(normalizeHotkey("Control+Alt+KeyE")).toBe("ctrl+alt+e");
  expect(normalizeHotkey("cmd+shift+Digit1")).toBe("shift+super+1");
  expect(normalizeHotkey("alt + ctrl + Space")).toBe("ctrl+alt+space");
  expect(normalizeHotkey("win+option+F5")).toBe("alt+super+f5");
});

test("formats for display", () => {
  expect(formatHotkey("ctrl+alt+e")).toBe("Ctrl+Alt+E");
  expect(formatHotkey("ctrl+space")).toBe("Ctrl+Space");
  expect(formatHotkey("super+shift+f5")).toBe("Shift+Win+F5");
});

test("formats with symbols on a Mac", () => {
  expect(formatHotkey("ctrl+alt+e", true)).toBe("⌃⌥E");
  expect(formatHotkey("super+shift+f5", true)).toBe("⇧⌘F5");
  expect(formatHotkey("ctrl+space", true)).toBe("⌃Space");
});

test("knows shortcuts other apps take", () => {
  expect(hotkeyConflict("Ctrl+Alt+Space")).toContain("Claude");
  expect(hotkeyConflict("alt+ctrl+r")).toContain("Word");
  expect(hotkeyConflict("ctrl+alt+e")).toBeNull();
});
