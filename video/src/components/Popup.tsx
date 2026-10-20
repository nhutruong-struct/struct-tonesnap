import React from "react";
import { C } from "../theme";
import { Logo } from "./ui";
import { Lang, Mode, TONES, VARIANTS } from "../data";

const Seg: React.FC<{ items: string[]; active: number }> = ({ items, active }) => (
  <span
    style={{
      display: "inline-flex",
      padding: 3,
      gap: 2,
      background: C.surface3,
      borderRadius: 8,
      fontSize: 13,
    }}
  >
    {items.map((label, i) => (
      <span
        key={label}
        style={{
          padding: "4px 12px",
          borderRadius: 6,
          background: i === active ? C.surface : "transparent",
          color: i === active ? C.accentText : C.text2,
          fontWeight: i === active ? 600 : 500,
          boxShadow: i === active ? "0 1px 2px rgba(20,24,40,0.12)" : "none",
        }}
      >
        {label}
      </span>
    ))}
  </span>
);

const Chip: React.FC<{ children: React.ReactNode; safe?: boolean }> = ({ children, safe }) => (
  <span
    style={{
      fontSize: 12,
      padding: "3px 10px",
      borderRadius: 999,
      background: safe ? C.safeBg : C.surface2,
      color: safe ? C.safe : C.text2,
      border: `1px solid ${C.border}`,
    }}
  >
    {children}
  </span>
);

const Btn: React.FC<{ primary?: boolean; disabled?: boolean; pressed?: boolean; children: string }> = ({
  primary,
  disabled,
  pressed,
  children,
}) => (
  <span
    style={{
      fontSize: 12.5,
      fontWeight: 600,
      padding: "4px 12px",
      borderRadius: 6,
      background: primary ? C.accent : C.surface,
      color: primary ? "#fff" : C.text,
      border: `1px solid ${primary ? C.accent : C.controlBorder}`,
      opacity: disabled ? 0.45 : 1,
      transform: pressed ? "scale(0.94)" : "none",
    }}
  >
    {children}
  </span>
);

const Cap: React.FC<{ children: string }> = ({ children }) => (
  <span
    style={{
      fontSize: 11,
      padding: "1px 6px",
      borderRadius: 4,
      background: C.surface,
      border: `1px solid ${C.controlBorder}`,
      borderBottomWidth: 2,
      color: C.text,
    }}
  >
    {children}
  </span>
);

export type PopupProps = {
  app: string;
  lang: Lang;
  mode: Mode;
  /** Characters written so far for each tone. */
  chars: [number, number, number] | number[];
  hover?: number;
  pressed?: number;
  /** Footer right side: shown once every tone is written. */
  timing?: string;
  width?: number;
};

/** The ToneSnap popup, drawn like src/popup at its real 540 px width. */
export const Popup: React.FC<PopupProps> = ({ app, lang, mode, chars, hover, pressed, timing, width = 540 }) => {
  const texts = VARIANTS[lang][mode];
  const writing = chars.some((n, i) => n < texts[i].length);
  return (
    <div
      style={{
        width,
        background: C.bg,
        borderRadius: 14,
        border: `1px solid ${C.controlBorder}`,
        boxShadow: "0 24px 60px rgba(20,24,40,0.28)",
        overflow: "hidden",
        color: C.text,
        fontSize: 14,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px", background: C.surface }}>
        <Logo size={20} />
        <span style={{ fontWeight: 600 }}>{app}</span>
        <span style={{ flex: 1 }} />
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke={C.text2}
          strokeWidth="2"
          strokeLinecap="round"
        >
          <circle cx="12" cy="12" r="3" />
          <path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1 7 17M17 7l2.1-2.1" />
        </svg>
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke={C.text2}
          strokeWidth="2"
          strokeLinecap="round"
        >
          <path d="M6 6l12 12M18 6 6 18" />
        </svg>
      </div>
      <div style={{ padding: "12px 14px", display: "flex", flexDirection: "column", gap: 10 }}>
        <div style={{ display: "flex", gap: 6 }}>
          <Chip>Read directly</Chip>
          <Chip>Context: surrounding text</Chip>
        </div>
        <div style={{ display: "flex", gap: 10 }}>
          <Seg items={["Vietnamese", "English", "Japanese"]} active={lang === "en" ? 1 : 2} />
          <Seg items={["Translate", "Rewrite"]} active={mode === "translate" ? 0 : 1} />
        </div>
        {TONES.map((tone, i) => {
          const n = chars[i] ?? 0;
          const full = n >= texts[i].length;
          const started = n > 0;
          return (
            <div
              key={tone}
              style={{
                borderRadius: 12,
                padding: "10px 12px",
                border: full ? `1px solid ${hover === i ? C.accent : C.border}` : `1px dashed ${C.controlBorder}`,
                background: full ? (hover === i ? C.accentSoft : C.surface) : "transparent",
                boxShadow: full ? "0 1px 2px rgba(20,24,40,0.06)" : "none",
                minHeight: 74,
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
                <span
                  style={{
                    width: 20,
                    height: 20,
                    borderRadius: 6,
                    background: C.accentSoft,
                    color: C.accentText,
                    fontSize: 12,
                    fontWeight: 700,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                  }}
                >
                  {i + 1}
                </span>
                <span style={{ fontWeight: 600 }}>{tone}</span>
                <span style={{ flex: 1 }} />
                <Btn primary disabled={!full} pressed={pressed === i}>
                  Apply
                </Btn>
                <Btn disabled={!full}>Copy</Btn>
              </div>
              {started ? (
                <div style={{ lineHeight: 1.45, minHeight: 20 }}>
                  {texts[i].slice(0, n)}
                  {!full && <span style={{ color: C.accent }}>▍</span>}
                </div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: 6, paddingTop: 4 }}>
                  <span style={{ height: 8, width: "85%", borderRadius: 4, background: C.surface3 }} />
                  <span style={{ height: 8, width: "55%", borderRadius: 4, background: C.surface3 }} />
                </div>
              )}
            </div>
          );
        })}
      </div>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 12,
          padding: "8px 14px",
          borderTop: `1px solid ${C.border}`,
          background: C.surface,
          fontSize: 12,
          color: C.text2,
        }}
      >
        <span style={{ display: "flex", gap: 4, alignItems: "center" }}>
          <Cap>1–9</Cap> Apply
        </span>
        <span style={{ display: "flex", gap: 4, alignItems: "center" }}>
          <Cap>Shift+1–9</Cap> Copy
        </span>
        <span style={{ display: "flex", gap: 4, alignItems: "center" }}>
          <Cap>R</Cap> Regenerate
        </span>
        <span style={{ display: "flex", gap: 4, alignItems: "center" }}>
          <Cap>Esc</Cap> Close
        </span>
        <span style={{ flex: 1 }} />
        <span>{writing ? "Writing…" : (timing ?? "")}</span>
      </div>
    </div>
  );
};
