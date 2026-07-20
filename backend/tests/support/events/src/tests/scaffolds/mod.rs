//! Test-only owner delivery scaffolds; they contain no domain behaviour.

mod catalog;
mod owners;
mod postgres;

pub(crate) use catalog::OwnerDeliveryScaffoldV1;
