CREATE TABLE IF NOT EXISTS contact_identity_candidates (
    identity_candidate_id TEXT PRIMARY KEY,
    candidate_kind TEXT NOT NULL,
    left_contact_id TEXT NOT NULL REFERENCES contacts(contact_id) ON DELETE CASCADE,
    right_contact_id TEXT REFERENCES contacts(contact_id) ON DELETE CASCADE,
    email_address TEXT,
    evidence_summary TEXT NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'suggested',
    event_id TEXT,
    actor_id TEXT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT contact_identity_candidate_kind_check
        CHECK (candidate_kind IN ('merge_contacts', 'attach_email_address', 'split_contact')),
    CONSTRAINT contact_identity_review_state_check
        CHECK (review_state IN ('suggested', 'user_confirmed', 'user_rejected')),
    CONSTRAINT contact_identity_confidence_check
        CHECK (confidence >= 0.0 AND confidence <= 1.0),
    CONSTRAINT contact_identity_candidate_id_not_empty
        CHECK (length(trim(identity_candidate_id)) > 0),
    CONSTRAINT contact_identity_left_contact_not_empty
        CHECK (length(trim(left_contact_id)) > 0),
    CONSTRAINT contact_identity_evidence_not_empty
        CHECK (length(trim(evidence_summary)) > 0),
    CONSTRAINT contact_identity_merge_has_right_contact
        CHECK (candidate_kind <> 'merge_contacts' OR right_contact_id IS NOT NULL)
);

CREATE UNIQUE INDEX IF NOT EXISTS contact_identity_merge_pair_idx
    ON contact_identity_candidates (
        candidate_kind,
        LEAST(left_contact_id, COALESCE(right_contact_id, left_contact_id)),
        GREATEST(left_contact_id, COALESCE(right_contact_id, left_contact_id))
    )
    WHERE candidate_kind = 'merge_contacts';

CREATE INDEX IF NOT EXISTS contact_identity_review_state_idx
    ON contact_identity_candidates (review_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS contact_identity_left_contact_idx
    ON contact_identity_candidates (left_contact_id);

CREATE INDEX IF NOT EXISTS contact_identity_right_contact_idx
    ON contact_identity_candidates (right_contact_id);
