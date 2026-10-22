//! Collects the selection and its context from the foreground app, trying the cheapest and most
//! precise source first: the accessibility API (UI Automation, macOS Accessibility) → clipboard
//! (Ctrl+C / Cmd+C) → on-device OCR for context. The same steps on every platform; the OS work
//! goes through the `platform` facade.

use std::time::Duration;

use serde::Serialize;

use crate::geometry::Rect;
use crate::platform::{self, Accessible, Foreground, Monitor, Target};
use crate::settings::Settings;

/// Where the selected text came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionSource {
    Uia,
    Clipboard,
}

/// Where the surrounding text came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSource {
    Uia,
    Ocr,
    None,
}

#[derive(Debug)]
pub struct Captured {
    /// The app Apply pastes back into.
    pub target: Target,
    pub app: String,
    pub selection: String,
    pub before: String,
    pub after: String,
    pub screen_text: String,
    /// `None` when the control does not say; Apply is still offered.
    pub editable: Option<bool>,
    /// Where the selection is on screen, when the app reports it.
    pub bounds: Option<Rect>,
    pub cursor: (i32, i32),
    pub source: SelectionSource,
    pub context_source: ContextSource,
}

/// What started the capture, which decides how far it may go to read the selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// The hotkey or the icon: the user asked, so the selection may be copied with Ctrl+C / Cmd+C.
    Asked,
    /// Auto mode: nobody asked, so only what the accessibility API reports, and only when it is
    /// at least `autoMinChars` long. A synthetic Ctrl+C would interrupt a terminal.
    Auto,
}

#[derive(Debug)]
pub enum CaptureError {
    /// The hotkey was pressed while one of our own windows was in front.
    OwnWindow,
    NoSelection,
    PasswordField,
    BlockedApp(String),
    /// The Accessibility permission is missing (macOS).
    AccessibilityDenied,
    /// Auto mode only: the accessibility API reported no selection, or a shorter one than
    /// `autoMinChars`; the floating icon is offered instead.
    LeftToIcon,
}

/// The name the popup shows for an app the OS does not name itself (Windows).
pub fn friendly_app_name(process: &str) -> String {
    let name = match process {
        "winword.exe" => "Microsoft Word",
        "outlook.exe" | "olk.exe" => "Outlook",
        "powerpnt.exe" => "PowerPoint",
        "excel.exe" => "Excel",
        "onenote.exe" => "OneNote",
        "ms-teams.exe" | "teams.exe" => "Microsoft Teams",
        "slack.exe" => "Slack",
        "chrome.exe" => "Google Chrome",
        "msedge.exe" => "Microsoft Edge",
        "firefox.exe" => "Firefox",
        "acrord32.exe" | "acrobat.exe" => "Adobe Acrobat",
        "notepad.exe" => "Notepad",
        "code.exe" => "VS Code",
        "notion.exe" => "Notion",
        "obsidian.exe" => "Obsidian",
        "zalo.exe" => "Zalo",
        "telegram.exe" => "Telegram",
        "discord.exe" => "Discord",
        other => return other.trim_end_matches(".exe").to_string(),
    };
    name.to_string()
}

fn last_chars(s: &str, n: usize) -> String {
    let count = s.chars().count();
    s.chars().skip(count.saturating_sub(n)).collect()
}

fn first_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// The screen area to OCR for context: generously around the selection, or around the pointer
/// when the app does not say where the selection is, kept on the monitor.
fn ocr_area(selection: Option<Rect>, cursor: (i32, i32), monitor: Rect, scale: f64) -> Rect {
    let pad_x = (420.0 * scale) as i32;
    let pad_y = (200.0 * scale) as i32;
    let area = match selection {
        Some(bounds) => bounds.inflate(pad_x, pad_y),
        None => Rect::around(cursor, 2 * pad_x, pad_y),
    };
    area.intersect(monitor)
}

/// Cuts OCR output down to the part around the selection. OCR text differs slightly from the
/// real text, so the match ignores whitespace and case and only uses the selection's start.
fn focus_screen_text(lines: &[String], selection: &str, n: usize) -> String {
    let text: Vec<char> = lines.join("\n").chars().collect();
    let mut normalized: Vec<char> = Vec::new();
    let mut origin: Vec<usize> = Vec::new();
    for (i, c) in text.iter().enumerate() {
        if !c.is_whitespace() {
            normalized.push(c.to_lowercase().next().unwrap_or(*c));
            origin.push(i);
        }
    }
    let needle: Vec<char> = selection
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_lowercase().next().unwrap_or(c))
        .take(24)
        .collect();

    let found = (needle.len() >= 6)
        .then(|| {
            normalized
                .windows(needle.len())
                .position(|w| w == needle.as_slice())
        })
        .flatten();
    let (start, end) = match found {
        Some(k) => {
            let begin = origin[k];
            let finish = (begin + selection.chars().count()).min(text.len());
            (begin.saturating_sub(n), (finish + n).min(text.len()))
        }
        None => (0, (2 * n).min(text.len())),
    };
    text[start..end].iter().collect()
}

/// What a capture needs from the OS. `Os` is the real desktop; tests drive the capture with a
/// fake one, so its decisions are tested without other apps.
trait Desktop {
    fn has_accessibility(&self) -> bool;
    fn foreground(&self) -> Option<Foreground>;
    fn wait_modifiers_released(&self);
    fn cursor(&self) -> (i32, i32);
    fn read_focused(&self, target: Target, context_chars: usize) -> Accessible;
    fn copy_selection(&self) -> Option<String>;
    fn monitor_at(&self, x: i32, y: i32) -> Monitor;
    fn recognize_text(&self, area: Rect, scale: f64) -> Option<Vec<String>>;
}

struct Os;

impl Desktop for Os {
    fn has_accessibility(&self) -> bool {
        platform::has_accessibility()
    }
    fn foreground(&self) -> Option<Foreground> {
        platform::foreground()
    }
    fn wait_modifiers_released(&self) {
        // The hotkey's own modifiers would turn the synthetic copy into another shortcut.
        platform::wait_modifiers_released(Duration::from_millis(600));
    }
    fn cursor(&self) -> (i32, i32) {
        platform::cursor()
    }
    fn read_focused(&self, target: Target, context_chars: usize) -> Accessible {
        platform::read_focused(target, context_chars)
    }
    fn copy_selection(&self) -> Option<String> {
        platform::copy_selection()
    }
    fn monitor_at(&self, x: i32, y: i32) -> Monitor {
        platform::monitor_at(x, y)
    }
    fn recognize_text(&self, area: Rect, scale: f64) -> Option<Vec<String>> {
        platform::recognize_text(area, scale)
    }
}

/// Captures the selection and its context from the app in front.
pub fn capture(settings: &Settings, trigger: Trigger) -> Result<Captured, CaptureError> {
    capture_from(&Os, settings, trigger)
}

fn capture_from(
    desktop: &impl Desktop,
    settings: &Settings,
    trigger: Trigger,
) -> Result<Captured, CaptureError> {
    if !desktop.has_accessibility() {
        return Err(CaptureError::AccessibilityDenied);
    }
    let front = desktop.foreground().ok_or(CaptureError::NoSelection)?;
    if front.own {
        return Err(CaptureError::OwnWindow);
    }
    let app = front
        .name
        .unwrap_or_else(|| friendly_app_name(&front.process));
    if settings.is_blocked_app(&front.process) {
        return Err(CaptureError::BlockedApp(app));
    }

    desktop.wait_modifiers_released();
    let cursor = desktop.cursor();
    let n = settings.context_chars as usize;

    let focused = desktop.read_focused(front.target, n);
    if focused.is_password {
        return Err(CaptureError::PasswordField);
    }
    // `Uia` stands for the platform's accessibility API on every system.
    let (selection, source) = if !focused.selection.trim().is_empty() {
        (focused.selection.clone(), SelectionSource::Uia)
    } else if trigger == Trigger::Auto {
        return Err(CaptureError::LeftToIcon);
    } else {
        match desktop.copy_selection() {
            Some(text) => (text, SelectionSource::Clipboard),
            None => return Err(CaptureError::NoSelection),
        }
    };
    if trigger == Trigger::Auto && selection.trim().chars().count() < settings.auto_min_chars() {
        return Err(CaptureError::LeftToIcon);
    }

    let mut captured = Captured {
        target: front.target,
        app,
        selection,
        before: String::new(),
        after: String::new(),
        screen_text: String::new(),
        editable: focused.editable,
        bounds: focused.bounds,
        cursor,
        source,
        context_source: ContextSource::None,
    };

    let has_context = !(focused.before.trim().is_empty() && focused.after.trim().is_empty());
    if source == SelectionSource::Uia && has_context {
        captured.before = last_chars(&focused.before, n);
        captured.after = first_chars(&focused.after, n);
        captured.context_source = ContextSource::Uia;
    } else if settings.ocr_fallback {
        let monitor = desktop.monitor_at(cursor.0, cursor.1);
        let area = ocr_area(focused.bounds, cursor, monitor.bounds, monitor.scale);
        if let Some(lines) = desktop.recognize_text(area, monitor.scale) {
            let text = focus_screen_text(&lines, &captured.selection, n);
            if !text.trim().is_empty() {
                captured.screen_text = text;
                captured.context_source = ContextSource::Ocr;
            }
        }
    }
    Ok(captured)
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;

    /// A desktop whose answers the test sets. It records whether Ctrl+C or OCR ran.
    struct FakeDesktop {
        access: bool,
        front: RefCell<Option<Foreground>>,
        focused: RefCell<Accessible>,
        clipboard: Option<String>,
        ocr: Option<Vec<String>>,
        copied: Cell<bool>,
        recognized: Cell<bool>,
    }

    impl Desktop for FakeDesktop {
        fn has_accessibility(&self) -> bool {
            self.access
        }
        fn foreground(&self) -> Option<Foreground> {
            self.front.take()
        }
        fn wait_modifiers_released(&self) {}
        fn cursor(&self) -> (i32, i32) {
            (500, 400)
        }
        fn read_focused(&self, _target: Target, _context_chars: usize) -> Accessible {
            self.focused.take()
        }
        fn copy_selection(&self) -> Option<String> {
            self.copied.set(true);
            self.clipboard.clone()
        }
        fn monitor_at(&self, _x: i32, _y: i32) -> Monitor {
            Monitor {
                work: MONITOR,
                bounds: MONITOR,
                scale: 1.0,
            }
        }
        fn recognize_text(&self, _area: Rect, _scale: f64) -> Option<Vec<String>> {
            self.recognized.set(true);
            self.ocr.clone()
        }
    }

    /// Slack in front, with "pls check my PR" selected in a control that gives its context.
    fn desktop() -> FakeDesktop {
        FakeDesktop {
            access: true,
            front: RefCell::new(Some(Foreground {
                target: Target::default(),
                process: "slack.exe".into(),
                name: None,
                own: false,
            })),
            focused: RefCell::new(Accessible {
                selection: "pls check my PR".into(),
                before: "Minh: back in 30 mins\n".into(),
                after: "\nthanks".into(),
                editable: Some(true),
                ..Accessible::default()
            }),
            clipboard: None,
            ocr: None,
            copied: Cell::new(false),
            recognized: Cell::new(false),
        }
    }

    /// The focused control gives no selection and no context.
    fn silent(mut desktop: FakeDesktop) -> FakeDesktop {
        desktop.focused = RefCell::new(Accessible::default());
        desktop
    }

    #[test]
    fn the_accessibility_api_comes_first() {
        let desktop = desktop();
        let captured = capture_from(&desktop, &Settings::default(), Trigger::Asked).unwrap();
        assert_eq!(captured.app, "Slack");
        assert_eq!(captured.selection, "pls check my PR");
        assert_eq!(captured.source, SelectionSource::Uia);
        assert_eq!(captured.context_source, ContextSource::Uia);
        assert_eq!(captured.after, "\nthanks");
        assert!(
            !desktop.copied.get(),
            "no Ctrl+C when the selection was read"
        );
        assert!(
            !desktop.recognized.get(),
            "no OCR when the app gave context"
        );
    }

    #[test]
    fn context_is_cut_to_the_configured_length() {
        let settings = Settings {
            context_chars: 7,
            ..Settings::default()
        };
        let captured = capture_from(&desktop(), &settings, Trigger::Asked).unwrap();
        assert_eq!(captured.before, "0 mins\n");
        assert_eq!(captured.after, "\nthanks");
    }

    #[test]
    fn the_clipboard_and_ocr_fill_in_when_the_app_says_nothing() {
        let mut desktop = silent(desktop());
        desktop.clipboard = Some("pls check my PR".into());
        desktop.ocr = Some(vec![
            "Minh: back in 30 mins".into(),
            "pls check my PR".into(),
        ]);
        let captured = capture_from(&desktop, &Settings::default(), Trigger::Asked).unwrap();
        assert_eq!(captured.source, SelectionSource::Clipboard);
        assert_eq!(captured.context_source, ContextSource::Ocr);
        assert!(captured.screen_text.contains("back in 30 mins"));
    }

    #[test]
    fn ocr_stays_off_when_the_settings_say_so() {
        let mut desktop = silent(desktop());
        desktop.clipboard = Some("pls check my PR".into());
        desktop.ocr = Some(vec!["anything".into()]);
        let settings = Settings {
            ocr_fallback: false,
            ..Settings::default()
        };
        let captured = capture_from(&desktop, &settings, Trigger::Asked).unwrap();
        assert_eq!(captured.context_source, ContextSource::None);
        assert!(!desktop.recognized.get());
    }

    #[test]
    fn nothing_selected_anywhere_is_reported() {
        let result = capture_from(&silent(desktop()), &Settings::default(), Trigger::Asked);
        assert!(matches!(result, Err(CaptureError::NoSelection)));
    }

    #[test]
    fn auto_mode_never_copies_and_leaves_unreadable_selections_to_the_icon() {
        let mut desktop = silent(desktop());
        desktop.clipboard = Some("pls check my PR".into());
        let result = capture_from(&desktop, &Settings::default(), Trigger::Auto);
        assert!(matches!(result, Err(CaptureError::LeftToIcon)));
        assert!(!desktop.copied.get(), "no synthetic Ctrl+C unasked");
    }

    #[test]
    fn auto_mode_leaves_short_selections_to_the_icon() {
        let settings = Settings {
            auto_min_chars: 16,
            ..Settings::default()
        };
        // "pls check my PR" is 15 characters.
        let short = capture_from(&desktop(), &settings, Trigger::Auto);
        assert!(matches!(short, Err(CaptureError::LeftToIcon)));

        let settings = Settings {
            auto_min_chars: 15,
            ..settings
        };
        let captured = capture_from(&desktop(), &settings, Trigger::Auto).unwrap();
        assert_eq!(captured.selection, "pls check my PR");
        let asked = capture_from(
            &desktop(),
            &Settings {
                auto_min_chars: 99,
                ..Settings::default()
            },
            Trigger::Asked,
        );
        assert!(asked.is_ok(), "the minimum only applies to Auto mode");
    }

    #[test]
    fn password_fields_are_never_read_or_copied() {
        let desktop = desktop();
        desktop.focused.borrow_mut().is_password = true;
        let result = capture_from(&desktop, &Settings::default(), Trigger::Asked);
        assert!(matches!(result, Err(CaptureError::PasswordField)));
        assert!(!desktop.copied.get());
    }

    #[test]
    fn blocked_apps_are_named_and_left_alone() {
        let desktop = desktop();
        let settings = Settings {
            blocked_apps: vec!["slack.exe".into()],
            ..Settings::default()
        };
        let result = capture_from(&desktop, &settings, Trigger::Asked);
        assert!(matches!(result, Err(CaptureError::BlockedApp(name)) if name == "Slack"));
        assert!(!desktop.copied.get());
    }

    #[test]
    fn our_own_windows_and_a_missing_permission_stop_the_capture() {
        let own = desktop();
        if let Some(front) = own.front.borrow_mut().as_mut() {
            front.own = true;
        }
        assert!(matches!(
            capture_from(&own, &Settings::default(), Trigger::Asked),
            Err(CaptureError::OwnWindow)
        ));

        let denied = FakeDesktop {
            access: false,
            ..desktop()
        };
        assert!(matches!(
            capture_from(&denied, &Settings::default(), Trigger::Asked),
            Err(CaptureError::AccessibilityDenied)
        ));
    }

    #[test]
    fn the_name_the_os_gives_wins_over_the_executable() {
        let desktop = desktop();
        if let Some(front) = desktop.front.borrow_mut().as_mut() {
            front.name = Some("Slack for Mac".into());
        }
        let captured = capture_from(&desktop, &Settings::default(), Trigger::Asked).unwrap();
        assert_eq!(captured.app, "Slack for Mac");
    }

    #[test]
    fn well_known_apps_get_a_friendly_name() {
        let names = [
            ("winword.exe", "Microsoft Word"),
            ("outlook.exe", "Outlook"),
            ("olk.exe", "Outlook"),
            ("ms-teams.exe", "Microsoft Teams"),
            ("teams.exe", "Microsoft Teams"),
            ("slack.exe", "Slack"),
            ("chrome.exe", "Google Chrome"),
            ("msedge.exe", "Microsoft Edge"),
            ("acrord32.exe", "Adobe Acrobat"),
            ("code.exe", "VS Code"),
        ];
        for (process, name) in names {
            assert_eq!(friendly_app_name(process), name, "{process}");
        }
    }

    #[test]
    fn other_apps_show_their_executable_name_without_the_extension() {
        assert_eq!(friendly_app_name("mspaint.exe"), "mspaint");
        assert_eq!(friendly_app_name("tool"), "tool");
    }

    #[test]
    fn context_is_cut_by_characters_not_bytes() {
        let text = "đường Nguyễn Huệ";
        assert_eq!(first_chars(text, 5), "đường");
        assert_eq!(last_chars(text, 4), " Huệ");
        assert_eq!(last_chars("ab", 5), "ab");
        assert_eq!(first_chars("ab", 5), "ab");
    }

    #[test]
    fn focuses_on_the_selection() {
        let lines = vec![
            "Alice: are we still on for the demo?".to_string(),
            "Bob: pls check my PR, it urgent".to_string(),
            "Alice: sure".to_string(),
        ];
        let out = focus_screen_text(&lines, "pls check my PR, it urgent", 10);
        assert!(out.contains("pls check my PR, it urgent"));
        assert!(!out.contains("demo"));
    }

    #[test]
    fn falls_back_to_the_start_when_not_found() {
        let lines = vec!["something else entirely".to_string()];
        assert_eq!(
            focus_screen_text(&lines, "hello world again", 5),
            "something "
        );
    }

    const MONITOR: Rect = Rect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };

    #[test]
    fn ocr_area_pads_the_selection_and_stays_on_the_monitor() {
        let selection = Rect {
            left: 1000,
            top: 500,
            right: 1200,
            bottom: 520,
        };
        assert_eq!(
            ocr_area(Some(selection), (0, 0), MONITOR, 1.0),
            Rect {
                left: 580,
                top: 300,
                right: 1620,
                bottom: 720
            }
        );
        let near_edge = Rect {
            left: 10,
            top: 10,
            right: 50,
            bottom: 30,
        };
        let area = ocr_area(Some(near_edge), (0, 0), MONITOR, 1.0);
        assert_eq!((area.left, area.top), (0, 0));
    }

    #[test]
    fn ocr_area_without_bounds_is_wide_around_the_pointer() {
        assert_eq!(
            ocr_area(None, (960, 540), MONITOR, 1.0),
            Rect {
                left: 120,
                top: 340,
                right: 1800,
                bottom: 740
            }
        );
    }
}
