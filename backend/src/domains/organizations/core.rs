use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

// ── OrganizationIdentity ────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationIdentity {
    pub id: String,
    pub organization_id: String,
    pub identity_type: String,
    pub identity_value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub status: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgIdentityStore {
    pool: PgPool,
}
impl OrgIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrganizationIdentity>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, identity_type, identity_value, source, confidence, last_verified_at, status, metadata, created_at, updated_at FROM organization_identities WHERE organization_id = $1 ORDER BY identity_type")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrganizationIdentity {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    identity_type: r.try_get("identity_type")?,
                    identity_value: r.try_get("identity_value")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    last_verified_at: r.try_get("last_verified_at")?,
                    status: r.try_get("status")?,
                    metadata: r.try_get("metadata")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
    pub async fn upsert(
        &self,
        org_id: &str,
        itype: &str,
        ivalue: &str,
        source: &str,
    ) -> Result<OrganizationIdentity, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_identities (organization_id, identity_type, identity_value, source) VALUES ($1,$2,$3,$4) ON CONFLICT (identity_type, identity_value) WHERE status='active' DO UPDATE SET updated_at=now() RETURNING id::text, organization_id, identity_type, identity_value, source, confidence, last_verified_at, status, metadata, created_at, updated_at")
            .bind(org_id).bind(itype).bind(ivalue).bind(source).fetch_one(&self.pool).await?;
        Ok(OrganizationIdentity {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            identity_type: row.try_get("identity_type")?,
            identity_value: row.try_get("identity_value")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            last_verified_at: row.try_get("last_verified_at")?,
            status: row.try_get("status")?,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

// ── OrganizationAlias ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationAlias {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub alias_type: String,
    pub source: String,
    pub confidence: f64,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgAliasStore {
    pool: PgPool,
}
impl OrgAliasStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrganizationAlias>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, alias_type, source, confidence, valid_from, valid_to, created_at FROM organization_aliases WHERE organization_id=$1 ORDER BY name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrganizationAlias {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    name: r.try_get("name")?,
                    alias_type: r.try_get("alias_type")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    valid_from: r.try_get("valid_from")?,
                    valid_to: r.try_get("valid_to")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        org_id: &str,
        name: &str,
        alias_type: &str,
        source: &str,
    ) -> Result<OrganizationAlias, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_aliases (organization_id, name, alias_type, source) VALUES ($1,$2,$3,$4) RETURNING id::text, organization_id, name, alias_type, source, confidence, valid_from, valid_to, created_at")
            .bind(org_id).bind(name).bind(alias_type).bind(source).fetch_one(&self.pool).await?;
        Ok(OrganizationAlias {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            name: row.try_get("name")?,
            alias_type: row.try_get("alias_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            valid_from: row.try_get("valid_from")?,
            valid_to: row.try_get("valid_to")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ── OrganizationDomain ─────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationDomain {
    pub id: String,
    pub organization_id: String,
    pub domain: String,
    pub domain_type: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgDomainStore {
    pool: PgPool,
}
impl OrgDomainStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrganizationDomain>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, domain, domain_type, source, confidence, last_verified_at, created_at FROM organization_domains WHERE organization_id=$1 ORDER BY domain_type, domain")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrganizationDomain {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    domain: r.try_get("domain")?,
                    domain_type: r.try_get("domain_type")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    last_verified_at: r.try_get("last_verified_at")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        org_id: &str,
        domain: &str,
        domain_type: &str,
        source: &str,
    ) -> Result<OrganizationDomain, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_domains (organization_id, domain, domain_type, source) VALUES ($1,$2,$3,$4) RETURNING id::text, organization_id, domain, domain_type, source, confidence, last_verified_at, created_at")
            .bind(org_id).bind(domain).bind(domain_type).bind(source).fetch_one(&self.pool).await?;
        Ok(OrganizationDomain {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            domain: row.try_get("domain")?,
            domain_type: row.try_get("domain_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            last_verified_at: row.try_get("last_verified_at")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ── OrganizationDepartment ─────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgDepartment {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_department_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgDepartmentStore {
    pool: PgPool,
}
impl OrgDepartmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgDepartment>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, description, parent_department_id::text, created_at FROM organization_departments WHERE organization_id=$1 ORDER BY name")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgDepartment {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    name: r.try_get("name")?,
                    description: r.try_get("description")?,
                    parent_department_id: r.try_get("parent_department_id")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        org_id: &str,
        name: &str,
        description: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<OrgDepartment, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_departments (organization_id, name, description, parent_department_id) VALUES ($1,$2,$3,$4::uuid) RETURNING id::text, organization_id, name, description, parent_department_id::text, created_at")
            .bind(org_id).bind(name).bind(description).bind(parent_id).fetch_one(&self.pool).await?;
        Ok(OrgDepartment {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            parent_department_id: row.try_get("parent_department_id")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// ── OrganizationContactLink ─────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgContactLink {
    pub id: String,
    pub organization_id: String,
    pub person_id: String,
    pub role: Option<String>,
    pub department: Option<String>,
    pub source: String,
    pub confidence: f64,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgContactLinkStore {
    pool: PgPool,
}
impl OrgContactLinkStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list_by_org(&self, org_id: &str) -> Result<Vec<OrgContactLink>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, person_id, role, department, source, confidence, valid_from, valid_to, is_primary, created_at, updated_at FROM organization_contact_links WHERE organization_id=$1 ORDER BY is_primary DESC, role")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgContactLink {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    person_id: r.try_get("person_id")?,
                    role: r.try_get("role")?,
                    department: r.try_get("department")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    valid_from: r.try_get("valid_from")?,
                    valid_to: r.try_get("valid_to")?,
                    is_primary: r.try_get("is_primary")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
    pub async fn link(
        &self,
        org_id: &str,
        person_id: &str,
        role: Option<&str>,
        dept: Option<&str>,
    ) -> Result<OrgContactLink, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_contact_links (organization_id, person_id, role, department) VALUES ($1,$2,$3,$4) ON CONFLICT (organization_id, person_id, role) DO UPDATE SET department=EXCLUDED.department, updated_at=now() RETURNING id::text, organization_id, person_id, role, department, source, confidence, valid_from, valid_to, is_primary, created_at, updated_at")
            .bind(org_id).bind(person_id).bind(role).bind(dept).fetch_one(&self.pool).await?;
        Ok(OrgContactLink {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            person_id: row.try_get("person_id")?,
            role: row.try_get("role")?,
            department: row.try_get("department")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            valid_from: row.try_get("valid_from")?,
            valid_to: row.try_get("valid_to")?,
            is_primary: row.try_get("is_primary")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
    pub async fn set_primary(&self, org_id: &str, person_id: &str) -> Result<(), OrgCoreError> {
        sqlx::query(
            "UPDATE organization_contact_links SET is_primary=false WHERE organization_id=$1",
        )
        .bind(org_id)
        .execute(&self.pool)
        .await?;
        sqlx::query("UPDATE organization_contact_links SET is_primary=true WHERE organization_id=$1 AND person_id=$2").bind(org_id).bind(person_id).execute(&self.pool).await?;
        Ok(())
    }
}

// ── RelatedOrganization ────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelatedOrganization {
    pub id: String,
    pub organization_id: String,
    pub related_organization_id: String,
    pub relation_type: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RelatedOrgStore {
    pool: PgPool,
}
impl RelatedOrgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<RelatedOrganization>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, related_organization_id, relation_type, source, confidence, created_at FROM related_organizations WHERE organization_id=$1")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(RelatedOrganization {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    related_organization_id: r.try_get("related_organization_id")?,
                    relation_type: r.try_get("relation_type")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
    pub async fn relate(
        &self,
        org_id: &str,
        related_id: &str,
        rel_type: &str,
    ) -> Result<RelatedOrganization, OrgCoreError> {
        let row = sqlx::query("INSERT INTO related_organizations (organization_id, related_organization_id, relation_type) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING RETURNING id::text, organization_id, related_organization_id, relation_type, source, confidence, created_at")
            .bind(org_id).bind(related_id).bind(rel_type).fetch_one(&self.pool).await?;
        Ok(RelatedOrganization {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            related_organization_id: row.try_get("related_organization_id")?,
            relation_type: row.try_get("relation_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug, Error)]
pub enum OrgCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
