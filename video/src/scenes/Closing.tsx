import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { C } from "../theme";
import { prog } from "../anim";
import { Keycap, Logo, Pop, SceneFade } from "../components/ui";

const StepCard: React.FC<{ n: number; title: string; start: number; children: React.ReactNode }> = ({
  n,
  title,
  start,
  children,
}) => {
  const frame = useCurrentFrame();
  const s = prog(frame, start, 18);
  return (
    <div
      style={{
        flex: 1,
        background: C.surface,
        border: `1px solid ${C.border}`,
        borderRadius: 24,
        boxShadow: "0 18px 44px rgba(20,24,40,0.12)",
        padding: "36px 36px 40px",
        display: "flex",
        flexDirection: "column",
        gap: 22,
        opacity: s,
        transform: `translateY(${(1 - s) * 40}px)`,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 18 }}>
        <span
          style={{
            width: 56,
            height: 56,
            borderRadius: "50%",
            background: C.gradient,
            color: "#fff",
            fontSize: 32,
            fontWeight: 700,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          {n}
        </span>
        <span style={{ fontSize: 40, fontWeight: 700 }}>{title}</span>
      </div>
      {children}
    </div>
  );
};

export const Setup: React.FC = () => {
  const frame = useCurrentFrame();
  const head = prog(frame, 0, 16);
  const dots = Math.min(24, Math.max(0, Math.floor((frame - 110) / 2)));
  return (
    <SceneFade>
      <AbsoluteFill style={{ padding: "0 120px", gap: 64, justifyContent: "center" }}>
        <div style={{ fontSize: 80, fontWeight: 750, textAlign: "center", opacity: head }}>Ready in a minute</div>
        <div style={{ display: "flex", gap: 40, height: 470, alignItems: "stretch" }}>
          <StepCard n={1} title="Install" start={14}>
            <div style={{ fontSize: 32, color: C.text2, lineHeight: 1.45 }}>
              Download the installer from the latest GitHub release. Windows 10 or 11.
            </div>
            <div
              style={{
                marginTop: "auto",
                alignSelf: "flex-start",
                background: C.accent,
                color: "#fff",
                fontSize: 30,
                fontWeight: 650,
                padding: "14px 30px",
                borderRadius: 12,
              }}
            >
              ToneSnap_x64-setup.exe
            </div>
          </StepCard>
          <StepCard n={2} title="Add your key" start={44}>
            <div style={{ fontSize: 32, color: C.text2, lineHeight: 1.45 }}>
              Open Settings from the tray, pick Gemini (free), and paste your API key.
            </div>
            <div
              style={{
                marginTop: "auto",
                border: `2px solid ${C.controlBorder}`,
                borderRadius: 12,
                padding: "14px 22px",
                fontSize: 32,
                letterSpacing: 4,
                color: C.text,
                minHeight: 30,
                background: C.surface2,
              }}
            >
              {"•".repeat(dots)}
            </div>
          </StepCard>
          <StepCard n={3} title="Use it" start={74}>
            <div style={{ fontSize: 32, color: C.text2, lineHeight: 1.45 }}>
              Select text anywhere, then press the hotkey.
            </div>
            <div
              style={{
                marginTop: "auto",
                display: "flex",
                gap: 12,
                alignItems: "center",
                fontSize: 28,
                color: C.text2,
              }}
            >
              <Keycap label="Ctrl" size={30} />+<Keycap label="Alt" size={30} />+<Keycap label="E" size={30} />
            </div>
          </StepCard>
        </div>
      </AbsoluteFill>
    </SceneFade>
  );
};

export const Outro: React.FC = () => {
  const frame = useCurrentFrame();
  const a = prog(frame, 30, 18);
  const b = prog(frame, 56, 18);
  return (
    <SceneFade>
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", flexDirection: "column", gap: 22 }}>
        <Pop start={2}>
          <Logo size={170} />
        </Pop>
        <div
          style={{
            fontSize: 110,
            fontWeight: 750,
            letterSpacing: -1.5,
            opacity: a,
            transform: `translateY(${(1 - a) * 20}px)`,
          }}
        >
          ToneSnap
        </div>
        <div style={{ fontSize: 42, color: C.text2, opacity: a }}>Free and open source · Windows 10/11 · MIT</div>
        <div
          style={{
            marginTop: 26,
            fontSize: 40,
            fontWeight: 650,
            color: "#fff",
            background: C.gradient,
            padding: "20px 48px",
            borderRadius: 999,
            opacity: b,
            transform: `translateY(${(1 - b) * 20}px)`,
          }}
        >
          github.com/nhutruong-struct/struct-tonesnap
        </div>
      </AbsoluteFill>
    </SceneFade>
  );
};
