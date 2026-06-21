use super::super::*;

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct CertsQuery {
    pub(super) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct CertsListResponse {
    pub(super) items: Vec<crate::domains::communications::signatures::CertificateRecord>,
}

pub(crate) async fn get_v1_certs(
    State(state): State<AppState>,
) -> Result<Json<CertsListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::signatures::CertificateStore,
    >(pool);
    Ok(Json(CertsListResponse {
        items: store.list().await?,
    }))
}

#[derive(Deserialize)]
pub(crate) struct NewCertRequest {
    pub(super) cert_id: String,
    pub(super) owner_name: String,
    pub(super) issuer: String,
    pub(super) serial_number: Option<String>,
    pub(super) fingerprint_sha256: Option<String>,
    pub(super) valid_from: Option<DateTime<Utc>>,
    pub(super) valid_until: Option<DateTime<Utc>>,
    pub(super) cert_type: Option<String>,
    pub(super) provider: Option<String>,
    pub(super) storage_kind: Option<String>,
    pub(super) storage_ref: Option<String>,
    pub(super) trust_status: Option<String>,
    pub(super) is_revoked: Option<bool>,
    pub(super) usage: Option<Vec<String>>,
    pub(super) linked_message_id: Option<String>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn post_v1_cert(
    State(state): State<AppState>,
    Json(req): Json<NewCertRequest>,
) -> Result<Json<crate::domains::communications::signatures::CertificateRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::signatures::CertificateStore,
    >(pool);
    Ok(Json(
        store
            .upsert(&crate::domains::communications::signatures::NewCertificate {
                cert_id: req.cert_id,
                owner_name: req.owner_name,
                issuer: req.issuer,
                serial_number: req.serial_number,
                fingerprint_sha256: req.fingerprint_sha256,
                valid_from: req.valid_from,
                valid_until: req.valid_until,
                cert_type: req
                    .cert_type
                    .as_deref()
                    .and_then(crate::domains::communications::signatures::CertificateType::parse)
                    .unwrap_or(crate::domains::communications::signatures::CertificateType::Unknown),
                provider: req
                    .provider
                    .as_deref()
                    .and_then(crate::domains::communications::signatures::CertificateProvider::parse)
                    .unwrap_or(crate::domains::communications::signatures::CertificateProvider::Other),
                storage_kind: req
                    .storage_kind
                    .as_deref()
                    .and_then(crate::domains::communications::signatures::CertificateStorageKind::parse)
                    .unwrap_or(
                        crate::domains::communications::signatures::CertificateStorageKind::EncryptedVault,
                    ),
                storage_ref: req.storage_ref,
                trust_status: req
                    .trust_status
                    .as_deref()
                    .and_then(crate::domains::communications::signatures::TrustStatus::parse)
                    .unwrap_or(crate::domains::communications::signatures::TrustStatus::Untrusted),
                is_revoked: req.is_revoked.unwrap_or(false),
                usage: req.usage.unwrap_or_default(),
                linked_message_id: req.linked_message_id,
                metadata: req.metadata.unwrap_or(serde_json::json!({})),
            })
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct ExpiringQuery {
    pub(super) days: Option<i64>,
}

pub(crate) async fn get_v1_certs_expiring(
    State(state): State<AppState>,
    Query(query): Query<ExpiringQuery>,
) -> Result<Json<CertsListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::signatures::CertificateStore,
    >(pool);
    Ok(Json(CertsListResponse {
        items: store.expiring_soon(query.days.unwrap_or(90)).await?,
    }))
}

pub(crate) async fn get_v1_signature_check(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<crate::domains::communications::signatures::SignatureDetection>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::domains::communications::signatures::SignatureDetector::detect_in_message(
            &msg.body_text,
            "",
        ),
    ))
}

pub(crate) async fn get_v1_spf_dkim(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let auth = crate::domains::communications::spf_dkim::parse_auth_headers(&msg.body_text);
    let risk = crate::domains::communications::spf_dkim::assess_auth_risk(&auth);
    Ok(Json(serde_json::json!({"auth": auth, "risk": risk})))
}
