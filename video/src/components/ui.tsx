import React from "react";
import { AbsoluteFill, Img, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import { FONT, C } from "../theme";
import { prog } from "../anim";

export const Logo: React.FC<{ size: number }> = ({ size }) => (
  <Img src={staticFile("icon.svg")} style={{ width: size, height: size, display: "block" }} />
);

/** Soft gradient background with two slow blobs. */
export const Background: React.FC = () => {
  const frame = useCurrentFrame();
  const t = frame / 300;
  return (
    <AbsoluteFill style={{ background: C.bg, overflow: "hidden" }}>
      <div
        style={{
          position: "absolute",
          width: 900,
          height: 900,
          borderRadius: "50%",
          left: -200 + Math.sin(t) * 60,
          top: -300 + Math.cos(t * 0.8) * 50,
          background: "radial-gradient(circle, rgba(59,123,255,0.18), rgba(59,123,255,0) 70%)",
        }}
      />
      <div
        style={{
          position: "absolute",
          width: 1000,
          height: 1000,
          borderRadius: "50%",
          right: -300 + Math.cos(t) * 60,
          bottom: -400 + Math.sin(t * 0.7) * 50,
          background: "radial-gradient(circle, rgba(122,92,250,0.18), rgba(122,92,250,0) 70%)",
        }}
      />
    </AbsoluteFill>
  );
};

/** Fades a scene in and out over `edge` frames; the scene's own length comes from its Sequence. */
export const SceneFade: React.FC<{ edge?: number; children: React.ReactNode }> = ({ edge = 10, children }) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const opacity = interpolate(frame, [0, edge, durationInFrames - edge, durationInFrames], [0, 1, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  return <AbsoluteFill style={{ opacity, fontFamily: FONT }}>{children}</AbsoluteFill>;
};

export const Cursor: React.FC<{ x: number; y: number; opacity?: number; pressed?: boolean }> = ({
  x,
  y,
  opacity = 1,
  pressed = false,
}) => (
  <div
    style={{
      position: "absolute",
      left: x,
      top: y,
      opacity,
      transform: `scale(${pressed ? 0.88 : 1})`,
      transformOrigin: "2px 2px",
      filter: "drop-shadow(0 3px 5px rgba(20,24,40,0.35))",
      zIndex: 50,
    }}
  >
    <svg width="34" height="34" viewBox="0 0 24 24">
      <path
        d="M3 2 L3 18 L7.5 14 L10.5 21 L13 20 L10 13 L16 13 Z"
        fill="#fff"
        stroke="#171a22"
        strokeWidth="1.5"
        strokeLinejoin="round"
      />
    </svg>
  </div>
);

export const Ripple: React.FC<{ x: number; y: number; start: number }> = ({ x, y, start }) => {
  const frame = useCurrentFrame();
  const t = prog(frame, start, 18);
  if (frame < start || t >= 1) return null;
  return (
    <div
      style={{
        position: "absolute",
        left: x - 30,
        top: y - 30,
        width: 60,
        height: 60,
        borderRadius: "50%",
        border: `4px solid ${C.accent}`,
        opacity: 1 - t,
        transform: `scale(${0.3 + t * 1.2})`,
        zIndex: 49,
      }}
    />
  );
};

export const Keycap: React.FC<{ label: string; size?: number; pressed?: boolean }> = ({
  label,
  size = 40,
  pressed,
}) => (
  <span
    style={{
      display: "inline-flex",
      alignItems: "center",
      justifyContent: "center",
      minWidth: size * 1.5,
      height: size * 1.4,
      padding: `0 ${size * 0.4}px`,
      borderRadius: size * 0.28,
      background: pressed ? C.accent : "#fff",
      color: pressed ? "#fff" : C.text,
      border: `2px solid ${pressed ? C.accent : C.controlBorder}`,
      borderBottomWidth: pressed ? 2 : 6,
      fontSize: size,
      fontWeight: 600,
      transform: pressed ? "translateY(4px)" : "none",
      boxShadow: "0 6px 14px rgba(20,24,40,0.12)",
    }}
  >
    {label}
  </span>
);

export const Caption: React.FC<{ step?: number; title: string; sub?: string; start?: number }> = ({
  step,
  title,
  sub,
  start = 4,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - start, fps, config: { damping: 200 } });
  return (
    <div
      style={{
        position: "absolute",
        left: 0,
        right: 0,
        bottom: 70,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 10,
        opacity: s,
        transform: `translateY(${(1 - s) * 30}px)`,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 22 }}>
        {step !== undefined && (
          <span
            style={{
              width: 64,
              height: 64,
              borderRadius: "50%",
              background: C.gradient,
              color: "#fff",
              fontSize: 36,
              fontWeight: 700,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            {step}
          </span>
        )}
        <span style={{ fontSize: 54, fontWeight: 650, color: C.text, letterSpacing: -0.5 }}>{title}</span>
      </div>
      {sub && <span style={{ fontSize: 30, color: C.text2 }}>{sub}</span>}
    </div>
  );
};

export const Pop: React.FC<{ start: number; children: React.ReactNode; style?: React.CSSProperties }> = ({
  start,
  children,
  style,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const s = spring({ frame: frame - start, fps, config: { damping: 14, stiffness: 160 } });
  return <div style={{ transform: `scale(${Math.max(0, s)})`, opacity: Math.min(1, s * 2), ...style }}>{children}</div>;
};
