//! Pure session state-machine contracts and checkpoint/snapshot models.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    New,
    Active,
    Paused,
    Completed,
}
impl Default for SessionState {
    fn default() -> Self {
        Self::New
    }
}

pub trait Session {
    fn state(&self) -> SessionState;
    fn transition(&mut self, next: SessionState) -> Result<(), SessionTransitionError>;
    fn session_id(&self) -> &str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionTransitionError {
    pub from: SessionState,
    pub to: SessionState,
}
impl std::fmt::Display for SessionTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid session transition: {:?} -> {:?}",
            self.from, self.to
        )
    }
}
impl std::error::Error for SessionTransitionError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checkpoint {
    pub schema: String,
    pub name: String,
    pub note: Option<String>,
    pub token_estimate: u64,
    pub message_count: usize,
    pub at_ms: i64,
    #[serde(skip_serializing, default)]
    pub entry_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: String,
    pub state: SessionState,
    pub message_count: usize,
    pub leaf_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestSession {
        id: String,
        state: SessionState,
    }
    impl Session for TestSession {
        fn state(&self) -> SessionState {
            self.state
        }
        fn transition(&mut self, next: SessionState) -> Result<(), SessionTransitionError> {
            let valid = matches!(
                (self.state, next),
                (SessionState::New, SessionState::Active)
                    | (
                        SessionState::Active,
                        SessionState::Paused | SessionState::Completed
                    )
                    | (
                        SessionState::Paused,
                        SessionState::Active | SessionState::Completed
                    )
            );
            if valid {
                self.state = next;
                Ok(())
            } else {
                Err(SessionTransitionError {
                    from: self.state,
                    to: next,
                })
            }
        }
        fn session_id(&self) -> &str {
            &self.id
        }
    }
    #[test]
    fn lifecycle_rejects_invalid_transitions() {
        let mut s = TestSession {
            id: "s".into(),
            state: SessionState::New,
        };
        assert!(s.transition(SessionState::Completed).is_err());
        s.transition(SessionState::Active).unwrap();
        s.transition(SessionState::Paused).unwrap();
        assert_eq!(s.state(), SessionState::Paused);
    }
    #[test]
    fn snapshot_round_trips() {
        let x = SessionSnapshot {
            session_id: "s".into(),
            state: SessionState::Active,
            message_count: 2,
            leaf_id: None,
        };
        assert_eq!(
            serde_json::from_str::<SessionSnapshot>(&serde_json::to_string(&x).unwrap()).unwrap(),
            x
        );
    }
}
