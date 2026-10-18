import type { Settings } from "../../lib/api";
import { LANGUAGE_NAMES } from "../../lib/i18n";
import { Card, Field, Section, ToggleField, type SectionProps } from "../fields";

type InterfaceSectionProps = SectionProps & {
  /** Whether ToneSnap starts at sign-in; `null` hides the option when Windows cannot tell. */
  launchAtLogin: boolean | null;
  onLaunchAtLoginChange: (on: boolean) => void;
};

export function InterfaceSection({ settings, update, t, launchAtLogin, onLaunchAtLoginChange }: InterfaceSectionProps) {
  return (
    <Section title={t("section.interface")} description={t("section.interface.desc")}>
      <Card>
        <Field label={t("uiLanguage.label")}>
          <select
            value={settings.uiLanguage}
            onChange={(e) => update("uiLanguage", e.target.value as Settings["uiLanguage"])}
          >
            <option value="auto">{t("uiLanguage.auto")}</option>
            <option value="vi">{LANGUAGE_NAMES.vi}</option>
            <option value="en">{LANGUAGE_NAMES.en}</option>
          </select>
        </Field>
        {launchAtLogin !== null && (
          <ToggleField
            label={t("launchAtLogin.label")}
            hint={t("launchAtLogin.help")}
            checked={launchAtLogin}
            onChange={onLaunchAtLoginChange}
          />
        )}
      </Card>
    </Section>
  );
}
