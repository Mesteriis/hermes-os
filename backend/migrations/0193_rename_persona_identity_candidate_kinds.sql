-- Rename Persona identity candidate kind values to Persona-native terminology.
--
-- Candidate ids may still contain historical `merge_persons` / `split_person`
-- fragments because they can be externally referenced. The active kind value is
-- migrated and new writers use `merge_personas` / `split_persona`.

UPDATE persona_identity_candidates
SET candidate_kind = CASE candidate_kind
    WHEN 'merge_persons' THEN 'merge_personas'
    WHEN 'split_person' THEN 'split_persona'
    ELSE candidate_kind
END
WHERE candidate_kind IN ('merge_persons', 'split_person');

ALTER TABLE persona_identity_candidates
    DROP CONSTRAINT IF EXISTS person_identity_candidate_kind_check;

ALTER TABLE persona_identity_candidates
    DROP CONSTRAINT IF EXISTS persona_identity_candidate_kind_check;

ALTER TABLE persona_identity_candidates
    ADD CONSTRAINT persona_identity_candidate_kind_check
        CHECK (candidate_kind IN ('merge_personas', 'attach_email_address', 'split_persona'));

ALTER TABLE persona_identity_candidates
    DROP CONSTRAINT IF EXISTS person_identity_merge_has_right_person;

ALTER TABLE persona_identity_candidates
    DROP CONSTRAINT IF EXISTS persona_identity_merge_has_right_person;

ALTER TABLE persona_identity_candidates
    DROP CONSTRAINT IF EXISTS persona_identity_merge_has_right_persona;

ALTER TABLE persona_identity_candidates
    ADD CONSTRAINT persona_identity_merge_has_right_persona
        CHECK (candidate_kind <> 'merge_personas' OR right_person_id IS NOT NULL);

DROP INDEX IF EXISTS person_identity_merge_pair_idx;
DROP INDEX IF EXISTS persona_identity_merge_pair_idx;

CREATE UNIQUE INDEX persona_identity_merge_pair_idx
    ON persona_identity_candidates (
        candidate_kind,
        LEAST(left_person_id, COALESCE(right_person_id, left_person_id)),
        GREATEST(left_person_id, COALESCE(right_person_id, left_person_id))
    )
    WHERE candidate_kind = 'merge_personas';
