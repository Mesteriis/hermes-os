ALTER TABLE mail_sensitive_forwarding_policies
    ADD COLUMN IF NOT EXISTS include_message_body BOOLEAN NOT NULL DEFAULT false;
