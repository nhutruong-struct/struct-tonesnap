import { Card, Field, Section, ToggleField, type SectionProps } from "../fields";

type PrivacySectionProps = SectionProps & {
  /** Custom sensitive terms, one per line. */
  terms: string;
  onTermsChange: (text: string) => void;
  /** Blocked process names, one per line. */
  blocked: string;
  onBlockedChange: (text: string) => void;
};

export function PrivacySection(props: PrivacySectionProps) {
  const { settings, update, t } = props;
  return (
    <Section title={t("section.privacy")} description={t("section.privacy.desc")}>
      <Card>
        <ToggleField
          label={t("redact.label")}
          hint={t("redact.help")}
          checked={settings.redact}
          onChange={(checked) => update("redact", checked)}
        />
        <ToggleField
          label={t("preview.label")}
          hint={t("preview.help")}
          checked={settings.previewBeforeSend}
          onChange={(checked) => update("previewBeforeSend", checked)}
        />
      </Card>
      <Card>
        <Field label={t("terms.label")} stack>
          <textarea
            rows={4}
            placeholder={t("terms.placeholder")}
            value={props.terms}
            onChange={(e) => props.onTermsChange(e.target.value)}
          />
        </Field>
        <Field label={t("blocked.label")} stack>
          <textarea rows={4} value={props.blocked} onChange={(e) => props.onBlockedChange(e.target.value)} />
        </Field>
      </Card>
    </Section>
  );
}
