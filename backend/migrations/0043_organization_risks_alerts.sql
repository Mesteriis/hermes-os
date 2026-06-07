-- Phase 7: Risks and alerts

CREATE TABLE IF NOT EXISTS organization_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    risk_type TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT,
    CONSTRAINT org_risks_severity_check CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT org_risks_confidence_range CHECK (confidence >= 0 AND confidence <= 1)
);

CREATE TABLE IF NOT EXISTS organization_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id TEXT NOT NULL REFERENCES organizations(organization_id) ON DELETE CASCADE,
    alert_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    severity TEXT NOT NULL DEFAULT 'medium',
    acknowledged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT org_alerts_severity_check CHECK (severity IN ('low', 'medium', 'high', 'critical'))
);

CREATE INDEX IF NOT EXISTS org_risks_org_id_idx ON organization_risks (organization_id);
CREATE INDEX IF NOT EXISTS org_alerts_org_id_idx ON organization_alerts (organization_id);
