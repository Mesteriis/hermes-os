-- Phase 5-6: Finance, contracts, compliance, services, products, enrichment

CREATE TABLE IF NOT EXISTS organization_financial_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE UNIQUE,
    bank_name TEXT,
    iban_masked TEXT,
    bic TEXT,
    payment_terms TEXT,
    currency TEXT DEFAULT 'EUR',
    billing_email TEXT,
    billing_address TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS organization_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    contract_type TEXT NOT NULL,
    title TEXT NOT NULL,
    signed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active',
    document_reference TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_contracts_status_check CHECK (status IN ('draft', 'active', 'expired', 'terminated', 'renewed'))
);

CREATE TABLE IF NOT EXISTS organization_compliance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    compliance_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    document_reference TEXT,
    expires_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_compliance_status_check CHECK (status IN ('compliant', 'pending', 'expired', 'not_applicable'))
);

CREATE TABLE IF NOT EXISTS organization_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    service_name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    started_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS organization_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    product_name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS organization_enrichment_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    url TEXT,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    confidence REAL NOT NULL DEFAULT 0.5,
    status TEXT NOT NULL DEFAULT 'pending',
    last_checked_at TIMESTAMPTZ,
    applied_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_enrichment_confidence_range CHECK (confidence >= 0 AND confidence <= 1),
    CONSTRAINT org_enrichment_status_check CHECK (status IN ('pending', 'applied', 'rejected', 'conflict'))
);

CREATE INDEX IF NOT EXISTS org_contracts_org_id_idx ON organization_contracts (organization_id);
CREATE INDEX IF NOT EXISTS org_compliance_org_id_idx ON organization_compliance (organization_id);
CREATE INDEX IF NOT EXISTS org_enrichment_org_id_idx ON organization_enrichment_results (organization_id);
