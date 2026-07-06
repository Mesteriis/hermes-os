use super::*;

#[derive(Serialize)]
pub(crate) struct ApplicationSettingsResponse {
    pub(crate) items: Vec<ApplicationSetting>,
}

#[derive(Serialize)]
pub(crate) struct ApplicationAccountsResponse {
    pub(crate) items: Vec<ProviderAccount>,
}

#[derive(Deserialize)]
pub(crate) struct ApplicationAccountUpdateRequest {
    pub(crate) display_name: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ApplicationSettingUpdateRequest {
    pub(crate) value: Value,
}

#[derive(Deserialize)]
pub(crate) struct AppendEventRequest {
    pub(crate) event_id: String,
    pub(crate) event_type: String,
    #[serde(default = "default_schema_version")]
    pub(crate) schema_version: i32,
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) source: Value,
    pub(crate) actor: Option<Value>,
    pub(crate) subject: Value,
    #[serde(default = "empty_json_object")]
    pub(crate) payload: Value,
    #[serde(default = "empty_json_object")]
    pub(crate) provenance: Value,
    pub(crate) causation_id: Option<String>,
    pub(crate) correlation_id: Option<String>,
}

impl AppendEventRequest {
    pub(crate) fn into_new_event(self) -> Result<NewEventEnvelope, EventEnvelopeError> {
        let mut builder = NewEventEnvelope::builder(
            self.event_id,
            self.event_type,
            self.occurred_at,
            self.source,
            self.subject,
        )
        .schema_version(self.schema_version)
        .payload(self.payload)
        .provenance(self.provenance);

        if let Some(actor) = self.actor {
            builder = builder.actor(actor);
        }

        if let Some(causation_id) = self.causation_id {
            builder = builder.causation_id(causation_id);
        }

        if let Some(correlation_id) = self.correlation_id {
            builder = builder.correlation_id(correlation_id);
        }

        builder.build()
    }
}

#[derive(Serialize)]
pub(crate) struct AppendEventResponse {
    pub(crate) event_id: String,
    pub(crate) position: i64,
}

#[derive(Deserialize)]
pub(crate) struct AuditEventsQuery {
    pub(crate) target_id: Option<String>,
    pub(crate) actor_id: Option<String>,
    pub(crate) after_audit_id: Option<i64>,
    pub(crate) limit: Option<u32>,
}

#[derive(Serialize)]
pub(crate) struct AuditEventsResponse {
    pub(crate) items: Vec<ApiAuditRecord>,
}

#[derive(Serialize)]
pub(crate) struct V1StatusResponse {
    pub(crate) version: &'static str,
    pub(crate) surfaces: V1Surfaces,
    pub(crate) vault_status: VaultStatus,
}

#[derive(Serialize)]
pub(crate) struct V1Surfaces {
    pub(crate) messages: bool,
    pub(crate) persons: bool,
    pub(crate) search: bool,
    pub(crate) documents: bool,
    pub(crate) account_setup: bool,
}
