import { Easing, interpolate } from "remotion";

export const ease = Easing.bezier(0.2, 0, 0, 1);

/** Eased 0..1 progress of `frame` between `start` and `start + dur`. */
export const prog = (frame: number, start: number, dur: number): number =>
  interpolate(frame, [start, start + dur], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: ease,
  });

export const lerp = (a: number, b: number, t: number): number => a + (b - a) * t;

export type Key = { f: number; x: number; y: number };

/** Position along eased segments between keyframes (frames ascending). */
export function along(frame: number, keys: Key[]): { x: number; y: number } {
  if (frame <= keys[0].f) return { x: keys[0].x, y: keys[0].y };
  for (let i = 1; i < keys.length; i++) {
    if (frame <= keys[i].f) {
      const a = keys[i - 1];
      const b = keys[i];
      const t = prog(frame, a.f, b.f - a.f);
      return { x: lerp(a.x, b.x, t), y: lerp(a.y, b.y, t) };
    }
  }
  const last = keys[keys.length - 1];
  return { x: last.x, y: last.y };
}

/** Characters written so far in each of several texts streamed one after another. */
export function stream(frame: number, start: number, lengths: number[], cpf = 1.3, gap = 10): number[] {
  const out: number[] = [];
  let at = start;
  for (const len of lengths) {
    out.push(Math.max(0, Math.min(len, Math.floor((frame - at) * cpf))));
    at += len / cpf + gap;
  }
  return out;
}

/** Frame at which `stream` finishes. */
export function streamEnd(start: number, lengths: number[], cpf = 1.3, gap = 10): number {
  return start + lengths.reduce((s, l) => s + l / cpf + gap, 0);
}
