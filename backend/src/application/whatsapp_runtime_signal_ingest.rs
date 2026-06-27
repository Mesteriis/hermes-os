use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionPort,
};
use crate::domains::signal_hub::{SignalHubError, dispatch_whatsapp_raw_signal};
use crate::integrations::whatsapp::runtime::{
    WhatsAppRuntimeEventSink, WhatsAppRuntimeEventSinkError, WhatsAppRuntimeEventSinkFuture,
    WhatsAppSanitizedRuntimeEventDto,
};
use crate::platform::communications::NewRawCommunicationRecord;

#[derive(Clone)]
pub(crate) struct WhatsappRuntimeSignalIngestService {
    pool: PgPool,
}

impl WhatsappRuntimeSignalIngestService {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn ingest_sanitized_runtime_event(
        &self,
        dto: WhatsAppSanitizedRuntimeEventDto,
    ) -> Result<WhatsappRuntimeSignalIngestResult, WhatsappRuntimeSignalIngestError> {
        dto.assert_event_spine_contract();
        let raw = native_runtime_raw_record(&dto);
        let stored_raw = CommunicationIngestionPort::new(self.pool.clone())
            .record_raw_source(&raw)
            .await?;
        let Some(accepted_event) =
            dispatch_whatsapp_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(WhatsappRuntimeSignalIngestError::SignalControlBlocked);
        };
        if accepted_event.event_type != dto.accepted_event_kind {
            return Err(
                WhatsappRuntimeSignalIngestError::AcceptedEventKindMismatch {
                    expected: dto.accepted_event_kind,
                    actual: accepted_event.event_type,
                },
            );
        }
        Ok(WhatsappRuntimeSignalIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            accepted_event_id: accepted_event.event_id,
        })
    }
}

impl WhatsAppRuntimeEventSink for WhatsappRuntimeSignalIngestService {
    fn accept<'a>(
        &'a self,
        dto: WhatsAppSanitizedRuntimeEventDto,
    ) -> WhatsAppRuntimeEventSinkFuture<'a> {
        Box::pin(async move {
            self.ingest_sanitized_runtime_event(dto)
                .await
                .map(|_| ())
                .map_err(WhatsappRuntimeSignalIngestError::into_sink_error)
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WhatsappRuntimeSignalIngestResult {
    pub(crate) raw_record_id: String,
    pub(crate) accepted_event_id: String,
}

#[derive(Debug, Error)]
pub(crate) enum WhatsappRuntimeSignalIngestError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error("whatsapp native runtime signal was blocked by Signal Hub controls")]
    SignalControlBlocked,

    #[error("accepted event kind mismatch: expected {expected}, got {actual}")]
    AcceptedEventKindMismatch {
        expected: &'static str,
        actual: String,
    },
}

impl WhatsappRuntimeSignalIngestError {
    fn into_sink_error(self) -> WhatsAppRuntimeEventSinkError {
        match self {
            Self::Communication(_) => {
                WhatsAppRuntimeEventSinkError::new("native_md_raw_record_append_failed")
            }
            Self::SignalHub(_) => {
                WhatsAppRuntimeEventSinkError::new("native_md_signal_hub_dispatch_failed")
            }
            Self::SignalControlBlocked => {
                WhatsAppRuntimeEventSinkError::new("native_md_signal_hub_control_blocked")
            }
            Self::AcceptedEventKindMismatch { .. } => {
                WhatsAppRuntimeEventSinkError::new("native_md_signal_hub_event_kind_mismatch")
            }
        }
    }
}

fn native_runtime_raw_record(dto: &WhatsAppSanitizedRuntimeEventDto) -> NewRawCommunicationRecord {
    let payload = native_runtime_raw_payload(dto);
    NewRawCommunicationRecord::new(
        native_runtime_raw_record_id(dto),
        dto.account_id.clone(),
        dto.raw_record_kind,
        dto.provider_event_id.clone(),
        native_runtime_source_fingerprint(dto),
        "whatsapp_native_md_live",
        payload,
    )
    .provenance(json!({
        "provider": dto.provider_shape,
        "provider_shape": dto.provider_shape,
        "runtime_driver": dto.runtime_driver,
        "account_id": dto.account_id,
        "provider_event_name": dto.provider_event_name,
        "event_family": dto.event_family,
        "observed_source": dto.bridge_dispatch.observed_source,
        "runtime_bridge": {
            "endpoint_path": dto.bridge_dispatch.endpoint_path,
            "request_kind": dto.bridge_dispatch.request_kind,
            "observed_source": dto.bridge_dispatch.observed_source,
        },
        "raw_signal_event_kind": dto.raw_signal_event_kind,
        "accepted_event_kind": dto.accepted_event_kind,
        "payload_policy": "sanitized_metadata_only",
        "captured_by": "application.whatsapp_runtime_signal_ingest",
    }))
}

fn native_runtime_raw_payload(dto: &WhatsAppSanitizedRuntimeEventDto) -> Value {
    let mut payload = json!({
        "provider_event_id": dto.provider_event_id,
        "provider_shape": dto.provider_shape,
        "runtime_driver": dto.runtime_driver,
        "provider_event_name": dto.provider_event_name,
        "event_family": dto.event_family,
        "runtime_event_kind": format!("native_md.{}", dto.event_family),
        "metadata": redact_secret_like_metadata(dto.metadata.clone()),
    });

    if dto.raw_record_kind == "whatsapp_web_runtime_event" {
        let (runtime_status, lifecycle_state, severity) = runtime_event_state(dto);
        payload["runtime_status"] = json!(runtime_status);
        payload["lifecycle_state"] = json!(lifecycle_state);
        payload["severity"] = json!(severity);
    }

    payload
}

fn runtime_event_state(
    dto: &WhatsAppSanitizedRuntimeEventDto,
) -> (&'static str, &'static str, &'static str) {
    if let Some(override_state) = sanitized_runtime_state_override(dto) {
        return override_state;
    }
    match dto.provider_event_name {
        "PairingQrCode" => ("qr_pending", "qr_pending", "info"),
        "PairingCode" => ("pair_code_pending", "pair_code_pending", "info"),
        "PairSuccess" => ("linked", "linked", "info"),
        "Connected" => ("available", "available", "info"),
        "HistorySync" | "OfflineSyncPreview" | "OfflineSyncCompleted" => {
            ("syncing", "syncing", "info")
        }
        "LoggedOut" => ("revoked", "revoked", "warning"),
        "Disconnected" | "StreamReplaced" | "TemporaryBan" | "ConnectFailure" | "StreamError" => {
            ("degraded", "degraded", "warning")
        }
        _ => ("degraded", "degraded", "warning"),
    }
}

fn sanitized_runtime_state_override(
    dto: &WhatsAppSanitizedRuntimeEventDto,
) -> Option<(&'static str, &'static str, &'static str)> {
    let runtime_status = dto.metadata.get("runtime_status").and_then(Value::as_str)?;
    let lifecycle_state = dto
        .metadata
        .get("lifecycle_state")
        .and_then(Value::as_str)?;
    let severity = dto.metadata.get("severity").and_then(Value::as_str)?;
    Some((
        allowed_runtime_status(runtime_status)?,
        allowed_lifecycle_state(lifecycle_state)?,
        allowed_runtime_severity(severity)?,
    ))
}

fn allowed_runtime_status(value: &str) -> Option<&'static str> {
    match value {
        "available" => Some("available"),
        "degraded" => Some("degraded"),
        "revoked" => Some("revoked"),
        "stopped" => Some("stopped"),
        "syncing" => Some("syncing"),
        "linked" => Some("linked"),
        "qr_pending" => Some("qr_pending"),
        "pair_code_pending" => Some("pair_code_pending"),
        _ => None,
    }
}

fn allowed_lifecycle_state(value: &str) -> Option<&'static str> {
    match value {
        "available" => Some("available"),
        "degraded" => Some("degraded"),
        "recovering" => Some("recovering"),
        "revoked" => Some("revoked"),
        "stopped" => Some("stopped"),
        "syncing" => Some("syncing"),
        "linked" => Some("linked"),
        "qr_pending" => Some("qr_pending"),
        "pair_code_pending" => Some("pair_code_pending"),
        _ => None,
    }
}

fn allowed_runtime_severity(value: &str) -> Option<&'static str> {
    match value {
        "info" => Some("info"),
        "warning" => Some("warning"),
        "blocked" => Some("blocked"),
        _ => None,
    }
}

fn native_runtime_raw_record_id(dto: &WhatsAppSanitizedRuntimeEventDto) -> String {
    stable_whatsapp_native_id(
        "raw:v5:whatsapp_native_md",
        &[
            dto.account_id.as_str(),
            dto.raw_record_kind,
            dto.provider_event_id.as_str(),
        ],
    )
}

fn native_runtime_source_fingerprint(dto: &WhatsAppSanitizedRuntimeEventDto) -> String {
    stable_whatsapp_native_id("sha256", &[dto.source_fingerprint_seed.as_str()])
}

fn stable_whatsapp_native_id(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.trim().as_bytes());
        hasher.update(b"\0");
    }
    format!("{prefix}:{:x}", hasher.finalize())
}

fn redact_secret_like_metadata(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    if is_secret_like_key(&key) {
                        (key, Value::String("[redacted]".to_owned()))
                    } else {
                        (key, redact_secret_like_metadata(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_secret_like_metadata).collect())
        }
        other => other,
    }
}

fn is_secret_like_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "access_token"
            | "refresh_token"
            | "session_key"
            | "session_material"
            | "authorization"
            | "cookie"
            | "token"
            | "secret"
            | "secret_key"
            | "media_key"
            | "direct_path"
            | "static_url"
            | "url"
            | "password"
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use testkit::context::TestContext;

    use super::*;
    use crate::domains::communications::core::{
        CommunicationProviderAccountStore, CommunicationProviderKind, NewProviderAccount,
    };
    use crate::integrations::whatsapp::runtime::WhatsAppRuntimeBridgeDispatch;
    use crate::platform::storage::Database;

    #[tokio::test]
    async fn sanitized_native_runtime_event_enters_raw_evidence_and_signal_hub_idempotently() {
        let test_context = TestContext::new().await;
        let database_url = test_context.connection_string();
        let database = Database::connect(Some(&database_url))
            .await
            .expect("database connection");
        let pool = database.pool().expect("configured pool").clone();
        let account_id = "whatsapp-native-sink-account";
        let provider_event_id = "native-md-provider-event-1";

        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(
                &NewProviderAccount::new(
                    account_id,
                    CommunicationProviderKind::WhatsappWeb,
                    "WhatsApp Native Sink",
                    "wa-native-sink",
                )
                .config(json!({
                    "provider_shape": "whatsapp_native_md",
                    "runtime_kind": "native_md",
                })),
            )
            .await
            .expect("provider account");

        let service = WhatsappRuntimeSignalIngestService::new(pool.clone());
        let dto = WhatsAppSanitizedRuntimeEventDto {
            account_id: account_id.to_owned(),
            provider_event_id: provider_event_id.to_owned(),
            provider_shape: "whatsapp_native_md",
            runtime_driver: "native_md_test_driver",
            provider_event_name: "PairingCode",
            event_family: "authentication",
            raw_record_kind: "whatsapp_web_runtime_event",
            raw_signal_event_kind: "signal.raw.whatsapp.runtime_event.observed",
            accepted_event_kind: "signal.accepted.whatsapp.runtime_event",
            source_fingerprint_seed:
                "source_fingerprint:v5:whatsapp_web_runtime_event:account:PairingCode:event"
                    .to_owned(),
            bridge_dispatch: WhatsAppRuntimeBridgeDispatch::new(
                "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
                "NewWhatsappWebRuntimeEvent",
                "provider_observed.runtime_bridge_runtime_event",
            ),
            metadata: json!({
                "payload_policy": "sanitized_metadata_only",
                "message_body": "excluded",
                "media_bytes": "excluded",
                "session_material": "excluded",
                "raw_provider_payload": "excluded",
                "session_key": "do-not-store",
                "nested": {
                    "refresh_token": "do-not-store"
                }
            }),
        };

        let first = service
            .ingest_sanitized_runtime_event(dto.clone())
            .await
            .expect("first runtime signal ingest");
        let second = service
            .ingest_sanitized_runtime_event(dto)
            .await
            .expect("duplicate runtime signal ingest");
        assert_eq!(first, second);

        let raw_payload: Value = sqlx::query_scalar(
            r#"
            SELECT payload
            FROM communication_raw_records
            WHERE raw_record_id = $1
            "#,
        )
        .bind(&first.raw_record_id)
        .fetch_one(&pool)
        .await
        .expect("raw payload");
        assert_eq!(raw_payload["provider_event_id"], json!(provider_event_id));
        assert_eq!(raw_payload["runtime_status"], json!("pair_code_pending"));
        assert_eq!(raw_payload["lifecycle_state"], json!("pair_code_pending"));
        assert_eq!(raw_payload["metadata"]["session_key"], json!("[redacted]"));
        assert_eq!(
            raw_payload["metadata"]["nested"]["refresh_token"],
            json!("[redacted]")
        );

        let event_counts: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT event_type, COUNT(*)::bigint
            FROM event_log
            WHERE event_type IN (
                'signal.raw.whatsapp.runtime_event.observed',
                'signal.accepted.whatsapp.runtime_event'
            )
              AND (
                    subject->>'raw_record_id' = $1
                    OR event_id = $2
                  )
            GROUP BY event_type
            ORDER BY event_type
            "#,
        )
        .bind(&first.raw_record_id)
        .bind(&first.accepted_event_id)
        .fetch_all(&pool)
        .await
        .expect("event counts");
        assert!(event_counts.iter().any(|(event_type, count)| {
            event_type == "signal.raw.whatsapp.runtime_event.observed" && *count == 1
        }));
        assert!(event_counts.iter().any(|(event_type, count)| {
            event_type == "signal.accepted.whatsapp.runtime_event" && *count == 1
        }));

        let recovered_dto = WhatsAppSanitizedRuntimeEventDto {
            account_id: account_id.to_owned(),
            provider_event_id: "native-md-recovered-event-1".to_owned(),
            provider_shape: "whatsapp_native_md",
            runtime_driver: "native_md_test_driver",
            provider_event_name: "NativeMdRuntimeRecovered",
            event_family: "runtime.lifecycle",
            raw_record_kind: "whatsapp_web_runtime_event",
            raw_signal_event_kind: "signal.raw.whatsapp.runtime_event.observed",
            accepted_event_kind: "signal.accepted.whatsapp.runtime_event",
            source_fingerprint_seed:
                "source_fingerprint:v5:whatsapp_web_runtime_event:account:recovered:event"
                    .to_owned(),
            bridge_dispatch: WhatsAppRuntimeBridgeDispatch::new(
                "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
                "NewWhatsappWebRuntimeEvent",
                "provider_observed.runtime_bridge_runtime_event",
            ),
            metadata: json!({
                "payload_policy": "sanitized_metadata_only",
                "message_body": "excluded",
                "media_bytes": "excluded",
                "session_material": "excluded",
                "raw_provider_payload": "excluded",
                "provider_event_kind": "connection.recovered",
                "runtime_event_kind": "connection.recovered",
                "runtime_status": "available",
                "lifecycle_state": "available",
                "severity": "info",
            }),
        };
        let recovered = service
            .ingest_sanitized_runtime_event(recovered_dto)
            .await
            .expect("recovered runtime signal ingest");
        let recovered_payload: Value = sqlx::query_scalar(
            r#"
            SELECT payload
            FROM communication_raw_records
            WHERE raw_record_id = $1
            "#,
        )
        .bind(&recovered.raw_record_id)
        .fetch_one(&pool)
        .await
        .expect("recovered raw payload");
        assert_eq!(recovered_payload["runtime_status"], json!("available"));
        assert_eq!(recovered_payload["lifecycle_state"], json!("available"));
        assert_eq!(recovered_payload["severity"], json!("info"));
        assert_eq!(
            recovered_payload["metadata"]["provider_event_kind"],
            json!("connection.recovered")
        );
    }
}
