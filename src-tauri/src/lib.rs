mod session;
mod settings;

use serde::Serialize;
use session::{Fixture, FixtureWork, Session, SessionAction, SessionError, SessionSnapshot};
use settings::{Preferences, PreferencesUpdate, RecoveryNotice, SettingsRepository};
use std::{sync::Mutex, time::Duration};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

const SNAPSHOT_EVENT: &str = "shadowmind:snapshot";
const FIXTURE_CONTENT: &str = include_str!("../../fixtures/workspace-session.json");

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ErrorCategory {
    Validation,
    Permission,
    Storage,
    Internal,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicError {
    code: &'static str,
    category: ErrorCategory,
    message: &'static str,
    recoverable: bool,
    capability: &'static str,
}

impl PublicError {
    fn invalid_settings() -> Self {
        Self {
            code: "invalid_appearance",
            category: ErrorCategory::Validation,
            message: "Choose a supported theme and text size.",
            recoverable: true,
            capability: "preferences",
        }
    }

    fn save_failed() -> Self {
        Self {
            code: "settings_write_failed",
            category: ErrorCategory::Storage,
            message: "Settings could not be saved. Check available storage and try again.",
            recoverable: true,
            capability: "preferences",
        }
    }

    fn app_unavailable() -> Self {
        Self {
            code: "workspace_unavailable",
            category: ErrorCategory::Internal,
            message: "Restart ShadowMind to reopen the workspace.",
            recoverable: true,
            capability: "workspace",
        }
    }
}

impl From<SessionError> for PublicError {
    fn from(error: SessionError) -> Self {
        let (code, category, message) = match error {
            SessionError::InvalidTransition => (
                "invalid_transition",
                ErrorCategory::Validation,
                "That action is unavailable in the current session state.",
            ),
            SessionError::StaleSession => (
                "stale_session",
                ErrorCategory::Validation,
                "The session has changed. Check the current session controls.",
            ),
            SessionError::FixtureConsentRequired => (
                "fixture_consent_required",
                ErrorCategory::Permission,
                "Confirm use of the bundled synthetic inputs before starting.",
            ),
        };
        Self {
            code,
            category,
            message,
            recoverable: true,
            capability: "session",
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    schema_version: u8,
    revision: u64,
    preferences: Preferences,
    recovery_notice: Option<RecoveryNotice>,
    session: SessionSnapshot,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotEvent {
    schema_version: u8,
    kind: &'static str,
    origin: &'static str,
    snapshot: Snapshot,
}

struct Foundation {
    revision: u64,
    preferences: Preferences,
    recovery_notice: Option<RecoveryNotice>,
    storage: SettingsRepository,
    session: Session,
}

impl Foundation {
    fn open(storage: SettingsRepository) -> Self {
        let (preferences, recovery_notice) = storage.load();
        Self {
            revision: 0,
            preferences,
            recovery_notice,
            storage,
            session: Session::default(),
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            schema_version: 1,
            revision: self.revision,
            preferences: self.preferences,
            recovery_notice: self.recovery_notice,
            session: self.session.snapshot(),
        }
    }

    fn update_preferences(&mut self, update: PreferencesUpdate) -> Result<Snapshot, PublicError> {
        if !update.appearance.is_valid() {
            return Err(PublicError::invalid_settings());
        }
        let mut next = self.preferences;
        next.appearance = update.appearance;
        next.onboarding_complete |= update.complete_onboarding;
        self.storage
            .save(next)
            .map_err(|_| PublicError::save_failed())?;
        self.preferences = next;
        self.recovery_notice = None;
        self.revision += 1;
        Ok(self.snapshot())
    }
}

#[tauri::command]
fn snapshot(foundation: State<'_, Mutex<Foundation>>) -> Result<Snapshot, PublicError> {
    let foundation = foundation
        .lock()
        .map_err(|_| PublicError::app_unavailable())?;
    Ok(foundation.snapshot())
}

#[tauri::command]
async fn update_preferences(
    foundation: State<'_, Mutex<Foundation>>,
    update: PreferencesUpdate,
) -> Result<Snapshot, PublicError> {
    let mut foundation = foundation
        .lock()
        .map_err(|_| PublicError::app_unavailable())?;
    foundation.update_preferences(update)
}

#[tauri::command]
fn session_action(
    app: AppHandle,
    foundation: State<'_, Mutex<Foundation>>,
    action: SessionAction,
) -> Result<Snapshot, PublicError> {
    let mut foundation = foundation
        .lock()
        .map_err(|_| PublicError::app_unavailable())?;
    let work = foundation.session.apply(action)?;
    foundation.revision += 1;
    if let Some(work) = work {
        tauri::async_runtime::spawn(complete_fixture(app, work));
    }
    Ok(foundation.snapshot())
}

async fn complete_fixture(app: AppHandle, work: FixtureWork) {
    let FixtureWork {
        session_id,
        request_id,
        cancellation,
    } = work;
    tokio::select! {
        biased;
        _ = cancellation => return,
        _ = tokio::time::sleep(Duration::from_millis(400)) => {}
    }
    let fixture = serde_json::from_str::<Fixture>(FIXTURE_CONTENT).ok();
    let state = app.state::<Mutex<Foundation>>();
    let Ok(mut foundation) = state.lock() else {
        return;
    };
    if foundation.session.finish(session_id, request_id, fixture) {
        foundation.revision += 1;
        let event = SnapshotEvent {
            schema_version: 1,
            kind: "snapshotUpdated",
            origin: "session",
            snapshot: foundation.snapshot(),
        };
        drop(foundation);
        let _ = app.emit_to("main", SNAPSHOT_EVENT, event);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_config_dir()?;
            app.manage(Mutex::new(Foundation::open(SettingsRepository::new(
                directory,
            ))));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event
                && let Some(state) = window.try_state::<Mutex<Foundation>>()
                && let Ok(mut state) = state.lock()
            {
                state.session.stop();
            }
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            update_preferences,
            session_action,
        ])
        .run(tauri::generate_context!())
        .expect("launch ShadowMind");
}
