import { act, cleanup } from "@testing-library/react";
import { emit } from "@tauri-apps/api/event";
import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { afterEach } from "vitest";

type Args = Record<string, unknown>;
type Handler = (args: Args) => unknown;

export interface Call {
  cmd: string;
  args: Args;
}

afterEach(async () => {
  cleanup();
  // Unmounting unsubscribes asynchronously, through the mocks; let that finish first.
  await new Promise((resolve) => setTimeout(resolve, 0));
  clearMocks();
});

/**
 * Stands in for the Rust side: `handlers` answer commands by name (a missing handler returns
 * nothing) and every command is recorded. Events go through Tauri's mocked event system, so
 * `sendEvent` reaches the components' `listen` subscriptions.
 */
export function fakeBackend(window: "popup" | "settings", handlers: Record<string, Handler> = {}) {
  const calls: Call[] = [];
  mockWindows(window);
  mockIPC(
    (cmd, args) => {
      calls.push({ cmd, args: (args ?? {}) as Args });
      return handlers[cmd]?.((args ?? {}) as Args);
    },
    { shouldMockEvents: true },
  );
  return {
    calls,
    /** Arguments of every call to `cmd`, oldest first. */
    callsTo: (cmd: string) => calls.filter((c) => c.cmd === cmd).map((c) => c.args),
  };
}

/** Lets pending promises (subscriptions, command results) settle and React re-render. */
export async function settle() {
  await act(() => new Promise((resolve) => setTimeout(resolve, 0)));
}

/** Emits an event the way Rust does, then lets the UI react to it. */
export async function sendEvent(name: string, payload: unknown) {
  await act(() => emit(name, payload));
  await settle();
}
