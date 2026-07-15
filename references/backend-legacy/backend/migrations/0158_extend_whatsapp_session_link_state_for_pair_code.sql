ALTER TABLE whatsapp_web_sessions
    DROP CONSTRAINT IF EXISTS whatsapp_web_sessions_link_state;

ALTER TABLE whatsapp_web_sessions
    ADD CONSTRAINT whatsapp_web_sessions_link_state CHECK (
        link_state IN (
            'fixture',
            'qr_pending',
            'pair_code_pending',
            'linked',
            'degraded',
            'revoked',
            'blocked'
        )
    );
