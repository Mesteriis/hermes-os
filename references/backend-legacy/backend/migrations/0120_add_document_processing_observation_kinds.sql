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
        'observation_kind:v1:document_processing_job',
        'DOCUMENT_PROCESSING_JOB',
        'Document processing job',
        1,
        'document',
        'Document processing job queued as durable execution evidence.'
    ),
    (
        'observation_kind:v1:document_processing_job_status',
        'DOCUMENT_PROCESSING_JOB_STATUS',
        'Document processing job status',
        1,
        'document',
        'Document processing job lifecycle state captured as durable execution evidence.'
    )
ON CONFLICT (kind_definition_id) DO UPDATE SET
    code = EXCLUDED.code,
    name = EXCLUDED.name,
    version = EXCLUDED.version,
    category = EXCLUDED.category,
    description = EXCLUDED.description,
    updated_at = now();
