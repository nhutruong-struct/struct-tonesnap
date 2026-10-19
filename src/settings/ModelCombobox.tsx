import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";

interface ModelComboboxProps {
  value: string;
  options: string[];
  onChange: (value: string) => void;
  /** Names the arrow button, which opens the list of suggestions. */
  toggleLabel: string;
}

/**
 * A text field with a list of suggestions that is drawn by the app, so it follows the design tokens
 * (a native `<datalist>` cannot be styled and hides every suggestion that does not match the text).
 * The user can still type any value; the list always shows every suggestion.
 */
export function ModelCombobox({ value, options, onChange, toggleLabel }: ModelComboboxProps) {
  const listId = useId();
  const root = useRef<HTMLDivElement>(null);
  const list = useRef<HTMLUListElement>(null);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);

  useEffect(() => {
    if (!open) return;
    // A list that opens near the bottom of the page scrolls into view instead of being cut off.
    list.current?.scrollIntoView?.({ block: "nearest" });
  }, [open]);

  const show = () => {
    setActive(Math.max(0, options.indexOf(value)));
    setOpen(true);
  };

  const choose = (option: string) => {
    onChange(option);
    setOpen(false);
  };

  const move = (step: number) => {
    if (!open) return show();
    setActive((index) => (index + step + options.length) % options.length);
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        move(1);
        break;
      case "ArrowUp":
        e.preventDefault();
        move(-1);
        break;
      case "Enter": {
        const option = options[active];
        if (open && option !== undefined) {
          e.preventDefault();
          choose(option);
        }
        break;
      }
      case "Escape":
        if (open) {
          e.preventDefault();
          setOpen(false);
        }
        break;
      default:
        break;
    }
  };

  return (
    <div
      className="combo"
      ref={root}
      onBlur={(e) => {
        if (!root.current?.contains(e.relatedTarget)) setOpen(false);
      }}
    >
      <input
        role="combobox"
        aria-expanded={open}
        aria-controls={listId}
        aria-autocomplete="none"
        aria-activedescendant={open && active >= 0 ? `${listId}-${active}` : undefined}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={onKeyDown}
        spellCheck={false}
        autoComplete="off"
      />
      <button
        type="button"
        className="combo-toggle"
        tabIndex={-1}
        aria-label={toggleLabel}
        aria-expanded={open}
        aria-controls={listId}
        // Keeps the focus in the text field, so the list does not close before the click lands.
        onMouseDown={(e) => e.preventDefault()}
        onClick={() => (open ? setOpen(false) : show())}
      />
      {open && (
        <ul className="combo-list" id={listId} role="listbox" ref={list}>
          {options.map((option, index) => (
            <li
              key={option}
              id={`${listId}-${index}`}
              role="option"
              aria-selected={option === value}
              className={index === active ? "combo-option active" : "combo-option"}
              onMouseDown={(e) => e.preventDefault()}
              onMouseEnter={() => setActive(index)}
              onClick={() => choose(option)}
            >
              {option}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
