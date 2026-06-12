use std::collections::BTreeSet;

use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::mail::core::{CommunicationIngestionStore, StoredRawCommunicationRecord};
use crate::domains::mail::ingestion::analyze_ingested_message;
use crate::domains::mail::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage,
    parse_raw_email_message_from_blob, project_parsed_raw_email_message,
};
use crate::domains::mail::rfc822::{ParsedEmailAttachment, ParsedEmailAttachmentDisposition};
use crate::domains::mail::storage::{
    AttachmentSafetyScanError, AttachmentSafetyScanRequest, AttachmentSafetyScanStatus,
    AttachmentSafetyScanner, LocalMailBlobStore, MailAttachmentDisposition, MailStorageError,
    MailStorageStore, NewMailAttachment, NewMailBlob, NoopAttachmentSafetyScanner,
};
use crate::domains::mail::sync::{
    EmailSyncBatch, EmailSyncRecordError, record_email_sync_batch_with_mail_blobs,
};
use crate::domains::persons::api::{PersonProjectionError, PersonProjectionStore};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailSyncPipelineReport {
    pub imported_records: usize,
    pub raw_blobs_upserted: usize,
    pub projected_messages: usize,
    pub attachment_blobs_upserted: usize,
    pub attachments_extracted: usize,
    pub attachments_not_scanned: usize,
    pub upserted_persons: usize,
    pub upserted_person_identities: usize,
    pub upserted_message_participants: usize,
    pub upserted_relationship_events: usize,
    pub upserted_organizations: usize,
    pub upserted_organization_contact_links: usize,
    pub checkpoint_saved: bool,
}

pub async fn project_email_sync_batch_with_mail_blobs(
    pool: PgPool,
    blob_store: &LocalMailBlobStore,
    account_id: &str,
    import_batch_id: impl AsRef<str>,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncPipelineReport, EmailSyncPipelineError> {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let mail_store = MailStorageStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let person_store = PersonProjectionStore::new(pool.clone());
    let attachment_scanner = NoopAttachmentSafetyScanner;
    let import_report = record_email_sync_batch_with_mail_blobs(
        &communication_store,
        &mail_store,
        blob_store,
        account_id,
        import_batch_id.as_ref(),
        batch,
    )
    .await?;

    let projection_report = project_raw_records(
        &message_store,
        &mail_store,
        blob_store,
        &import_report.raw_records,
        &attachment_scanner,
    )
    .await?;
    let knowledge_report =
        project_message_knowledge(&pool, &person_store, &projection_report.projected_messages)
            .await?;

    Ok(EmailSyncPipelineReport {
        imported_records: import_report.inserted_or_existing_records,
        raw_blobs_upserted: import_report.blobs_upserted,
        projected_messages: projection_report.projected_messages.len(),
        attachment_blobs_upserted: projection_report.attachment_blobs_upserted,
        attachments_extracted: projection_report.attachments_extracted,
        attachments_not_scanned: projection_report.attachments_not_scanned,
        upserted_persons: knowledge_report.upserted_persons,
        upserted_person_identities: knowledge_report.upserted_person_identities,
        upserted_message_participants: knowledge_report.upserted_message_participants,
        upserted_relationship_events: knowledge_report.upserted_relationship_events,
        upserted_organizations: knowledge_report.upserted_organizations,
        upserted_organization_contact_links: knowledge_report.upserted_organization_contact_links,
        checkpoint_saved: import_report.checkpoint_saved,
    })
}

#[derive(Default)]
struct MessageKnowledgeReport {
    upserted_persons: usize,
    upserted_person_identities: usize,
    upserted_message_participants: usize,
    upserted_relationship_events: usize,
    upserted_organizations: usize,
    upserted_organization_contact_links: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EmailParticipant {
    email_address: String,
    display_name: Option<String>,
    role: &'static str,
}

async fn project_message_knowledge(
    pool: &PgPool,
    person_store: &PersonProjectionStore,
    messages: &[ProjectedMessage],
) -> Result<MessageKnowledgeReport, EmailSyncPipelineError> {
    let mut report = MessageKnowledgeReport::default();
    let mut seen_persons = BTreeSet::new();

    for message in messages {
        let mut participants = Vec::new();
        participants.push(parse_email_participant(&message.sender, "sender")?);
        for recipient in &message.recipients {
            participants.push(parse_email_participant(recipient, "recipient")?);
        }

        for participant in participants {
            let person = person_store
                .upsert_email_person(&participant.email_address)
                .await?;
            if seen_persons.insert(person.person_id.clone()) {
                report.upserted_persons += 1;
                report.upserted_person_identities += 1;
            }
            if upsert_message_participant(pool, message, &person.person_id, &participant).await? {
                report.upserted_message_participants += 1;
            }
            if insert_relationship_event(pool, message, &person.person_id, &participant).await? {
                report.upserted_relationship_events += 1;
            }
            if let Some(domain) = organization_domain_for_email(&participant.email_address) {
                let organization_id = upsert_email_domain_organization(pool, &domain).await?;
                if organization_id.is_some() {
                    report.upserted_organizations += 1;
                }
                let organization_id =
                    organization_id.unwrap_or_else(|| organization_id_for_domain(&domain));
                let _ = upsert_organization_domain(pool, &organization_id, &domain).await?;
                if upsert_organization_contact_link(pool, &organization_id, &person.person_id)
                    .await?
                {
                    report.upserted_organization_contact_links += 1;
                }
            }
        }
    }

    Ok(report)
}

fn parse_email_participant(
    raw: &str,
    role: &'static str,
) -> Result<EmailParticipant, EmailSyncPipelineError> {
    let trimmed = raw.trim();
    let (display_name, email) = if let Some((name, tail)) = trimmed.rsplit_once('<') {
        if let Some((addr, _)) = tail.split_once('>') {
            (clean_display_name(name), addr.trim())
        } else {
            (None, trimmed)
        }
    } else {
        (None, trimmed)
    };
    let email_address = email.trim_matches('"').trim().to_ascii_lowercase();
    if email_address.is_empty() || !email_address.contains('@') {
        return Err(EmailSyncPipelineError::InvalidParticipantEmail(
            raw.to_owned(),
        ));
    }
    Ok(EmailParticipant {
        email_address,
        display_name,
        role,
    })
}

fn clean_display_name(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"').trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

async fn upsert_message_participant(
    pool: &PgPool,
    message: &ProjectedMessage,
    person_id: &str,
    participant: &EmailParticipant,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO communication_message_participants (
            message_id, person_id, email_address, display_name, role, source, confidence
        )
        VALUES ($1, $2, $3, $4, $5, 'email_sync', 1.0)
        ON CONFLICT (message_id, person_id, role, email_address)
        DO UPDATE SET
            display_name = EXCLUDED.display_name,
            source = EXCLUDED.source,
            confidence = EXCLUDED.confidence,
            updated_at = now()
        RETURNING (xmax = 0) AS inserted
        "#,
    )
    .bind(&message.message_id)
    .bind(person_id)
    .bind(&participant.email_address)
    .bind(participant.display_name.as_deref())
    .bind(participant.role)
    .fetch_one(pool)
    .await?;
    row.try_get::<bool, _>("inserted")
}

async fn insert_relationship_event(
    pool: &PgPool,
    message: &ProjectedMessage,
    person_id: &str,
    participant: &EmailParticipant,
) -> Result<bool, sqlx::Error> {
    let event_type = if participant.role == "sender" {
        "email_sent"
    } else {
        "email_received"
    };
    let title = if participant.role == "sender" {
        "Sent email"
    } else {
        "Received email"
    };
    let occurred_at = message.occurred_at.unwrap_or(message.projected_at);
    let result = sqlx::query(
        r#"
        INSERT INTO relationship_events (
            person_id, event_type, title, description, occurred_at, source,
            related_entity_id, related_entity_kind, metadata
        )
        SELECT $1, $2, $3, $4, $5, 'email_sync', $6, 'communication_message', '{}'::jsonb
        WHERE NOT EXISTS (
            SELECT 1
            FROM relationship_events
            WHERE person_id = $1
              AND event_type = $2
              AND related_entity_id = $6
              AND related_entity_kind = 'communication_message'
        )
        "#,
    )
    .bind(person_id)
    .bind(event_type)
    .bind(title)
    .bind(Some(format!("Email subject: {}", message.subject)))
    .bind(occurred_at)
    .bind(&message.message_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

fn organization_domain_for_email(email_address: &str) -> Option<String> {
    let domain = email_address.split('@').nth(1)?.trim().to_ascii_lowercase();
    if domain.is_empty() || is_public_mail_domain(&domain) {
        None
    } else {
        Some(domain)
    }
}

fn is_public_mail_domain(domain: &str) -> bool {
    matches!(
        domain,
        "gmail.com"
            | "googlemail.com"
            | "icloud.com"
            | "me.com"
            | "mac.com"
            | "outlook.com"
            | "hotmail.com"
            | "live.com"
            | "yahoo.com"
            | "proton.me"
            | "protonmail.com"
            | "mail.ru"
            | "yandex.ru"
    )
}

fn organization_id_for_domain(domain: &str) -> String {
    format!("org:v1:email-domain:{}:{domain}", domain.len())
}

async fn upsert_email_domain_organization(
    pool: &PgPool,
    domain: &str,
) -> Result<Option<String>, sqlx::Error> {
    let organization_id = organization_id_for_domain(domain);
    let row = sqlx::query(
        r#"
        INSERT INTO organizations (organization_id, display_name, org_type, website)
        VALUES ($1, $2, 'company', $3)
        ON CONFLICT (organization_id)
        DO UPDATE SET
            updated_at = now(),
            last_interaction_at = now(),
            interaction_count = organizations.interaction_count + 1
        RETURNING organization_id, (xmax = 0) AS inserted
        "#,
    )
    .bind(&organization_id)
    .bind(domain)
    .bind(format!("https://{domain}"))
    .fetch_one(pool)
    .await?;
    let inserted = row.try_get::<bool, _>("inserted")?;
    Ok(if inserted {
        Some(row.try_get("organization_id")?)
    } else {
        None
    })
}

async fn upsert_organization_domain(
    pool: &PgPool,
    organization_id: &str,
    domain: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO organization_domains (organization_id, domain, domain_type, source)
        SELECT $1, $2, 'email', 'email_sync'
        WHERE NOT EXISTS (
            SELECT 1
            FROM organization_domains
            WHERE organization_id = $1
              AND domain = $2
              AND domain_type != 'former'
        )
        "#,
    )
    .bind(organization_id)
    .bind(domain)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO organization_identities (organization_id, identity_type, identity_value, source)
        VALUES ($1, 'email_domain', $2, 'email_sync')
        ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
        DO UPDATE SET organization_id = EXCLUDED.organization_id, source = EXCLUDED.source, updated_at = now()
        "#,
    )
    .bind(organization_id)
    .bind(domain)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

async fn upsert_organization_contact_link(
    pool: &PgPool,
    organization_id: &str,
    person_id: &str,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO organization_contact_links (organization_id, person_id, role, source, confidence)
        VALUES ($1, $2, 'email_participant', 'email_sync', 1.0)
        ON CONFLICT (organization_id, person_id, role)
        DO UPDATE SET
            source = EXCLUDED.source,
            confidence = EXCLUDED.confidence,
            updated_at = now()
        RETURNING (xmax = 0) AS inserted
        "#,
    )
    .bind(organization_id)
    .bind(person_id)
    .fetch_one(pool)
    .await?;
    row.try_get::<bool, _>("inserted")
}

#[derive(Default)]
struct RawRecordProjectionReport {
    projected_messages: Vec<ProjectedMessage>,
    attachment_blobs_upserted: usize,
    attachments_extracted: usize,
    attachments_not_scanned: usize,
}

async fn project_raw_records(
    message_store: &MessageProjectionStore,
    mail_store: &MailStorageStore,
    blob_store: &LocalMailBlobStore,
    raw_records: &[StoredRawCommunicationRecord],
    attachment_scanner: &impl AttachmentSafetyScanner,
) -> Result<RawRecordProjectionReport, EmailSyncPipelineError> {
    let mut report = RawRecordProjectionReport::default();
    for raw_record in raw_records {
        let parsed = parse_raw_email_message_from_blob(blob_store, raw_record).await?;
        let message = project_parsed_raw_email_message(message_store, raw_record, &parsed).await?;
        let _analysis = analyze_ingested_message(message_store, &message).await?;
        let attachment_report = project_attachments(
            mail_store,
            blob_store,
            raw_record,
            &message,
            &parsed.attachments,
            attachment_scanner,
        )
        .await?;

        report.attachment_blobs_upserted += attachment_report.attachment_blobs_upserted;
        report.attachments_extracted += attachment_report.attachments_extracted;
        report.attachments_not_scanned += attachment_report.attachments_not_scanned;
        report.projected_messages.push(message);
    }
    Ok(report)
}

#[derive(Default)]
struct AttachmentProjectionReport {
    attachment_blobs_upserted: usize,
    attachments_extracted: usize,
    attachments_not_scanned: usize,
}

async fn project_attachments(
    mail_store: &MailStorageStore,
    blob_store: &LocalMailBlobStore,
    raw_record: &StoredRawCommunicationRecord,
    message: &ProjectedMessage,
    attachments: &[ParsedEmailAttachment],
    attachment_scanner: &impl AttachmentSafetyScanner,
) -> Result<AttachmentProjectionReport, EmailSyncPipelineError> {
    let mut report = AttachmentProjectionReport::default();

    for parsed_attachment in attachments {
        let local_blob = blob_store.put_blob(&parsed_attachment.body_bytes).await?;
        let blob = mail_store
            .upsert_blob(
                &NewMailBlob::from_local_blob(&local_blob)
                    .content_type(&parsed_attachment.content_type),
            )
            .await?;
        let scan_report = attachment_scanner.scan(&AttachmentSafetyScanRequest {
            provider_attachment_id: &parsed_attachment.provider_attachment_id,
            filename: parsed_attachment.filename.as_deref(),
            content_type: &parsed_attachment.content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &blob.sha256,
            storage_kind: &blob.storage_kind,
            storage_path: &blob.storage_path,
            bytes: &parsed_attachment.body_bytes,
        })?;
        let scan_status = scan_report.status;

        let mut attachment = NewMailAttachment::new(
            &message.message_id,
            &raw_record.raw_record_id,
            &blob.blob_id,
            &parsed_attachment.provider_attachment_id,
            &parsed_attachment.content_type,
            local_blob.size_bytes,
            &blob.sha256,
        )
        .disposition(mail_attachment_disposition(parsed_attachment.disposition))
        .scan_report(scan_report);

        if let Some(filename) = &parsed_attachment.filename {
            attachment = attachment.filename(filename);
        }

        mail_store.upsert_attachment(&attachment).await?;
        report.attachment_blobs_upserted += 1;
        report.attachments_extracted += 1;
        if scan_status == AttachmentSafetyScanStatus::NotScanned {
            report.attachments_not_scanned += 1;
        }
    }

    Ok(report)
}

fn mail_attachment_disposition(
    disposition: ParsedEmailAttachmentDisposition,
) -> MailAttachmentDisposition {
    match disposition {
        ParsedEmailAttachmentDisposition::Attachment => MailAttachmentDisposition::Attachment,
        ParsedEmailAttachmentDisposition::Inline => MailAttachmentDisposition::Inline,
        ParsedEmailAttachmentDisposition::Unknown => MailAttachmentDisposition::Unknown,
    }
}

#[derive(Debug, Error)]
pub enum EmailSyncPipelineError {
    #[error(transparent)]
    Sync(#[from] EmailSyncRecordError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    Contact(#[from] PersonProjectionError),

    #[error(transparent)]
    MailStorage(#[from] MailStorageError),

    #[error(transparent)]
    AttachmentScan(#[from] AttachmentSafetyScanError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid email participant address: {0}")]
    InvalidParticipantEmail(String),
}
