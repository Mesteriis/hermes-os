//! Bounded, process-local sessions for the private registration socket.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SESSION_TTL: Duration = Duration::from_secs(60);
const MAX_SESSIONS: usize = 32;
const MAX_BEGINS_PER_MINUTE: usize = 16;

pub struct RegistrationSessions {
    sessions: HashMap<String, Session>,
    begins: VecDeque<Instant>,
}

struct Session {
    expires_at: Instant,
    state: SessionState,
}

enum SessionState {
    Open,
    Describing,
    Registered(String),
}

impl RegistrationSessions {
    pub fn begin(&mut self) -> Result<(String, u64), String> {
        self.purge();
        let now = Instant::now();
        while self
            .begins
            .front()
            .is_some_and(|item| now.duration_since(*item) >= Duration::from_secs(60))
        {
            self.begins.pop_front();
        }
        if self.begins.len() >= MAX_BEGINS_PER_MINUTE || self.sessions.len() >= MAX_SESSIONS {
            return Err("registration_rate_limited".to_owned());
        }
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
        let id = bytes
            .iter()
            .map(|item| format!("{item:02x}"))
            .collect::<String>();
        self.begins.push_back(now);
        self.sessions.insert(
            id.clone(),
            Session {
                expires_at: now + SESSION_TTL,
                state: SessionState::Open,
            },
        );
        let expires = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis() as u64
            + SESSION_TTL.as_millis() as u64;
        Ok((id, expires))
    }
    pub fn start_describe(&mut self, session_id: &str) -> Result<(), String> {
        let session = self.session_mut(session_id)?;
        if !matches!(session.state, SessionState::Open) {
            return Err("registration_session_unavailable".to_owned());
        }
        session.state = SessionState::Describing;
        Ok(())
    }

    pub fn record(&mut self, session_id: &str, registration_id: String) -> Result<(), String> {
        let session = self.session_mut(session_id)?;
        if !matches!(session.state, SessionState::Describing) {
            return Err("registration_session_unavailable".to_owned());
        }
        session.state = SessionState::Registered(registration_id);
        Ok(())
    }
    pub fn registration_id(&mut self, session_id: &str) -> Result<String, String> {
        match &self.session_mut(session_id)?.state {
            SessionState::Registered(registration_id) => Ok(registration_id.clone()),
            SessionState::Open | SessionState::Describing => {
                Err("registration_session_unavailable".to_owned())
            }
        }
    }
    fn session_mut(&mut self, session_id: &str) -> Result<&mut Session, String> {
        self.purge();
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| "registration_session_unavailable".to_owned())
    }
    fn purge(&mut self) {
        let now = Instant::now();
        self.sessions.retain(|_, item| item.expires_at > now);
    }
}

impl Default for RegistrationSessions {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            begins: VecDeque::new(),
        }
    }
}
