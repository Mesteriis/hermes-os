//! Container lifecycle primitives for Hermes integration-test sessions.
//!
//! This crate deliberately has no dependency on application or domain crates.
//! It can start PostgreSQL and NATS and expose their session-scoped endpoints;
//! `hermes-backend-testkit` owns migrations and domain-specific fixtures.

pub mod containers;
