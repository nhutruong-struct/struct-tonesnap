import type { RequestEvent, Session, Settings } from "../lib/api";

/** The Rust defaults (`Settings::default()`), with English for stable assertions. */
export function settings(overrides: Partial<Settings> = {}): Settings {
  return {
    uiLanguage: "en",
    triggerMode: "icon",
    hotkey: "ctrl+alt+e",
    autoDelayMs: 400,
    autoMinChars: 8,
    provider: "gemini",
    model: "claude-opus-5-5",
    effort: "low",
    geminiModel: "gemini-3.8-flash",
    groqModel: "openai/gpt-oss-20b",
    openrouterModel: "openai/gpt-oss-20b:free",
    targetLanguage: "vi",
    mode: "translate",
    tones: ["concise", "natural", "formal"],
    contextChars: 600,
    ocrFallback: true,
    redact: true,
    customSensitiveTerms: [],
    previewBeforeSend: false,
    blockedApps: ["keepass.exe"],
    ...overrides,
  };
}

export function session(overrides: Partial<Session> = {}): Session {
  return {
    id: 1,
    app: "Slack",
    source: "uia",
    contextSource: "uia",
    editable: true,
    selection: "pls review my PR, an@corp.vn",
    payload: {
      app: "Slack",
      contextBefore: "",
      selection: "pls review my PR, [EMAIL_1]",
      contextAfter: "",
      screenText: "",
      targetLanguage: "vi",
      mode: "translate",
      tones: ["concise", "formal"],
    },
    redactions: [["[EMAIL_1]", "an@corp.vn"]],
    preview: false,
    model: "gemini-3.8-flash",
    ...overrides,
  };
}

export function requestEvent(overrides: Partial<RequestEvent> = {}): RequestEvent {
  return { sessionId: 1, requestId: 1, ...overrides };
}

/** A complete model answer for the tones in `session()`. */
export const ANSWER = JSON.stringify({
  variants: {
    concise: { text: "Please review my PR, [EMAIL_1]." },
    formal: { text: "Could you please review my PR, [EMAIL_1]?" },
  },
});
