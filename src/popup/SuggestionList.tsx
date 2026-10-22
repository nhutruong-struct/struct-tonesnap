import type { Translate } from "../lib/i18n";
import { toneLabel } from "../lib/tones";
import type { Variant } from "./suggestions";

interface VariantListProps {
  variants: Variant[];
  /** The field cannot be edited, so only Copy is offered. */
  readOnly: boolean;
  onApply: (index: number) => void;
  onCopy: (index: number) => void;
  t: Translate;
}

export function VariantList({ variants, readOnly, onApply, onCopy, t }: VariantListProps) {
  return (
    <ol className="variants">
      {variants.map((v, i) => (
        <li key={v.tone} className={v.ready ? "variant" : "variant pending"}>
          <div className="variant-head">
            <span className="badge">{i + 1}</span>
            <span className="tone">{toneLabel(t, v.tone)}</span>
            <span className="spacer" />
            {!readOnly && (
              <button className="primary small" disabled={!v.ready || v.missing.length > 0} onClick={() => onApply(i)}>
                {t("popup.apply")}
              </button>
            )}
            <button className="small" disabled={!v.ready} onClick={() => onCopy(i)}>
              {t("popup.copy")}
            </button>
          </div>
          {v.text ? <p className="variant-text">{v.text}</p> : <div className="skeleton" />}
          {v.missing.length > 0 && (
            <p className="variant-warning" role="alert">
              {t("popup.missingRedacted", { values: v.missing.join(", ") })}
            </p>
          )}
        </li>
      ))}
    </ol>
  );
}
