//! Session Actor
//!
//! Manages user sessions using the Actor Model.
//! Each session is an independent actor with its own state.

use actix::prelude::*;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::models::User;

// ─────────────────────────────────────────────────────────────────────────────
// Session Actor Messages
// ─────────────────────────────────────────────────────────────────────────────

/// Create a new session
#[derive(Message)]
#[rtype(result = "Result<Session, SessionError>")]
pub struct CreateSession {
    pub user: User,
    pub ttl_seconds: Option<i64>,
}

/// Get session by ID
#[derive(Message)]
#[rtype(result = "Option<Session>")]
pub struct GetSession {
    pub session_id: Uuid,
}

/// Refresh session TTL
#[derive(Message)]
#[rtype(result = "Result<Session, SessionError>")]
pub struct RefreshSession {
    pub session_id: Uuid,
}

/// Destroy session
#[derive(Message)]
#[rtype(result = "bool")]
pub struct DestroySession {
    pub session_id: Uuid,
}

/// Clean up expired sessions
#[derive(Message)]
#[rtype(result = "usize")]
pub struct CleanupExpired;

// ─────────────────────────────────────────────────────────────────────────────
// Session Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user: User,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    #[error("Session expired")]
    Expired,
    #[error("Session creation failed: {0}")]
    CreationFailed(String),
}

impl Session {
    pub fn new(user: User, ttl_seconds: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id: user.id,
            user,
            created_at: now,
            expires_at: now + Duration::seconds(ttl_seconds),
            last_accessed: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn refresh(&mut self, ttl_seconds: i64) {
        let now = Utc::now();
        self.last_accessed = now;
        self.expires_at = now + Duration::seconds(ttl_seconds);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Session Actor
// ─────────────────────────────────────────────────────────────────────────────

/// Individual session actor - one per active session
pub struct SessionActor {
    session: Session,
}

impl SessionActor {
    pub fn new(session: Session) -> Self {
        Self { session }
    }
}

impl Actor for SessionActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::debug!("SessionActor started for user: {}", self.session.user_id);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::debug!("SessionActor stopped for user: {}", self.session.user_id);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Session Manager Actor
// ─────────────────────────────────────────────────────────────────────────────

/// Session manager - maintains all active sessions
pub struct SessionManager {
    sessions: HashMap<Uuid, Session>,
    default_ttl: i64,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            default_ttl: 3600 * 24, // 24 hours
        }
    }
}

impl Actor for SessionManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::info!("SessionManager started");
        
        // Schedule periodic cleanup of expired sessions
        ctx.run_interval(std::time::Duration::from_secs(300), |act, _ctx| {
            let expired: Vec<Uuid> = act
                .sessions
                .iter()
                .filter(|(_, s)| s.is_expired())
                .map(|(id, _)| *id)
                .collect();

            for id in &expired {
                act.sessions.remove(id);
            }

            if !expired.is_empty() {
                tracing::info!("Cleaned up {} expired sessions", expired.len());
            }
        });
    }
}

impl Supervised for SessionManager {
    fn restarting(&mut self, _ctx: &mut Self::Context) {
        tracing::warn!("SessionManager restarting due to failure");
    }
}

impl SystemService for SessionManager {}

// ─────────────────────────────────────────────────────────────────────────────
// Message Handlers
// ─────────────────────────────────────────────────────────────────────────────

impl Handler<CreateSession> for SessionManager {
    type Result = Result<Session, SessionError>;

    fn handle(&mut self, msg: CreateSession, _ctx: &mut Self::Context) -> Self::Result {
        let ttl = msg.ttl_seconds.unwrap_or(self.default_ttl);
        let session = Session::new(msg.user, ttl);
        
        tracing::debug!("Created session {} for user {}", session.id, session.user_id);
        
        self.sessions.insert(session.id, session.clone());
        Ok(session)
    }
}

impl Handler<GetSession> for SessionManager {
    type Result = Option<Session>;

    fn handle(&mut self, msg: GetSession, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions
            .get(&msg.session_id)
            .filter(|s| !s.is_expired())
            .cloned()
    }
}

impl Handler<RefreshSession> for SessionManager {
    type Result = Result<Session, SessionError>;

    fn handle(&mut self, msg: RefreshSession, _ctx: &mut Self::Context) -> Self::Result {
        match self.sessions.get_mut(&msg.session_id) {
            Some(session) if !session.is_expired() => {
                session.refresh(self.default_ttl);
                Ok(session.clone())
            }
            Some(_) => Err(SessionError::Expired),
            None => Err(SessionError::NotFound),
        }
    }
}

impl Handler<DestroySession> for SessionManager {
    type Result = bool;

    fn handle(&mut self, msg: DestroySession, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.session_id).is_some()
    }
}

impl Handler<CleanupExpired> for SessionManager {
    type Result = usize;

    fn handle(&mut self, _msg: CleanupExpired, _ctx: &mut Self::Context) -> Self::Result {
        let before = self.sessions.len();
        self.sessions.retain(|_, s| !s.is_expired());
        before - self.sessions.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Methods
// ─────────────────────────────────────────────────────────────────────────────

impl SessionManager {
    /// Start the session manager as a system service
    pub fn start() -> Addr<Self> {
        Self::from_registry()
    }

    /// Get session manager address
    pub fn get() -> Addr<Self> {
        Self::from_registry()
    }
}
