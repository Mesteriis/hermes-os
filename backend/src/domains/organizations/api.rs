use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Organization {
    pub organization_id: String,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub org_type: Option<String>,
    pub status: String,
    pub country: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub timezone: Option<String>,
    pub trust_score: Option<i16>,
    pub health_status: Option<String>,
    pub priority: Option<String>,
    pub notes: Option<String>,
    pub tags: Value,
    pub org_metadata: Value,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub interaction_count: i32,
    pub registration_number: Option<String>,
    pub country_of_registration: Option<String>,
    pub vat: Option<String>,
    pub cif: Option<String>,
    pub nif: Option<String>,
    pub tax_id: Option<String>,
    pub legal_address: Option<String>,
    pub registry_source: Option<String>,
    pub registry_last_verified: Option<DateTime<Utc>>,
    pub communication_style: Option<String>,
    pub verbosity: Option<String>,
    pub formality: Option<String>,
    pub secondary_languages: Option<Value>,
    pub preferred_tone: Option<String>,
    pub official_style_required: Option<bool>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub watchlist: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrganizationStore {
    pool: PgPool,
}

impl OrganizationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        display_name: &str,
        org_type: Option<&str>,
    ) -> Result<Organization, OrganizationError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let org_id = format!("org:v1:{ts:x}");
        let row = sqlx::query(
            "INSERT INTO organizations (organization_id, display_name, org_type)
             VALUES ($1, $2, $3)
             RETURNING organization_id, display_name, legal_name, org_type, status, country, city,
                       address, website, industry, description, primary_language, timezone,
                       trust_score, health_status, priority, notes, tags, org_metadata,
                       last_interaction_at, interaction_count,
                       registration_number, country_of_registration, vat, cif, nif, tax_id,
                       legal_address, registry_source, registry_last_verified,
                       communication_style, verbosity, formality, secondary_languages,
                       preferred_tone, official_style_required,
                       last_health_check, watchlist, created_at, updated_at",
        )
        .bind(&org_id)
        .bind(display_name)
        .bind(org_type)
        .fetch_one(&self.pool)
        .await?;
        row_to_org(row)
    }

    pub async fn get(
        &self,
        organization_id: &str,
    ) -> Result<Option<Organization>, OrganizationError> {
        let row = sqlx::query(
            "SELECT organization_id, display_name, legal_name, org_type, status, country, city,
                    address, website, industry, description, primary_language, timezone,
                    trust_score, health_status, priority, notes, tags, org_metadata,
                    last_interaction_at, interaction_count,
                    registration_number, country_of_registration, vat, cif, nif, tax_id,
                    legal_address, registry_source, registry_last_verified,
                    communication_style, verbosity, formality, secondary_languages,
                    preferred_tone, official_style_required,
                    last_health_check, watchlist, created_at, updated_at
             FROM organizations WHERE organization_id = $1",
        )
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_org).transpose()
    }

    pub async fn list(
        &self,
        org_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Organization>, OrganizationError> {
        let limit = limit.clamp(1, 100);
        let rows =
            if let Some(t) = org_type {
                sqlx::query(
                "SELECT organization_id, display_name, legal_name, org_type, status, country, city,
                        address, website, industry, description, primary_language, timezone,
                        trust_score, health_status, priority, notes, tags, org_metadata,
                        last_interaction_at, interaction_count,
                        registration_number, country_of_registration, vat, cif, nif, tax_id,
                        legal_address, registry_source, registry_last_verified,
                        communication_style, verbosity, formality, secondary_languages,
                        preferred_tone, official_style_required,
                        last_health_check, watchlist, created_at, updated_at
                 FROM organizations WHERE org_type = $1 ORDER BY interaction_count DESC LIMIT $2"
            ).bind(t).bind(limit).fetch_all(&self.pool).await?
            } else {
                sqlx::query(
                "SELECT organization_id, display_name, legal_name, org_type, status, country, city,
                        address, website, industry, description, primary_language, timezone,
                        trust_score, health_status, priority, notes, tags, org_metadata,
                        last_interaction_at, interaction_count,
                        registration_number, country_of_registration, vat, cif, nif, tax_id,
                        legal_address, registry_source, registry_last_verified,
                        communication_style, verbosity, formality, secondary_languages,
                        preferred_tone, official_style_required,
                        last_health_check, watchlist, created_at, updated_at
                 FROM organizations ORDER BY interaction_count DESC LIMIT $1"
            ).bind(limit).fetch_all(&self.pool).await?
            };
        rows.into_iter().map(row_to_org).collect()
    }

    pub async fn update(
        &self,
        organization_id: &str,
        update: &OrganizationUpdate,
    ) -> Result<Organization, OrganizationError> {
        let row = sqlx::query(
            "UPDATE organizations SET
                display_name = COALESCE($2, display_name),
                legal_name = COALESCE($3, legal_name),
                org_type = COALESCE($4, org_type),
                status = COALESCE($5, status),
                country = COALESCE($6, country),
                city = COALESCE($7, city),
                address = COALESCE($8, address),
                website = COALESCE($9, website),
                industry = COALESCE($10, industry),
                description = COALESCE($11, description),
                primary_language = COALESCE($12, primary_language),
                timezone = COALESCE($13, timezone),
                priority = COALESCE($14, priority),
                notes = COALESCE($15, notes),
                tags = COALESCE($16, tags),
                org_metadata = COALESCE($17, org_metadata),
                updated_at = now()
             WHERE organization_id = $1
             RETURNING organization_id, display_name, legal_name, org_type, status, country, city,
                       address, website, industry, description, primary_language, timezone,
                       trust_score, health_status, priority, notes, tags, org_metadata,
                       last_interaction_at, interaction_count,
                       registration_number, country_of_registration, vat, cif, nif, tax_id,
                       legal_address, registry_source, registry_last_verified,
                       communication_style, verbosity, formality, secondary_languages,
                       preferred_tone, official_style_required,
                       last_health_check, watchlist, created_at, updated_at",
        )
        .bind(organization_id)
        .bind(update.display_name.as_deref())
        .bind(update.legal_name.as_deref())
        .bind(update.org_type.as_deref())
        .bind(update.status.as_deref())
        .bind(update.country.as_deref())
        .bind(update.city.as_deref())
        .bind(update.address.as_deref())
        .bind(update.website.as_deref())
        .bind(update.industry.as_deref())
        .bind(update.description.as_deref())
        .bind(update.primary_language.as_deref())
        .bind(update.timezone.as_deref())
        .bind(update.priority.as_deref())
        .bind(update.notes.as_deref())
        .bind(update.tags.as_ref())
        .bind(update.org_metadata.as_ref())
        .fetch_one(&self.pool)
        .await?;
        row_to_org(row)
    }

    pub async fn archive(&self, organization_id: &str) -> Result<(), OrganizationError> {
        sqlx::query("UPDATE organizations SET status = 'archived', updated_at = now() WHERE organization_id = $1")
            .bind(organization_id).execute(&self.pool).await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct OrganizationUpdate {
    pub display_name: Option<String>,
    pub legal_name: Option<String>,
    pub org_type: Option<String>,
    pub status: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub timezone: Option<String>,
    pub priority: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Value>,
    pub org_metadata: Option<Value>,
}

fn row_to_org(row: PgRow) -> Result<Organization, OrganizationError> {
    Ok(Organization {
        organization_id: row.try_get("organization_id")?,
        display_name: row.try_get("display_name")?,
        legal_name: row.try_get("legal_name")?,
        org_type: row.try_get("org_type")?,
        status: row.try_get("status")?,
        country: row.try_get("country")?,
        city: row.try_get("city")?,
        address: row.try_get("address")?,
        website: row.try_get("website")?,
        industry: row.try_get("industry")?,
        description: row.try_get("description")?,
        primary_language: row.try_get("primary_language")?,
        timezone: row.try_get("timezone")?,
        trust_score: row.try_get("trust_score")?,
        health_status: row.try_get("health_status")?,
        priority: row.try_get("priority")?,
        notes: row.try_get("notes")?,
        tags: row.try_get("tags")?,
        org_metadata: row.try_get("org_metadata")?,
        last_interaction_at: row.try_get("last_interaction_at")?,
        interaction_count: row.try_get("interaction_count")?,
        registration_number: row.try_get("registration_number")?,
        country_of_registration: row.try_get("country_of_registration")?,
        vat: row.try_get("vat")?,
        cif: row.try_get("cif")?,
        nif: row.try_get("nif")?,
        tax_id: row.try_get("tax_id")?,
        legal_address: row.try_get("legal_address")?,
        registry_source: row.try_get("registry_source")?,
        registry_last_verified: row.try_get("registry_last_verified")?,
        communication_style: row.try_get("communication_style")?,
        verbosity: row.try_get("verbosity")?,
        formality: row.try_get("formality")?,
        secondary_languages: row.try_get("secondary_languages")?,
        preferred_tone: row.try_get("preferred_tone")?,
        official_style_required: row.try_get("official_style_required")?,
        last_health_check: row.try_get("last_health_check")?,
        watchlist: row.try_get("watchlist")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum OrganizationError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("organization not found")]
    NotFound,
}
