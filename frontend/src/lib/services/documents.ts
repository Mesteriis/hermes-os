import {
	fetchDocumentProcessingJobs,
	fetchDocumentProcessing,
	retryDocumentProcessingJob,
	type DocumentProcessingJob,
	type DocumentProcessingRecord
} from '$lib/api';

export async function loadDocumentProcessingJobs(): Promise<{
	jobs: DocumentProcessingJob[];
	error: string;
}> {
	try {
		const response = await fetchDocumentProcessingJobs(50);
		return { jobs: response.items, error: '' };
	} catch (error) {
		return {
			jobs: [],
			error: error instanceof Error ? error.message : 'Unknown document processing jobs error'
		};
	}
}

export async function reloadSelectedDocumentProcessingDetail(
	documentId: string | undefined
): Promise<{
	detail: DocumentProcessingRecord | null;
	error: string;
}> {
	if (!documentId) {
		return { detail: null, error: '' };
	}

	try {
		const detail = await fetchDocumentProcessing(documentId);
		return { detail, error: '' };
	} catch (error) {
		return {
			detail: null,
			error: error instanceof Error ? error.message : 'Unknown document processing detail error'
		};
	}
}

export async function retryFailedDocumentProcessingJob(
	job: DocumentProcessingJob
): Promise<{ error: string }> {
	try {
		await retryDocumentProcessingJob(job.job_id, {
			command_id: `document-processing-retry-${Date.now()}-${job.job_id}`
		});
		return { error: '' };
	} catch (error) {
		return {
			error: error instanceof Error ? error.message : 'Unknown document processing retry error'
		};
	}
}
