import React from "react";
import { C } from "../theme";
import { Cursor, Logo } from "./ui";

const Avatar: React.FC<{ color: string; letter: string }> = ({ color, letter }) => (
  <span
    style={{
      width: 52,
      height: 52,
      borderRadius: 12,
      background: color,
      color: "#fff",
      fontWeight: 700,
      fontSize: 24,
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      flexShrink: 0,
    }}
  >
    {letter}
  </span>
);

const Msg: React.FC<{ color: string; letter: string; name: string; text: string }> = ({
  color,
  letter,
  name,
  text,
}) => (
  <div style={{ display: "flex", gap: 16 }}>
    <Avatar color={color} letter={letter} />
    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
      <span style={{ fontSize: 22, fontWeight: 700 }}>
        {name} <span style={{ fontWeight: 400, color: C.text3, fontSize: 18 }}>10:42</span>
      </span>
      <span style={{ fontSize: 24, color: C.text2, lineHeight: 1.35 }}>{text}</span>
    </div>
  </div>
);

export type ChatProps = {
  /** Text in the message box. */
  draft: string;
  /** 0..1: how much of the draft is highlighted as the selection. */
  selected?: number;
  /** Blink the caret at the end of the draft. */
  caret?: boolean;
  /** Draft text that just landed, flashed as a highlight (0..1, fading out). */
  flash?: number;
  /** Floating ToneSnap icon scale (0 = hidden), anchored to the end of the selection. */
  icon?: number;
  /** Mouse pointer placed inside the draft: `x` is a CSS `left` within the draft, `y` px from its top. */
  cursor?: { x: string; y: number; pressed?: boolean; opacity?: number };
  /** Click ripple on the floating icon, 0..1. */
  ripple?: number;
  width?: number;
  height?: number;
};

/** A generic team-chat window; the message box at the bottom is where the demo happens. */
export const ChatWindow: React.FC<ChatProps> = ({
  draft,
  selected = 0,
  caret,
  flash = 0,
  icon = 0,
  cursor,
  ripple = 0,
  width = 880,
  height = 700,
}) => (
  <div
    style={{
      width,
      height,
      background: C.surface,
      borderRadius: 18,
      border: `1px solid ${C.controlBorder}`,
      boxShadow: "0 30px 70px rgba(20,24,40,0.22)",
      overflow: "hidden",
      display: "flex",
      flexDirection: "column",
      color: C.text,
    }}
  >
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 14,
        padding: "18px 26px",
        borderBottom: `1px solid ${C.border}`,
        background: C.surface2,
      }}
    >
      <span style={{ display: "flex", gap: 8 }}>
        {["#ff6b6b", "#ffc24b", "#4cd964"].map((c) => (
          <span key={c} style={{ width: 14, height: 14, borderRadius: "50%", background: c }} />
        ))}
      </span>
      <span style={{ fontSize: 24, fontWeight: 700, marginLeft: 12 }}># dev-team</span>
      <span style={{ fontSize: 18, color: C.text3 }}>Team chat</span>
    </div>
    <div style={{ flex: 1, padding: "28px 30px", display: "flex", flexDirection: "column", gap: 26 }}>
      <Msg
        color="#e5765b"
        letter="M"
        name="Mark"
        text="The release build is green. Who can look at the open PRs today?"
      />
      <Msg color="#3aa0a5" letter="L" name="Lan" text="I am in a meeting until noon, ping me after that." />
    </div>
    <div style={{ padding: "0 30px 30px" }}>
      <div
        style={{
          border: `2px solid ${C.controlBorder}`,
          borderRadius: 14,
          padding: "22px 24px",
          minHeight: 70,
          fontSize: 30,
          position: "relative",
        }}
      >
        <span
          style={{
            position: "relative",
            display: "inline-block",
            lineHeight: 1.3,
            backgroundImage: `linear-gradient(${C.select}, ${C.select})`,
            backgroundRepeat: "no-repeat",
            backgroundSize: `${selected * 100}% 100%`,
            outline: flash > 0 ? `${flash * 6}px solid rgba(79,95,232,${flash * 0.35})` : "none",
            borderRadius: 4,
          }}
        >
          {draft}
          {caret && <span style={{ borderRight: `3px solid ${C.text}`, marginLeft: 2 }} />}
          {icon > 0 && (
            <span
              style={{
                position: "absolute",
                right: -62,
                top: -62,
                transform: `scale(${icon})`,
                filter: "drop-shadow(0 6px 12px rgba(20,24,40,0.3))",
                zIndex: 5,
              }}
            >
              <Logo size={52} />
            </span>
          )}
          {ripple > 0 && ripple < 1 && (
            <span
              style={{
                position: "absolute",
                right: -62 - 4,
                top: -62 - 4,
                width: 60,
                height: 60,
                borderRadius: "50%",
                border: `4px solid ${C.accent}`,
                opacity: 1 - ripple,
                transform: `scale(${0.4 + ripple * 1.4})`,
                zIndex: 4,
              }}
            />
          )}
          {cursor && (
            <span style={{ position: "absolute", left: cursor.x, top: cursor.y, zIndex: 20, pointerEvents: "none" }}>
              <Cursor x={0} y={0} pressed={cursor.pressed} opacity={cursor.opacity ?? 1} />
            </span>
          )}
        </span>
      </div>
    </div>
  </div>
);
