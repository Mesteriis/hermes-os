CREATE TABLE IF NOT EXISTS email_templates (
    template_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    subject_template TEXT NOT NULL,
    body_template TEXT NOT NULL DEFAULT '',
    variables JSONB NOT NULL DEFAULT '[]'::jsonb,
    language TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT email_templates_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT email_templates_subject_not_empty CHECK (length(trim(subject_template)) > 0),
    CONSTRAINT email_templates_variables_is_array CHECK (jsonb_typeof(variables) = 'array')
);
