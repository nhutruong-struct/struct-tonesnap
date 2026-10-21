//! Collects the selection and its context from the foreground app, trying the cheapest and most
//! precise source first: UI Automation → clipboard (Ctrl+C) → on-device OCR for context.

use serde::Serialize;

use crate::geometry::Rect;
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
    pub hwnd: isize,
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

#[derive(Debug)]
pub enum CaptureError {
    /// The hotkey was pressed while one of our own windows was in front.
    OwnWindow,
    NoSelection,
    PasswordField,
    BlockedApp(String),
    /// The Accessibility permission is missing (macOS).
    #[cfg_attr(
        not(target_os = "macos"),
        allow(dead_code, reason = "only macOS asks the user for a permission")
    )]
    AccessibilityDenied,
}

#[cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "only the Windows capture uses it; macOS names apps itself and keeps no window rectangles"
    )
)]
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

#[cfg(windows)]
pub fn capture(settings: &Settings) -> Result<Captured, CaptureError> {
    use std::time::Duration;

    use crate::platform::win::{clipboard, input, ocr, uia, window};

    let hwnd = window::foreground();
    if window::is_own_window(hwnd) {
        return Err(CaptureError::OwnWindow);
    }
    let process = window::process_name(hwnd).unwrap_or_default();
    if settings.is_blocked_app(&process) {
        return Err(CaptureError::BlockedApp(friendly_app_name(&process)));
    }
    let app = friendly_app_name(&process);

    input::wait_modifiers_released(Duration::from_millis(600));
    let cursor = window::cursor();
    let n = settings.context_chars as usize;

    let ui = uia::capture(n as i32);
    if ui.is_password {
        return Err(CaptureError::PasswordField);
    }

    let (selection, source) = if !ui.selection.trim().is_empty() {
        (ui.selection.clone(), SelectionSource::Uia)
    } else {
        match clipboard::copy_selection() {
            Some(text) => (text, SelectionSource::Clipboard),
            None => return Err(CaptureError::NoSelection),
        }
    };

    let mut captured = Captured {
        hwnd,
        app,
        selection,
        before: String::new(),
        after: String::new(),
        screen_text: String::new(),
        editable: ui.editable,
        bounds: ui.bounds,
        cursor,
        source,
        context_source: ContextSource::None,
    };

    let uia_has_context = !(ui.before.trim().is_empty() && ui.after.trim().is_empty());
    if source == SelectionSource::Uia && uia_has_context {
        captured.before = last_chars(&ui.before, n);
        captured.after = first_chars(&ui.after, n);
        captured.context_source = ContextSource::Uia;
    } else if settings.ocr_fallback {
        let monitor = window::monitor_at(cursor.0, cursor.1);
        let area = ocr_area(ui.bounds, cursor, monitor.bounds, monitor.scale);
        if let Ok(lines) = ocr::recognize(area) {
            let text = focus_screen_text(&lines, &captured.selection, n);
            if !text.trim().is_empty() {
                captured.screen_text = text;
                captured.context_source = ContextSource::Ocr;
            }
        }
    }
    Ok(captured)
}

/// The selection and its context: the Accessibility API first, then the clipboard (Cmd+C) for
/// the selection, and on-device OCR for context when the app does not give any.
#[cfg(target_os = "macos")]
pub fn capture(settings: &Settings) -> Result<Captured, CaptureError> {
    use std::time::Duration;

    use crate::platform::mac::{self, clipboard, input};

    if !mac::is_trusted() {
        return Err(CaptureError::AccessibilityDenied);
    }
    let front = mac::frontmost().ok_or(CaptureError::NoSelection)?;
    // The popup does not make ToneSnap the active app, so ask who holds the keyboard too.
    if front.is_own() || mac::own_window_in_front() == Some(true) {
        return Err(CaptureError::OwnWindow);
    }
    if settings.is_blocked_app(&front.process) {
        return Err(CaptureError::BlockedApp(front.name));
    }
    input::wait_modifiers_released(Duration::from_millis(600));
    let cursor = mac::cursor();
    let n = settings.context_chars as usize;

    let ax = mac::capture(front.pid, n);
    if ax.is_password {
        return Err(CaptureError::PasswordField);
    }
    let (selection, source) = if !ax.selection.trim().is_empty() {
        (ax.selection.clone(), SelectionSource::Uia)
    } else {
        match clipboard::copy_selection() {
            Some(text) => (text, SelectionSource::Clipboard),
            None => return Err(CaptureError::NoSelection),
        }
    };

    let mut captured = Captured {
        // The process id plays the part of Windows' window handle.
        hwnd: front.pid as isize,
        app: front.name,
        selection,
        before: String::new(),
        after: String::new(),
        screen_text: String::new(),
        editable: ax.editable,
        bounds: ax.bounds,
        cursor,
        // `Uia` stands for the platform's accessibility API on both systems.
        source,
        context_source: ContextSource::None,
    };
    let ax_has_context = !(ax.before.trim().is_empty() && ax.after.trim().is_empty());
    if source == SelectionSource::Uia && ax_has_context {
        captured.before = last_chars(&ax.before, n);
        captured.after = first_chars(&ax.after, n);
        captured.context_source = ContextSource::Uia;
    } else if settings.ocr_fallback {
        let monitor = mac::monitor_at(cursor.0, cursor.1);
        let area = ocr_area(ax.bounds, cursor, monitor.bounds, monitor.scale);
        if let Some(lines) = mac::ocr::recognize(area, monitor.scale) {
            let text = focus_screen_text(&lines, &captured.selection, n);
            if !text.trim().is_empty() {
                captured.screen_text = text;
                captured.context_source = ContextSource::Ocr;
            }
        }
    }
    Ok(captured)
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn capture(_settings: &Settings) -> Result<Captured, CaptureError> {
    Err(CaptureError::NoSelection)
}

#[cfg(test)]
mod tests {
    use super::*;

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
