import { useEffect, useRef, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { hotkeyConflict } from "../lib/hotkey";
import { Icon } from "../lib/icons";
import type { Settings } from "../lib/api";
import { AiSection } from "./sections/AiSection";
import { ContextSection } from "./sections/ContextSection";
import { InterfaceSection } from "./sections/InterfaceSection";
import { PermissionsSection, usePermissions } from "./sections/PermissionsSection";
import { PrivacySection } from "./sections/PrivacySection";
import { TonesSection } from "./sections/TonesSection";
import { TriggerSection } from "./sections/TriggerSection";
import { UpdatesSection } from "./sections/UpdatesSection";
import { SettingsNav, type Attention, type SectionId } from "./SettingsNav";
import { describeCommandError, describeSettingsIssue, useSettingsForm, type SettingsForm } from "./useSettingsForm";

export default function SettingsPage() {
  const form = useSettingsForm();
  const { settings, lang, t } = form;

  const title = t("settings.windowTitle");
  useEffect(() => {
    document.documentElement.lang = lang;
    void getCurrentWindow().setTitle(title);
  }, [lang, title]);

  if (!settings) return <div className="settings loading">{t("settings.loading")}</div>;

  // Remounted on every load, so drafts such as an unsaved API key and the open section do not linger.
  return <SettingsBody key={form.loadCount} form={form} settings={settings} />;
}

function SettingsBody({ form, settings }: { form: SettingsForm; settings: Settings }) {
  const { update, t } = form;
  const [chosen, setChosen] = useState<SectionId | null>(null);
  const [landing, setLanding] = useState<SectionId | null>(null);
  const scroll = useRef<HTMLDivElement>(null);
  const permissions = usePermissions();

  const open = (id: SectionId) => {
    setChosen(id);
    // The next section starts at its top, whatever the last one was scrolled to.
    if (scroll.current) scroll.current.scrollTop = 0;
  };

  // Without the Accessibility permission nothing works, so the first look goes to its section.
  // Decided once, when the permissions first arrive, so granting it does not move the page.
  const missingPermission = permissions?.required === true && !permissions.accessibility;
  if (landing === null && permissions) setLanding(missingPermission ? "permissions" : "interface");
  const active = chosen ?? landing ?? "interface";

  // A hotkey problem is explained in the Trigger section; the dot leads the user there.
  const attention: Attention = form.hotkeyError
    ? { trigger: "bad" }
    : hotkeyConflict(settings.hotkey)
      ? { trigger: "warn" }
      : {};
  if (missingPermission) attention.permissions = "bad";

  const section = { settings, update, t };
  // Every section stays mounted, so what the user typed in one is still there when they come back.
  const panels: [SectionId, ReactNode][] = [
    [
      "interface",
      <InterfaceSection
        key="interface"
        {...section}
        launchAtLogin={form.launchAtLogin}
        onLaunchAtLoginChange={form.changeLaunchAtLogin}
      />,
    ],
    ["permissions", <PermissionsSection key="permissions" permissions={permissions} t={t} />],
    ["ai", <AiSection key="ai" {...section} onStatus={form.setStatus} />],
    ["trigger", <TriggerSection key="trigger" {...section} hotkeyError={form.hotkeyError} />],
    ["tones", <TonesSection key="tones" {...section} />],
    ["context", <ContextSection key="context" {...section} />],
    [
      "privacy",
      <PrivacySection
        key="privacy"
        {...section}
        terms={form.terms}
        onTermsChange={form.setTerms}
        blocked={form.blocked}
        onBlockedChange={form.setBlocked}
      />,
    ],
    ["updates", <UpdatesSection key="updates" {...section} />],
  ];

  return (
    <div className="settings">
      <SettingsNav
        active={active}
        onSelect={open}
        attention={attention}
        hotkey={settings.hotkey}
        showPermissions={permissions?.required === true}
        t={t}
      />
      <div className="settings-main">
        {(form.issue || form.hotkeyError) && (
          <div className="banners">
            {form.issue && (
              <div className="banner warn">
                <Icon name="warning" />
                <span>{describeSettingsIssue(t, form.issue)}</span>
              </div>
            )}
            {form.hotkeyError && (
              <div className="banner bad">
                <Icon name="warning" />
                <span>{describeCommandError(t, form.hotkeyError)}</span>
              </div>
            )}
          </div>
        )}
        <div className="settings-scroll" ref={scroll}>
          {panels.map(([id, panel]) => (
            <div key={id} hidden={active !== id}>
              {panel}
            </div>
          ))}
        </div>
        <div className="save-bar">
          {form.status && <span className={form.status.ok ? "status ok" : "status bad"}>{form.status.text}</span>}
          <button className="primary" onClick={() => void form.save()}>
            {t("settings.save")}
          </button>
        </div>
      </div>
    </div>
  );
}
