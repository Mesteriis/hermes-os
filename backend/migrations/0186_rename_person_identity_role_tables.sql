-- Rename legacy Person identity/role storage to Persona storage.

ALTER TABLE IF EXISTS person_identities RENAME TO persona_identities;
ALTER TABLE IF EXISTS person_identity_candidates RENAME TO persona_identity_candidates;
ALTER TABLE IF EXISTS person_roles RENAME TO persona_roles;

ALTER INDEX IF EXISTS person_identities_type_value_idx RENAME TO persona_identities_type_value_idx;
ALTER INDEX IF EXISTS person_identities_person_id_idx RENAME TO persona_identities_person_id_idx;
ALTER INDEX IF EXISTS person_identity_merge_pair_idx RENAME TO persona_identity_merge_pair_idx;
ALTER INDEX IF EXISTS contact_identity_review_state_idx RENAME TO persona_identity_review_state_idx;
ALTER INDEX IF EXISTS contact_identity_left_contact_idx RENAME TO persona_identity_left_person_idx;
ALTER INDEX IF EXISTS contact_identity_right_contact_idx RENAME TO persona_identity_right_person_idx;
ALTER INDEX IF EXISTS person_roles_person_id_idx RENAME TO persona_roles_person_id_idx;
ALTER INDEX IF EXISTS person_roles_role_idx RENAME TO persona_roles_role_idx;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_identities_type_check') THEN
        ALTER TABLE persona_identities
            RENAME CONSTRAINT person_identities_type_check TO persona_identities_type_check;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_identities_status_check') THEN
        ALTER TABLE persona_identities
            RENAME CONSTRAINT person_identities_status_check TO persona_identities_status_check;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_identities_metadata_is_object') THEN
        ALTER TABLE persona_identities
            RENAME CONSTRAINT person_identities_metadata_is_object TO persona_identities_metadata_is_object;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_identity_candidate_kind_check') THEN
        ALTER TABLE persona_identity_candidates
            RENAME CONSTRAINT person_identity_candidate_kind_check TO persona_identity_candidate_kind_check;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_identity_merge_has_right_person') THEN
        ALTER TABLE persona_identity_candidates
            RENAME CONSTRAINT person_identity_merge_has_right_person TO persona_identity_merge_has_right_person;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'contact_identity_review_state_check') THEN
        ALTER TABLE persona_identity_candidates
            RENAME CONSTRAINT contact_identity_review_state_check TO persona_identity_review_state_check;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'contact_identity_confidence_check') THEN
        ALTER TABLE persona_identity_candidates
            RENAME CONSTRAINT contact_identity_confidence_check TO persona_identity_confidence_check;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'contact_identity_candidate_id_not_empty') THEN
        ALTER TABLE persona_identity_candidates
            RENAME CONSTRAINT contact_identity_candidate_id_not_empty TO persona_identity_candidate_id_not_empty;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'contact_identity_left_contact_not_empty') THEN
        ALTER TABLE persona_identity_candidates
            RENAME CONSTRAINT contact_identity_left_contact_not_empty TO persona_identity_left_person_not_empty;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'contact_identity_evidence_not_empty') THEN
        ALTER TABLE persona_identity_candidates
            RENAME CONSTRAINT contact_identity_evidence_not_empty TO persona_identity_evidence_not_empty;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'person_roles_unique') THEN
        ALTER TABLE persona_roles
            RENAME CONSTRAINT person_roles_unique TO persona_roles_unique;
    END IF;
END $$;
