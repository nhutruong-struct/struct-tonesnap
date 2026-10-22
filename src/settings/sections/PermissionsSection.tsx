import { useEffect, useState } from "react";
import { api, type PermissionKind, type Permissions } from "../../lib/api";
import type { Key, Translate } from "../../lib/i18n";
import { Card, Section } from "../fields";

/** How often the status is read again while Settings is open: the user grants it elsewhere. */
const POLL_MS = 1500;

/** The permissions, kept current while the section is mounted and when the window regains focus. */
export function usePermissions(): Permissions | null {
  const [permissions, setPermissions] = useState<Permissions | null>(null);
  useEffect(() => {
    let alive = true;
    const read = () =>
      void api.permissions().then((latest) => {
        if (alive) setPermissions(latest);
      });
    read();
    const timer = window.setInterval(read, POLL_MS);
    window.addEventListener("focus", read);
    return () => {
      alive = false;
      window.clearInterval(timer);
      window.removeEventListener("focus", read);
    };
  }, []);
  return permissions;
}

const ROWS: { kind: PermissionKind; field: keyof Omit<Permissions, "required"> }[] = [
  { kind: "accessibility", field: "accessibility" },
  { kind: "screen_recording", field: "screenRecording" },
];

export function PermissionsSection({ permissions, t }: { permissions: Permissions | null; t: Translate }) {
  return (
    <Section title={t("section.permissions")} description={t("section.permissions.desc")}>
      <Card>
        {ROWS.map(({ kind, field }) => (
          <PermissionRow key={kind} kind={kind} allowed={permissions?.[field] ?? false} t={t} />
        ))}
      </Card>
    </Section>
  );
}

function PermissionRow({ kind, allowed, t }: { kind: PermissionKind; allowed: boolean; t: Translate }) {
  return (
    <div className="item field">
      <span className="item-text">
        <span className="item-label">{t(`permission.${kind}.label` as Key)}</span>
        <small>{t(`permission.${kind}.hint` as Key)}</small>
      </span>
      <div className="row">
        <span className={allowed ? "chip safe" : "chip warn"}>
          {t(allowed ? "permission.allowed" : "permission.denied")}
        </span>
        {!allowed && (
          <>
            <button onClick={() => void api.requestPermission(kind)}>{t("permission.allow")}</button>
            <button onClick={() => void api.openPermissionSettings(kind)}>{t("permission.openSettings")}</button>
          </>
        )}
      </div>
    </div>
  );
}
