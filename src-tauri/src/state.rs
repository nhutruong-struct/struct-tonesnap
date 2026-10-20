//! State shared by the commands, the capture thread and the request tasks.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::SeqCst};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};

use crate::commands::CommandError;
use crate::session::Session;
use crate::settings::{Settings, SettingsIssue};

pub struct AppState {
    settings: Mutex<Settings>,
    /// The selection the popup is working on.
    session: Mutex<Option<Session>>,
    session_seq: AtomicU64,
    /// Id of the current request. Bumping it cancels the running one.
    request_seq: AtomicU64,
    capturing: AtomicBool,
    popup_shown_at: Mutex<Instant>,
    /// Why the configured hotkey is not active, if it is not.
    hotkey_error: Mutex<Option<CommandError>>,
    /// What was wrong with the saved settings, until they are saved again.
    settings_issue: Mutex<Option<SettingsIssue>>,
}

pub fn state(app: &AppHandle) -> tauri::State<'_, AppState> {
    app.state::<AppState>()
}

impl AppState {
    pub fn new(settings: Settings, settings_issue: Option<SettingsIssue>) -> Self {
        Self {
            settings: Mutex::new(settings),
            session: Mutex::new(None),
            session_seq: AtomicU64::new(0),
            request_seq: AtomicU64::new(0),
            capturing: AtomicBool::new(false),
            popup_shown_at: Mutex::new(Instant::now()),
            hotkey_error: Mutex::new(None),
            settings_issue: Mutex::new(settings_issue),
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    /// Replaces the settings with ones that were just saved, which also settles any issue
    /// found when loading them.
    pub fn set_saved_settings(&self, settings: Settings) {
        *self.settings.lock().unwrap() = settings;
        *self.settings_issue.lock().unwrap() = None;
    }

    pub fn settings_issue(&self) -> Option<SettingsIssue> {
        self.settings_issue.lock().unwrap().clone()
    }

    pub fn session(&self) -> Option<Session> {
        self.session.lock().unwrap().clone()
    }

    pub fn set_session(&self, session: Session) {
        *self.session.lock().unwrap() = Some(session);
    }

    pub fn next_session_id(&self) -> u64 {
        self.session_seq.fetch_add(1, SeqCst) + 1
    }

    /// Starts a new request, which supersedes (and so cancels) any running one.
    pub fn begin_request(&self) -> u64 {
        self.request_seq.fetch_add(1, SeqCst) + 1
    }

    pub fn is_current_request(&self, id: u64) -> bool {
        self.request_seq.load(SeqCst) == id
    }

    /// Stops the running request, if any: its stream ends at the next chunk.
    pub fn cancel_request(&self) {
        self.request_seq.fetch_add(1, SeqCst);
    }

    /// Marks a capture as running; false when one already is.
    pub fn try_begin_capture(&self) -> bool {
        !self.capturing.swap(true, SeqCst)
    }

    pub fn is_capturing(&self) -> bool {
        self.capturing.load(SeqCst)
    }

    pub fn end_capture(&self) {
        self.capturing.store(false, SeqCst);
    }

    pub fn mark_popup_shown(&self) {
        *self.popup_shown_at.lock().unwrap() = Instant::now();
    }

    pub fn popup_shown_within(&self, window: Duration) -> bool {
        self.popup_shown_at.lock().unwrap().elapsed() <= window
    }

    pub fn hotkey_error(&self) -> Option<CommandError> {
        self.hotkey_error.lock().unwrap().clone()
    }

    pub fn set_hotkey_error(&self, error: Option<CommandError>) {
        *self.hotkey_error.lock().unwrap() = error;
    }
}
