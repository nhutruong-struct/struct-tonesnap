import type { Mode, TargetLanguage, ToneId } from "../../lib/api";
import { TONE_IDS, toneDescription, toneLabel } from "../../lib/tones";
import { CheckField, Card, Field, Section, type SectionProps } from "../fields";

const LANGUAGES: TargetLanguage[] = ["vi", "en", "ja"];
const MODES: Mode[] = ["translate", "rewrite"];

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
        <Field label={t("target.label")} hint={<small>{t("target.hint")}</small>}>
          <select
            value={settings.targetLanguage}
            onChange={(e) => update("targetLanguage", e.target.value as TargetLanguage)}
          >
            {LANGUAGES.map((id) => (
              <option key={id} value={id}>
                {t(`target.${id}`)}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t("mode.label")} hint={<small>{t(`mode.${settings.mode}.hint`)}</small>}>
          <select value={settings.mode} onChange={(e) => update("mode", e.target.value as Mode)}>
            {MODES.map((id) => (
              <option key={id} value={id}>
                {t(`mode.${id}`)}
              </option>
            ))}
          </select>
        </Field>
      </Card>
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
    </Section>
  );
}
