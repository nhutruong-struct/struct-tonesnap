/** Hotkeys are stored as lower-case "ctrl+alt+e", which the Rust side parses. */

const MODIFIER_ORDER = ["ctrl", "alt", "shift", "super"];
const MODIFIER_ALIASES: Record<string, string> = {
  control: "ctrl",
  option: "alt",
  meta: "super",
  cmd: "super",
  command: "super",
  win: "super",
};

export function normalizeHotkey(combo: string): string {
  const parts = combo
    .split("+")
    .map((p) => p.trim().toLowerCase())
    .filter(Boolean);
  const modifiers = new Set<string>();
  let key = "";
  for (const part of parts) {
    const modifier = MODIFIER_ALIASES[part] ?? part;
    if (MODIFIER_ORDER.includes(modifier)) modifiers.add(modifier);
    else key = part.replace(/^(key|digit)(?=.)/, "");
  }
  return [...MODIFIER_ORDER.filter((m) => modifiers.has(m)), key].filter(Boolean).join("+");
}

const DISPLAY: Record<string, string> = { ctrl: "Ctrl", alt: "Alt", shift: "Shift", super: "Win" };

/** "ctrl+alt+e" → "Ctrl+Alt+E" */
export function formatHotkey(combo: string): string {
  return normalizeHotkey(combo)
    .split("+")
    .map((p) => DISPLAY[p] ?? (p.length === 1 ? p.toUpperCase() : p[0].toUpperCase() + p.slice(1)))
    .join("+");
}

/**
 * Shortcuts other popular apps grab, often with low-level keyboard hooks that Windows
 * cannot report as a conflict. Keys are normalized hotkeys.
 */
const KNOWN_CONFLICTS: Record<string, string> = {
  "ctrl+alt+space": "Claude desktop (Quick Entry)",
  "ctrl+space": "VS Code, Rider, Visual Studio (IntelliSense), Windows IME",
  "alt+space": "PowerToys Run, ChatGPT, Copilot",
  "ctrl+shift+space": "Microsoft Word",
  "ctrl+shift+e": "Microsoft Word (Track Changes)",
  "ctrl+alt+r": "Microsoft Word (®)",
  "ctrl+alt+c": "Microsoft Word (©)",
  "ctrl+alt+t": "Microsoft Word (™)",
};

export function hotkeyConflict(combo: string): string | null {
  return KNOWN_CONFLICTS[normalizeHotkey(combo)] ?? null;
}
