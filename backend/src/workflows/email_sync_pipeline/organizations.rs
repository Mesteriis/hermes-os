use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::domains::mail::messages::ProjectedMessage;
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewState, RelationshipStore,
};

use super::errors::EmailSyncPipelineError;
use super::participants::EmailParticipant;

#[derive(Default)]
pub(crate) struct OrganizationProjectionReport {
    pub(crate) upserted_organizations: usize,
    pub(crate) upserted_organization_contact_links: usize,
}

pub(crate) async fn project_email_participant_organization(
    pool: &PgPool,
    person_id: &str,
    message: &ProjectedMessage,
    participant: &EmailParticipant,
) -> Result<OrganizationProjectionReport, EmailSyncPipelineError> {
    let Some(domain) = organization_domain_for_email(&participant.email_address) else {
        return Ok(OrganizationProjectionReport::default());
    };

    let organization_id = upsert_email_domain_organization(pool, &domain).await?;
    let organization_inserted = organization_id.is_some();
    let organization_id = organization_id.unwrap_or_else(|| organization_id_for_domain(&domain));
    let _ = upsert_organization_domain(pool, &organization_id, &domain).await?;
    let contact_link_inserted =
        upsert_organization_contact_link(pool, &organization_id, person_id, message, participant)
            .await?;

    Ok(OrganizationProjectionReport {
        upserted_organizations: usize::from(organization_inserted),
        upserted_organization_contact_links: usize::from(contact_link_inserted),
    })
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
    message: &ProjectedMessage,
    participant: &EmailParticipant,
) -> Result<bool, EmailSyncPipelineError> {
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO organization_contact_links (organization_id, person_id, role, source, confidence)
        VALUES ($1, $2, 'email_participant', 'email_sync', 1.0)
        ON CONFLICT (organization_id, person_id, role)
        DO UPDATE SET
            source = EXCLUDED.source,
            confidence = EXCLUDED.confidence,
            updated_at = now()
        RETURNING
            id::text,
            organization_id,
            person_id,
            role,
            source,
            confidence::float8 AS confidence,
            valid_from,
            valid_to,
            (xmax = 0) AS inserted
        "#,
    )
    .bind(organization_id)
    .bind(person_id)
    .fetch_one(&mut *transaction)
    .await?;
    let link_id: String = row.try_get("id")?;
    let source: String = row.try_get("source")?;
    let role: Option<String> = row.try_get("role")?;
    let confidence: f64 = row.try_get("confidence")?;
    let valid_from = row.try_get("valid_from")?;
    let valid_to = row.try_get("valid_to")?;
    let inserted: bool = row.try_get("inserted")?;

    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: person_id.to_owned(),
        target_entity_kind: RelationshipEntityKind::Organization,
        target_entity_id: organization_id.to_owned(),
        relationship_type: "member_of".to_owned(),
        trust_score: 0.5,
        strength_score: 0.55,
        confidence,
        review_state: RelationshipReviewState::SystemAccepted,
        valid_from,
        valid_to,
        metadata: json!({
            "compatibility_table": "organization_contact_links",
            "compatibility_record_id": link_id,
            "source": source,
            "role": role,
            "participant_role": participant.role,
            "participant_email": participant.email_address
        }),
    };
    let evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        message.message_id.clone(),
    )
    .excerpt(format!(
        "Email participant {} was associated with an organization domain.",
        participant.email_address
    ))
    .metadata(json!({
        "compatibility_table": "organization_contact_links",
        "compatibility_record_id": link_id,
        "organization_id": organization_id,
        "person_id": person_id,
        "participant_role": participant.role,
        "participant_email": participant.email_address
    }));

    RelationshipStore::upsert_with_evidence_in_transaction(
        &mut transaction,
        &relationship,
        &[evidence],
    )
    .await?;
    transaction.commit().await?;

    Ok(inserted)
}
