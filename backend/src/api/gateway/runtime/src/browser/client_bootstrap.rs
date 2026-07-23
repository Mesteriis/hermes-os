use bytes::Bytes;
use hermes_gateway_protocol::v1::{
    ClientBootstrapRequestV1, ClientBootstrapResponseV1, ClientModuleBootstrapV1,
    ClientModuleSettingsBootstrapV1, ClientSettingValueEntryV1, ClientSettingValueV1 as WireValue,
    ClientSettingsApplyStateV1, ClientSurfaceAvailabilityStateV1 as WireSurfaceState,
    ClientSurfaceAvailabilityV1, ClientSurfaceIdV1 as WireSurfaceId,
    ClientSystemComponentIdV1 as WireSystemComponentId,
    ClientSystemComponentStateV1 as WireSystemComponentState, ClientSystemComponentStatusV1,
    client_setting_value_v1::Value,
};
use hermes_gateway_session_contract::{
    BrowserAuthenticationAuthority, ClientBootstrapAuthority, ClientBootstrapProjectionV1,
    ClientModuleProjectionV1, ClientModuleSettingsProjectionV1,
    ClientSettingValueEntryV1 as ProjectionEntry, ClientSettingValueV1 as ProjectionValue,
    ClientSurfaceAvailabilityProjectionV1,
    ClientSurfaceAvailabilityStateV1 as ProjectionSurfaceState,
    ClientSurfaceIdV1 as ProjectionSurfaceId,
    ClientSystemComponentIdV1 as ProjectionSystemComponentId,
    ClientSystemComponentStateV1 as ProjectionSystemComponentState,
    ClientSystemComponentStatusProjectionV1,
};
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, HeaderName};
use hyper::{Method, Request, Response, StatusCode};
use prost::Message;

use crate::{GatewayHttpResponse, SharedBrowserGatewaySessionService, full_gateway_body};

const BOOTSTRAP_PATH: &str = "/hermes.gateway.v1.ClientBootstrapService/GetBootstrap";
const MAX_REQUEST_BYTES: usize = 1_024;
const CONNECT_PROTOCOL_VERSION: HeaderName = HeaderName::from_static("connect-protocol-version");
const CONNECT_ERROR_CODE: HeaderName = HeaderName::from_static("connect-error-code");

/// ConnectRPC route for authenticated, client-safe module composition.
pub struct ClientBootstrapRouter<A> {
    service: SharedBrowserGatewaySessionService<A>,
}

impl<A> Clone for ClientBootstrapRouter<A> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
        }
    }
}

impl<A> ClientBootstrapRouter<A>
where
    A: BrowserAuthenticationAuthority + ClientBootstrapAuthority,
{
    #[must_use]
    pub const fn from_shared(service: SharedBrowserGatewaySessionService<A>) -> Self {
        Self { service }
    }

    pub async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let (parts, body) = request.into_parts();
        if parts.method != Method::POST
            || parts.uri.path() != BOOTSTRAP_PATH
            || parts.uri.query().is_some()
        {
            return not_found();
        }
        if !is_protobuf(&parts.headers) {
            return invalid_argument();
        }
        let cookie = parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok());
        let Ok(body) = Limited::new(body, MAX_REQUEST_BYTES).collect().await else {
            return invalid_argument();
        };
        let body = body.to_bytes();
        if !body.is_empty() || ClientBootstrapRequestV1::decode(body).is_err() {
            return invalid_argument();
        }
        let session = match self.service.authorize_request(cookie) {
            Ok(session) => session,
            Err(_) => return unauthenticated(),
        };
        self.service
            .client_bootstrap_for(&session)
            .map(response)
            .unwrap_or_else(|_| unavailable())
    }
}

fn response(projection: ClientBootstrapProjectionV1) -> GatewayHttpResponse {
    let response = ClientBootstrapResponseV1 {
        major: 1,
        modules: projection.modules().iter().map(module).collect(),
        surfaces: projection.surfaces().iter().map(surface).collect(),
        system_status: projection
            .system_status()
            .iter()
            .map(system_status)
            .collect(),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/proto")
        .header(CACHE_CONTROL, "no-store")
        .header(CONNECT_PROTOCOL_VERSION, "1")
        .body(full_gateway_body(response.encode_to_vec()))
        .expect("Gateway Connect response is valid")
}

fn system_status(
    status: &ClientSystemComponentStatusProjectionV1,
) -> ClientSystemComponentStatusV1 {
    ClientSystemComponentStatusV1 {
        component_id: system_component_id(status.component_id()) as i32,
        state: system_component_state(status.state()) as i32,
        sanitized_reason_code: status
            .sanitized_reason_code()
            .unwrap_or_default()
            .to_owned(),
    }
}

fn system_component_id(value: ProjectionSystemComponentId) -> WireSystemComponentId {
    match value {
        ProjectionSystemComponentId::Kernel => WireSystemComponentId::Kernel,
        ProjectionSystemComponentId::ControlStore => WireSystemComponentId::ControlStore,
        ProjectionSystemComponentId::ModuleControlPlane => {
            WireSystemComponentId::ModuleControlPlane
        }
        ProjectionSystemComponentId::Gateway => WireSystemComponentId::Gateway,
        ProjectionSystemComponentId::Vault => WireSystemComponentId::Vault,
        ProjectionSystemComponentId::StorageControl => WireSystemComponentId::StorageControl,
        ProjectionSystemComponentId::Postgresql => WireSystemComponentId::Postgresql,
        ProjectionSystemComponentId::Pgbouncer => WireSystemComponentId::Pgbouncer,
        ProjectionSystemComponentId::Nats => WireSystemComponentId::Nats,
        ProjectionSystemComponentId::EventHub => WireSystemComponentId::EventHub,
        ProjectionSystemComponentId::Scheduler => WireSystemComponentId::Scheduler,
        ProjectionSystemComponentId::Clock => WireSystemComponentId::Clock,
        ProjectionSystemComponentId::Blob => WireSystemComponentId::Blob,
        ProjectionSystemComponentId::Telemetry => WireSystemComponentId::Telemetry,
        ProjectionSystemComponentId::Sse => WireSystemComponentId::Sse,
    }
}

fn system_component_state(value: ProjectionSystemComponentState) -> WireSystemComponentState {
    match value {
        ProjectionSystemComponentState::Healthy => WireSystemComponentState::Healthy,
        ProjectionSystemComponentState::Degraded => WireSystemComponentState::Degraded,
        ProjectionSystemComponentState::Unavailable => WireSystemComponentState::Unavailable,
        ProjectionSystemComponentState::NotAdmitted => WireSystemComponentState::NotAdmitted,
    }
}

fn surface(surface: &ClientSurfaceAvailabilityProjectionV1) -> ClientSurfaceAvailabilityV1 {
    ClientSurfaceAvailabilityV1 {
        surface_id: surface_id(surface.surface_id()) as i32,
        state: surface_state(surface.state()) as i32,
        sanitized_reason_code: surface
            .sanitized_reason_code()
            .unwrap_or_default()
            .to_owned(),
        supported_client_contract_major: surface.supported_client_contract_major(),
    }
}

fn surface_id(value: ProjectionSurfaceId) -> WireSurfaceId {
    match value {
        ProjectionSurfaceId::Dashboard => WireSurfaceId::Dashboard,
        ProjectionSurfaceId::Communications => WireSurfaceId::Communications,
        ProjectionSurfaceId::Review => WireSurfaceId::Review,
        ProjectionSurfaceId::Personas => WireSurfaceId::Personas,
        ProjectionSurfaceId::Knowledge => WireSurfaceId::Knowledge,
        ProjectionSurfaceId::Tasks => WireSurfaceId::Tasks,
        ProjectionSurfaceId::Calendar => WireSurfaceId::Calendar,
        ProjectionSurfaceId::Documents => WireSurfaceId::Documents,
        ProjectionSurfaceId::Settings => WireSurfaceId::Settings,
    }
}

fn surface_state(value: ProjectionSurfaceState) -> WireSurfaceState {
    match value {
        ProjectionSurfaceState::Available => WireSurfaceState::Available,
        ProjectionSurfaceState::NotAdmitted => WireSurfaceState::NotAdmitted,
        ProjectionSurfaceState::Starting => WireSurfaceState::Starting,
        ProjectionSurfaceState::Blocked => WireSurfaceState::Blocked,
        ProjectionSurfaceState::Unavailable => WireSurfaceState::Unavailable,
    }
}

fn module(module: &ClientModuleProjectionV1) -> ClientModuleBootstrapV1 {
    ClientModuleBootstrapV1 {
        registration_id: module.registration_id().to_owned(),
        module_id: module.module_id().to_owned(),
        grant_epoch: module.grant_epoch(),
        capability_ids: module.capability_ids().to_vec(),
        sections_enabled: module.sections_enabled(),
        settings: module.settings().map(settings),
    }
}

fn settings(settings: &ClientModuleSettingsProjectionV1) -> ClientModuleSettingsBootstrapV1 {
    ClientModuleSettingsBootstrapV1 {
        schema_major: settings.schema_major(),
        schema_revision: settings.schema_revision(),
        desired_revision: settings.desired_revision(),
        effective_revision: settings.effective_revision(),
        apply_state: apply_state(settings.apply_state()) as i32,
        sanitized_reason_code: settings
            .sanitized_reason_code()
            .unwrap_or_default()
            .to_owned(),
        values: settings.values().iter().map(setting).collect(),
    }
}

fn setting(entry: &ProjectionEntry) -> ClientSettingValueEntryV1 {
    ClientSettingValueEntryV1 {
        setting_id: entry.setting_id().to_owned(),
        value: Some(WireValue {
            value: Some(value(entry.value())),
        }),
        display_name: entry.display_name().to_owned(),
        editable: entry.editable(),
    }
}

fn value(value: &ProjectionValue) -> Value {
    match value {
        ProjectionValue::Boolean(value) => Value::BooleanValue(*value),
        ProjectionValue::SignedInteger(value) => Value::SignedIntegerValue(*value),
        ProjectionValue::UnsignedInteger(value) => Value::UnsignedIntegerValue(*value),
        ProjectionValue::Decimal(value) => Value::DecimalValue(value.clone()),
        ProjectionValue::String(value) => Value::StringValue(value.clone()),
        ProjectionValue::DurationMillis(value) => Value::DurationMillis(*value),
        ProjectionValue::TimestampUnixMillis(value) => Value::TimestampUnixMillis(*value),
        ProjectionValue::Enum(value) => Value::EnumValue(value.clone()),
        ProjectionValue::ResourceReference(value) => Value::ResourceReference(value.clone()),
    }
}

fn apply_state(value: &str) -> ClientSettingsApplyStateV1 {
    match value {
        "current" => ClientSettingsApplyStateV1::Current,
        "pending_validation" => ClientSettingsApplyStateV1::PendingValidation,
        "pending_apply" => ClientSettingsApplyStateV1::PendingApply,
        "applying" => ClientSettingsApplyStateV1::Applying,
        "awaiting_external_restart" => ClientSettingsApplyStateV1::AwaitingExternalRestart,
        "blocked_config" => ClientSettingsApplyStateV1::BlockedConfig,
        _ => ClientSettingsApplyStateV1::Unspecified,
    }
}

fn is_protobuf(headers: &hyper::HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| {
            matches!(
                value.trim(),
                "application/proto" | "application/connect+proto"
            )
        })
}

fn invalid_argument() -> GatewayHttpResponse {
    connect_error(StatusCode::BAD_REQUEST, "invalid_argument")
}
fn unauthenticated() -> GatewayHttpResponse {
    connect_error(StatusCode::UNAUTHORIZED, "unauthenticated")
}
fn unavailable() -> GatewayHttpResponse {
    connect_error(StatusCode::SERVICE_UNAVAILABLE, "unavailable")
}
fn not_found() -> GatewayHttpResponse {
    connect_error(StatusCode::NOT_FOUND, "unimplemented")
}
fn connect_error(status: StatusCode, code: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header(CACHE_CONTROL, "no-store")
        .header(CONNECT_PROTOCOL_VERSION, "1")
        .header(CONNECT_ERROR_CODE, code)
        .body(full_gateway_body(Bytes::new()))
        .expect("Gateway Connect error is valid")
}
