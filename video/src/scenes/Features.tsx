import React from "react";
import { AbsoluteFill, Sequence, useCurrentFrame } from "remotion";
import { C } from "../theme";
import { prog } from "../anim";
import { Logo, SceneFade } from "../components/ui";

const Card: React.FC<{ children: React.ReactNode; style?: React.CSSProperties }> = ({ children, style }) => (
  <div
    style={{
      background: C.surface,
      border: `1px solid ${C.border}`,
      borderRadius: 22,
      boxShadow: "0 18px 44px rgba(20,24,40,0.12)",
      ...style,
    }}
  >
    {children}
  </div>
);

const Layout: React.FC<{ kicker: string; title: string; body: string; children: React.ReactNode }> = ({
  kicker,
  title,
  body,
  children,
}) => {
  const frame = useCurrentFrame();
  const t = prog(frame, 4, 18);
  const v = prog(frame, 14, 22);
  return (
    <SceneFade edge={12}>
      <AbsoluteFill style={{ flexDirection: "row", alignItems: "center", padding: "0 140px", gap: 90 }}>
        <div style={{ flex: "0 0 700px", opacity: t, transform: `translateX(${(1 - t) * -40}px)` }}>
          <div
            style={{ fontSize: 30, fontWeight: 700, color: C.accentText, letterSpacing: 2, textTransform: "uppercase" }}
          >
            {kicker}
          </div>
          <div style={{ fontSize: 82, fontWeight: 750, lineHeight: 1.05, margin: "16px 0 26px", color: C.text }}>
            {title}
          </div>
          <div style={{ fontSize: 36, lineHeight: 1.42, color: C.text2 }}>{body}</div>
        </div>
        <div style={{ flex: 1, opacity: v, transform: `translateX(${(1 - v) * 50}px)` }}>{children}</div>
      </AbsoluteFill>
    </SceneFade>
  );
};

const APPS: { name: string; color: string; glyph: string }[] = [
  { name: "Chat", color: "#e5765b", glyph: "#" },
  { name: "Email", color: "#3b7bff", glyph: "@" },
  { name: "Meetings", color: "#7a5cfa", glyph: "▶" },
  { name: "Browser", color: "#2fa36b", glyph: "◎" },
  { name: "Docs", color: "#d9a21b", glyph: "¶" },
  { name: "Notes", color: "#3aa0a5", glyph: "✎" },
];

const AnyApp: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <Layout
      kicker="Works everywhere"
      title="Any app. Any text field."
      body="Reads your selection through UI Automation, then the clipboard, then on-device OCR. Your clipboard is always restored."
    >
      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 28 }}>
        {APPS.map((a, i) => {
          const s = prog(frame, 22 + i * 6, 16);
          return (
            <Card
              key={a.name}
              style={{
                padding: "34px 20px",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: 18,
                opacity: s,
                transform: `scale(${0.85 + 0.15 * s})`,
              }}
            >
              <span
                style={{
                  width: 96,
                  height: 96,
                  borderRadius: 24,
                  background: a.color,
                  color: "#fff",
                  fontSize: 52,
                  fontWeight: 700,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                {a.glyph}
              </span>
              <span style={{ fontSize: 32, fontWeight: 600 }}>{a.name}</span>
            </Card>
          );
        })}
      </div>
    </Layout>
  );
};

const Privacy: React.FC = () => {
  const frame = useCurrentFrame();
  const swap = prog(frame, 46, 14);
  const restore = prog(frame, 84, 14);
  const Tag: React.FC<{ children: string }> = ({ children }) => (
    <span style={{ background: C.safeBg, color: C.safe, borderRadius: 8, padding: "0 10px", fontWeight: 700 }}>
      {children}
    </span>
  );
  return (
    <Layout
      kicker="Private by design"
      title="Sensitive data stays on your PC."
      body="Emails, phone numbers, IDs, card numbers and API keys are masked before anything is sent, and restored when you see the result. No logs, no telemetry."
    >
      <Card style={{ padding: "38px 44px", display: "flex", flexDirection: "column", gap: 30 }}>
        <div style={{ fontSize: 24, fontWeight: 700, color: C.text3, textTransform: "uppercase", letterSpacing: 1.5 }}>
          {restore > 0.5 ? "What you see" : swap > 0.5 ? "What the AI receives" : "What you write"}
        </div>
        <div style={{ fontSize: 40, lineHeight: 1.5 }}>
          Please send it to {swap > 0.5 && restore < 0.5 ? <Tag>[EMAIL_1]</Tag> : <b>an.nguyen@example.com</b>} or call{" "}
          {swap > 0.5 && restore < 0.5 ? <Tag>[PHONE_1]</Tag> : <b>0912 345 678</b>}.
        </div>
        <div style={{ display: "flex", gap: 14, opacity: swap }}>
          {["Emails", "Phones", "IDs", "Cards", "URLs", "API keys"].map((k) => (
            <span
              key={k}
              style={{ fontSize: 24, background: C.surface2, borderRadius: 999, padding: "6px 18px", color: C.text2 }}
            >
              {k}
            </span>
          ))}
        </div>
      </Card>
    </Layout>
  );
};

const PROVIDERS = [
  { name: "Google Gemini", note: "Free tier · default", color: "#3b7bff" },
  { name: "Groq", note: "Free tier", color: "#e5765b" },
  { name: "OpenRouter", note: "Free models", color: "#7a5cfa" },
  { name: "Claude API", note: "Paid", color: "#d9a21b" },
];

const OwnKey: React.FC = () => {
  const frame = useCurrentFrame();
  const key = prog(frame, 70, 16);
  return (
    <Layout
      kicker="Bring your own key"
      title="You choose the AI."
      body="Pick a provider and paste your key. It is kept in Windows Credential Manager, never in a file."
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 22 }}>
        {PROVIDERS.map((p, i) => {
          const s = prog(frame, 20 + i * 8, 16);
          return (
            <Card
              key={p.name}
              style={{
                padding: "22px 30px",
                display: "flex",
                alignItems: "center",
                gap: 24,
                opacity: s,
                transform: `translateY(${(1 - s) * 24}px)`,
              }}
            >
              <span style={{ width: 18, height: 18, borderRadius: "50%", background: p.color }} />
              <span style={{ fontSize: 36, fontWeight: 650, flex: 1 }}>{p.name}</span>
              <span style={{ fontSize: 28, color: C.text2 }}>{p.note}</span>
            </Card>
          );
        })}
        <div style={{ display: "flex", alignItems: "center", gap: 16, fontSize: 30, color: C.safe, opacity: key }}>
          <span style={{ fontSize: 40 }}>🔑</span> Stored in Windows Credential Manager
        </div>
      </div>
    </Layout>
  );
};

const Updates: React.FC = () => {
  const frame = useCurrentFrame();
  const bar = prog(frame, 24, 50);
  const ready = prog(frame, 78, 14);
  return (
    <Layout
      kicker="Always current"
      title="Updates itself."
      body="New versions come from the GitHub releases, are signed and verified, and install while you are not using ToneSnap. Prefer to be asked first? One switch."
    >
      <Card style={{ padding: "40px 44px", display: "flex", flexDirection: "column", gap: 28 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 24 }}>
          <Logo size={84} />
          <div>
            <div style={{ fontSize: 42, fontWeight: 700 }}>ToneSnap 1.1.1</div>
            <div style={{ fontSize: 28, color: C.text2 }}>
              {ready > 0.5 ? "Ready to install" : "Downloading in the background…"}
            </div>
          </div>
        </div>
        <div style={{ height: 16, borderRadius: 8, background: C.surface3, overflow: "hidden" }}>
          <div style={{ height: "100%", width: `${bar * 100}%`, background: C.gradient, borderRadius: 8 }} />
        </div>
        <div
          style={{
            display: "flex",
            justifyContent: "flex-end",
            opacity: ready,
            transform: `translateY(${(1 - ready) * 12}px)`,
          }}
        >
          <span
            style={{
              background: C.accent,
              color: "#fff",
              fontSize: 30,
              fontWeight: 650,
              padding: "14px 34px",
              borderRadius: 12,
            }}
          >
            Update &amp; Restart
          </span>
        </div>
      </Card>
    </Layout>
  );
};

export const FEATURE_FRAMES = 120;

export const Features: React.FC = () => (
  <>
    {[AnyApp, Privacy, OwnKey, Updates].map((Scene, i) => (
      <Sequence key={i} from={i * FEATURE_FRAMES} durationInFrames={FEATURE_FRAMES}>
        <Scene />
      </Sequence>
    ))}
  </>
);
