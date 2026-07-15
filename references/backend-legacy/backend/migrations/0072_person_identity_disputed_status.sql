ALTER TABLE person_identities
    DROP CONSTRAINT IF EXISTS person_identities_status_check;

ALTER TABLE person_identities
    ADD CONSTRAINT person_identities_status_check CHECK (status IN (
        'active', 'outdated', 'unreachable', 'blocked', 'disputed'
    ));
