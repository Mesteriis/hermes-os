CREATE TABLE IF NOT EXISTS document_processing_jobs (
    job_id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    step TEXT NOT NULL,
    status TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    last_error_summary TEXT,
    queued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT document_processing_step_check
        CHECK (step IN ('extract_text', 'ocr')),
    CONSTRAINT document_processing_status_check
        CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'skipped')),
    CONSTRAINT document_processing_attempts_check
        CHECK (attempts >= 0 AND max_attempts >= 1 AND attempts <= max_attempts),
    CONSTRAINT document_processing_job_id_not_empty
        CHECK (length(trim(job_id)) > 0),
    CONSTRAINT document_processing_document_id_not_empty
        CHECK (length(trim(document_id)) > 0),
    CONSTRAINT document_processing_document_step_unique
        UNIQUE (document_id, step)
);

CREATE INDEX IF NOT EXISTS document_processing_status_idx
    ON document_processing_jobs (status, queued_at);

CREATE INDEX IF NOT EXISTS document_processing_document_idx
    ON document_processing_jobs (document_id);

CREATE TABLE IF NOT EXISTS document_artifacts (
    artifact_id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
    job_id TEXT NOT NULL REFERENCES document_processing_jobs(job_id) ON DELETE CASCADE,
    artifact_kind TEXT NOT NULL,
    content_sha256 TEXT NOT NULL,
    text_content TEXT,
    storage_kind TEXT,
    storage_path TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT document_artifact_kind_check
        CHECK (artifact_kind IN ('extracted_text', 'ocr_text')),
    CONSTRAINT document_artifact_id_not_empty
        CHECK (length(trim(artifact_id)) > 0),
    CONSTRAINT document_artifact_sha_not_empty
        CHECK (length(trim(content_sha256)) > 0),
    CONSTRAINT document_artifact_text_or_storage
        CHECK (text_content IS NOT NULL OR storage_path IS NOT NULL)
);

CREATE UNIQUE INDEX IF NOT EXISTS document_artifacts_document_kind_idx
    ON document_artifacts (document_id, artifact_kind);

CREATE INDEX IF NOT EXISTS document_artifacts_job_idx
    ON document_artifacts (job_id);
