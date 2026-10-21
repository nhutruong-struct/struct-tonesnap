import { formatHotkey } from "../lib/hotkey";
import type { Key, Translate } from "../lib/i18n";
import { Icon, Logo, type IconName } from "../lib/icons";

export const SECTION_IDS = [
  "interface",
  "permissions",
  "ai",
  "trigger",
  "tones",
  "context",
  "privacy",
  "updates",
] as const;
export type SectionId = (typeof SECTION_IDS)[number];

const ITEMS: Record<SectionId, { icon: IconName; label: Key }> = {
  interface: { icon: "sliders", label: "section.interface" },
  permissions: { icon: "lock", label: "section.permissions" },
  ai: { icon: "sparkles", label: "section.ai" },
  trigger: { icon: "keyboard", label: "section.trigger" },
  tones: { icon: "type", label: "section.tones" },
  context: { icon: "layers", label: "section.context" },
  privacy: { icon: "shield", label: "section.privacy" },
  updates: { icon: "refresh", label: "section.updates" },
};

/** Sections that hold a problem the user should look at, with how bad it is. */
export type Attention = Partial<Record<SectionId, "warn" | "bad">>;

interface SettingsNavProps {
  active: SectionId;
  onSelect: (id: SectionId) => void;
  attention: Attention;
  hotkey: string;
  /** Whether the OS has permissions to grant, which adds their section. */
  showPermissions: boolean;
  t: Translate;
}

/** The sidebar: the app name, one button per section, and a reminder of the hotkey. */
export function SettingsNav({ active, onSelect, attention, hotkey, showPermissions, t }: SettingsNavProps) {
  return (
    <nav className="settings-nav" aria-label={t("settings.windowTitle")}>
      <div className="brand">
        <Logo size={28} />
        <span className="brand-name">ToneSnap</span>
      </div>
      {SECTION_IDS.filter((id) => id !== "permissions" || showPermissions).map((id) => {
        const { icon, label } = ITEMS[id];
        const level = active === id ? undefined : attention[id];
        return (
          <button
            key={id}
            className="nav-item"
            title={t(label)}
            aria-current={active === id ? "page" : undefined}
            onClick={() => onSelect(id)}
          >
            <Icon name={icon} size={18} />
            <span className="nav-label">{t(label)}</span>
            {level && <span className={`nav-dot ${level}`} role="img" aria-label={t("settings.needsAttention")} />}
          </button>
        );
      })}
      <p className="nav-tip">
        {t("settings.leadStart")} <kbd>{formatHotkey(hotkey)}</kbd> {t("settings.leadEnd")}
      </p>
    </nav>
  );
}
