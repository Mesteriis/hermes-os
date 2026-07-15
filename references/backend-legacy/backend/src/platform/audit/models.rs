use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::constants::API_FRONTEND_ACTOR_KIND;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApiAuditRecord {
    pub audit_id: i64,
    pub recorded_at: DateTime<Utc>,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub operation: String,
    pub method: String,
    pub path_template: String,
    pub target_kind: String,
    pub target_id: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewApiAuditRecord {
    pub(super) actor_kind: String,
    pub(super) actor_id: String,
    pub(super) operation: String,
    pub(super) method: String,
    pub(super) path_template: String,
    pub(super) target_kind: String,
    pub(super) target_id: Option<String>,
    pub(super) metadata: Value,
}

impl NewApiAuditRecord {
    pub(super) fn new(
        actor_id: impl Into<String>,
        operation: impl Into<String>,
        method: impl Into<String>,
        path_template: impl Into<String>,
        target_kind: impl Into<String>,
        target_id: Option<String>,
        metadata: Value,
    ) -> Self {
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: operation.into(),
            method: method.into(),
            path_template: path_template.into(),
            target_kind: target_kind.into(),
            target_id,
            metadata,
        }
    }
}
