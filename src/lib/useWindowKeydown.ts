import { useEffect, useLayoutEffect, useRef } from "react";

/** Calls the latest `handler` for every keydown in the window, subscribing only once. */
export function useWindowKeydown(handler: (e: KeyboardEvent) => void) {
  const latest = useRef(handler);
  useLayoutEffect(() => {
    latest.current = handler;
  });
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => latest.current(e);
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
