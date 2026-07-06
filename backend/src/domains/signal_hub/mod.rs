mod ai;
mod capabilities;
mod connections;
mod controls;
mod fixture_source;
mod fixtures;
mod health;
mod mail;
mod policies;
mod profiles;
mod service;
mod store;
mod telegram;
mod whatsapp;
mod zulip;

pub use self::ai::{dispatch_ai_helper_signal, dispatch_ai_helper_signal_best_effort};
pub use self::capabilities::SignalHubCapabilityService;
pub use self::connections::SignalHubConnectionService;
pub use self::controls::{
    SignalHubControlRequest, SignalHubControlResult, SignalHubControlService,
};
pub use self::fixture_source::{
    SignalFixtureEmission, SignalFixtureEmitRequest, SignalFixtureSource,
    SignalFixtureSourceService,
};
pub use self::fixtures::{SystemSourceFixture, system_source_fixtures};
pub use self::health::SignalHubHealthService;
pub use self::mail::{
    MailDeliverySignalRequest, dispatch_mail_delivery_event_signal, dispatch_mail_raw_signal,
};
pub use self::policies::{
    SignalPolicy, SignalPolicyDecision, SignalPolicyEvaluator, SignalPolicyMode, SignalPolicyScope,
};
pub use self::profiles::SignalHubProfileService;
pub use self::service::{
    SIGNAL_HUB_RAW_SIGNAL_CONSUMER, SignalHubSignalService, SignalProcessingOutcome,
    process_signal_hub_raw_event, signal_hub_raw_dispatcher_allows_processing,
};
pub use self::store::SignalHubStore as SignalHubPort;
pub(crate) use self::store::event_type_pattern_matches;
pub use self::store::{
    FixtureRestoreReport, SignalCapability, SignalCapabilityUpsert, SignalConnection,
    SignalConnectionCreate, SignalConnectionUpdate, SignalHealth, SignalHealthCheckRequest,
    SignalHealthSnapshotWrite, SignalHubError, SignalHubStore, SignalProfile, SignalProfileCreate,
    SignalProfilePolicy, SignalProfileSummary, SignalProfileUpdate, SignalReplayRequest,
    SignalReplayRequestCreate, SignalRuntimeState, SignalRuntimeStateUpdate, SignalSource,
};
pub use self::telegram::dispatch_telegram_raw_signal;
pub use self::whatsapp::dispatch_whatsapp_raw_signal;
pub use self::zulip::dispatch_zulip_raw_signal;
