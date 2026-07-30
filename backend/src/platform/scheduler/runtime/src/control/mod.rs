//! Authenticated inherited Kernel control surface for Scheduler.

mod clock;
mod framing;
mod handshake;
mod runtime;
mod schedule_control;
mod schedules;
mod transient_retry;
mod vault_route;
mod workers;

pub(crate) use runtime::serve_inherited;
