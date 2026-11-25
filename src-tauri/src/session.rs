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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SessionError {
    InvalidTransition,
    StaleSession,
    FixtureConsentRequired,
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

pub struct FixtureWork {
    pub session_id: Uuid,
    pub request_id: Uuid,
    pub cancellation: oneshot::Receiver<()>,
}

struct PendingFixture {
    id: Uuid,
    cancellation: oneshot::Sender<()>,
}

#[derive(Default)]
pub struct Session {
    view: SessionSnapshot,
    pending: Option<PendingFixture>,
}

impl Session {
    pub fn snapshot(&self) -> SessionSnapshot {
        self.view.clone()
    }

    pub fn apply(&mut self, action: SessionAction) -> Result<Option<FixtureWork>, SessionError> {
        match action {
            SessionAction::Configure { mode }
                if matches!(self.view.phase, Phase::Idle | Phase::Stopped) =>
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
                self.pending = Some(PendingFixture {
                    id: request_id,
                    cancellation: sender,
                });
                self.view.fixture = None;
                self.view.fixture_status = FixtureStatus::Processing;
                return Ok(Some(FixtureWork {
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
        self.view.fixture = None;
        self.view.fixture_status = FixtureStatus::Ready;
        self.view.phase = Phase::Stopped;
    }

    fn cancel_fixture(&mut self) {
        if let Some(request) = self.pending.take() {
            let _ = request.cancellation.send(());
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
