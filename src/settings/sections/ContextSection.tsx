import { Card, Field, Section, ToggleField, type SectionProps } from "../fields";

export function ContextSection({ settings, update, t }: SectionProps) {
  return (
    <Section title={t("section.context")} description={t("section.context.desc")}>
      <Card>
        <Field label={t("contextChars.label")}>
          <input
            type="number"
            min={0}
            max={3000}
            step={100}
            value={settings.contextChars}
            onChange={(e) => update("contextChars", Number(e.target.value))}
          />
        </Field>
        <ToggleField
          label={t("ocr.label")}
          hint={t("ocr.help")}
          checked={settings.ocrFallback}
          onChange={(checked) => update("ocrFallback", checked)}
        />
      </Card>
    </Section>
  );
}
