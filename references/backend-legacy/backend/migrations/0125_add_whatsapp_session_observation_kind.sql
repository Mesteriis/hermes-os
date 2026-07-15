INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES (
    'okd_whatsapp_web_session_v1',
    'WHATSAPP_WEB_SESSION',
    'WhatsApp Web Session',
    1,
    'communications',
    'Canonical evidence for WhatsApp Web session lifecycle materialization.'
)
ON CONFLICT (code, version) DO NOTHING;
