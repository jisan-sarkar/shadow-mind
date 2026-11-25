use crate::media_input::{AudioChoice, MediaSource, NativeStatus};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InputMode {
    Screen,
    Audio,
    Both,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InputKind {
    Screen,
    Audio,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    #[default]
    Idle,
    Configuring,
    Active,
    Paused,
    Stopping,
    Stopped,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FixtureStatus {
    #[default]
    Ready,
    Processing,
    Complete,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub question: String,
    pub answer: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub id: Option<Uuid>,
    pub mode: Option<InputMode>,
    pub phase: Phase,
    pub fixture_status: FixtureStatus,
    pub fixture: Option<Fixture>,
    pub screen_status: NativeStatus,
    pub audio_status: NativeStatus,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SessionError {
    InvalidTransition,
    StaleSession,
    FixtureConsentRequired,
    ImageConsentRequired,
    NativeConsentRequired,
    SourceUnavailable,
}

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum SessionAction {
    Configure {
        mode: InputMode,
    },
    Start {
        session_id: Uuid,
        confirm_fixtures: bool,
    },
    Pause {
        session_id: Uuid,
    },
    Resume {
        session_id: Uuid,
    },
    Stop {
        session_id: Uuid,
    },
    RunFixture {
        session_id: Uuid,
    },
    CancelFixture {
        session_id: Uuid,
    },
}

pub struct SessionWork {
    pub session_id: Uuid,
    pub request_id: Uuid,
    pub cancellation: oneshot::Receiver<()>,
}

struct PendingWork {
    id: Uuid,
    cancellation: Option<oneshot::Sender<()>>,
}

#[derive(Default)]
pub struct Session {
    view: SessionSnapshot,
    pending: Option<PendingWork>,
    pending_image: Option<PendingWork>,
    screen: Option<PendingWork>,
    audio: Option<PendingWork>,
    audio_choices: Vec<AudioChoice>,
}

impl Session {
    pub fn snapshot(&self) -> SessionSnapshot {
        self.view.clone()
    }

    pub fn apply(&mut self, action: SessionAction) -> Result<Option<SessionWork>, SessionError> {
        match action {
            SessionAction::Configure { mode }
                if matches!(self.view.phase, Phase::Idle | Phase::Stopped)
                    && self.screen.is_none()
                    && self.audio.is_none() =>
            {
                self.view = SessionSnapshot {
                    id: Some(Uuid::new_v4()),
                    mode: Some(mode),
                    phase: Phase::Configuring,
                    ..SessionSnapshot::default()
                };
            }
            SessionAction::Configure { .. } => return Err(SessionError::InvalidTransition),
            SessionAction::Start {
                session_id,
                confirm_fixtures,
            } => {
                self.require_session(session_id)?;
                if self.view.phase != Phase::Configuring {
                    return Err(SessionError::InvalidTransition);
                }
                if !confirm_fixtures {
                    return Err(SessionError::FixtureConsentRequired);
                }
                self.view.phase = Phase::Active;
            }
            SessionAction::Pause { session_id } => {
                self.require_session(session_id)?;
                if !matches!(self.view.phase, Phase::Active | Phase::Paused) {
                    return Err(SessionError::InvalidTransition);
                }
                self.cancel_fixture();
                self.cancel_image();
                self.stop_native();
                self.view.phase = Phase::Paused;
            }
            SessionAction::Resume { session_id } => {
                self.require_session(session_id)?;
                if self.view.phase != Phase::Paused {
                    return Err(SessionError::InvalidTransition);
                }
                self.view.phase = Phase::Active;
            }
            SessionAction::Stop { session_id } => {
                self.require_session(session_id)?;
                self.stop();
            }
            SessionAction::RunFixture { session_id } => {
                self.require_session(session_id)?;
                if self.view.phase != Phase::Active || self.pending.is_some() {
                    return Err(SessionError::InvalidTransition);
                }
                let request_id = Uuid::new_v4();
                let (sender, cancellation) = oneshot::channel();
                self.pending = Some(PendingWork {
                    id: request_id,
                    cancellation: Some(sender),
                });
                self.view.fixture = None;
                self.view.fixture_status = FixtureStatus::Processing;
                return Ok(Some(SessionWork {
                    session_id,
                    request_id,
                    cancellation,
                }));
            }
            SessionAction::CancelFixture { session_id } => {
                self.require_session(session_id)?;
                if self.view.phase != Phase::Active {
                    return Err(SessionError::InvalidTransition);
                }
                self.cancel_fixture();
            }
        }
        Ok(None)
    }

    pub fn require_native(&self, id: Uuid, kind: InputKind) -> Result<(), SessionError> {
        self.require_session(id)?;
        let allowed = matches!(
            (kind, self.view.mode),
            (_, Some(InputMode::Both))
                | (InputKind::Screen, Some(InputMode::Screen))
                | (InputKind::Audio, Some(InputMode::Audio))
        );
        if self.view.phase == Phase::Active && allowed {
            Ok(())
        } else {
            Err(SessionError::InvalidTransition)
        }
    }

    pub fn store_audio_choices(
        &mut self,
        id: Uuid,
        choices: Vec<AudioChoice>,
    ) -> Result<Vec<AudioChoice>, SessionError> {
        self.require_native(id, InputKind::Audio)?;
        self.audio_choices = choices.clone();
        Ok(choices)
    }

    pub fn begin_native(
        &mut self,
        session_id: Uuid,
        kind: InputKind,
        confirmed: bool,
        source_id: Option<Uuid>,
    ) -> Result<(SessionWork, MediaSource), SessionError> {
        self.require_native(session_id, kind)?;
        if !confirmed {
            return Err(SessionError::NativeConsentRequired);
        }
        if self.native_slot(kind).is_some() {
            return Err(SessionError::InvalidTransition);
        }
        let source = match kind {
            InputKind::Screen if source_id.is_none() => MediaSource::Screen,
            InputKind::Audio => {
                let choice = self
                    .audio_choices
                    .iter()
                    .find(|choice| Some(choice.id) == source_id)
                    .ok_or(SessionError::SourceUnavailable)?;
                MediaSource::Audio(choice.target.clone())
            }
            InputKind::Screen => return Err(SessionError::SourceUnavailable),
        };
        let request_id = Uuid::new_v4();
        let (sender, cancellation) = oneshot::channel();
        *self.native_slot(kind) = Some(PendingWork {
            id: request_id,
            cancellation: Some(sender),
        });
        *self.native_status(kind) = NativeStatus::Connecting;
        Ok((
            SessionWork {
                session_id,
                request_id,
                cancellation,
            },
            source,
        ))
    }

    pub fn update_native(
        &mut self,
        session_id: Uuid,
        request_id: Uuid,
        kind: InputKind,
        status: NativeStatus,
    ) -> bool {
        if self.view.id != Some(session_id)
            || self
                .native_slot(kind)
                .as_ref()
                .is_none_or(|work| work.id != request_id)
        {
            return false;
        }
        let shutting_down = *self.native_status(kind) == NativeStatus::Disconnecting;
        if status == NativeStatus::Receiving {
            if shutting_down || self.require_native(session_id, kind).is_err() {
                return false;
            }
            *self.native_status(kind) = status;
            return true;
        }
        self.native_slot(kind).take();
        if kind == InputKind::Audio {
            self.audio_choices.clear();
        }
        *self.native_status(kind) = if shutting_down {
            NativeStatus::Idle
        } else {
            status
        };
        true
    }

    pub fn stop_native_for(&mut self, id: Uuid) -> Result<(), SessionError> {
        self.require_session(id)?;
        self.stop_native();
        Ok(())
    }

    pub fn stop_native(&mut self) -> bool {
        let mut changed = false;
        for kind in [InputKind::Screen, InputKind::Audio] {
            let pending = self.native_slot(kind);
            let desired = if let Some(work) = pending {
                if let Some(sender) = work.cancellation.take() {
                    let _ = sender.send(());
                }
                NativeStatus::Disconnecting
            } else {
                NativeStatus::Idle
            };
            let status = self.native_status(kind);
            changed |= *status != desired;
            *status = desired;
        }
        self.audio_choices.clear();
        changed
    }

    fn native_slot(&mut self, kind: InputKind) -> &mut Option<PendingWork> {
        match kind {
            InputKind::Screen => &mut self.screen,
            InputKind::Audio => &mut self.audio,
        }
    }

    fn native_status(&mut self, kind: InputKind) -> &mut NativeStatus {
        match kind {
            InputKind::Screen => &mut self.view.screen_status,
            InputKind::Audio => &mut self.view.audio_status,
        }
    }

    pub fn begin_image(
        &mut self,
        session_id: Uuid,
        authorized: bool,
    ) -> Result<SessionWork, SessionError> {
        self.require_session(session_id)?;
        if self.view.phase != Phase::Active
            || !matches!(self.view.mode, Some(InputMode::Screen | InputMode::Both))
            || self.pending_image.is_some()
        {
            return Err(SessionError::InvalidTransition);
        }
        if !authorized {
            return Err(SessionError::ImageConsentRequired);
        }
        let request_id = Uuid::new_v4();
        let (sender, cancellation) = oneshot::channel();
        self.pending_image = Some(PendingWork {
            id: request_id,
            cancellation: Some(sender),
        });
        Ok(SessionWork {
            session_id,
            request_id,
            cancellation,
        })
    }

    pub fn finish_image(&mut self, session_id: Uuid, request_id: Uuid) -> bool {
        if self.view.id != Some(session_id)
            || self.view.phase != Phase::Active
            || self
                .pending_image
                .as_ref()
                .is_none_or(|work| work.id != request_id)
        {
            return false;
        }
        self.pending_image = None;
        true
    }

    fn cancel_image(&mut self) {
        if let Some(sender) = self.pending_image.take().and_then(|work| work.cancellation) {
            let _ = sender.send(());
        }
    }

    fn require_session(&self, id: Uuid) -> Result<(), SessionError> {
        if self.view.id == Some(id) {
            Ok(())
        } else {
            Err(SessionError::StaleSession)
        }
    }

    pub fn stop(&mut self) {
        if matches!(self.view.phase, Phase::Idle | Phase::Stopped) {
            return;
        }
        self.view.phase = Phase::Stopping;
        self.cancel_fixture();
        self.cancel_image();
        self.stop_native();
        self.view.fixture = None;
        self.view.fixture_status = FixtureStatus::Ready;
        self.view.phase = Phase::Stopped;
    }

    fn cancel_fixture(&mut self) {
        if let Some(request) = self.pending.take() {
            if let Some(sender) = request.cancellation {
                let _ = sender.send(());
            }
            self.view.fixture_status = FixtureStatus::Cancelled;
        }
    }

    pub fn finish(&mut self, session_id: Uuid, request_id: Uuid, fixture: Option<Fixture>) -> bool {
        if self.view.id != Some(session_id)
            || self.view.phase != Phase::Active
            || self
                .pending
                .as_ref()
                .is_none_or(|request| request.id != request_id)
        {
            return false;
        }
        self.pending = None;
        self.view.fixture_status = if fixture.is_some() {
            FixtureStatus::Complete
        } else {
            FixtureStatus::Failed
        };
        self.view.fixture = fixture;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configure(session: &mut Session) -> Uuid {
        session
            .apply(SessionAction::Configure {
                mode: InputMode::Both,
            })
            .unwrap();
        session.snapshot().id.unwrap()
    }

    fn start(session: &mut Session, session_id: Uuid) {
        session
            .apply(SessionAction::Start {
                session_id,
                confirm_fixtures: true,
            })
            .unwrap();
    }

    #[test]
    fn lifecycle_requires_confirmation_and_rejects_stale_commands() {
        let mut session = Session::default();
        let first = configure(&mut session);
        assert_eq!(session.snapshot().phase, Phase::Configuring);
        assert_eq!(
            session
                .apply(SessionAction::Start {
                    session_id: first,
                    confirm_fixtures: false
                })
                .err(),
            Some(SessionError::FixtureConsentRequired)
        );
        start(&mut session, first);
        session
            .apply(SessionAction::Pause { session_id: first })
            .unwrap();
        assert_eq!(session.snapshot().phase, Phase::Paused);
        session
            .apply(SessionAction::Resume { session_id: first })
            .unwrap();
        session
            .apply(SessionAction::Stop { session_id: first })
            .unwrap();
        session
            .apply(SessionAction::Stop { session_id: first })
            .unwrap();
        assert_eq!(session.snapshot().phase, Phase::Stopped);
        let next = configure(&mut session);
        assert_ne!(first, next);
        assert_eq!(
            session
                .apply(SessionAction::Stop { session_id: first })
                .err(),
            Some(SessionError::StaleSession)
        );
        assert_eq!(session.snapshot().phase, Phase::Configuring);
    }

    #[test]
    fn native_sources_cancel_independently_and_gate_restart_until_cleanup() {
        let mut session = Session::default();
        let id = configure(&mut session);
        start(&mut session, id);
        assert_eq!(
            session
                .begin_native(id, InputKind::Screen, false, None)
                .err(),
            Some(SessionError::NativeConsentRequired)
        );
        assert_eq!(
            session
                .begin_native(id, InputKind::Audio, true, Some(Uuid::new_v4()))
                .err(),
            Some(SessionError::SourceUnavailable)
        );
        let choice_id = Uuid::new_v4();
        session
            .store_audio_choices(
                id,
                vec![AudioChoice {
                    id: choice_id,
                    label: "Fixture".into(),
                    target: "fixture-target".into(),
                }],
            )
            .unwrap();
        let (mut screen, _) = session
            .begin_native(id, InputKind::Screen, true, None)
            .unwrap();
        let (audio, _) = session
            .begin_native(id, InputKind::Audio, true, Some(choice_id))
            .unwrap();
        assert!(session.update_native(
            id,
            screen.request_id,
            InputKind::Screen,
            NativeStatus::Receiving
        ));
        assert!(session.update_native(
            id,
            audio.request_id,
            InputKind::Audio,
            NativeStatus::Unavailable
        ));
        assert_eq!(session.snapshot().screen_status, NativeStatus::Receiving);
        session.stop();
        assert_eq!(screen.cancellation.try_recv(), Ok(()));
        assert_eq!(
            session.snapshot().screen_status,
            NativeStatus::Disconnecting
        );
        assert!(
            session
                .apply(SessionAction::Configure {
                    mode: InputMode::Screen
                })
                .is_err()
        );
        assert!(!session.update_native(
            id,
            screen.request_id,
            InputKind::Screen,
            NativeStatus::Receiving
        ));
        assert!(session.update_native(
            id,
            screen.request_id,
            InputKind::Screen,
            NativeStatus::Idle
        ));
        assert_eq!(session.snapshot().screen_status, NativeStatus::Idle);
        assert!(
            session
                .apply(SessionAction::Configure {
                    mode: InputMode::Screen
                })
                .is_ok()
        );
    }

    #[test]
    fn image_authorization_is_session_scoped_and_cancelled_on_pause() {
        let mut session = Session::default();
        let id = configure(&mut session);
        assert_eq!(
            session.begin_image(id, true).err(),
            Some(SessionError::InvalidTransition)
        );
        start(&mut session, id);
        assert_eq!(
            session.begin_image(id, false).err(),
            Some(SessionError::ImageConsentRequired)
        );
        assert_eq!(
            session.begin_image(Uuid::new_v4(), true).err(),
            Some(SessionError::StaleSession)
        );
        let mut work = session.begin_image(id, true).unwrap();
        assert!(session.begin_image(id, true).is_err());
        session
            .apply(SessionAction::Pause { session_id: id })
            .unwrap();
        assert_eq!(work.cancellation.try_recv(), Ok(()));
        assert!(!session.finish_image(id, work.request_id));
        session.stop();
        session
            .apply(SessionAction::Configure {
                mode: InputMode::Audio,
            })
            .unwrap();
        let audio_id = session.snapshot().id.unwrap();
        start(&mut session, audio_id);
        assert_eq!(
            session.begin_image(audio_id, true).err(),
            Some(SessionError::InvalidTransition)
        );
    }

    #[test]
    fn pause_and_close_cancel_pending_work_and_ignore_late_results() {
        let mut session = Session::default();
        let id = configure(&mut session);
        start(&mut session, id);
        let mut work = session
            .apply(SessionAction::RunFixture { session_id: id })
            .unwrap()
            .unwrap();
        assert!(
            session
                .apply(SessionAction::RunFixture { session_id: id })
                .is_err()
        );
        session
            .apply(SessionAction::Pause { session_id: id })
            .unwrap();
        assert_eq!(work.cancellation.try_recv(), Ok(()));
        session
            .apply(SessionAction::Resume { session_id: id })
            .unwrap();
        assert!(!session.finish(id, work.request_id, None));
        assert_eq!(session.snapshot().fixture_status, FixtureStatus::Cancelled);
        let mut work = session
            .apply(SessionAction::RunFixture { session_id: id })
            .unwrap()
            .unwrap();
        session.stop();
        assert_eq!(work.cancellation.try_recv(), Ok(()));
        assert!(!session.finish(id, work.request_id, None));
        assert_eq!(session.snapshot().fixture, None);
    }
}
