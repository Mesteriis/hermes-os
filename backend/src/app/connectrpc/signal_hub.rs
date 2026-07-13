use std::sync::Arc;

use chrono::{DateTime, Utc};
use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, Router as ConnectRouter, ServiceRequest,
    ServiceResult,
};
use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::app::signal_hub_support::run_signal_hub_health_check;
use crate::application::signal_hub_replay::SignalHubReplayService;
use crate::domains::signal_hub::capabilities::SignalHubCapabilityService;
use crate::domains::signal_hub::connections::SignalHubConnectionService;
use crate::domains::signal_hub::controls::{SignalHubControlRequest, SignalHubControlService};
use crate::domains::signal_hub::fixture_source::{
    SignalFixtureEmitRequest, SignalFixtureSource, SignalFixtureSourceService,
};
use crate::domains::signal_hub::health::SignalHubHealthService;
use crate::domains::signal_hub::profiles::SignalHubProfileService;
use crate::domains::signal_hub::store::{
    SignalCapability, SignalConnection, SignalConnectionCreate, SignalConnectionUpdate,
    SignalHealth, SignalHealthCheckRequest as DomainSignalHealthCheckRequest, SignalHubError,
    SignalHubStore, SignalProfileCreate, SignalProfilePolicy, SignalProfileSummary,
    SignalProfileUpdate, SignalReplayRequest, SignalReplayRequestCreate, SignalRuntimeState,
    SignalRuntimeStateUpdate, SignalSource,
};
use crate::platform::config::AppConfig;
use crate::platform::settings::ApplicationSettingsStore;
use hermes_connectrpc_contracts::hermes::signal_hub::v1::{
    ApplyProfileRequest, ApplyProfileResponse, CreateConnectionRequest, CreateConnectionResponse,
    CreatePolicyRequest, CreatePolicyResponse, CreateProfileRequest, CreateProfileResponse,
    DisableSignalsRequest, DisableSignalsResponse, DisableSourceRequest, DisableSourceResponse,
    EmitFixtureSignalRequest, EmitFixtureSignalResponse, EnableSignalsRequest,
    EnableSignalsResponse, EnableSourceRequest, EnableSourceResponse, GetSourceRequest,
    GetSourceResponse, ListCapabilitiesRequest, ListCapabilitiesResponse, ListConnectionsRequest,
    ListConnectionsResponse, ListFixtureSourcesRequest, ListFixtureSourcesResponse,
    ListHealthRequest, ListHealthResponse, ListPoliciesRequest, ListPoliciesResponse,
    ListProfilesRequest, ListProfilesResponse, ListReplayRequestsRequest,
    ListReplayRequestsResponse, ListRuntimeStatesRequest, ListRuntimeStatesResponse,
    ListSourcesRequest, ListSourcesResponse, MuteSignalsRequest, MuteSignalsResponse,
    PauseSignalsRequest, PauseSignalsResponse, RemoveConnectionRequest, RemoveConnectionResponse,
    RemoveProfileRequest, RemoveProfileResponse, RequestReplayRequest, RequestReplayResponse,
    RestoreSystemFixtureRequest, RestoreSystemFixtureResponse, ResumeSignalsRequest,
    ResumeSignalsResponse, RunHealthCheckRequest, RunHealthCheckResponse,
    SignalCapability as ProtoSignalCapability, SignalConnection as ProtoSignalConnection,
    SignalFixtureSource as ProtoSignalFixtureSource, SignalHealth as ProtoSignalHealth,
    SignalHubService, SignalHubServiceExt, SignalPolicy as ProtoSignalPolicy,
    SignalProfile as ProtoSignalProfile, SignalProfilePolicy as ProtoSignalProfilePolicy,
    SignalReplayRequest as ProtoSignalReplayRequest, SignalRuntimeState as ProtoSignalRuntimeState,
    SignalSource as ProtoSignalSource, UnmuteSignalsRequest, UnmuteSignalsResponse,
    UpdateConnectionRequest, UpdateConnectionResponse, UpdateProfileRequest, UpdateProfileResponse,
    UpdateRuntimeStateRequest, UpdateRuntimeStateResponse,
};
use hermes_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use hermes_signal_hub_postgres::raw_signals::adapter::RawSignalStore;

pub(crate) fn register(
    router: ConnectRouter,
    pool: Option<PgPool>,
    config: AppConfig,
) -> ConnectRouter {
    let Some(pool) = pool else {
        return router;
    };

    Arc::new(SignalHubConnectService::new(pool, config)).register(router)
}

struct SignalHubConnectService {
    config: AppConfig,
    capability_service: SignalHubCapabilityService,
    fixture_service: SignalFixtureSourceService,
    connection_service: SignalHubConnectionService,
    control_service: SignalHubControlService,
    profile_service: SignalHubProfileService,
    replay_service: SignalHubReplayService,
    store: SignalHubStore,
}

impl SignalHubConnectService {
    fn new(pool: PgPool, config: AppConfig) -> Self {
        let store = SignalHubStore::new(pool.clone());
        Self {
            config,
            capability_service: SignalHubCapabilityService::new(store.clone()),
            fixture_service: SignalFixtureSourceService::new(
                store.clone(),
                hermes_events_postgres::store::EventStore::new(pool.clone()),
            ),
            connection_service: SignalHubConnectionService::new(
                store.clone(),
                hermes_events_postgres::store::EventStore::new(pool.clone()),
            ),
            control_service: SignalHubControlService::new(
                store.clone(),
                hermes_events_postgres::store::EventStore::new(pool.clone()),
            ),
            profile_service: SignalHubProfileService::new(
                store.clone(),
                ApplicationSettingsStore::new(pool.clone()),
                hermes_events_postgres::store::EventStore::new(pool.clone()),
            ),
            replay_service: SignalHubReplayService::new(
                store.clone(),
                hermes_events_postgres::store::EventStore::new(pool),
            ),
            store,
        }
    }
}

#[allow(refining_impl_trait)]
impl SignalHubService for SignalHubConnectService {
    async fn list_sources(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListSourcesRequest>,
    ) -> ServiceResult<ListSourcesResponse> {
        let items = self
            .store
            .list_sources()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListSourcesResponse {
            items: items.into_iter().map(proto_source).collect(),
            ..Default::default()
        })
    }

    async fn get_source(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetSourceRequest>,
    ) -> ServiceResult<GetSourceResponse> {
        let item = self
            .store
            .get_source(req.code)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(GetSourceResponse {
            item: Some(proto_source(item)).into(),
            ..Default::default()
        })
    }

    async fn list_capabilities(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListCapabilitiesRequest>,
    ) -> ServiceResult<ListCapabilitiesResponse> {
        let items = self
            .capability_service
            .list_capabilities(req.source_code, req.connection_id)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListCapabilitiesResponse {
            items: items.into_iter().map(proto_capability).collect(),
            ..Default::default()
        })
    }

    async fn list_fixture_sources(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListFixtureSourcesRequest>,
    ) -> ServiceResult<ListFixtureSourcesResponse> {
        let items = self
            .fixture_service
            .list_fixture_sources()
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListFixtureSourcesResponse {
            items: items.into_iter().map(proto_fixture_source).collect(),
            ..Default::default()
        })
    }

    async fn list_connections(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListConnectionsRequest>,
    ) -> ServiceResult<ListConnectionsResponse> {
        let items = self
            .store
            .list_connections()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListConnectionsResponse {
            items: items.into_iter().map(proto_connection).collect(),
            ..Default::default()
        })
    }

    async fn list_profiles(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListProfilesRequest>,
    ) -> ServiceResult<ListProfilesResponse> {
        let items = self
            .profile_service
            .list_profiles()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListProfilesResponse {
            items: items.into_iter().map(proto_profile).collect(),
            ..Default::default()
        })
    }

    async fn apply_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ApplyProfileRequest>,
    ) -> ServiceResult<ApplyProfileResponse> {
        let item = self
            .profile_service
            .apply_profile(req.code)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ApplyProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn create_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateProfileRequest>,
    ) -> ServiceResult<CreateProfileResponse> {
        let item = self
            .profile_service
            .create_profile(&SignalProfileCreate {
                code: req.code.to_owned(),
                display_name: req.display_name.to_owned(),
                description: req.description.to_owned(),
                source_policies: req
                    .source_policies
                    .iter()
                    .map(|policy| {
                        Ok(SignalProfilePolicy {
                            scope: SignalPolicyScope::parse(policy.scope).ok_or_else(|| {
                                SignalHubError::InvalidPolicyScope(policy.scope.to_string())
                            })?,
                            source_code: policy.source_code.map(str::to_owned),
                            connection_id: policy.connection_id.map(str::to_owned),
                            event_pattern: policy.event_pattern.map(str::to_owned),
                            mode: SignalPolicyMode::parse(policy.mode).ok_or_else(|| {
                                SignalHubError::InvalidPolicyMode(policy.mode.to_string())
                            })?,
                            reason: policy.reason.to_owned(),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(signal_hub_connect_error)?,
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(CreateProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn update_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateProfileRequest>,
    ) -> ServiceResult<UpdateProfileResponse> {
        let source_policies = if req.update_source_policies {
            Some(
                req.source_policies
                    .iter()
                    .map(|policy| {
                        Ok(SignalProfilePolicy {
                            scope: SignalPolicyScope::parse(policy.scope).ok_or_else(|| {
                                SignalHubError::InvalidPolicyScope(policy.scope.to_string())
                            })?,
                            source_code: policy.source_code.map(str::to_owned),
                            connection_id: policy.connection_id.map(str::to_owned),
                            event_pattern: policy.event_pattern.map(str::to_owned),
                            mode: SignalPolicyMode::parse(policy.mode).ok_or_else(|| {
                                SignalHubError::InvalidPolicyMode(policy.mode.to_string())
                            })?,
                            reason: policy.reason.to_owned(),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(signal_hub_connect_error)?,
            )
        } else {
            None
        };

        let item = self
            .profile_service
            .update_profile(&SignalProfileUpdate {
                code: req.code.to_owned(),
                display_name: req.display_name.map(ToOwned::to_owned),
                description: req.description.map(ToOwned::to_owned),
                source_policies,
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(UpdateProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn remove_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RemoveProfileRequest>,
    ) -> ServiceResult<RemoveProfileResponse> {
        let item = self
            .profile_service
            .remove_profile(req.code)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(RemoveProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn create_connection(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateConnectionRequest>,
    ) -> ServiceResult<CreateConnectionResponse> {
        let item = self
            .connection_service
            .create_connection(&SignalConnectionCreate {
                source_code: req.source_code.to_owned(),
                display_name: req.display_name.to_owned(),
                status: req.status.to_owned(),
                profile: req.profile.map(ToOwned::to_owned),
                settings: parse_metadata_json(req.settings_json).map_err(invalid_argument_error)?,
                secret_ref: req.secret_ref.map(ToOwned::to_owned),
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(CreateConnectionResponse {
            item: Some(proto_connection(item)).into(),
            ..Default::default()
        })
    }

    async fn update_connection(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateConnectionRequest>,
    ) -> ServiceResult<UpdateConnectionResponse> {
        let item = self
            .connection_service
            .update_connection(&SignalConnectionUpdate {
                id: req.id.to_owned(),
                display_name: req.display_name.map(ToOwned::to_owned),
                status: req.status.map(ToOwned::to_owned),
                profile: req.profile.map(ToOwned::to_owned),
                settings: req
                    .settings_json
                    .map(parse_metadata_json)
                    .transpose()
                    .map_err(invalid_argument_error)?,
                secret_ref: req.secret_ref.map(ToOwned::to_owned),
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(UpdateConnectionResponse {
            item: Some(proto_connection(item)).into(),
            ..Default::default()
        })
    }

    async fn remove_connection(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RemoveConnectionRequest>,
    ) -> ServiceResult<RemoveConnectionResponse> {
        let item = self
            .connection_service
            .remove_connection(req.id)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(RemoveConnectionResponse {
            item: Some(proto_connection(item)).into(),
            ..Default::default()
        })
    }

    async fn enable_source(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, EnableSourceRequest>,
    ) -> ServiceResult<EnableSourceResponse> {
        let item = self
            .control_service
            .enable_source(req.source_code, req.reason)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(EnableSourceResponse {
            source_code: item.source_code.unwrap_or(req.source_code.to_owned()),
            cleared_count: u32::try_from(item.cleared_count).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn disable_source(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DisableSourceRequest>,
    ) -> ServiceResult<DisableSourceResponse> {
        let item = self
            .control_service
            .disable_source(req.source_code, req.reason)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(DisableSourceResponse {
            source_code: item.source_code.unwrap_or(req.source_code.to_owned()),
            policy_id: item.policy_id.unwrap_or_default(),
            ..Default::default()
        })
    }

    async fn disable_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, DisableSignalsRequest>,
    ) -> ServiceResult<DisableSignalsResponse> {
        let result = self
            .control_service
            .disable_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(DisableSignalsResponse {
            policy_id: result.policy_id,
            ..Default::default()
        })
    }

    async fn enable_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, EnableSignalsRequest>,
    ) -> ServiceResult<EnableSignalsResponse> {
        let result = self
            .control_service
            .enable_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(EnableSignalsResponse {
            cleared_count: u32::try_from(result.cleared_count).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn mute_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, MuteSignalsRequest>,
    ) -> ServiceResult<MuteSignalsResponse> {
        let item = self
            .control_service
            .mute_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(MuteSignalsResponse {
            policy_id: item.policy_id,
            ..Default::default()
        })
    }

    async fn unmute_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UnmuteSignalsRequest>,
    ) -> ServiceResult<UnmuteSignalsResponse> {
        let item = self
            .control_service
            .unmute_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(UnmuteSignalsResponse {
            cleared_count: u32::try_from(item.cleared_count).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn pause_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, PauseSignalsRequest>,
    ) -> ServiceResult<PauseSignalsResponse> {
        let item = self
            .control_service
            .pause_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(PauseSignalsResponse {
            policy_id: item.policy_id,
            ..Default::default()
        })
    }

    async fn resume_signals(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ResumeSignalsRequest>,
    ) -> ServiceResult<ResumeSignalsResponse> {
        let item = self
            .control_service
            .resume_signals(&control_request(
                req.scope,
                req.source_code,
                req.connection_id,
                req.event_pattern,
                req.reason,
            )?)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ResumeSignalsResponse {
            cleared_count: u32::try_from(item.cleared_count).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn list_health(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListHealthRequest>,
    ) -> ServiceResult<ListHealthResponse> {
        let items = self
            .store
            .list_health()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListHealthResponse {
            items: items.into_iter().map(proto_health).collect(),
            ..Default::default()
        })
    }

    async fn run_health_check(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RunHealthCheckRequest>,
    ) -> ServiceResult<RunHealthCheckResponse> {
        let item = run_signal_hub_health_check(
            &self.config,
            self.store.pool().clone(),
            &DomainSignalHealthCheckRequest {
                source_code: req.source_code.to_owned(),
                connection_id: req.connection_id.map(ToOwned::to_owned),
                runtime_kind: None,
            },
        )
        .await
        .map_err(signal_hub_connect_error)?;
        Response::ok(RunHealthCheckResponse {
            item: Some(proto_health(item)).into(),
            ..Default::default()
        })
    }

    async fn list_runtime_states(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListRuntimeStatesRequest>,
    ) -> ServiceResult<ListRuntimeStatesResponse> {
        let items = self
            .store
            .list_runtime_states()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListRuntimeStatesResponse {
            items: items.into_iter().map(proto_runtime_state).collect(),
            ..Default::default()
        })
    }

    async fn update_runtime_state(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateRuntimeStateRequest>,
    ) -> ServiceResult<UpdateRuntimeStateResponse> {
        let metadata = parse_metadata_json(req.metadata_json).map_err(invalid_argument_error)?;
        let item = self
            .store
            .set_runtime_state(&SignalRuntimeStateUpdate {
                source_code: req.source_code.to_owned(),
                runtime_kind: req.runtime_kind.to_owned(),
                state: req.state.to_owned(),
                metadata,
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(UpdateRuntimeStateResponse {
            item: Some(proto_runtime_state(item)).into(),
            ..Default::default()
        })
    }

    async fn restore_system_fixture(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, RestoreSystemFixtureRequest>,
    ) -> ServiceResult<RestoreSystemFixtureResponse> {
        let report = self
            .store
            .restore_system_sources()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(RestoreSystemFixtureResponse {
            sources_created: u32::try_from(report.sources_created).unwrap_or(u32::MAX),
            sources_repaired: u32::try_from(report.sources_repaired).unwrap_or(u32::MAX),
            profiles_created: u32::try_from(report.profiles_created).unwrap_or(u32::MAX),
            profiles_repaired: u32::try_from(report.profiles_repaired).unwrap_or(u32::MAX),
            ..Default::default()
        })
    }

    async fn list_policies(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListPoliciesRequest>,
    ) -> ServiceResult<ListPoliciesResponse> {
        let items = RawSignalStore::new(self.store.pool().clone())
            .list_active_policies()
            .await
            .map_err(SignalHubError::from)
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListPoliciesResponse {
            items: items.into_iter().map(proto_policy).collect(),
            ..Default::default()
        })
    }

    async fn create_policy(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreatePolicyRequest>,
    ) -> ServiceResult<CreatePolicyResponse> {
        let policy = SignalPolicy {
            scope: parse_policy_scope(req.scope).map_err(invalid_argument_error)?,
            source_code: req.source_code.map(ToOwned::to_owned),
            connection_id: req.connection_id.map(ToOwned::to_owned),
            event_pattern: req.event_pattern.map(ToOwned::to_owned),
            mode: parse_policy_mode(req.mode).map_err(invalid_argument_error)?,
            reason: non_empty_reason(req.reason),
            expires_at: parse_optional_timestamp(req.expires_at, "expires_at")
                .map_err(invalid_argument_error)?,
        };
        let id = self
            .store
            .create_policy(&policy)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(CreatePolicyResponse {
            id: id.to_string(),
            ..Default::default()
        })
    }

    async fn list_replay_requests(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListReplayRequestsRequest>,
    ) -> ServiceResult<ListReplayRequestsResponse> {
        let items = self
            .store
            .list_replay_requests()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListReplayRequestsResponse {
            items: items.into_iter().map(proto_replay_request).collect(),
            ..Default::default()
        })
    }

    async fn request_replay(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, RequestReplayRequest>,
    ) -> ServiceResult<RequestReplayResponse> {
        let metadata = parse_metadata_json(req.metadata_json).map_err(invalid_argument_error)?;
        let item = self
            .replay_service
            .request_replay(&SignalReplayRequestCreate {
                source_code: req.source_code.map(ToOwned::to_owned),
                connection_id: req.connection_id.map(ToOwned::to_owned),
                event_pattern: req.event_pattern.map(ToOwned::to_owned),
                from_position: req.from_position,
                to_position: req.to_position,
                from_time: parse_optional_timestamp(req.from_time, "from_time")
                    .map_err(invalid_argument_error)?,
                to_time: parse_optional_timestamp(req.to_time, "to_time")
                    .map_err(invalid_argument_error)?,
                target_consumer: req.target_consumer.map(ToOwned::to_owned),
                target_projection: req.target_projection.map(ToOwned::to_owned),
                requested_by: "hermes-frontend".to_owned(),
                metadata,
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(RequestReplayResponse {
            item: Some(proto_replay_request(item)).into(),
            ..Default::default()
        })
    }

    async fn emit_fixture_signal(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, EmitFixtureSignalRequest>,
    ) -> ServiceResult<EmitFixtureSignalResponse> {
        let item = self
            .fixture_service
            .emit_fixture(&SignalFixtureEmitRequest {
                fixture_id: req.fixture_id.to_owned(),
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(EmitFixtureSignalResponse {
            fixture_id: item.fixture_id,
            raw_event_id: item.raw_event_id,
            event_type: item.event_type,
            source_code: item.source_code,
            correlation_id: item.correlation_id,
            ..Default::default()
        })
    }
}

fn proto_source(source: SignalSource) -> ProtoSignalSource {
    ProtoSignalSource {
        id: source.id.to_string(),
        code: source.code,
        display_name: source.display_name,
        category: source.category,
        source_kind: source.source_kind,
        default_enabled: source.default_enabled,
        supports_connections: source.supports_connections,
        supports_runtime: source.supports_runtime,
        supports_replay: source.supports_replay,
        supports_pause: source.supports_pause,
        supports_mute: source.supports_mute,
        capability_schema_version: source.capability_schema_version,
        created_at: timestamp_to_string(source.created_at),
        updated_at: timestamp_to_string(source.updated_at),
        ..Default::default()
    }
}

fn proto_connection(connection: SignalConnection) -> ProtoSignalConnection {
    ProtoSignalConnection {
        id: connection.id.to_string(),
        source_code: connection.source_code,
        display_name: connection.display_name,
        status: connection.status,
        profile: connection.profile,
        secret_ref: connection.secret_ref,
        connected_at: connection.connected_at.map(timestamp_to_string),
        last_seen_at: connection.last_seen_at.map(timestamp_to_string),
        last_signal_at: connection.last_signal_at.map(timestamp_to_string),
        last_sync_at: connection.last_sync_at.map(timestamp_to_string),
        created_at: timestamp_to_string(connection.created_at),
        updated_at: timestamp_to_string(connection.updated_at),
        settings_json: connection.settings.to_string(),
        ..Default::default()
    }
}

fn proto_fixture_source(fixture: SignalFixtureSource) -> ProtoSignalFixtureSource {
    ProtoSignalFixtureSource {
        fixture_id: fixture.fixture_id,
        source_code: fixture.source_code,
        event_type: fixture.event_type,
        correlation_id: fixture.correlation_id,
        occurred_at: timestamp_to_string(fixture.occurred_at),
        summary: fixture.summary,
        ..Default::default()
    }
}

fn proto_profile(profile: SignalProfileSummary) -> ProtoSignalProfile {
    ProtoSignalProfile {
        id: profile.id,
        code: profile.code,
        display_name: profile.display_name,
        description: profile.description,
        policy_count: u32::try_from(profile.policy_count).unwrap_or(u32::MAX),
        is_system: profile.is_system,
        is_active: profile.is_active,
        created_at: timestamp_to_string(profile.created_at),
        updated_at: timestamp_to_string(profile.updated_at),
        source_policies: profile
            .source_policies
            .into_iter()
            .map(proto_profile_policy)
            .collect(),
        ..Default::default()
    }
}

fn proto_capability(capability: SignalCapability) -> ProtoSignalCapability {
    ProtoSignalCapability {
        id: capability.id,
        source_code: capability.source_code,
        connection_id: capability.connection_id,
        capability: capability.capability,
        state: capability.state,
        reason: capability.reason,
        requires_confirmation: capability.requires_confirmation,
        action_class: capability.action_class,
        updated_at: timestamp_to_string(capability.updated_at),
        ..Default::default()
    }
}

fn proto_profile_policy(policy: SignalProfilePolicy) -> ProtoSignalProfilePolicy {
    ProtoSignalProfilePolicy {
        scope: policy.scope.as_str().to_owned(),
        source_code: policy.source_code,
        connection_id: policy.connection_id,
        event_pattern: policy.event_pattern,
        mode: policy.mode.as_str().to_owned(),
        reason: policy.reason,
        ..Default::default()
    }
}

fn proto_health(health: SignalHealth) -> ProtoSignalHealth {
    ProtoSignalHealth {
        id: health.id.to_string(),
        source_code: health.source_code,
        connection_id: health.connection_id.map(|id| id.to_string()),
        level: health.level,
        summary: health.summary,
        last_ok_at: health.last_ok_at.map(timestamp_to_string),
        last_failure_at: health.last_failure_at.map(timestamp_to_string),
        failure_count: health.failure_count,
        consecutive_failure_count: health.consecutive_failure_count,
        next_retry_at: health.next_retry_at.map(timestamp_to_string),
        updated_at: timestamp_to_string(health.updated_at),
        evidence_json: health.evidence.to_string(),
        ..Default::default()
    }
}

fn proto_runtime_state(runtime: SignalRuntimeState) -> ProtoSignalRuntimeState {
    ProtoSignalRuntimeState {
        id: runtime.id.to_string(),
        source_code: runtime.source_code,
        connection_id: runtime.connection_id.map(|id| id.to_string()),
        runtime_kind: runtime.runtime_kind,
        state: runtime.state,
        metadata_json: runtime.metadata.to_string(),
        updated_at: timestamp_to_string(runtime.updated_at),
        last_started_at: runtime.last_started_at.map(timestamp_to_string),
        last_stopped_at: runtime.last_stopped_at.map(timestamp_to_string),
        last_heartbeat_at: runtime.last_heartbeat_at.map(timestamp_to_string),
        last_error_at: runtime.last_error_at.map(timestamp_to_string),
        last_error_code: runtime.last_error_code,
        last_error_message_redacted: runtime.last_error_message_redacted,
        ..Default::default()
    }
}

fn proto_policy(policy: SignalPolicy) -> ProtoSignalPolicy {
    ProtoSignalPolicy {
        scope: policy.scope.as_str().to_owned(),
        source_code: policy.source_code,
        connection_id: policy.connection_id,
        event_pattern: policy.event_pattern,
        mode: policy.mode.as_str().to_owned(),
        reason: policy.reason,
        expires_at: policy.expires_at.map(timestamp_to_string),
        ..Default::default()
    }
}

fn proto_replay_request(request: SignalReplayRequest) -> ProtoSignalReplayRequest {
    ProtoSignalReplayRequest {
        id: request.id,
        source_code: request.source_code,
        connection_id: request.connection_id,
        event_pattern: request.event_pattern,
        from_position: request.from_position,
        to_position: request.to_position,
        from_time: request.from_time.map(timestamp_to_string),
        to_time: request.to_time.map(timestamp_to_string),
        target_consumer: request.target_consumer,
        target_projection: request.target_projection,
        status: request.status,
        requested_by: request.requested_by,
        requested_at: timestamp_to_string(request.requested_at),
        started_at: request.started_at.map(timestamp_to_string),
        completed_at: request.completed_at.map(timestamp_to_string),
        last_error_redacted: request.last_error_redacted,
        replayed_count: request.replayed_count,
        metadata_json: request.metadata.to_string(),
        ..Default::default()
    }
}

fn timestamp_to_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

fn parse_metadata_json(value: &str) -> Result<Value, String> {
    if value.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }

    let parsed: Value = serde_json::from_str(value)
        .map_err(|error| format!("metadata_json must be valid JSON: {error}"))?;
    if !parsed.is_object() {
        return Err("metadata_json must be a JSON object".to_owned());
    }
    Ok(parsed)
}

fn parse_optional_timestamp(
    value: Option<&str>,
    field_name: &str,
) -> Result<Option<DateTime<Utc>>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    DateTime::parse_from_rfc3339(value)
        .map(|parsed| Some(parsed.with_timezone(&Utc)))
        .map_err(|error| format!("{field_name} must be a valid RFC3339 timestamp: {error}"))
}

fn parse_policy_scope(value: &str) -> Result<SignalPolicyScope, String> {
    SignalPolicyScope::parse(value.trim())
        .ok_or_else(|| format!("invalid signal policy scope: {value}"))
}

fn parse_policy_mode(value: &str) -> Result<SignalPolicyMode, String> {
    SignalPolicyMode::parse(value.trim())
        .ok_or_else(|| format!("invalid signal policy mode: {value}"))
}

fn control_request(
    scope: &str,
    source_code: Option<&str>,
    connection_id: Option<&str>,
    event_pattern: Option<&str>,
    reason: Option<&str>,
) -> Result<SignalHubControlRequest, ConnectError> {
    Ok(SignalHubControlRequest {
        scope: parse_policy_scope(scope).map_err(invalid_argument_error)?,
        source_code: source_code
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        connection_id: connection_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        event_pattern: event_pattern
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        reason: reason
            .map(non_empty_reason)
            .unwrap_or_else(|| "owner control".to_owned()),
    })
}

fn non_empty_reason(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "user policy".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn signal_hub_connect_error(error: SignalHubError) -> ConnectError {
    if error.is_invalid_request() {
        return invalid_argument_error(error.to_string());
    }
    if error.is_not_found() {
        return ConnectError::new(ErrorCode::NotFound, error.to_string());
    }
    if error.is_failed_precondition() {
        return ConnectError::new(ErrorCode::FailedPrecondition, error.to_string());
    }

    match error {
        SignalHubError::Sqlx(_)
        | SignalHubError::EventStore(_)
        | SignalHubError::Envelope(_)
        | SignalHubError::Json(_)
        | SignalHubError::Toml(_)
        | SignalHubError::Settings(_)
        | SignalHubError::RawSignalPersistence(_) => {
            ConnectError::new(ErrorCode::Internal, error.to_string())
        }
        SignalHubError::InvalidRawSignalEventType(_)
        | SignalHubError::MissingSourceCode
        | SignalHubError::InvalidPolicyScope(_)
        | SignalHubError::InvalidPolicyMode(_)
        | SignalHubError::InvalidConnectionId(_)
        | SignalHubError::InvalidConnectionStatus(_)
        | SignalHubError::SourceNotFound(_)
        | SignalHubError::SourceDoesNotSupportConnections(_)
        | SignalHubError::ConnectionNotFound(_)
        | SignalHubError::InvalidRuntimeState(_)
        | SignalHubError::InvalidRuntimeId(_)
        | SignalHubError::InvalidHealthId(_)
        | SignalHubError::InvalidReplayRequest(_)
        | SignalHubError::FixtureNotFound(_)
        | SignalHubError::InvalidFixtureCatalog(_)
        | SignalHubError::ProfileNotFound(_)
        | SignalHubError::InvalidProfileDefinition(_)
        | SignalHubError::SystemProfileImmutable(_)
        | SignalHubError::EmptyField(_) => unreachable!("classified above"),
    }
}

fn invalid_argument_error(message: impl Into<String>) -> ConnectError {
    ConnectError::new(ErrorCode::InvalidArgument, message.into())
}
