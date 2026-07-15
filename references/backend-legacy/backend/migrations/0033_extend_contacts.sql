ALTER TABLE contacts
    ADD COLUMN IF NOT EXISTS language TEXT,
    ADD COLUMN IF NOT EXISTS tone TEXT,
    ADD COLUMN IF NOT EXISTS trust_score SMALLINT,
    ADD COLUMN IF NOT EXISTS avg_response_hours DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS preferred_channel TEXT,
    ADD COLUMN IF NOT EXISTS last_interaction_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS interaction_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS frequent_topics JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS writing_style TEXT,
    ADD COLUMN IF NOT EXISTS contact_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS is_favorite BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS notes TEXT;

ALTER TABLE contacts
    ADD CONSTRAINT contacts_trust_score_range CHECK (trust_score IS NULL OR (trust_score >= 0 AND trust_score <= 100)),
    ADD CONSTRAINT contacts_contact_metadata_is_object CHECK (jsonb_typeof(contact_metadata) = 'object');

CREATE INDEX IF NOT EXISTS contacts_trust_score_idx ON contacts (trust_score DESC NULLS LAST) WHERE trust_score IS NOT NULL;
CREATE INDEX IF NOT EXISTS contacts_last_interaction_idx ON contacts (last_interaction_at DESC NULLS LAST) WHERE last_interaction_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS contacts_favorite_idx ON contacts (contact_id) WHERE is_favorite = true;
