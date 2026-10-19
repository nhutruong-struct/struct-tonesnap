import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, test } from "vitest";
import type { Settings, SettingsIssue } from "../lib/api";
import { translator } from "../lib/i18n";
import { fakeBackend, sendEvent, settle } from "../test/backend";
import { settings as savedSettings } from "../test/fixtures";
import SettingsPage from "./SettingsPage";

interface Setup {
  settings?: Partial<Settings>;
  issue?: SettingsIssue | null;
  save?: () => unknown;
  /** What Windows reports for starting at sign-in. */
  launchAtLogin?: () => unknown;
  setLaunchAtLogin?: () => unknown;
}

async function open(setup: Setup = {}) {
  const keys = new Set<string>();
  const backend = fakeBackend("settings", {
    get_settings: () => savedSettings(setup.settings),
    hotkey_error: () => null,
    settings_issue: () => setup.issue ?? null,
    has_api_key: (args) => keys.has(args.provider as string),
    set_api_key: (args) => {
      if (args.key === "") keys.delete(args.provider as string);
      else keys.add(args.provider as string);
    },
    save_settings: setup.save ?? (() => null),
    "plugin:autostart|is_enabled": setup.launchAtLogin ?? (() => false),
    "plugin:autostart|enable": setup.setLaunchAtLogin ?? (() => null),
    "plugin:autostart|disable": setup.setLaunchAtLogin ?? (() => null),
  });
  render(<SettingsPage />);
  await settle();
  return backend;
}

const saved = (backend: Awaited<ReturnType<typeof open>>) =>
  backend.callsTo("save_settings").map((args) => args.newSettings as Settings);

const hidden = (backend: Awaited<ReturnType<typeof open>>) => backend.callsTo("plugin:window|hide").length;

async function click(text: string) {
  act(() => {
    fireEvent.click(screen.getByText(text));
  });
  await settle();
}

/** Opens a section from the sidebar. */
async function openSection(name: RegExp) {
  act(() => {
    fireEvent.click(screen.getByRole("button", { name }));
  });
  await settle();
}

function field(label: string): HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement {
  const span = screen.getByText(label);
  const control = span.closest("label")?.querySelector("input, textarea, select");
  if (!control) throw new Error(`no control for ${label}`);
  return control as HTMLInputElement;
}

describe("loading", () => {
  test("shows the saved values", async () => {
    const backend = await open();
    expect((field("Hotkey") as HTMLInputElement).value).toBe("Ctrl+Alt+E");
    expect(backend.callsTo("plugin:window|set_title")[0]).toMatchObject({ value: "ToneSnap – Settings" });
  });

  test("names the settings that went back to their defaults by their labels", async () => {
    await open({ issue: { kind: "invalid_fields", fields: ["hotkey", "tones", "somethingNew"] } });
    expect(screen.getByText(/went back to their defaults: Hotkey; Language and tones; somethingNew\./)).toBeTruthy();
  });

  test("explains a settings file that could not be read", async () => {
    await open({ issue: { kind: "unreadable" } });
    expect(screen.getByText(/could not be read/)).toBeTruthy();
  });
});

describe("sections", () => {
  const heading = (name: string) => screen.queryByRole("heading", { name });
  const navItem = (name: RegExp) => screen.getByRole("button", { name });
  async function goTo(name: RegExp) {
    act(() => {
      fireEvent.click(navItem(name));
    });
    await settle();
  }

  test("one section is open at a time, and what was typed in the others stays", async () => {
    await open();
    expect(heading("Interface")).toBeTruthy();
    expect(heading("AI connection")).toBeNull();

    await goTo(/AI connection/);
    expect(heading("AI connection")).toBeTruthy();
    expect(heading("Interface")).toBeNull();
    fireEvent.change(field("API key"), { target: { value: "draft" } });

    await goTo(/Privacy/);
    await goTo(/AI connection/);
    expect((field("API key") as HTMLInputElement).value).toBe("draft");
  });

  test("a hotkey problem in a closed section is marked on its sidebar item", async () => {
    await open();
    const attention = () => within(navItem(/Popup trigger/)).queryByRole("img", { name: "Needs attention" });
    expect(attention()).toBeNull();

    const hotkey = field("Hotkey");
    fireEvent.keyDown(hotkey, { key: "r", code: "KeyR", ctrlKey: true, altKey: true });
    expect(attention()).toBeTruthy();

    await goTo(/Popup trigger/);
    expect(attention()).toBeNull();
  });

  test("reopening starts at the first section again", async () => {
    await open();
    await goTo(/Privacy/);
    await sendEvent("settings-opened", null);
    expect(heading("Interface")).toBeTruthy();
    expect(heading("Privacy")).toBeNull();
  });
});

describe("saving", () => {
  test("sends cleaned lists and tones in their canonical order, then closes", async () => {
    const backend = await open({ settings: { tones: ["formal"] } });
    fireEvent.change(field("Extra terms to mask (one per line)"), { target: { value: "  Falcon \n\n Osprey " } });
    fireEvent.change(field("Blocked apps (process name, one per line)"), {
      target: { value: "KeePass.exe\nBitwarden.EXE" },
    });
    await click("Concise");
    await click("Save Settings");

    const [sent] = saved(backend);
    expect(sent.customSensitiveTerms).toEqual(["Falcon", "Osprey"]);
    expect(sent.blockedApps).toEqual(["keepass.exe", "bitwarden.exe"]);
    expect(sent.tones).toEqual(["concise", "formal"]);
    expect(hidden(backend)).toBe(1);
  });

  test("starts in Vietnamese and translating, and saves the language and mode that were picked", async () => {
    const backend = await open();
    expect((field("Default target language") as HTMLSelectElement).value).toBe("vi");
    expect((field("Default mode") as HTMLSelectElement).value).toBe("translate");
    expect(screen.getByText(/Stays close to the meaning/)).toBeTruthy();

    fireEvent.change(field("Default target language"), { target: { value: "ja" } });
    fireEvent.change(field("Default mode"), { target: { value: "rewrite" } });
    expect(screen.getByText(/Rephrases freely/)).toBeTruthy();
    await click("Save Settings");

    const [sent] = saved(backend);
    expect(sent.targetLanguage).toBe("ja");
    expect(sent.mode).toBe("rewrite");
    expect(sent.tones).toEqual(["concise", "natural", "formal"]);
  });

  test("a new hotkey is captured from the keyboard and flagged when another app uses it", async () => {
    const backend = await open();
    const hotkey = field("Hotkey") as HTMLInputElement;
    fireEvent.keyDown(hotkey, { key: "Control", code: "ControlLeft", ctrlKey: true });
    expect(hotkey.value).toBe("Ctrl+Alt+E");
    fireEvent.keyDown(hotkey, { key: "r", code: "KeyR", ctrlKey: true, altKey: true });
    expect(hotkey.value).toBe("Ctrl+Alt+R");
    expect(screen.getByText(/Commonly used by Microsoft Word/)).toBeTruthy();
    await click("Save Settings");
    expect(saved(backend)[0].hotkey).toBe("ctrl+alt+r");
  });

  test("a hotkey another app holds keeps the window open with an explanation", async () => {
    const backend = await open({
      save: () => Promise.reject({ kind: "hotkey_taken", hotkey: "ctrl+alt+e", detail: "taken" }),
    });
    await click("Save Settings");
    expect(screen.getByText(/Ctrl\+Alt\+E is already used by another app/)).toBeTruthy();
    expect(hidden(backend)).toBe(0);
  });

  test("a failed write is reported next to the Save button", async () => {
    await open({ save: () => Promise.reject({ kind: "save_failed", hotkey: "", detail: "disk full" }) });
    await click("Save Settings");
    expect(screen.getByText("Could not save settings: disk full")).toBeTruthy();
  });

  test("a successful save clears the settings warning", async () => {
    await open({ issue: { kind: "broken", backup: "C:\\settings.broken.json" } });
    expect(screen.getByText(/C:\\settings.broken.json/)).toBeTruthy();
    await click("Save Settings");
    expect(screen.queryByText(/C:\\settings.broken.json/)).toBeNull();
  });
});

describe("starting with Windows", () => {
  const LABEL = "Start ToneSnap when I sign in to Windows";
  const checkbox = () => screen.getByRole<HTMLInputElement>("switch", { name: new RegExp(LABEL) });
  const changes = (backend: Awaited<ReturnType<typeof open>>) => [
    ...backend.callsTo("plugin:autostart|enable").map(() => "enable"),
    ...backend.callsTo("plugin:autostart|disable").map(() => "disable"),
  ];

  test("changes only when the settings are saved", async () => {
    const backend = await open();
    expect(checkbox().checked).toBe(false);
    await click(LABEL);
    expect(changes(backend)).toEqual([]);
    await click("Save Settings");
    expect(changes(backend)).toEqual(["enable"]);
    expect(hidden(backend)).toBe(1);
  });

  test("is left alone when the user did not change it", async () => {
    const backend = await open({ launchAtLogin: () => true });
    expect(checkbox().checked).toBe(true);
    await click("Save Settings");
    expect(changes(backend)).toEqual([]);
  });

  test("a change Windows refuses is reported and keeps the window open", async () => {
    const backend = await open({ launchAtLogin: () => true, setLaunchAtLogin: () => Promise.reject("access denied") });
    await click(LABEL);
    await click("Save Settings");
    expect(changes(backend)).toEqual(["disable"]);
    expect(screen.getByText(/starting with Windows could not be changed: access denied/)).toBeTruthy();
    expect(hidden(backend)).toBe(0);
  });

  test("is hidden when Windows cannot tell", async () => {
    await open({ launchAtLogin: () => Promise.reject("no registry") });
    expect(screen.queryByText(LABEL)).toBeNull();
  });
});

describe("AI providers", () => {
  async function openAi(setup: Setup = {}) {
    const backend = await open(setup);
    await openSection(/AI connection/);
    return backend;
  }

  test("Gemini is the default, with a free-tier notice and a link to get a key", async () => {
    await openAi();
    expect(screen.getByLabelText<HTMLInputElement>(/Google Gemini/).checked).toBe(true);
    expect(screen.getByText("Get a free key at aistudio.google.com/apikey")).toBeTruthy();
    expect(screen.getByText(/Free tiers usually let the provider use what you send/)).toBeTruthy();
    expect((field("Model") as HTMLInputElement).value).toBe("gemini-3.8-flash");
  });

  test("each provider keeps its own key and model", async () => {
    const backend = await openAi();
    await click("Groq – free tier, very fast");
    fireEvent.change(field("Model"), { target: { value: "openai/gpt-oss-120b" } });
    fireEvent.change(field("API key"), { target: { value: "  gsk-test  " } });
    await click("Save Key");
    expect(backend.callsTo("set_api_key")).toEqual([{ provider: "groq", key: "  gsk-test  " }]);

    await click("OpenRouter – free models");
    expect((field("API key") as HTMLInputElement).placeholder).toBe("sk-or-…");
    expect((field("Model") as HTMLInputElement).value).toBe("openai/gpt-oss-20b:free");

    await click("Save Settings");
    const settings = saved(backend)[0];
    expect(settings.provider).toBe("openrouter");
    expect(settings.groqModel).toBe("openai/gpt-oss-120b");
    expect(settings.geminiModel).toBe("gemini-3.8-flash");
  });

  test("a provider that has a saved key says so", async () => {
    const backend = await openAi();
    fireEvent.change(field("API key"), { target: { value: "AIza-test" } });
    await click("Save Key");
    expect(backend.callsTo("set_api_key")).toEqual([{ provider: "gemini", key: "AIza-test" }]);
    expect((field("API key") as HTMLInputElement).placeholder).toBe("Saved (enter a new key to replace it)");

    await click("Remove");
    expect(backend.callsTo("set_api_key")[1]).toEqual({ provider: "gemini", key: "" });
    expect(screen.getByText("API key removed.")).toBeTruthy();
  });
});

describe("Claude", () => {
  test("stores the API key in the credential store, never in the settings", async () => {
    const backend = await open();
    await openSection(/AI connection/);
    await click("Claude API – paid, highest quality");
    expect(screen.queryByText(/Free tiers usually/)).toBeNull();
    const input = field("API key") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "  test-key  " } });
    await click("Save Key");
    expect(backend.callsTo("set_api_key")).toEqual([{ provider: "claude", key: "  test-key  " }]);
    expect(screen.getByText("API key saved to Windows Credential Manager.")).toBeTruthy();
    expect(input.value).toBe("");
    expect(input.placeholder).toBe("Saved (enter a new key to replace it)");

    await click("Save Settings");
    expect(JSON.stringify(saved(backend)[0])).not.toContain("test-key");
    expect(saved(backend)[0].provider).toBe("claude");
  });

  test("Haiku cannot change the effort", async () => {
    await open({ settings: { provider: "claude", model: "claude-haiku-4-5" } });
    await openSection(/AI connection/);
    expect((field("Reasoning effort") as HTMLSelectElement).disabled).toBe(true);
    expect(screen.getByText("Haiku 4.5 does not support this option.")).toBeTruthy();
  });
});

describe("reopening", () => {
  test("drops unsaved edits and reloads what is saved", async () => {
    const backend = await open();
    await click("Natural");
    await openSection(/AI connection/);
    await click("Claude API – paid, highest quality");
    fireEvent.change(field("API key"), { target: { value: "draft" } });

    await sendEvent("settings-opened", null);
    expect(backend.callsTo("get_settings")).toHaveLength(2);
    expect(screen.getByLabelText<HTMLInputElement>(/Google Gemini/).checked).toBe(true);
    expect((field("API key") as HTMLInputElement).value).toBe("");
    await click("Save Settings");
    expect(saved(backend)[0].tones).toEqual(savedSettings().tones);
  });
});

test("the interface language preview follows the unsaved choice", async () => {
  await open();
  fireEvent.change(field("Interface language"), { target: { value: "vi" } });
  await settle();
  expect(screen.getByText(translator("vi")("settings.save"))).toBeTruthy();
});
