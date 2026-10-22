import type { KeyboardEvent } from "react";
import type { CommandError, TriggerMode } from "../../lib/api";
import { formatHotkey, hotkeyConflict, normalizeHotkey } from "../../lib/hotkey";
import { Card, Field, RadioField, Section, type SectionProps } from "../fields";

const TRIGGERS: TriggerMode[] = ["hotkey", "icon", "auto", "off"];

const MODIFIER_KEYS = ["Control", "Alt", "Shift", "Meta"];

/** Converts a keydown into the normalized "ctrl+alt+e" form the Rust side parses. */
function comboFromEvent(e: KeyboardEvent): string | null {
  if (MODIFIER_KEYS.includes(e.key)) return null;
  const parts = [];
  if (e.ctrlKey) parts.push("ctrl");
  if (e.altKey) parts.push("alt");
  if (e.shiftKey) parts.push("shift");
  if (e.metaKey) parts.push("super");
  if (parts.length === 0) return null;
  parts.push(e.code);
  return normalizeHotkey(parts.join("+"));
}

export function TriggerSection({
  settings,
  update,
  t,
  hotkeyError,
}: SectionProps & { hotkeyError: CommandError | null }) {
  const conflict = hotkeyConflict(settings.hotkey);
  return (
    <Section title={t("section.trigger")} description={t("section.trigger.desc")}>
      <Card>
        {TRIGGERS.map((id) => (
          <RadioField
            key={id}
            name="trigger"
            label={t(`trigger.${id}`)}
            hint={t(`trigger.${id}.hint`)}
            checked={settings.triggerMode === id}
            onSelect={() => update("triggerMode", id)}
          />
        ))}
      </Card>
      <Card>
        <Field
          label={t("hotkey.label")}
          hint={
            <>
              {conflict && <small className="status bad">{t("hotkey.conflict", { apps: conflict })}</small>}
              <small>{t("hotkey.help")}</small>
            </>
          }
        >
          <input
            readOnly
            className={hotkeyError || conflict ? "hotkey invalid" : "hotkey"}
            value={formatHotkey(settings.hotkey)}
            onKeyDown={(e) => {
              const combo = comboFromEvent(e);
              if (combo) {
                e.preventDefault();
                update("hotkey", combo);
              }
            }}
          />
        </Field>
        <Field label={t("autoDelay.label")}>
          <input
            type="number"
            min={100}
            max={3000}
            step={50}
            disabled={settings.triggerMode !== "auto"}
            value={settings.autoDelayMs}
            onChange={(e) => update("autoDelayMs", Number(e.target.value))}
          />
        </Field>
        <Field label={t("autoMinChars.label")} hint={<small>{t("autoMinChars.hint")}</small>}>
          <input
            type="number"
            min={1}
            max={200}
            disabled={settings.triggerMode !== "auto"}
            value={settings.autoMinChars}
            onChange={(e) => update("autoMinChars", Number(e.target.value))}
          />
        </Field>
      </Card>
    </Section>
  );
}
