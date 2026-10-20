import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { C } from "../theme";
import { prog } from "../anim";
import { Logo, Pop, SceneFade } from "../components/ui";
import { SOURCE } from "../data";

export const Hook: React.FC = () => {
  const frame = useCurrentFrame();
  const a = prog(frame, 6, 18);
  const b = prog(frame, 50, 18);
  const bubble = prog(frame, 84, 18);
  const dots = Math.floor(frame / 8) % 4;
  return (
    <SceneFade>
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", gap: 28, flexDirection: "column" }}>
        <div
          style={{
            fontSize: 92,
            fontWeight: 700,
            color: C.text,
            opacity: a,
            transform: `translateY(${(1 - a) * 30}px)`,
          }}
        >
          Writing English at work?
        </div>
        <div
          style={{
            fontSize: 92,
            fontWeight: 700,
            opacity: b,
            transform: `translateY(${(1 - b) * 30}px)`,
            background: C.gradient,
            WebkitBackgroundClip: "text",
            color: "transparent",
          }}
        >
          Not sure how it sounds?
        </div>
        <div
          style={{
            marginTop: 40,
            opacity: bubble,
            transform: `translateY(${(1 - bubble) * 24}px)`,
            display: "flex",
            alignItems: "center",
            gap: 24,
          }}
        >
          <span
            style={{
              fontSize: 40,
              background: C.surface,
              border: `2px solid ${C.controlBorder}`,
              borderRadius: 18,
              padding: "16px 28px",
              boxShadow: "0 12px 30px rgba(20,24,40,0.12)",
            }}
          >
            {SOURCE}
          </span>
          <span style={{ fontSize: 56, color: C.text3, width: 120 }}>{".".repeat(dots)}?</span>
        </div>
      </AbsoluteFill>
    </SceneFade>
  );
};

export const Intro: React.FC = () => {
  const frame = useCurrentFrame();
  const t = prog(frame, 40, 20);
  const u = prog(frame, 62, 20);
  return (
    <SceneFade>
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", flexDirection: "column", gap: 18 }}>
        <Pop start={4}>
          <Logo size={220} />
        </Pop>
        <div
          style={{
            fontSize: 132,
            fontWeight: 750,
            letterSpacing: -2,
            color: C.text,
            opacity: t,
            transform: `translateY(${(1 - t) * 24}px)`,
          }}
        >
          ToneSnap
        </div>
        <div
          style={{
            fontSize: 46,
            color: C.text2,
            textAlign: "center",
            lineHeight: 1.4,
            opacity: u,
            transform: `translateY(${(1 - u) * 24}px)`,
          }}
        >
          Select text in any app.
          <br />
          Get it back in the tone you need.
        </div>
      </AbsoluteFill>
    </SceneFade>
  );
};
