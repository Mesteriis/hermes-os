-- Phase 1: Organization identities, aliases, departments, contacts, domains

CREATE TABLE IF NOT EXISTS organization_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    identity_type TEXT NOT NULL,
    identity_value TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_identities_type_check CHECK (identity_type IN (
        'domain', 'website', 'email_domain', 'support_email', 'billing_email', 'legal_email',
        'phone', 'vat', 'cif', 'nif', 'registry_number',
        'github_org', 'linkedin_page', 'twitter', 'mastodon',
        'support_portal', 'customer_portal', 'tax_portal', 'app_portal'
    )),
    CONSTRAINT org_identities_status_check CHECK (status IN ('active', 'outdated', 'unreachable', 'blocked')),
    CONSTRAINT org_identities_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_identities_metadata_is_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS org_identities_type_value_idx ON organization_identities (identity_type, identity_value) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS org_identities_org_id_idx ON organization_identities (organization_id);

CREATE TABLE IF NOT EXISTS organization_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    alias_type TEXT DEFAULT 'trading',
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_aliases_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_aliases_type_check CHECK (alias_type IN ('legal', 'trading', 'brand', 'former'))
);

CREATE INDEX IF NOT EXISTS org_aliases_org_id_idx ON organization_aliases (organization_id);

CREATE TABLE IF NOT EXISTS organization_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    domain TEXT NOT NULL,
    domain_type TEXT DEFAULT 'primary',
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_domains_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_domains_type_check CHECK (domain_type IN ('primary', 'additional', 'email', 'portal', 'former'))
);

CREATE INDEX IF NOT EXISTS org_domains_org_id_idx ON organization_domains (organization_id);
CREATE UNIQUE INDEX IF NOT EXISTS org_domains_unique_active ON organization_domains (organization_id, domain) WHERE domain_type != 'former';

CREATE TABLE IF NOT EXISTS organization_departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    parent_department_id UUID REFERENCES organization_departments(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_departments_name_not_empty CHECK (length(trim(name)) > 0)
);

CREATE INDEX IF NOT EXISTS org_departments_org_id_idx ON organization_departments (organization_id);

CREATE TABLE IF NOT EXISTS organization_contact_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES persons(person_id) ON DELETE CASCADE,
    role TEXT,
    department TEXT,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    valid_from TIMESTAMPTZ DEFAULT now(),
    valid_to TIMESTAMPTZ,
    is_primary BOOL NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT org_contact_links_unique UNIQUE (organization_id, person_id, role),
    CONSTRAINT org_contact_links_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS org_contact_links_org_id_idx ON organization_contact_links (organization_id);
CREATE INDEX IF NOT EXISTS org_contact_links_person_id_idx ON organization_contact_links (person_id);

CREATE TABLE IF NOT EXISTS related_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    related_organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    confidence REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT related_orgs_type_check CHECK (relation_type IN ('parent', 'subsidiary', 'division', 'partner', 'supplier', 'customer')),
    CONSTRAINT related_orgs_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE INDEX IF NOT EXISTS related_orgs_org_id_idx ON related_organizations (organization_id);
