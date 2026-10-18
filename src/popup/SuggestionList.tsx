import type { Suggestions } from "../lib/api";
import type { Translate } from "../lib/i18n";
import { restore } from "../lib/redaction";
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
              <button className="primary small" disabled={!v.ready} onClick={() => onApply(i)}>
                {t("popup.apply")}
              </button>
            )}
            <button className="small" disabled={!v.ready} onClick={() => onCopy(i)}>
              {t("popup.copy")}
            </button>
          </div>
          {v.text ? <p className="variant-text">{v.text}</p> : <div className="skeleton" />}
          {v.note && <p className="note">{v.note}</p>}
        </li>
      ))}
    </ol>
  );
}

interface IssueListProps {
  issues: Suggestions["issues"];
  redactions: [string, string][];
  t: Translate;
}

/** Mistakes in the original text, with fixes and explanations. */
export function IssueList({ issues, redactions, t }: IssueListProps) {
  if (!issues?.length) return null;
  return (
    <section className="issues">
      <h3>{t("popup.issues")}</h3>
      <ul>
        {issues.map((issue, i) => (
          <li key={i}>
            <span className="from">{restore(issue.original ?? "", redactions)}</span>
            <span className="arrow">→</span>
            <span className="to">{restore(issue.fix ?? "", redactions)}</span>
            {issue.explanation && <p>{restore(issue.explanation, redactions)}</p>}
          </li>
        ))}
      </ul>
    </section>
  );
}
