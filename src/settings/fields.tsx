import type { ReactNode } from "react";
import type { Settings } from "../lib/api";
import type { Translate } from "../lib/i18n";

/** What every Settings section receives. */
export interface SectionProps {
  settings: Settings;
  update: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
  t: Translate;
}

/** A message in the save bar. */
export interface Status {
  ok: boolean;
  text: string;
}

/** A section's title and one line saying what it is for, above its cards. */
export function Section(props: { title: string; description: string; children: ReactNode }) {
  return (
    <section className="section">
      <div className="section-head">
        <h2>{props.title}</h2>
        <p>{props.description}</p>
      </div>
      {props.children}
    </section>
  );
}

/** A group of rows, divided by thin lines. */
export function Card({ children }: { children: ReactNode }) {
  return <div className="card">{children}</div>;
}

/** A line of explanation or a message between the rows of a card. */
export function Note({ bad, children }: { bad?: boolean; children: ReactNode }) {
  return <p className={bad ? "item-note status bad" : "item-note"}>{children}</p>;
}

/** The label of a row with its hint under it. */
function ItemText({ label, children }: { label: string; children?: ReactNode }) {
  return (
    <span className="item-text">
      <span className="item-label">{label}</span>
      {children}
    </span>
  );
}

/**
 * A row with a label and its hint on the left and one control on the right; `stack` puts the
 * control under the label at full width, for text areas and long values.
 */
export function Field(props: { label: string; hint?: ReactNode; stack?: boolean; children: ReactNode }) {
  return (
    <label className={props.stack ? "item field stack" : "item field"}>
      <ItemText label={props.label}>{props.hint}</ItemText>
      {props.children}
    </label>
  );
}

interface ChoiceProps {
  label: string;
  hint: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

/** An on/off setting: the label and hint on the left, a switch on the right. */
export function ToggleField(props: ChoiceProps) {
  return (
    <label className="item toggle">
      <ItemText label={props.label}>
        <small>{props.hint}</small>
      </ItemText>
      <input type="checkbox" role="switch" checked={props.checked} onChange={(e) => props.onChange(e.target.checked)} />
    </label>
  );
}

/** One of several things that can be ticked together: a checkbox, then the label and hint. */
export function CheckField(props: ChoiceProps) {
  return (
    <label className="item check">
      <input type="checkbox" checked={props.checked} onChange={(e) => props.onChange(e.target.checked)} />
      <ItemText label={props.label}>
        <small>{props.hint}</small>
      </ItemText>
    </label>
  );
}

/** One option of a radio group: a radio button, then the label, an optional badge and the hint. */
export function RadioField(props: {
  name: string;
  label: string;
  hint: string;
  checked: boolean;
  onSelect: () => void;
  disabled?: boolean;
  badge?: ReactNode;
}) {
  return (
    <label className={props.disabled ? "item radio disabled" : "item radio"}>
      <input
        type="radio"
        name={props.name}
        disabled={props.disabled}
        checked={props.checked}
        onChange={props.onSelect}
      />
      <span className="item-text">
        <span className="item-label">
          {props.label}
          {props.badge}
        </span>
        <small>{props.hint}</small>
      </span>
    </label>
  );
}
