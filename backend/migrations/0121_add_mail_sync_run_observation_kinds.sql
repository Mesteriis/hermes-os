INSERT INTO observation_kind_definitions (
    kind_definition_id,
    code,
    name,
    version,
    category,
    description
)
VALUES
    (
        'okd_communication_mail_sync_run_v1',
        'COMMUNICATION_MAIL_SYNC_RUN',
        'Communication Mail Sync Run',
        1,
        'communications',
        'Canonical evidence for mail background sync run creation.'
    ),
    (
        'okd_communication_mail_sync_run_status_v1',
        'COMMUNICATION_MAIL_SYNC_RUN_STATUS',
        'Communication Mail Sync Run Status',
        1,
        'communications',
        'Canonical evidence for mail background sync run lifecycle transitions.'
    )
ON CONFLICT (code, version) DO NOTHING;
