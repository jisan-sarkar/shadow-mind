mod capture_indicator;
mod image_input;
mod media_input;
mod session;
mod settings;

use gtk::prelude::*;
use image_input::{ImageInputError, ImageMetadata};
use media_input::{AudioChoice, NativeStatus};
use serde::Serialize;
use session::{
    Fixture, InputKind, Session, SessionAction, SessionError, SessionSnapshot, SessionWork,
};
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
    Unavailable,
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
            SessionError::ImageConsentRequired => (
                "image_consent_required",
                ErrorCategory::Permission,
                "Confirm that you have permission to inspect the selected image.",
            ),
            SessionError::NativeConsentRequired => (
                "native_consent_required",
                ErrorCategory::Permission,
                "Confirm that you have permission to use this source.",
            ),
            SessionError::SourceUnavailable => (
                "native_source_unavailable",
                ErrorCategory::Unavailable,
                "The source is unavailable. Refresh the source list or choose an approved source again.",
            ),
            SessionError::CleanupFailed => (
                "source_release_failed",
                ErrorCategory::Unavailable,
                "Close and reopen ShadowMind before using this input again.",
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

impl From<ImageInputError> for PublicError {
    fn from(error: ImageInputError) -> Self {
        let (code, category, message) = match error {
            ImageInputError::Unsupported => (
                "image_format_unsupported",
                ErrorCategory::Validation,
                "Choose a local PNG or JPEG image.",
            ),
            ImageInputError::Unreadable => (
                "image_unreadable",
                ErrorCategory::Unavailable,
                "The selected image could not be read.",
            ),
            ImageInputError::TooLarge => (
                "image_too_large",
                ErrorCategory::Validation,
                "Choose an image up to 16 MiB and 4096 pixels on each side.",
            ),
            ImageInputError::ChooserUnavailable => (
                "image_chooser_unavailable",
                ErrorCategory::Unavailable,
                "The desktop file chooser is unavailable. Try again.",
            ),
            ImageInputError::CleanupFailed => (
                "image_chooser_release_failed",
                ErrorCategory::Unavailable,
                "Close and reopen ShadowMind before opening the file chooser again.",
            ),
        };
        Self {
            code,
            category,
            message,
            recoverable: true,
            capability: "image",
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

#[tauri::command]
async fn choose_image(
    foundation: State<'_, Mutex<Foundation>>,
    session_id: uuid::Uuid,
    authorized: bool,
) -> Result<Option<ImageMetadata>, PublicError> {
    let SessionWork {
        request_id,
        cancellation,
        ..
    } = foundation
        .lock()
        .map_err(|_| PublicError::app_unavailable())?
        .session
        .begin_image(session_id, authorized)?;
    let result = image_input::choose_and_inspect(cancellation).await;
    let still_current = foundation
        .lock()
        .map_err(|_| PublicError::app_unavailable())?
        .session
        .finish_image(
            session_id,
            request_id,
            matches!(result, Err(ImageInputError::CleanupFailed)),
        );
    if still_current {
        result.map_err(Into::into)
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn list_audio_sources(
    foundation: State<'_, Mutex<Foundation>>,
    session_id: uuid::Uuid,
) -> Result<Vec<AudioChoice>, PublicError> {
    foundation
        .lock()
        .map_err(|_| PublicError::app_unavailable())?
        .session
        .require_native(session_id, InputKind::Audio)?;
    let choices = tauri::async_runtime::spawn_blocking(media_input::audio_choices)
        .await
        .map_err(|_| SessionError::SourceUnavailable)?
        .map_err(|_| SessionError::SourceUnavailable)?;
    let choices = foundation
        .lock()
        .map_err(|_| PublicError::app_unavailable())?
        .session
        .store_audio_choices(session_id, choices)?;
    Ok(choices)
}

#[tauri::command]
fn start_native_input(
    app: AppHandle,
    foundation: State<'_, Mutex<Foundation>>,
    session_id: uuid::Uuid,
    kind: InputKind,
    confirmed: bool,
    source_id: Option<uuid::Uuid>,
) -> Result<Snapshot, PublicError> {
    let mut foundation = foundation
        .lock()
        .map_err(|_| PublicError::app_unavailable())?;
    let (work, source) = foundation
        .session
        .begin_native(session_id, kind, confirmed, source_id)?;
    foundation.revision += 1;
    let initial = foundation.snapshot();
    drop(foundation);
    tauri::async_runtime::spawn(async move {
        let SessionWork {
            session_id,
            request_id,
            cancellation,
        } = work;
        let status = media_input::receive(source, cancellation, || {
            update_native_status(&app, session_id, request_id, kind, NativeStatus::Receiving);
        })
        .await;
        update_native_status(&app, session_id, request_id, kind, status);
    });
    Ok(initial)
}

#[tauri::command]
fn stop_native_inputs(
    foundation: State<'_, Mutex<Foundation>>,
    session_id: uuid::Uuid,
) -> Result<Snapshot, PublicError> {
    let mut foundation = foundation
        .lock()
        .map_err(|_| PublicError::app_unavailable())?;
    foundation.session.stop_native_for(session_id)?;
    foundation.revision += 1;
    Ok(foundation.snapshot())
}

fn update_native_status(
    app: &AppHandle,
    session_id: uuid::Uuid,
    request_id: uuid::Uuid,
    kind: InputKind,
    status: NativeStatus,
) {
    let state = app.state::<Mutex<Foundation>>();
    let Ok(mut foundation) = state.lock() else {
        return;
    };
    if foundation
        .session
        .update_native(session_id, request_id, kind, status)
    {
        foundation.revision += 1;
        let snapshot = foundation.snapshot();
        drop(foundation);
        emit_session_snapshot(app, snapshot);
    }
}

fn emit_session_snapshot(app: &AppHandle, snapshot: Snapshot) {
    let event = SnapshotEvent {
        schema_version: 1,
        kind: "snapshotUpdated",
        origin: "session",
        snapshot,
    };
    let _ = app.emit_to("main", SNAPSHOT_EVENT, event);
}

fn update_lifecycle(app: &AppHandle, change: impl FnOnce(&mut Session) -> bool) {
    let state = app.state::<Mutex<Foundation>>();
    let Ok(mut foundation) = state.lock() else {
        return;
    };
    if change(&mut foundation.session) {
        foundation.revision += 1;
        let snapshot = foundation.snapshot();
        drop(foundation);
        emit_session_snapshot(app, snapshot);
    }
}

async fn complete_fixture(app: AppHandle, work: SessionWork) {
    let SessionWork {
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
        let snapshot = foundation.snapshot();
        drop(foundation);
        emit_session_snapshot(&app, snapshot);
    }
}

async fn wait_for_input_cleanup(app: &AppHandle) {
    let state = app.state::<Mutex<Foundation>>();
    while state.lock().is_ok_and(|state| state.session.draining()) {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_config_dir()?;
            let mut foundation = Foundation::open(SettingsRepository::new(directory));
            foundation.session.set_desktop_suspended(true);
            app.manage(Mutex::new(foundation));
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let app = &handle;
                media_input::watch_desktop(
                    || update_lifecycle(app, |session| session.set_desktop_suspended(true)),
                    || update_lifecycle(app, |session| session.set_desktop_suspended(false)),
                    || wait_for_input_cleanup(app),
                )
                .await;
            });
            let window = app
                .get_webview_window("main")
                .ok_or("missing main window")?;
            let handle = app.handle().clone();
            window
                .gtk_window()?
                .connect_window_state_event(move |_, event| {
                    if event
                        .new_window_state()
                        .contains(gtk::gdk::WindowState::ICONIFIED)
                    {
                        update_lifecycle(&handle, Session::pause);
                    }
                    gtk::glib::Propagation::Proceed
                });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let mut begin_close = false;
                update_lifecycle(window.app_handle(), |session| {
                    begin_close = session.begin_close();
                    begin_close
                });
                api.prevent_close();
                if begin_close {
                    let window = window.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = tokio::time::timeout(
                            Duration::from_secs(7),
                            wait_for_input_cleanup(window.app_handle()),
                        )
                        .await;
                        let _ = window.destroy();
                    });
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            update_preferences,
            session_action,
            choose_image,
            list_audio_sources,
            start_native_input,
            stop_native_inputs,
        ])
        .run(tauri::generate_context!())
        .expect("launch ShadowMind");
}
