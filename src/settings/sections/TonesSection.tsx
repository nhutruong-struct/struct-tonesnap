import type { Settings, ToneId } from "../../lib/api";
import { TONE_IDS, toneDescription, toneLabel } from "../../lib/tones";
import { CheckField, Card, Field, Section, type SectionProps } from "../fields";

export function TonesSection({ settings, update, t }: SectionProps) {
  const toggleTone = (id: ToneId) => {
    const selected = settings.tones.includes(id) ? settings.tones.filter((x) => x !== id) : [...settings.tones, id];
    // Keep the canonical order so the popup is predictable.
    update(
      "tones",
      TONE_IDS.filter((x) => selected.includes(x)),
    );
  };

  return (
    <Section title={t("section.tones")} description={t("section.tones.desc")}>
      <Card>
        {TONE_IDS.map((id) => (
          <CheckField
            key={id}
            label={toneLabel(t, id)}
            hint={toneDescription(t, id)}
            checked={settings.tones.includes(id)}
            onChange={() => toggleTone(id)}
          />
        ))}
      </Card>
      <Card>
        <Field label={t("explain.label")}>
          <select
            value={settings.explainLanguage}
            onChange={(e) => update("explainLanguage", e.target.value as Settings["explainLanguage"])}
          >
            <option value="vi">{t("explain.vi")}</option>
            <option value="en">{t("explain.en")}</option>
            <option value="off">{t("explain.off")}</option>
          </select>
        </Field>
      </Card>
    </Section>
  );
}
