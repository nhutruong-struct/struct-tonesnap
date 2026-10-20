import type { Mode, RequestOptions, TargetLanguage } from "../lib/api";
import type { Translate } from "../lib/i18n";

const LANGUAGES: TargetLanguage[] = ["vi", "en", "ja"];
const MODES: Mode[] = ["translate", "rewrite"];

interface OptionsBarProps {
  options: RequestOptions;
  onChange: (change: Partial<RequestOptions>) => void;
  t: Translate;
}

/** Quick choice of the language the versions are written in, and of translating or rewriting. */
export function OptionsBar({ options, onChange, t }: OptionsBarProps) {
  return (
    <div className="options">
      <Segmented
        label={t("popup.options.language")}
        value={options.targetLanguage}
        choices={LANGUAGES.map((id) => ({ id, text: t(`target.${id}`) }))}
        onSelect={(targetLanguage) => onChange({ targetLanguage })}
      />
      <Segmented
        label={t("popup.options.mode")}
        value={options.mode}
        choices={MODES.map((id) => ({ id, text: t(`mode.${id}`) }))}
        onSelect={(mode) => onChange({ mode })}
      />
    </div>
  );
}

interface SegmentedProps<T extends string> {
  label: string;
  value: T;
  choices: { id: T; text: string }[];
  onSelect: (id: T) => void;
}

function Segmented<T extends string>({ label, value, choices, onSelect }: SegmentedProps<T>) {
  return (
    <div className="segmented" role="radiogroup" aria-label={label}>
      {choices.map(({ id, text }) => (
        <button key={id} role="radio" aria-checked={id === value} onClick={() => onSelect(id)}>
          {text}
        </button>
      ))}
    </div>
  );
}
