import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { openUrl } from "@tauri-apps/plugin-opener";

export type ToneId = "minimal" | "concise" | "natural" | "polite" | "formal" | "friendly";

export interface Payload {
  app: string;
  contextBefore: string;
  selection: string;
  contextAfter: string;
  screenText: string;
  tones: ToneId[];
  explanationLanguage: string;
}

export interface Session {
  id: number;
  app: string;
  source: "uia" | "clipboard";
  contextSource: "uia" | "ocr" | "none";
  editable: boolean | null;
  /** The original selection, before redaction. */
  selection: string;
  /** The selection is Vietnamese, so the suggestions are translations. */
  translated: boolean;
  payload: Payload;
  redactions: [string, string][];
  preview: boolean;
  model: string;
}

export interface RequestEvent {
  sessionId: number;
  requestId: number;
  text?: string | null;
  stopReason?: string | null;
  model?: string | null;
  firstTokenMs?: number | null;
  totalMs?: number | null;
  usage?: Record<string, unknown> | null;
  errorKind?: string | null;
  errorStatus?: number | null;
  errorMessage?: string | null;
}

export type NoticeKind = "no_selection" | "password_field" | "blocked_app" | "apply_failed";

export interface Notice {
  kind: NoticeKind;
  detail: string;
}

/** Model output, possibly still incomplete while streaming. */
export interface Suggestions {
  detected_language?: string;
  /** Keyed by tone id, in the requested order. */
  variants?: Partial<Record<string, { text?: string; note?: string }>>;
  issues?: { original?: string; fix?: string; explanation?: string }[];
}

export type ProviderId = "claude" | "gemini" | "groq" | "openrouter";

export type TriggerMode = "hotkey" | "icon" | "auto" | "off";

/** `auto` installs a new version when ToneSnap is idle; `manual` only says that it exists. */
export type UpdateMode = "auto" | "manual";

export interface Settings {
  uiLanguage: "auto" | "vi" | "en";
  triggerMode: TriggerMode;
  hotkey: string;
  autoDelayMs: number;
  autoMinChars: number;
  provider: ProviderId;
  /** Claude model id. */
  model: string;
  effort: string;
  geminiModel: string;
  groqModel: string;
  openrouterModel: string;
  tones: ToneId[];
  explainLanguage: "vi" | "en" | "off";
  contextChars: number;
  ocrFallback: boolean;
  redact: boolean;
  customSensitiveTerms: string[];
  previewBeforeSend: boolean;
  blockedApps: string[];
  updateMode: UpdateMode;
}

/** Rejection value of `saveSettings`, also returned by `hotkeyError`. */
export interface CommandError {
  kind: "hotkey_invalid" | "hotkey_taken" | "save_failed";
  hotkey: string;
  detail: string;
}

/** What was wrong with the saved settings file, until the settings are saved again. */
export type SettingsIssue =
  { kind: "invalid_fields"; fields: string[] } | { kind: "broken"; backup: string } | { kind: "unreadable" };

/** Where updating stands (`update.rs`). */
export type UpdateStatus =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "up_to_date" }
  | { kind: "available"; version: string }
  | { kind: "downloading"; version: string }
  | { kind: "ready"; version: string }
  | { kind: "installing"; version: string }
  | { kind: "failed"; step: "check" | "download" | "install"; detail: string };

export interface UpdateInfo {
  currentVersion: string;
  status: UpdateStatus;
  /** Installed with the MSI: updates need administrator rights, so they always wait for the user. */
  msiInstall: boolean;
}

export const api = {
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (newSettings: Settings) => invoke<void>("save_settings", { newSettings }),
  hotkeyError: () => invoke<CommandError | null>("hotkey_error"),
  settingsIssue: () => invoke<SettingsIssue | null>("settings_issue"),
  hasApiKey: (provider: ProviderId) => invoke<boolean>("has_api_key", { provider }),
  /** An empty key removes the provider's saved key. */
  setApiKey: (provider: ProviderId, key: string) => invoke<void>("set_api_key", { provider, key }),
  sendRequest: () => invoke<void>("send_request"),
  applyText: (text: string) => invoke<void>("apply_text", { text }),
  copyText: (text: string) => invoke<void>("copy_text", { text }),
  hidePopup: () => invoke<void>("hide_popup"),
  openSettings: () => invoke<void>("open_settings"),
  updateInfo: () => invoke<UpdateInfo>("update_info"),
  checkForUpdates: () => invoke<void>("check_for_updates"),
  /** Downloads the offered version if needed and installs it; ToneSnap restarts. */
  installUpdate: () => invoke<void>("install_update"),
  /** Whether Windows starts ToneSnap at sign-in. Windows keeps this choice, not settings.json. */
  launchAtLogin: () => isEnabled(),
  setLaunchAtLogin: (on: boolean) => (on ? enable() : disable()),
  /** Opens a web page in the default browser. */
  openUrl: (url: string) => openUrl(url),
};

type Listener<T> = (payload: T) => void;

/**
 * Typed subscriptions to the events Rust emits; each resolves to its unsubscribe function.
 * Event names stay literal so the harness can match them against the Rust `emit` calls.
 */
export const events = {
  session: (f: Listener<Session>) => listen<Session>("session", (e) => f(e.payload)),
  notice: (f: Listener<Notice>) => listen<Notice>("notice", (e) => f(e.payload)),
  requestStarted: (f: Listener<RequestEvent>) => listen<RequestEvent>("request-started", (e) => f(e.payload)),
  requestDelta: (f: Listener<RequestEvent>) => listen<RequestEvent>("request-delta", (e) => f(e.payload)),
  requestDone: (f: Listener<RequestEvent>) => listen<RequestEvent>("request-done", (e) => f(e.payload)),
  requestFailed: (f: Listener<RequestEvent>) => listen<RequestEvent>("request-failed", (e) => f(e.payload)),
  settingsOpened: (f: () => void) => listen("settings-opened", () => f()),
  settingsChanged: (f: Listener<Settings>) => listen<Settings>("settings-changed", (e) => f(e.payload)),
  updateStatus: (f: Listener<UpdateInfo>) => listen<UpdateInfo>("update-status", (e) => f(e.payload)),
};

/** Unsubscribes from all of `subscriptions`; the cleanup for a subscribing `useEffect`. */
export function unsubscribeAll(subscriptions: Promise<UnlistenFn>[]): () => void {
  return () => {
    for (const s of subscriptions) void s.then((unlisten) => unlisten());
  };
}
