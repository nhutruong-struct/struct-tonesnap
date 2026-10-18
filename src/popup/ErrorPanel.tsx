import { api, type RequestEvent } from "../lib/api";
import { hasKey, type Translate } from "../lib/i18n";
import { Icon } from "../lib/icons";

/** Errors that point at something the user can fix in Settings. */
const SETTINGS_ERRORS = ["no_api_key", "authentication_error", "permission_error", "not_found_error"];

function describeError(t: Translate, ev: RequestEvent) {
  const kind = ev.errorKind ?? "";
  const titleKey = `error.${kind}`;
  const detailKey = `error.${kind}.detail`;
  return {
    title: hasKey(titleKey) ? t(titleKey) : t("error.generic", { status: ev.errorStatus ?? "" }).trim(),
    detail: hasKey(detailKey) ? t(detailKey) : ev.errorMessage,
    fixInSettings: SETTINGS_ERRORS.includes(kind),
  };
}

export function ErrorPanel({ outcome, t }: { outcome: RequestEvent; t: Translate }) {
  const error = describeError(t, outcome);
  return (
    <section className="error">
      <Icon name="warning" size={20} />
      <div>
        <strong>{error.title}</strong>
        {error.detail && <p>{error.detail}</p>}
        <div className="actions">
          <button onClick={() => void api.sendRequest()}>{t("popup.retry")}</button>
          {error.fixInSettings && <button onClick={() => void api.openSettings()}>{t("popup.openSettings")}</button>}
        </div>
      </div>
    </section>
  );
}
