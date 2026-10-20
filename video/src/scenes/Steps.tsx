import React from "react";
import { spring, useCurrentFrame, useVideoConfig } from "remotion";
import { C } from "../theme";
import { lerp, prog, stream, streamEnd } from "../anim";
import { Caption, Keycap, SceneFade } from "../components/ui";
import { ChatWindow } from "../components/ChatWindow";
import { Popup } from "../components/Popup";
import { SOURCE, VARIANTS } from "../data";

const CHAT = { left: 90, top: 150 };
const POPUP = { left: 1000, top: 130, scale: 1.3 };
const lens = (texts: readonly string[]) => texts.map((t) => t.length);

/** Step 1: select text. */
export const SelectStep: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const enter = prog(frame, 0, 22);
  const typed = Math.min(SOURCE.length, Math.max(0, Math.floor((frame - 20) * 0.7)));
  const selected = prog(frame, 112, 55);
  const iconPop = spring({ frame: frame - 172, fps, config: { damping: 12, stiffness: 170 } });
  const typing = frame < 20 + SOURCE.length / 0.7 + 4;
  // The pointer arrives left of the text, drags along it, then rests at the end.
  const approach = prog(frame, 64, 40);
  const dragging = frame >= 104;
  return (
    <SceneFade>
      <div
        style={{
          position: "absolute",
          left: CHAT.left + 440,
          top: CHAT.top,
          opacity: enter,
          transform: `translateY(${(1 - enter) * 40}px)`,
        }}
      >
        <ChatWindow
          draft={SOURCE.slice(0, typed)}
          caret={typing && Math.floor(frame / 15) % 2 === 0}
          selected={selected}
          icon={Math.max(0, iconPop)}
          cursor={
            frame >= 64
              ? {
                  x: dragging ? `${selected * 100}%` : `${lerp(-90, 0, approach)}px`,
                  y: lerp(140, 20, approach),
                  pressed: dragging && selected < 1,
                  opacity: prog(frame, 64, 10),
                }
              : undefined
          }
        />
      </div>
      <Caption
        step={1}
        title="Select text in any app"
        sub="Slack, Outlook, Teams, a browser — anywhere you can select"
        start={8}
      />
    </SceneFade>
  );
};

/** Step 2: click the icon or press the hotkey; the popup writes one version per tone. */
export const TriggerStep: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const texts = VARIANTS.en.translate;
  const L = lens(texts);
  const start = 84;
  const chars = stream(frame, start, L, 1.4, 12);
  const done = frame >= streamEnd(start, L, 1.4, 12);
  const approach = prog(frame, 6, 36);
  const pressedNow = frame >= 44 && frame < 54;
  const popIn = spring({ frame: frame - 50, fps, config: { damping: 18, stiffness: 120 } });
  const keys = prog(frame, 150, 14);
  const keyDown = frame >= 188 && frame < 204;
  const hover = done ? 1 : undefined;
  return (
    <SceneFade>
      <div style={{ position: "absolute", left: CHAT.left, top: CHAT.top }}>
        <ChatWindow
          draft={SOURCE}
          selected={1}
          icon={pressedNow ? 0.86 : 1}
          ripple={prog(frame, 44, 18) > 0 && frame >= 44 ? prog(frame, 44, 18) : 0}
          cursor={{
            x: `calc(100% + ${36 + lerp(420, 0, approach)}px)`,
            y: -36 + lerp(240, 0, approach),
            pressed: pressedNow,
            opacity: frame < 70 ? 1 : 1 - prog(frame, 70, 14),
          }}
        />
      </div>
      <div
        style={{
          position: "absolute",
          left: POPUP.left,
          top: POPUP.top,
          opacity: Math.min(1, popIn * 1.6),
          transform: `scale(${POPUP.scale * (0.9 + 0.1 * Math.min(1, popIn))}) translateY(${(1 - popIn) * 24}px)`,
          transformOrigin: "top left",
        }}
      >
        <Popup app="Team chat" lang="en" mode="translate" chars={chars} timing="Gemini · 1.4 s" hover={hover} />
      </div>
      <div
        style={{
          position: "absolute",
          top: 38,
          left: 0,
          right: 0,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          gap: 16,
          fontSize: 34,
          color: C.text2,
          opacity: keys,
        }}
      >
        <span>or press</span>
        <Keycap label="Ctrl" size={28} pressed={keyDown} />
        <span>+</span>
        <Keycap label="Alt" size={28} pressed={keyDown} />
        <span>+</span>
        <Keycap label="E" size={28} pressed={keyDown} />
      </div>
      <Caption
        step={2}
        title="Click the icon or press the hotkey"
        sub="One version per tone: Concise, Natural, Formal"
        start={10}
      />
    </SceneFade>
  );
};

/** Choose language and mode in the popup. */
export const OptionsStep: React.FC = () => {
  const frame = useCurrentFrame();
  const move = prog(frame, 0, 26);
  const scale = lerp(POPUP.scale, 1.36, move);
  const left = lerp(POPUP.left, (1920 - 540 * 1.36) / 2 + 250, move);
  const top = lerp(POPUP.top, 96, move);
  const phase = frame < 120 ? 0 : 1; // 0: Japanese translate, 1: English rewrite
  const lang = phase === 0 ? "ja" : "en";
  const mode = phase === 0 ? "translate" : "rewrite";
  const texts = VARIANTS[lang][mode];
  const start = phase === 0 ? 40 : 130;
  const chars = frame < 40 ? VARIANTS.en.translate.map((t) => t.length) : stream(frame, start, lens(texts), 1.1, 8);
  const shownLang = frame < 40 ? "en" : lang;
  const shownMode = frame < 40 ? "translate" : mode;
  const shownTexts = VARIANTS[shownLang][shownMode];
  const safeChars = frame < 40 ? shownTexts.map((t) => t.length) : chars;
  const call1 = prog(frame, 28, 14) * (1 - prog(frame, 116, 10));
  const call2 = prog(frame, 122, 14);
  return (
    <SceneFade>
      <div
        style={{
          position: "absolute",
          left: 90,
          top: 150,
          opacity: 0.28,
          filter: "blur(3px)",
        }}
      >
        <ChatWindow draft={SOURCE} selected={1} />
      </div>
      <div style={{ position: "absolute", left, top, transform: `scale(${scale})`, transformOrigin: "top left" }}>
        <Popup app="Team chat" lang={shownLang} mode={shownMode} chars={safeChars} timing="Gemini · 1.2 s" />
      </div>
      <Callout x={110} y={250} opacity={call1} title="Target language" body="Vietnamese, English or Japanese" />
      <Callout x={110} y={250} opacity={call2} title="Mode" body="Translate stays close. Rewrite rephrases freely." />
      <Caption
        title="Pick the language and mode, right in the popup"
        sub="No settings to open, no prompt to write"
        start={8}
      />
    </SceneFade>
  );
};

const Callout: React.FC<{ x: number; y: number; opacity: number; title: string; body: string }> = ({
  x,
  y,
  opacity,
  title,
  body,
}) => (
  <div
    style={{
      position: "absolute",
      left: x,
      top: y,
      width: 480,
      opacity,
      transform: `translateX(${(1 - opacity) * -30}px)`,
      background: C.surface,
      border: `2px solid ${C.accent}`,
      borderRadius: 18,
      padding: "22px 28px",
      boxShadow: "0 16px 36px rgba(79,95,232,0.2)",
    }}
  >
    <div style={{ fontSize: 34, fontWeight: 700, color: C.accentText }}>{title}</div>
    <div style={{ fontSize: 28, color: C.text2, marginTop: 6, lineHeight: 1.35 }}>{body}</div>
  </div>
);

/** Step 3: apply a version back into the original app. */
export const ApplyStep: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const texts = VARIANTS.en.rewrite;
  const full = texts.map((t) => t.length);
  const keyShow = prog(frame, 40, 12);
  const keyDown = frame >= 70 && frame < 88;
  const applied = frame >= 96;
  const close = prog(frame, 92, 14);
  const flash = applied ? 1 - prog(frame, 96, 40) : 0;
  const toast = spring({ frame: frame - 112, fps, config: { damping: 14, stiffness: 150 } });
  return (
    <SceneFade>
      <div style={{ position: "absolute", left: CHAT.left, top: CHAT.top }}>
        <ChatWindow draft={applied ? texts[1] : SOURCE} selected={applied ? 0 : 1} flash={flash} />
        <div
          style={{
            position: "absolute",
            right: 24,
            top: 8,
            opacity: Math.min(1, Math.max(0, toast)),
            transform: `scale(${0.8 + 0.2 * Math.min(1, Math.max(0, toast))})`,
            transformOrigin: "top right",
            background: C.safeBg,
            color: C.safe,
            border: `1px solid ${C.safe}`,
            borderRadius: 999,
            padding: "10px 22px",
            fontSize: 24,
            fontWeight: 600,
          }}
        >
          ✓ Pasted in place · clipboard restored
        </div>
      </div>
      <div
        style={{
          position: "absolute",
          left: POPUP.left,
          top: POPUP.top,
          opacity: 1 - close,
          transform: `scale(${POPUP.scale * (1 - 0.06 * close)})`,
          transformOrigin: "top left",
        }}
      >
        <Popup
          app="Team chat"
          lang="en"
          mode="rewrite"
          chars={full}
          timing="Gemini · 1.2 s"
          hover={frame >= 20 && frame < 92 ? 1 : undefined}
          pressed={keyDown ? 1 : undefined}
        />
      </div>
      <div
        style={{
          position: "absolute",
          top: 38,
          left: 0,
          right: 0,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          gap: 18,
          fontSize: 34,
          color: C.text2,
          opacity: keyShow * (1 - close),
        }}
      >
        <span>Press</span>
        <Keycap label="2" size={30} pressed={keyDown} />
        <span>or click Apply</span>
      </div>
      <Caption step={3} title="Apply pastes it back in place" sub="Press 1–9 to apply, Shift + 1–9 to copy" start={8} />
    </SceneFade>
  );
};
