import { ApiClient } from '../client';
import type {
	DocumentProcessingRecord,
	DocumentProcessingJobsResponse,
	DocumentProcessingRetryRequest,
	DocumentProcessingRetryResponse
} from '../types';

export async function fetchDocumentProcessing(documentId: string): Promise<DocumentProcessingRecord> {
	return ApiClient.instance.get<DocumentProcessingRecord>(
		`/api/v1/documents/${encodeURIComponent(documentId)}/processing`,
		'Document processing request failed'
	);
}

export async function fetchDocumentProcessingJobs(limit = 50): Promise<DocumentProcessingJobsResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<DocumentProcessingJobsResponse>(
		`/api/v1/document-processing/jobs?${params.toString()}`,
		'Document processing jobs request failed'
	);
}

export async function retryDocumentProcessingJob(
	jobId: string,
	request: DocumentProcessingRetryRequest
): Promise<DocumentProcessingRetryResponse> {
	return ApiClient.instance.post<DocumentProcessingRetryResponse>(
		`/api/v1/document-processing/jobs/${encodeURIComponent(jobId)}/retry`,
		request,
		'Document processing retry request failed'
	);
}
