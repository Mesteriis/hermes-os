//! Privacy-first public contract for bounded technical telemetry.

mod model;

pub use model::{
    TelemetryIdentityErrorV1, TelemetryPriorityV1, TelemetrySignalErrorV1, TelemetrySignalKindV1,
    TelemetrySignalV1, TelemetrySourceV1,
};
