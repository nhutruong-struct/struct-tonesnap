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
  test("shows each tone with redacted values restored, plus the issues", async () => {
    await answered();
    expect(screen.getByText("Please review my PR, an@corp.vn.")).toBeTruthy();
    expect(screen.getByText("Could you please review my PR, an@corp.vn?")).toBeTruthy();
    expect(screen.getByText("Fixed the spelling.")).toBeTruthy();
    expect(screen.getByText("Spell it out in writing.")).toBeTruthy();
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
