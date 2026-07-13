use std::future::Future;
use std::pin::Pin;

use hermes_events_api::EventEnvelope;
use thiserror::Error;

use crate::policies::SignalPolicy;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawSignalInput {
    pub event: EventEnvelope,
}

impl RawSignalInput {
    pub fn new(event: EventEnvelope) -> Self {
        Self { event }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RawSignalOutcome {
    Accepted { event_id: String },
    Rejected { reason: String },
    Muted { reason: String },
    Paused { reason: String },
}

#[derive(Debug, Error)]
#[error("signal hub raw-signal port error: {0}")]
pub struct RawSignalPortError(pub String);

impl RawSignalPortError {
    pub fn new(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

pub type RawSignalPortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RawSignalPortError>> + Send + 'a>>;

pub trait RawSignalCommandPort: Send + Sync {
    fn process_raw_signal<'a>(
        &'a self,
        input: &'a RawSignalInput,
    ) -> RawSignalPortFuture<'a, RawSignalOutcome>;
}

pub trait RawSignalRuntimeQueryPort: Send + Sync {
    fn allows_processing<'a>(&'a self) -> RawSignalPortFuture<'a, bool>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RawSignalPersistenceErrorKind {
    Storage,
    Serialization,
    InvalidConnectionId,
    InvalidPolicyScope,
    InvalidPolicyMode,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("raw signal persistence error: {message}")]
pub struct RawSignalPersistenceError {
    pub kind: RawSignalPersistenceErrorKind,
    pub message: String,
}

impl RawSignalPersistenceError {
    pub fn new(kind: RawSignalPersistenceErrorKind, error: impl std::fmt::Display) -> Self {
        Self {
            kind,
            message: error.to_string(),
        }
    }
}

pub type RawSignalPersistenceFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RawSignalPersistenceError>> + Send + 'a>>;

/// Persistence boundary required to turn a raw provider signal into a
/// canonical Signal Hub decision.
pub trait RawSignalPersistencePort: Send + Sync {
    fn resolve_connection_id<'a>(
        &'a self,
        source_code: &'a str,
        event: &'a EventEnvelope,
    ) -> RawSignalPersistenceFuture<'a, Option<String>>;

    fn list_active_policies<'a>(&'a self) -> RawSignalPersistenceFuture<'a, Vec<SignalPolicy>>;

    fn record_paused_event<'a>(
        &'a self,
        event: &'a EventEnvelope,
        source_code: &'a str,
        connection_id: Option<&'a str>,
        reason: &'a str,
    ) -> RawSignalPersistenceFuture<'a, ()>;
}
