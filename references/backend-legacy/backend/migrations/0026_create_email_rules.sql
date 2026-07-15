CREATE TABLE IF NOT EXISTS email_rules (
    rule_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description_nl TEXT NOT NULL DEFAULT '',
    conditions_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    actions_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    mode TEXT NOT NULL DEFAULT 'suggest',
    enabled BOOLEAN NOT NULL DEFAULT true,
    match_count BIGINT NOT NULL DEFAULT 0,
    last_matched_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_rules_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT email_rules_mode CHECK (mode IN ('suggest', 'ask_before_execute', 'auto_execute', 'dry_run')),
    CONSTRAINT email_rules_conditions_is_array CHECK (jsonb_typeof(conditions_json) = 'array'),
    CONSTRAINT email_rules_actions_is_array CHECK (jsonb_typeof(actions_json) = 'array')
);
