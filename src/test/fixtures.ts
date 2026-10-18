import type { RequestEvent, Session, Settings } from "../lib/api";

/** The Rust defaults (`Settings::default()`), with English for stable assertions. */
export function settings(overrides: Partial<Settings> = {}): Settings {
  return {
    uiLanguage: "en",
    triggerMode: "hotkey",
    hotkey: "ctrl+alt+e",
    autoDelayMs: 400,
    autoMinChars: 8,
    provider: "gemini",
    model: "claude-opus-5-5",
    effort: "low",
    geminiModel: "gemini-3.8-flash",
    groqModel: "openai/gpt-oss-20b",
    openrouterModel: "openai/gpt-oss-20b:free",
    tones: ["minimal", "concise", "polite"],
    explainLanguage: "vi",
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
    translated: false,
    payload: {
      app: "Slack",
      contextBefore: "",
      selection: "pls review my PR, [EMAIL_1]",
      contextAfter: "",
      screenText: "",
      tones: ["minimal", "polite"],
      explanationLanguage: "English",
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
  detected_language: "en",
  variants: {
    minimal: { text: "Please review my PR, [EMAIL_1].", note: "Fixed the spelling." },
    polite: { text: "Could you please review my PR, [EMAIL_1]?", note: "" },
  },
  issues: [{ original: "pls", fix: "please", explanation: "Spell it out in writing." }],
});
