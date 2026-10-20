import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { fakeBackend, sendEvent, settle } from "../test/backend";
import { ANSWER, requestEvent, session, settings } from "../test/fixtures";
import Popup from "./Popup";

let backend: ReturnType<typeof fakeBackend>;

beforeEach(async () => {
  backend = fakeBackend("popup", { get_settings: () => settings() });
  render(<Popup />);
  await settle();
});

/** A session whose request streamed `answer` and finished. */
async function answered(overrides = {}, answer = ANSWER) {
  await sendEvent("session", session(overrides));
  await sendEvent("request-started", requestEvent());
  await sendEvent("request-delta", requestEvent({ text: answer.slice(0, 40) }));
  await sendEvent("request-delta", requestEvent({ text: answer.slice(40) }));
  await sendEvent("request-done", requestEvent({ firstTokenMs: 800, totalMs: 2400, model: "gemini-3.8-flash" }));
}

function key(init: KeyboardEventInit) {
  act(() => {
    fireEvent.keyDown(window, init);
  });
}

describe("suggestions", () => {
  test("shows each tone with redacted values restored", async () => {
    await answered();
    expect(screen.getByText("Please review my PR, an@corp.vn.")).toBeTruthy();
    expect(screen.getByText("Could you please review my PR, an@corp.vn?")).toBeTruthy();
    expect(screen.getByText("0.8s / 2.4s · gemini-3.8-flash")).toBeTruthy();
  });

  test("number keys apply, Shift+number copies", async () => {
    await answered();
    key({ key: "1", code: "Digit1" });
    expect(backend.callsTo("apply_text")).toEqual([{ text: "Please review my PR, an@corp.vn." }]);
    key({ key: "!", code: "Digit2", shiftKey: true });
    expect(backend.callsTo("copy_text")).toEqual([{ text: "Could you please review my PR, an@corp.vn?" }]);
  });

  test("the Apply button pastes the variant", async () => {
    await answered();
    fireEvent.click(screen.getAllByText("Apply")[1]);
    expect(backend.callsTo("apply_text")).toEqual([{ text: "Could you please review my PR, an@corp.vn?" }]);
  });

  test("a read-only field offers Copy only, and number keys copy", async () => {
    await answered({ editable: false });
    expect(screen.queryByText("Apply")).toBeNull();
    key({ key: "1", code: "Digit1" });
    expect(backend.callsTo("apply_text")).toEqual([]);
    expect(backend.callsTo("copy_text")).toEqual([{ text: "Please review my PR, an@corp.vn." }]);
  });

  test("a variant still being written cannot be applied", async () => {
    await sendEvent("session", session());
    await sendEvent("request-started", requestEvent());
    await sendEvent("request-delta", requestEvent({ text: ANSWER.slice(0, ANSWER.indexOf('"formal"') - 2) }));
    key({ key: "1", code: "Digit1" });
    expect(backend.callsTo("apply_text")).toEqual([]);
    expect(screen.getByText("Writing…")).toBeTruthy();
  });

  test("events from a replaced request are ignored", async () => {
    await sendEvent("session", session());
    await sendEvent("request-started", requestEvent({ requestId: 2 }));
    await sendEvent("request-delta", requestEvent({ requestId: 1, text: ANSWER }));
    expect(screen.queryByText("Please review my PR, an@corp.vn.")).toBeNull();
    await sendEvent("request-started", requestEvent({ sessionId: 99, requestId: 3 }));
    await sendEvent("request-delta", requestEvent({ requestId: 2, text: ANSWER }));
    expect(screen.getByText("Please review my PR, an@corp.vn.")).toBeTruthy();
  });
});

describe("language and mode", () => {
  const radio = (name: string) => screen.getByRole("radio", { name });
  const sent = (backend: ReturnType<typeof fakeBackend>) => backend.callsTo("send_request");

  test("shows the language and mode the session starts with", async () => {
    await answered();
    expect(radio("Vietnamese").getAttribute("aria-checked")).toBe("true");
    expect(radio("English").getAttribute("aria-checked")).toBe("false");
    expect(radio("Translate").getAttribute("aria-checked")).toBe("true");
    expect(radio("Rewrite").getAttribute("aria-checked")).toBe("false");
  });

  test("choosing another language asks again in that language and keeps the mode", async () => {
    await answered();
    fireEvent.click(radio("Japanese"));
    expect(sent(backend)).toEqual([{ targetLanguage: "ja", mode: "translate" }]);
    expect(radio("Japanese").getAttribute("aria-checked")).toBe("true");
    expect(radio("Vietnamese").getAttribute("aria-checked")).toBe("false");
  });

  test("choosing rewrite asks again, and R then repeats the request with what is on screen", async () => {
    await answered();
    fireEvent.click(radio("Rewrite"));
    key({ key: "r", code: "KeyR" });
    expect(sent(backend)).toEqual([
      { targetLanguage: "vi", mode: "rewrite" },
      { targetLanguage: "vi", mode: "rewrite" },
    ]);
  });

  test("choosing what is already chosen sends nothing", async () => {
    await answered();
    fireEvent.click(radio("Vietnamese"));
    fireEvent.click(radio("Translate"));
    expect(sent(backend)).toEqual([]);
  });

  test("while previewing, a choice is only sent together with the data", async () => {
    await sendEvent("session", session({ preview: true }));
    fireEvent.click(radio("English"));
    expect(sent(backend)).toEqual([]);
    expect(screen.getByText("Target language").nextElementSibling?.textContent).toBe("English");
    key({ key: "Enter", code: "Enter" });
    expect(sent(backend)).toEqual([{ targetLanguage: "en", mode: "translate" }]);
  });

  test("a new session starts from its own language and mode again", async () => {
    await answered();
    fireEvent.click(radio("Japanese"));
    await answered({ id: 2, payload: { ...session().payload, targetLanguage: "en", mode: "rewrite" } });
    expect(radio("English").getAttribute("aria-checked")).toBe("true");
    expect(radio("Rewrite").getAttribute("aria-checked")).toBe("true");
  });
});

describe("keys", () => {
  test("Escape closes, R asks again", async () => {
    await answered();
    key({ key: "r", code: "KeyR" });
    expect(backend.callsTo("send_request")).toEqual([{ targetLanguage: "vi", mode: "translate" }]);
    key({ key: "r", code: "KeyR", ctrlKey: true });
    expect(backend.callsTo("send_request")).toHaveLength(1);
    key({ key: "Escape", code: "Escape" });
    expect(backend.callsTo("hide_popup")).toHaveLength(1);
  });
});

describe("footer", () => {
  test("lists the keys that work", async () => {
    await answered();
    expect(screen.getByText("Shift+1–9")).toBeTruthy();
    expect(screen.getByText("Regenerate")).toBeTruthy();
  });

  test("offers only Copy for a read-only field, and only Esc while previewing", async () => {
    await answered({ editable: false });
    expect(screen.queryByText("Shift+1–9")).toBeNull();
    expect(screen.queryByText("Apply")).toBeNull();

    await sendEvent("session", session({ preview: true }));
    expect(screen.queryByText("Regenerate")).toBeNull();
    expect(screen.getByText("Esc")).toBeTruthy();
  });
});

describe("preview", () => {
  test("shows what will be sent and sends on Enter", async () => {
    await sendEvent("session", session({ preview: true }));
    expect(screen.getByText("pls review my PR, [EMAIL_1]")).toBeTruthy();
    expect(backend.callsTo("send_request")).toHaveLength(0);
    key({ key: "r", code: "KeyR" });
    expect(backend.callsTo("send_request")).toHaveLength(0);
    key({ key: "Enter", code: "Enter" });
    expect(backend.callsTo("send_request")).toHaveLength(1);
  });
});

describe("errors", () => {
  test("a known error is explained, with Retry and a way to Settings", async () => {
    await sendEvent("session", session());
    await sendEvent("request-started", requestEvent());
    await sendEvent("request-failed", requestEvent({ errorKind: "authentication_error", errorStatus: 401 }));
    expect(screen.getByText("Invalid API key")).toBeTruthy();
    expect(screen.getByText("Check the key in Settings.")).toBeTruthy();
    fireEvent.click(screen.getByText("Retry (R)"));
    expect(backend.callsTo("send_request")).toHaveLength(1);
    fireEvent.click(screen.getByText("Open Settings"));
    expect(backend.callsTo("open_settings")).toHaveLength(1);
  });

  test("an unknown error shows its status and message", async () => {
    await sendEvent("session", session());
    await sendEvent("request-started", requestEvent());
    await sendEvent(
      "request-failed",
      requestEvent({ errorKind: "teapot_error", errorStatus: 418, errorMessage: "I am a teapot" }),
    );
    expect(screen.getByText("Error 418")).toBeTruthy();
    expect(screen.getByText("I am a teapot")).toBeTruthy();
    expect(screen.queryByText("Open Settings")).toBeNull();
  });
});

describe("notices", () => {
  test("a notice replaces the suggestions and closes on its own", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    await sendEvent("notice", { kind: "blocked_app", detail: "KeePass" });
    expect(screen.getByText(/KeePass/)).toBeTruthy();
    expect(backend.callsTo("hide_popup")).toHaveLength(0);
    await act(() => vi.advanceTimersByTimeAsync(3600));
    expect(backend.callsTo("hide_popup")).toHaveLength(1);
    vi.useRealTimers();
  });

  test("a new session clears the notice", async () => {
    await sendEvent("notice", { kind: "no_selection", detail: "" });
    await answered();
    expect(screen.getByText("Please review my PR, an@corp.vn.")).toBeTruthy();
  });
});

describe("header", () => {
  test("names the app and says where the selection and context came from", async () => {
    await answered();
    expect(screen.getByText("Slack")).toBeTruthy();
    expect(screen.getByText("Read directly")).toBeTruthy();
    expect(screen.getByText("Context: surrounding text")).toBeTruthy();
    expect(screen.getByText("1 sensitive items masked")).toBeTruthy();
    expect(screen.queryByText("Read-only – use Copy")).toBeNull();
  });

  test("tells a clipboard read without context, and a read-only field", async () => {
    await answered({ source: "clipboard", contextSource: "none", editable: false, redactions: [] });
    expect(screen.getByText("Via clipboard")).toBeTruthy();
    expect(screen.getByText("No context")).toBeTruthy();
    expect(screen.getByText("Read-only – use Copy")).toBeTruthy();
    expect(screen.queryByText(/sensitive items masked/)).toBeNull();
  });

  test("says when the context was read from the screen", async () => {
    await answered({ contextSource: "ocr" });
    expect(screen.getByText("Context: screen OCR")).toBeTruthy();
  });

  test("the gear opens Settings and the cross closes the popup", async () => {
    await answered();
    fireEvent.click(screen.getByTitle("Settings"));
    expect(backend.callsTo("open_settings")).toHaveLength(1);
    fireEvent.click(screen.getByTitle("Close (Esc)"));
    expect(backend.callsTo("hide_popup")).toHaveLength(1);
  });
});

describe("before a session", () => {
  test("asks for a selection, and the number keys do nothing", () => {
    expect(screen.getByText("Select some text, then press the hotkey.")).toBeTruthy();
    key({ key: "1", code: "Digit1" });
    key({ key: "r", code: "KeyR" });
    expect(backend.callsTo("apply_text")).toEqual([]);
    expect(backend.callsTo("send_request")).toEqual([]);
  });
});

describe("progress and timings", () => {
  test("shows the model from the session and a dash for a missing timing", async () => {
    await sendEvent("session", session());
    await sendEvent("request-started", requestEvent());
    await sendEvent("request-delta", requestEvent({ text: ANSWER }));
    await sendEvent("request-done", requestEvent({ firstTokenMs: null, totalMs: 1500 }));
    expect(screen.getByText("– / 1.5s · gemini-3.8-flash")).toBeTruthy();
  });

  test("a number key beyond the requested tones does nothing", async () => {
    await answered();
    key({ key: "5", code: "Digit5" });
    key({ key: "%", code: "Digit5", shiftKey: true });
    expect(backend.callsTo("apply_text")).toEqual([]);
    expect(backend.callsTo("copy_text")).toEqual([]);
  });
});

describe("more errors", () => {
  async function failedWith(event: Partial<ReturnType<typeof requestEvent>>) {
    await sendEvent("session", session());
    await sendEvent("request-started", requestEvent());
    await sendEvent("request-failed", requestEvent(event));
  }

  test("a rate limit can only be retried", async () => {
    await failedWith({ errorKind: "rate_limit_error", errorStatus: 429 });
    expect(screen.getByText("Rate limited")).toBeTruthy();
    expect(screen.getByText("Try again in a few seconds (R).")).toBeTruthy();
    expect(screen.getByText("Retry (R)")).toBeTruthy();
    expect(screen.queryByText("Open Settings")).toBeNull();
  });

  test("a missing key points to Settings", async () => {
    await failedWith({ errorKind: "no_api_key" });
    expect(screen.getByText("No API key yet")).toBeTruthy();
    expect(screen.getByText("Open Settings to enter your API key.")).toBeTruthy();
    fireEvent.click(screen.getByText("Open Settings"));
    expect(backend.callsTo("open_settings")).toHaveLength(1);
  });

  test("a connection error shows the message it came with", async () => {
    await failedWith({ errorKind: "network", errorMessage: "dns error" });
    expect(screen.getByText("Connection error")).toBeTruthy();
    expect(screen.getByText("dns error")).toBeTruthy();
  });

  test("a refused or cut-off answer is explained without a status", async () => {
    await failedWith({ errorKind: "max_tokens" });
    expect(screen.getByText("The result was too long and got cut off")).toBeTruthy();
    expect(screen.getByText("Select a shorter passage and try again.")).toBeTruthy();
  });

  test("retrying starts from a clean slate", async () => {
    await failedWith({ errorKind: "overloaded_error", errorStatus: 529 });
    fireEvent.click(screen.getByText("Retry (R)"));
    expect(backend.callsTo("send_request")).toEqual([{ targetLanguage: "vi", mode: "translate" }]);
    await sendEvent("request-started", requestEvent({ requestId: 2 }));
    expect(screen.queryByText("The AI service is overloaded")).toBeNull();
    expect(screen.getByText("Writing…")).toBeTruthy();
  });
});

describe("more preview", () => {
  test("lists the data that will be sent and hides empty context", async () => {
    await sendEvent("session", session({ preview: true }));
    expect(screen.getByText("Slack", { selector: "dd" })).toBeTruthy();
    expect(screen.getByText("Target language", { selector: "dt" })).toBeTruthy();
    expect(screen.getByText("Mode", { selector: "dt" })).toBeTruthy();
    expect(screen.queryByText("Before")).toBeNull();
    expect(screen.queryByText("After")).toBeNull();
    expect(screen.queryByText("Screen text (OCR)")).toBeNull();

    const payload = { ...session().payload, contextBefore: "Hi Sarah,", screenText: "Inbox" };
    await sendEvent("session", session({ id: 2, preview: true, payload }));
    expect(screen.getByText("Hi Sarah,")).toBeTruthy();
    expect(screen.getByText("Inbox")).toBeTruthy();
  });

  test("Cancel closes the popup without sending", async () => {
    await sendEvent("session", session({ preview: true }));
    fireEvent.click(screen.getByText("Cancel"));
    expect(backend.callsTo("hide_popup")).toHaveLength(1);
    expect(backend.callsTo("send_request")).toEqual([]);
  });
});

describe("more notices", () => {
  test("the Close button dismisses a notice at once", async () => {
    await sendEvent("notice", { kind: "password_field", detail: "" });
    expect(screen.getByText("ToneSnap never reads password fields.")).toBeTruthy();
    fireEvent.click(screen.getByText("Close"));
    expect(backend.callsTo("hide_popup")).toHaveLength(1);
  });

  test("says that a failed Apply left the text on the clipboard", async () => {
    await sendEvent("notice", { kind: "apply_failed", detail: "" });
    expect(screen.getByText(/It is on your clipboard; press Ctrl\+V to paste/)).toBeTruthy();
  });
});
