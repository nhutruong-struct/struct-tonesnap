import { useEffect, useReducer } from "react";
import { events, unsubscribeAll, type Notice, type RequestEvent, type RequestOptions, type Session } from "../lib/api";

export type Phase = "idle" | "preview" | "streaming" | "done" | "error";

export interface PopupState {
  session: Session | null;
  notice: Notice | null;
  phase: Phase;
  /** Model output received so far: JSON that may still be incomplete. */
  raw: string;
  /** The finished or failed request's last event, with its timings or error. */
  outcome: RequestEvent | null;
  /** The request on screen; events from requests it replaced are ignored. */
  requestId: number;
}

/** Events that belong to one request, after `request-started`. */
type RequestAction =
  | { type: "delta"; event: RequestEvent }
  | { type: "done"; event: RequestEvent }
  | { type: "failed"; event: RequestEvent };

type Action =
  | { type: "session"; session: Session }
  | { type: "notice"; notice: Notice }
  | { type: "options"; options: RequestOptions }
  | { type: "started"; event: RequestEvent }
  | RequestAction;

const INITIAL: PopupState = {
  session: null,
  notice: null,
  phase: "idle",
  raw: "",
  outcome: null,
  requestId: 0,
};

function reduce(state: PopupState, action: Action): PopupState {
  switch (action.type) {
    case "session":
      return { ...INITIAL, session: action.session, phase: action.session.preview ? "preview" : "streaming" };
    case "notice":
      return { ...state, notice: action.notice };
    case "options":
      if (!state.session) return state;
      return {
        ...state,
        session: { ...state.session, payload: { ...state.session.payload, ...action.options } },
      };
    case "started":
      if (action.event.sessionId !== state.session?.id) return state;
      return {
        ...state,
        requestId: action.event.requestId,
        raw: "",
        outcome: null,
        phase: "streaming",
      };
    case "delta":
    case "done":
    case "failed":
      // Ignore what is left of a request that has been replaced.
      return action.event.requestId === state.requestId ? reduceRequest(state, action) : state;
  }
}

function reduceRequest(state: PopupState, action: RequestAction): PopupState {
  switch (action.type) {
    case "delta":
      return { ...state, raw: state.raw + (action.event.text ?? "") };
    case "done":
      return { ...state, outcome: action.event, phase: "done" };
    case "failed":
      return { ...state, outcome: action.event, phase: "error" };
  }
}

/** What the popup shows, driven by the session, notice and request events from Rust. */
export function usePopupState(): PopupState & { setOptions: (options: RequestOptions) => void } {
  const [state, dispatch] = useReducer(reduce, INITIAL);
  useEffect(
    () =>
      unsubscribeAll([
        events.session((session) => dispatch({ type: "session", session })),
        events.notice((notice) => dispatch({ type: "notice", notice })),
        events.requestStarted((event) => dispatch({ type: "started", event })),
        events.requestDelta((event) => dispatch({ type: "delta", event })),
        events.requestDone((event) => dispatch({ type: "done", event })),
        events.requestFailed((event) => dispatch({ type: "failed", event })),
      ]),
    [],
  );
  return { ...state, setOptions: (options) => dispatch({ type: "options", options }) };
}
