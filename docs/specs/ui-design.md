# Interface design

> **Status:** implemented (2026-10-08 redesign).
> **Source:** `src/styles/` (`tokens.css`, `base.css`, `controls.css`, `popup.css`, `settings.css`), `src/lib/icons.tsx`, `src/settings/fields.tsx`, `src/settings/SettingsNav.tsx`.

How both windows look and why. The rules here keep new screens consistent: add a token before adding a one-off value, and reuse a shared component before writing new markup.

## Tokens

- **UI-01** Every colour, space, radius, font size and duration comes from `tokens.css`. Other stylesheets use `var(--…)` and no literal values, except inside the SVG data URI of the select arrow.
- **UI-02** Spacing is a 4 px grid: `--s-1` 4, `--s-2` 8, `--s-3` 12, `--s-4` 16, `--s-5` 20, `--s-6` 24, `--s-8` 32. Padding and gaps use these steps only. Radii: controls 6, fields and notes 8, cards 12, pills fully round.
- **UI-03** The accent follows the app icon (blue to violet). Status colours (`--safe`, `--warn`, `--bad`, each with a `-bg`) carry meaning only: masked data, read-only or coming soon, errors.
- **UI-04** Dark mode follows Windows (`prefers-color-scheme`). Both palettes keep text at WCAG AA (4.5:1) on the surface it is drawn on, and the outline of a checkbox, radio or switch at 3:1.
- **UI-05** The font is the system's (Segoe UI Variable, then Segoe UI). Sizes: 12 caption, 12.5 small, 14 body, 16 title, 20 headline. No web fonts, so nothing is downloaded.

## Controls

- **UI-10** Buttons: default (bordered), `primary` (accent fill; one per view or group), `ghost` (no border), `danger` (red text), `icon` (square, for the header), and `small`. All have hover, pressed and disabled states.
- **UI-11** Checkbox, radio and switch are drawn in CSS on top of the native inputs (`appearance: none`), so keyboard and screen-reader behavior is the browser's. The state shows in the control itself: fill and a mark that scales in.
- **UI-12** The focus ring is drawn only for keyboard focus (`:focus-visible`), 2 px outside the control. A mouse click never leaves a ring. Text fields and selects show a 2 px accent line on focus (their border plus 1 px); a transparent outline keeps it visible in Windows high-contrast mode.
- **UI-13** On/off settings are **switches** (`role="switch"`, a checkbox underneath). A choice among several, or one of many that can be ticked together (tones), is a **radio** or a **checkbox**.
- **UI-15** A text field with suggestions (the model name) is a **combobox** drawn by the app (`ModelCombobox`), not a `<datalist>`: the native list cannot take the tokens and shows only the suggestions that match the text. The arrow at the right edge opens the list, which always shows every suggestion, marks the current value and floats over the card (a card holding one is not clipped). Arrow keys move, Enter picks, Esc closes; the text can still be any value. Roles: `combobox` on the field, `listbox` and `option` on the list.
- **UI-14** Motion is 120 ms. With reduced motion requested, transitions and animations are effectively off.

## Popup

- **UI-20** Header: app icon, the app the text came from, then Settings and Close. Every part of it that is not a button is a drag handle.
- **UI-21** Under the header, chips say where the selection and context came from, how many values were masked and, for a read-only field, that only Copy works (POP-20).
- **UI-22** *Removed.* The popup does not quote the selection (POP-22).
- **UI-23** Each tone is a card: a number badge (also the key that applies it), the tone name, *Apply* (primary) and *Copy*, then the text. A card is highlighted when the pointer is over it. A card that is still being written has a dashed border and no fill, and its buttons are off (POP-32).
- **UI-24** *Removed.* Mistakes in the original are no longer shown (AI-06..08).
- **UI-25** The footer shows the keys as key caps with their effect; while previewing it shows only *Esc*. On the right: progress, or timings and the model (POP-23).
- **UI-26** Errors are a red-tinted card with an icon, a title, a detail and the buttons; notices are one line with an info icon and *Close*.

## Settings

- **UI-30** The window has a sidebar and one open section. The sidebar holds the app icon and name, one button per section (icon and name), and a reminder of the hotkey. It is 248 px wide, enough for the longest section name in both languages on one line. The open section is marked by a tinted row and a bar on its left.
- **UI-31** Every section stays mounted while another is open, so text typed in it (such as an API key not yet saved) is still there when the user returns. Reopening the window starts at the first section ([settings.md](settings.md) SET-41).
- **UI-32** A section is a title, a one-line description, and cards. A card is a list of rows divided by thin lines. A row has the label and its hint on the left and one control on the right; text areas go full width under their label (`stack`). Components: `Section`, `Card`, `Field`, `ToggleField`, `CheckField`, `RadioField`, `Note` in `src/settings/fields.tsx`.
- **UI-33** A problem the user must see (a damaged settings file, a hotkey another app holds) is a banner above the section, whichever section is open. A hotkey problem also puts a dot on the *Popup trigger* sidebar item while another section is open.
- **UI-34** The save bar is fixed under the sections: the last message on the left, *Save Settings* on the right, with the same 12 px of space above, below and beside the button.
- **UI-35** Below 680 px wide the sidebar shrinks to icons (names in tooltips); the window's minimum width is 560 px.

**Verified by:** `src/settings/SettingsPage.test.tsx` ("sections": one section open at a time and edits kept, the attention dot, reopening starts at the first section; the switch role of the start-at-sign-in option), `src/popup/Popup.test.tsx` ("footer": key hints, read-only, preview). Appearance itself is checked by looking at both windows in light and dark mode, in English and Vietnamese, at 540 × 500 (popup) and 760 × 780 and 560 × 640 (Settings).
