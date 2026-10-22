import { useEffect, useMemo } from "react";
import { api, type Notice, type RequestEvent, type RequestOptions, type Session } from "../lib/api";
import { translator, useSavedLang, type Translate } from "../lib/i18n";
import { Icon, Logo } from "../lib/icons";
import { useWindowKeydown } from "../lib/useWindowKeydown";
import { ErrorPanel } from "./ErrorPanel";
import { OptionsBar } from "./OptionsBar";
import { PreviewPanel } from "./PreviewPanel";
import { VariantList } from "./SuggestionList";
import { parseSuggestions, toVariants } from "./suggestions";
import { usePopupState, type Phase } from "./usePopupState";

function seconds(ms?: number | null) {
  return ms == null ? "–" : `${(ms / 1000).toFixed(1)}s`;
}

export default function Popup() {
  const t = translator(useSavedLang());
  const { session, notice, phase, raw, outcome, setOptions } = usePopupState();
  const suggestions = useMemo(() => parseSuggestions(raw), [raw]);

  // Notices are informational; close them on their own.
  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => void api.hidePopup(), 3500);
    return () => clearTimeout(timer);
  }, [notice]);

  const redactions = session?.redactions ?? [];
  const readOnly = session?.editable === false;
  const variants = toVariants(
    session?.payload.tones ?? [],
    suggestions,
    phase === "done",
    redactions,
    session?.payload.selection ?? "",
  );

  const copy = (i: number) => {
    if (variants[i]?.ready) void api.copyText(variants[i].text);
  };
  const options: RequestOptions | null = session && {
    targetLanguage: session.payload.targetLanguage,
    mode: session.payload.mode,
  };
  // Every request carries the language and mode on screen, so Rust never works from a stale choice.
  const send = () => {
    if (options) void api.sendRequest(options);
  };
  const choose = (change: Partial<RequestOptions>) => {
    if (!options) return;
    const next = { ...options, ...change };
    if (next.targetLanguage === options.targetLanguage && next.mode === options.mode) return;
    setOptions(next);
    // While previewing, nothing is sent until the user says so.
    if (phase !== "preview") void api.sendRequest(next);
  };
  const apply = (i: number) => {
    if (!variants[i]?.ready) return;
    if (readOnly) void api.copyText(variants[i].text);
    // A variant that lost a masked value is never pasted straight in; the user copies it and checks.
    else if (variants[i].missing.length === 0) void api.applyText(variants[i].text);
  };

  useWindowKeydown((e) => {
    if (e.key === "Escape") {
      void api.hidePopup();
    } else if (phase === "preview" && e.key === "Enter") {
      send();
    } else if ((e.key === "r" || e.key === "R") && !e.ctrlKey && session && phase !== "preview") {
      send();
    } else {
      const digit = /^Digit([1-9])$/.exec(e.code);
      if (!digit) return;
      const i = Number(digit[1]) - 1;
      if (e.shiftKey) copy(i);
      else apply(i);
    }
  });

  if (notice) return <NoticeView notice={notice} t={t} />;
  if (!session || !options) {
    return (
      <div className="popup empty">
        <Logo size={40} />
        <p>{t("popup.empty")}</p>
      </div>
    );
  }

  const writing = phase === "streaming" || phase === "done";
  return (
    <div className="popup">
      <PopupHeader session={session} t={t} />
      <PopupMeta session={session} readOnly={readOnly} t={t} />
      <OptionsBar options={options} onChange={choose} t={t} />
      <main>
        {phase === "preview" && <PreviewPanel payload={session.payload} onSend={send} t={t} />}
        {phase === "error" && outcome && <ErrorPanel outcome={outcome} onRetry={send} t={t} />}
        {writing && <VariantList variants={variants} readOnly={readOnly} onApply={apply} onCopy={copy} t={t} />}
      </main>
      <footer>
        <KeyHints readOnly={readOnly} preview={phase === "preview"} t={t} />
        <span className="timing">{status(t, phase, outcome, session)}</span>
      </footer>
    </div>
  );
}

/** Progress while writing; timings and the model once done. */
function status(t: Translate, phase: Phase, outcome: RequestEvent | null, session: Session) {
  if (phase === "streaming") return t("popup.writing");
  if (phase === "done") {
    return `${seconds(outcome?.firstTokenMs)} / ${seconds(outcome?.totalMs)} · ${outcome?.model ?? session.model}`;
  }
  return null;
}

function NoticeView({ notice, t }: { notice: Notice; t: Translate }) {
  return (
    <div className="popup notice-view">
      <Icon name="info" size={20} />
      <div className="notice-text">{t(`notice.${notice.kind}`, { app: notice.detail })}</div>
      <button onClick={() => void api.hidePopup()}>{t("popup.close")}</button>
    </div>
  );
}

/** The keys that work in the popup, so nobody has to remember them. */
function KeyHints({ readOnly, preview, t }: { readOnly: boolean; preview: boolean; t: Translate }) {
  // Nothing to pick or regenerate until the data has been sent.
  if (preview) {
    return (
      <div className="keys">
        <span>
          <kbd>Esc</kbd>
          {t("popup.close")}
        </span>
      </div>
    );
  }
  return (
    <div className="keys">
      <span>
        <kbd>1–9</kbd>
        {readOnly ? t("popup.copy") : t("popup.apply")}
      </span>
      {!readOnly && (
        <span>
          <kbd>Shift+1–9</kbd>
          {t("popup.copy")}
        </span>
      )}
      <span>
        <kbd>R</kbd>
        {t("popup.regenerate")}
      </span>
      <span>
        <kbd>Esc</kbd>
        {t("popup.close")}
      </span>
    </div>
  );
}

// Every part of the header that is not a button is a drag handle: Tauri drags only by an element
// that carries the attribute itself.
function PopupHeader({ session, t }: { session: Session; t: Translate }) {
  return (
    <header data-tauri-drag-region>
      <Logo size={20} />
      <strong className="app-name" data-tauri-drag-region>
        {session.app}
      </strong>
      <span className="spacer" data-tauri-drag-region />
      <div className="header-actions">
        <button className="icon" title={t("popup.settingsTitle")} onClick={() => void api.openSettings()}>
          <Icon name="settings" />
        </button>
        <button className="icon" title={t("popup.closeTitle")} onClick={() => void api.hidePopup()}>
          <Icon name="close" />
        </button>
      </div>
    </header>
  );
}

/** Where the selection and its context came from, and what was masked. */
function PopupMeta({ session, readOnly, t }: { session: Session; readOnly: boolean; t: Translate }) {
  return (
    <div className="meta" data-tauri-drag-region>
      <span className="chip">{t(`popup.source.${session.source}`)}</span>
      <span className="chip">{t(`popup.context.${session.contextSource}`)}</span>
      {session.redactions.length > 0 && (
        <span className="chip safe">
          <Icon name="shield" size={12} />
          {t("popup.redacted", { n: session.redactions.length })}
        </span>
      )}
      {readOnly && (
        <span className="chip warn">
          <Icon name="lock" size={12} />
          {t("popup.readOnly")}
        </span>
      )}
    </div>
  );
}
