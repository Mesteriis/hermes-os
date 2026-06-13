export type DocumentProcessingStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'skipped';

export type DocumentProcessingStep = 'extract_text' | 'ocr';

export type DocumentProcessingArtifactKind = 'extracted_text' | 'ocr_text';

export type DocumentProcessingJob = {
	job_id: string;
	document_id: string;
	step: DocumentProcessingStep;
	status: DocumentProcessingStatus;
	attempts: number;
	max_attempts: number;
	last_error_summary: string | null;
	queued_at: string;
	started_at: string | null;
	finished_at: string | null;
	created_at: string;
	updated_at: string;
};

export type DocumentProcessingArtifact = {
	artifact_id: string;
	document_id: string;
	job_id: string;
	artifact_kind: DocumentProcessingArtifactKind;
	content_sha256: string;
	text_content: string | null;
	storage_kind: string | null;
	storage_path: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
};

export type DocumentProcessingRecord = {
	document_id: string;
	jobs: DocumentProcessingJob[];
	artifacts: DocumentProcessingArtifact[];
};

export type DocumentProcessingJobsResponse = {
	items: DocumentProcessingJob[];
};

export type DocumentProcessingRetryRequest = {
	command_id: string;
};

export type DocumentProcessingRetryResponse = {
	job_id: string;
	status: DocumentProcessingStatus;
	event_id: string;
};
