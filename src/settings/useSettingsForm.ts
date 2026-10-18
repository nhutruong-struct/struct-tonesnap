import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, events, unsubscribeAll, type CommandError, type Settings, type SettingsIssue } from "../lib/api";
import { formatHotkey } from "../lib/hotkey";
import { resolveLang, translator, type Key, type Translate } from "../lib/i18n";
import type { Status } from "./fields";

function lines(text: string): string[] {
  return text
    .split("\n")
    .map((l) => l.trim())
    .filter(Boolean);
}

/** The label each saved field has in this window, for naming fields that were reset. */
const FIELD_LABELS: Partial<Record<keyof Settings, Key>> = {
  uiLanguage: "uiLanguage.label",
  triggerMode: "section.trigger",
  hotkey: "hotkey.label",
  autoDelayMs: "autoDelay.label",
  provider: "section.ai",
  model: "model.label",
  effort: "effort.label",
  geminiModel: "model.label",
  groqModel: "model.label",
  openrouterModel: "model.label",
  tones: "section.tones",
  explainLanguage: "explain.label",
  contextChars: "contextChars.label",
  ocrFallback: "ocr.label",
  redact: "redact.label",
  customSensitiveTerms: "terms.label",
  previewBeforeSend: "preview.label",
  blockedApps: "blocked.label",
};

function fieldLabel(t: Translate, field: string): string {
  const key = FIELD_LABELS[field as keyof Settings];
  return key ? t(key) : field;
}

export function describeSettingsIssue(t: Translate, issue: SettingsIssue): string {
  switch (issue.kind) {
    case "invalid_fields":
      return t("settings.issue.invalid_fields", { fields: issue.fields.map((f) => fieldLabel(t, f)).join("; ") });
    case "broken":
      return t("settings.issue.broken", { backup: issue.backup });
    case "unreadable":
      return t("settings.issue.unreadable");
  }
}

export function describeCommandError(t: Translate, err: CommandError): string {
  switch (err.kind) {
    case "hotkey_taken":
      return t("hotkey.taken", { hotkey: formatHotkey(err.hotkey) });
    case "hotkey_invalid":
      return t("hotkey.invalid", { hotkey: formatHotkey(err.hotkey), detail: err.detail });
    case "save_failed":
      return t("settings.saveFailed", { detail: err.detail });
  }
}

/**
 * The Settings form: the saved values, reloaded whenever the window opens, the user's unsaved
 * edits, and saving them.
 */
export function useSettingsForm() {
  const [settings, setSettings] = useState<Settings | null>(null);
  /** The two list fields, edited as one entry per line. */
  const [terms, setTerms] = useState("");
  const [blocked, setBlocked] = useState("");
  const [status, setStatus] = useState<Status | null>(null);
  const [hotkeyError, setHotkeyError] = useState<CommandError | null>(null);
  /** What was wrong with the saved file; cleared by the next successful save. */
  const [issue, setIssue] = useState<SettingsIssue | null>(null);
  /** Counts loads, so sections can drop their own drafts when the window opens again. */
  const [loadCount, setLoadCount] = useState(0);
  /** Starting at sign-in, as Windows reports it and as edited; `null` when Windows cannot tell. */
  const [savedLaunchAtLogin, setSavedLaunchAtLogin] = useState<boolean | null>(null);
  const [launchAtLogin, setLaunchAtLogin] = useState<boolean | null>(null);

  // The interface language follows the unsaved choice so users can see what they pick.
  const lang = resolveLang(settings?.uiLanguage);
  const t = translator(lang);

  useEffect(() => {
    const load = () => {
      void api.getSettings().then((s) => {
        setSettings(s);
        setTerms(s.customSensitiveTerms.join("\n"));
        setBlocked(s.blockedApps.join("\n"));
        setLoadCount((n) => n + 1);
      });
      void api.hotkeyError().then(setHotkeyError);
      void api.settingsIssue().then(setIssue);
      void api.launchAtLogin().then(
        (on) => {
          setSavedLaunchAtLogin(on);
          setLaunchAtLogin(on);
        },
        () => {
          setSavedLaunchAtLogin(null);
          setLaunchAtLogin(null);
        },
      );
      setStatus(null);
    };
    load();
    return unsubscribeAll([events.settingsOpened(load)]);
  }, []);

  const update = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    setSettings((s) => s && { ...s, [key]: value });
    setStatus(null);
  };

  const save = async () => {
    if (!settings) return;
    try {
      await api.saveSettings({
        ...settings,
        customSensitiveTerms: lines(terms),
        blockedApps: lines(blocked).map((b) => b.toLowerCase()),
      });
    } catch (e) {
      const err = e as CommandError;
      if (err.kind === "save_failed") setStatus({ ok: false, text: describeCommandError(t, err) });
      else setHotkeyError(err);
      return;
    }
    setHotkeyError(null);
    setIssue(null);
    // Windows keeps this choice itself; like every other edit, it applies only once saved.
    if (launchAtLogin !== null && launchAtLogin !== savedLaunchAtLogin) {
      try {
        await api.setLaunchAtLogin(launchAtLogin);
        setSavedLaunchAtLogin(launchAtLogin);
      } catch (e) {
        setStatus({ ok: false, text: t("launchAtLogin.failed", { detail: String(e) }) });
        return;
      }
    }
    await getCurrentWindow().hide();
  };

  const changeLaunchAtLogin = (on: boolean) => {
    setLaunchAtLogin(on);
    setStatus(null);
  };

  return {
    settings,
    update,
    terms,
    setTerms,
    blocked,
    setBlocked,
    status,
    setStatus,
    hotkeyError,
    issue,
    loadCount,
    launchAtLogin,
    changeLaunchAtLogin,
    save,
    lang,
    t,
  };
}

export type SettingsForm = ReturnType<typeof useSettingsForm>;
