import { useEffect, useState } from "react";
import { api, events, unsubscribeAll, type UpdateInfo, type UpdateMode, type UpdateStatus } from "../../lib/api";
import type { Translate } from "../../lib/i18n";
import { Card, Note, RadioField, Section, type SectionProps } from "../fields";

const MODES: UpdateMode[] = ["auto", "manual"];

export function describeUpdateStatus(t: Translate, status: UpdateStatus, mode: UpdateMode): string {
  switch (status.kind) {
    case "idle":
      return t("updates.status.idle");
    case "checking":
      return t("updates.status.checking");
    case "up_to_date":
      return t("updates.status.up_to_date");
    case "available":
      return t("updates.status.available", { version: status.version });
    case "downloading":
      return t("updates.status.downloading", { version: status.version });
    case "ready":
      return t(`updates.status.ready.${mode}`, { version: status.version });
    case "installing":
      return t("updates.status.installing", { version: status.version });
    case "failed":
      return t(`updates.status.failed.${status.step}`, { detail: status.detail });
  }
}

/** The running version and where updating stands, kept current by the `update-status` events. */
function useUpdateInfo() {
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  useEffect(() => {
    // An event is newer than the answer to a request made before it.
    let heardEvent = false;
    void api.updateInfo().then((answer) => {
      if (!heardEvent) setInfo(answer);
    });
    return unsubscribeAll([
      events.updateStatus((latest) => {
        heardEvent = true;
        setInfo(latest);
      }),
    ]);
  }, []);
  return info;
}

/**
 * The update mode, saved with the other settings, and the update status with its buttons, which
 * act at once like the API key buttons.
 */
export function UpdatesSection({ settings, update, t }: SectionProps) {
  const info = useUpdateInfo();
  const kind = info?.status.kind;
  const working = kind === "checking" || kind === "downloading" || kind === "installing";
  const installable = kind === "available" || kind === "ready";

  return (
    <Section title={t("section.updates")} description={t("section.updates.desc")}>
      <Card>
        {MODES.map((mode) => (
          <RadioField
            key={mode}
            name="updateMode"
            label={t(`updateMode.${mode}`)}
            hint={t(`updateMode.${mode}.hint`)}
            checked={settings.updateMode === mode}
            onSelect={() => update("updateMode", mode)}
          />
        ))}
        {info?.msiInstall && <Note>{t("updates.msiNote")}</Note>}
        {info && !info.supported && <Note>{t("updates.unsupported")}</Note>}
      </Card>
      {info?.supported && (
        <Card>
          <div className="item field stack">
            <span className="item-text">
              <span className="item-label">{t("updates.current", { version: info.currentVersion })}</span>
              <small role="status">
                {describeUpdateStatus(t, info.status, info.msiInstall ? "manual" : settings.updateMode)}
              </small>
            </span>
            <div className="row">
              <button disabled={working} onClick={() => void api.checkForUpdates()}>
                {t("updates.check")}
              </button>
              {installable && (
                <button className="primary" onClick={() => void api.installUpdate()}>
                  {t("updates.install")}
                </button>
              )}
            </div>
          </div>
        </Card>
      )}
    </Section>
  );
}
