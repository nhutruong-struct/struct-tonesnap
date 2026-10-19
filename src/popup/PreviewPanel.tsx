import { api, type Payload } from "../lib/api";
import type { Key, Translate } from "../lib/i18n";

/** Shows exactly what will be sent to the model, before anything is sent. */
export function PreviewPanel({ payload, onSend, t }: { payload: Payload; onSend: () => void; t: Translate }) {
  const rows: { label: Key; value: string; always?: boolean }[] = [
    { label: "preview.app", value: payload.app, always: true },
    { label: "preview.before", value: payload.contextBefore },
    { label: "preview.selection", value: payload.selection, always: true },
    { label: "preview.target", value: t(`target.${payload.targetLanguage}`), always: true },
    { label: "preview.mode", value: t(`mode.${payload.mode}`), always: true },
    { label: "preview.after", value: payload.contextAfter },
    { label: "preview.screen", value: payload.screenText },
  ];
  return (
    <section className="preview">
      <h3>{t("preview.title")}</h3>
      <dl>
        {rows
          .filter((row) => row.always || row.value)
          .map((row) => (
            <Row key={row.label} label={t(row.label)} value={row.value} />
          ))}
      </dl>
      <div className="actions">
        <button className="primary" onClick={onSend}>
          {t("preview.send")}
        </button>
        <button className="ghost" onClick={() => void api.hidePopup()}>
          {t("preview.cancel")}
        </button>
      </div>
    </section>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </>
  );
}
