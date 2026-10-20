import { useEffect, useState } from "react";
import { api, type ProviderId, type Settings } from "../../lib/api";
import type { Key } from "../../lib/i18n";
import { Card, Field, Note, RadioField, Section, type SectionProps, type Status } from "../fields";
import { ModelCombobox } from "../ModelCombobox";

const PROVIDERS: ProviderId[] = ["gemini", "groq", "openrouter", "claude"];

const KEY_PLACEHOLDERS: Record<ProviderId, string> = {
  claude: "sk-ant-…",
  gemini: "AIza…",
  groq: "gsk_…",
  openrouter: "sk-or-…",
};

/** Where each provider hands out API keys; the hint under the field names the same page. */
const KEY_URLS: Record<ProviderId, string> = {
  claude: "https://console.anthropic.com/settings/keys",
  gemini: "https://aistudio.google.com/apikey",
  groq: "https://console.groq.com/keys",
  openrouter: "https://openrouter.ai/keys",
};

const CLAUDE_MODELS: { id: string; label: string; hint: Key }[] = [
  { id: "claude-opus-5-5", label: "Claude Opus 5.5", hint: "model.opus" },
  { id: "claude-sonnet-5-5", label: "Claude Sonnet 5.5", hint: "model.sonnet" },
  { id: "claude-haiku-4-5", label: "Claude Haiku 4.5", hint: "model.haiku" },
];

/** Models offered as suggestions; free tiers change often, so any model name can be typed. */
const SUGGESTED_MODELS: Record<Exclude<ProviderId, "claude">, { field: keyof Settings; models: string[] }> = {
  gemini: {
    field: "geminiModel",
    models: [
      "gemini-3.8-flash",
      "gemini-3.6-flash",
      "gemini-3.5-flash-lite",
      "gemini-2.5-flash",
      "gemini-2.5-flash-lite",
    ],
  },
  groq: {
    field: "groqModel",
    models: ["openai/gpt-oss-20b", "openai/gpt-oss-120b", "qwen/qwen3.8-27b"],
  },
  openrouter: {
    field: "openrouterModel",
    models: ["openai/gpt-oss-20b:free", "openai/gpt-oss-120b:free", "meta-llama/llama-3.3-70b-instruct:free"],
  },
};

type AiSectionProps = SectionProps & { onStatus: (status: Status) => void };

export function AiSection({ settings, update, t, onStatus }: AiSectionProps) {
  const provider = settings.provider;
  return (
    <Section title={t("section.ai")} description={t("section.ai.desc")}>
      <Card>
        {PROVIDERS.map((p) => (
          <RadioField
            key={p}
            name="provider"
            label={t(`provider.${p}`)}
            hint={t(`provider.${p}.hint`)}
            checked={provider === p}
            onSelect={() => update("provider", p)}
          />
        ))}
      </Card>
      {/* The key belongs to the provider, so switching provider starts from that provider's state. */}
      <ApiKeyCard key={provider} provider={provider} t={t} onStatus={onStatus} />
      {provider === "claude" ? (
        <ClaudeModelCard settings={settings} update={update} t={t} />
      ) : (
        <ModelCard provider={provider} settings={settings} update={update} t={t} />
      )}
    </Section>
  );
}

function ApiKeyCard({
  provider,
  t,
  onStatus,
}: {
  provider: ProviderId;
  t: SectionProps["t"];
  onStatus: AiSectionProps["onStatus"];
}) {
  const [hasKey, setHasKey] = useState(false);
  const [keyDraft, setKeyDraft] = useState("");
  useEffect(() => {
    void api.hasApiKey(provider).then(setHasKey);
  }, [provider]);

  const saveKey = async (key: string) => {
    try {
      await api.setApiKey(provider, key);
      setHasKey(await api.hasApiKey(provider));
      setKeyDraft("");
      onStatus({ ok: true, text: key ? t("apiKey.saved") : t("apiKey.removed") });
    } catch (e) {
      onStatus({ ok: false, text: String(e) });
    }
  };

  return (
    <Card>
      <Field
        label={t("apiKey.label")}
        hint={
          <>
            <small>{t(`apiKey.get.${provider}`)}</small>
            <small>{t("apiKey.help")}</small>
          </>
        }
        stack
      >
        <div className="row">
          <input
            type="password"
            placeholder={hasKey ? t("apiKey.placeholderSaved") : KEY_PLACEHOLDERS[provider]}
            value={keyDraft}
            onChange={(e) => setKeyDraft(e.target.value)}
            autoComplete="off"
          />
          <button onClick={() => void api.openUrl(KEY_URLS[provider])}>{t("apiKey.getKey")}</button>
          <button className="primary" disabled={!keyDraft.trim()} onClick={() => void saveKey(keyDraft)}>
            {t("apiKey.save")}
          </button>
          {hasKey && (
            <button className="ghost danger" onClick={() => void saveKey("")}>
              {t("apiKey.remove")}
            </button>
          )}
        </div>
      </Field>
      {provider !== "claude" && <Note>{t("provider.freeNotice")}</Note>}
    </Card>
  );
}

function ModelCard({ provider, settings, update, t }: SectionProps & { provider: Exclude<ProviderId, "claude"> }) {
  const { field, models } = SUGGESTED_MODELS[provider];
  const value = settings[field] as string;
  return (
    <Card>
      <Field label={t("model.label")} hint={<small>{t("model.custom")}</small>} stack>
        <ModelCombobox
          value={value}
          options={models}
          onChange={(model) => update(field, model as Settings[typeof field])}
          toggleLabel={t("model.suggestions")}
        />
      </Field>
    </Card>
  );
}

function ClaudeModelCard({ settings, update, t }: SectionProps) {
  // Haiku 4.5 does not accept the effort setting.
  const isHaiku = settings.model.startsWith("claude-haiku");
  return (
    <Card>
      <Field label={t("model.label")}>
        <select value={settings.model} onChange={(e) => update("model", e.target.value)}>
          {CLAUDE_MODELS.map((m) => (
            <option key={m.id} value={m.id}>
              {m.label} – {t(m.hint)}
            </option>
          ))}
        </select>
      </Field>
      <Field label={t("effort.label")} hint={isHaiku && <small>{t("effort.haikuNote")}</small>}>
        <select value={settings.effort} disabled={isHaiku} onChange={(e) => update("effort", e.target.value)}>
          <option value="low">{t("effort.low")}</option>
          <option value="medium">{t("effort.medium")}</option>
          <option value="high">{t("effort.high")}</option>
        </select>
      </Field>
    </Card>
  );
}
