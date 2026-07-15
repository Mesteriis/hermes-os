ALTER TABLE person_identities
    DROP CONSTRAINT IF EXISTS person_identities_type_check;

ALTER TABLE person_identities
    ADD CONSTRAINT person_identities_type_check CHECK (identity_type IN (
        'email', 'telegram', 'whatsapp', 'phone',
        'github', 'linkedin', 'website',
        'mastodon', 'x', 'stackoverflow', 'habr',
        'medium', 'orcid', 'google_scholar',
        'document_mention', 'message_participant'
    ));
