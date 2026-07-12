CREATE TABLE IF NOT EXISTS mail_sensitive_forwarding_policies (
    policy_id TEXT PRIMARY KEY,
    source_account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    delivery_account_id TEXT NOT NULL REFERENCES communication_provider_accounts(account_id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    fixed_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    minimum_severity TEXT NOT NULL,
    subject_template TEXT NOT NULL,
    body_template TEXT NOT NULL,
    max_sends_per_hour INTEGER NOT NULL,
    quiet_hours JSONB NOT NULL DEFAULT '{}'::jsonb,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT mail_sensitive_forwarding_policy_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT mail_sensitive_forwarding_policy_recipients_is_array CHECK (jsonb_typeof(fixed_recipients) = 'array'),
    CONSTRAINT mail_sensitive_forwarding_policy_recipients_not_empty CHECK (jsonb_array_length(fixed_recipients) > 0),
    CONSTRAINT mail_sensitive_forwarding_policy_severity CHECK (minimum_severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT mail_sensitive_forwarding_policy_subject_not_blank CHECK (length(trim(subject_template)) > 0),
    CONSTRAINT mail_sensitive_forwarding_policy_body_not_blank CHECK (length(trim(body_template)) > 0),
    CONSTRAINT mail_sensitive_forwarding_policy_rate_positive CHECK (max_sends_per_hour > 0),
    CONSTRAINT mail_sensitive_forwarding_policy_quiet_hours_is_object CHECK (jsonb_typeof(quiet_hours) = 'object')
);

CREATE INDEX IF NOT EXISTS mail_sensitive_forwarding_policies_source_idx
    ON mail_sensitive_forwarding_policies (source_account_id, enabled, updated_at DESC);

CREATE TABLE IF NOT EXISTS mail_sensitive_forwarding_dispatches (
    dispatch_id TEXT PRIMARY KEY,
    policy_id TEXT NOT NULL REFERENCES mail_sensitive_forwarding_policies(policy_id) ON DELETE RESTRICT,
    message_id TEXT NOT NULL REFERENCES communication_messages(message_id) ON DELETE RESTRICT,
    outbox_id TEXT NOT NULL REFERENCES communication_outbox(outbox_id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT mail_sensitive_forwarding_dispatch_once UNIQUE (policy_id, message_id),
    CONSTRAINT mail_sensitive_forwarding_dispatch_outbox_unique UNIQUE (outbox_id)
);

CREATE INDEX IF NOT EXISTS mail_sensitive_forwarding_dispatches_policy_created_idx
    ON mail_sensitive_forwarding_dispatches (policy_id, created_at DESC);
