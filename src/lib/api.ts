import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { openUrl } from "@tauri-apps/plugin-opener";

export type ToneId = "concise" | "natural" | "formal";

/** The language every version is written in. */
export type TargetLanguage = "vi" | "en" | "ja";

/** Translate stays close to the original; rewrite may rephrase and restructure it. */
export type Mode = "translate" | "rewrite";

/** What the popup lets the user change for one request. */
export interface RequestOptions {
  targetLanguage: TargetLanguage;
  mode: Mode;
}

export interface Payload extends RequestOptions {
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
  targetLanguage: TargetLanguage;
  mode: Mode;
  tones: ToneId[];
  explainLanguage: "vi" | "en" | "off";
  contextChars: number;
  ocrFallback: boolean;
  redact: boolean;
  customSensitiveTerms: string[];
  previewBeforeSend: boolean;
  blockedApps: string[];
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

export const api = {
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (newSettings: Settings) => invoke<void>("save_settings", { newSettings }),
  hotkeyError: () => invoke<CommandError | null>("hotkey_error"),
  settingsIssue: () => invoke<SettingsIssue | null>("settings_issue"),
  hasApiKey: (provider: ProviderId) => invoke<boolean>("has_api_key", { provider }),
  /** An empty key removes the provider's saved key. */
  setApiKey: (provider: ProviderId, key: string) => invoke<void>("set_api_key", { provider, key }),
  /** Sends the session, or asks again, in the target language and mode the popup shows. */
  sendRequest: (options: RequestOptions) => invoke<void>("send_request", { ...options }),
  applyText: (text: string) => invoke<void>("apply_text", { text }),
  copyText: (text: string) => invoke<void>("copy_text", { text }),
  hidePopup: () => invoke<void>("hide_popup"),
  openSettings: () => invoke<void>("open_settings"),
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
};

/** Unsubscribes from all of `subscriptions`; the cleanup for a subscribing `useEffect`. */
export function unsubscribeAll(subscriptions: Promise<UnlistenFn>[]): () => void {
  return () => {
    for (const s of subscriptions) void s.then((unlisten) => unlisten());
  };
}
