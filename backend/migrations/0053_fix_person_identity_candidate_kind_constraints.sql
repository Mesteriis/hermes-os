UPDATE person_identity_candidates
SET candidate_kind = CASE candidate_kind
    WHEN 'merge_contacts' THEN 'merge_persons'
    WHEN 'split_contact' THEN 'split_person'
    ELSE candidate_kind
END
WHERE candidate_kind IN ('merge_contacts', 'split_contact');

ALTER TABLE person_identity_candidates
    DROP CONSTRAINT IF EXISTS contact_identity_candidate_kind_check;

ALTER TABLE person_identity_candidates
    DROP CONSTRAINT IF EXISTS person_identity_candidate_kind_check;

ALTER TABLE person_identity_candidates
    ADD CONSTRAINT person_identity_candidate_kind_check
        CHECK (candidate_kind IN ('merge_persons', 'attach_email_address', 'split_person'));

ALTER TABLE person_identity_candidates
    DROP CONSTRAINT IF EXISTS contact_identity_merge_has_right_contact;

ALTER TABLE person_identity_candidates
    DROP CONSTRAINT IF EXISTS person_identity_merge_has_right_person;

ALTER TABLE person_identity_candidates
    ADD CONSTRAINT person_identity_merge_has_right_person
        CHECK (candidate_kind <> 'merge_persons' OR right_person_id IS NOT NULL);

DROP INDEX IF EXISTS contact_identity_merge_pair_idx;
DROP INDEX IF EXISTS person_identity_merge_pair_idx;

CREATE UNIQUE INDEX person_identity_merge_pair_idx
    ON person_identity_candidates (
        candidate_kind,
        LEAST(left_person_id, COALESCE(right_person_id, left_person_id)),
        GREATEST(left_person_id, COALESCE(right_person_id, left_person_id))
    )
    WHERE candidate_kind = 'merge_persons';
