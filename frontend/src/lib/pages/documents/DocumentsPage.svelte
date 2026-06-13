<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as documentsService from '$lib/services/documents';
	import type { DocumentProcessingJob } from '$lib/api';
	import DocumentsSourceCards from './widgets/DocumentsSourceCards.svelte';
	import DocumentsNavigation from './widgets/DocumentsNavigation.svelte';
	import DocumentsList from './widgets/DocumentsList.svelte';
	import DocumentsProcessingJobs from './widgets/DocumentsProcessingJobs.svelte';
	import DocumentsInsights from './widgets/DocumentsInsights.svelte';
	import './documents.css';

	const _ = (key: string) => t($currentLocale, key);

	type Doc = { name: string; source: string; project: string; type: string; date: string; size: string; icon: string; tone: string };

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	let documentProcessingJobs = $state<DocumentProcessingJob[]>([]);
	let documentProcessingDetailError = $state('');
	let isDocumentProcessingJobsLoading = $state(false);
	let retryingDocumentProcessingJobId = $state<string | null>(null);

	let documents = $derived(
		documentProcessingJobs.map((job) => ({
			name: `${job.document_id} (${job.step})`,
			source: 'Hermes Hub',
			project: job.status,
			type: job.step,
			date: job.queued_at,
			size: job.last_error_summary || 'No errors',
			icon: 'tabler:file-text' as const,
			tone: job.status === 'succeeded' ? 'green' as const : job.status === 'failed' ? 'red' as const : 'amber' as const
		}))
	);

	async function retryFailedDocumentProcessingJob(job: DocumentProcessingJob) {
		if (retryingDocumentProcessingJobId === job.job_id) return;
		retryingDocumentProcessingJobId = job.job_id;
		documentProcessingDetailError = '';
		const result = await documentsService.retryFailedDocumentProcessingJob(job);
		documentProcessingDetailError = result.error;
		await loadDocumentProcessingJobs();
		if (retryingDocumentProcessingJobId === job.job_id) retryingDocumentProcessingJobId = null;
	}

	async function loadDocumentProcessingJobs() {
		isDocumentProcessingJobsLoading = true;
		const result = await documentsService.loadDocumentProcessingJobs();
		documentProcessingJobs = result.jobs;
		documentProcessingDetailError = result.error;
		isDocumentProcessingJobsLoading = false;
	}

	$effect(() => {
		loadDocumentProcessingJobs();
	});
</script>

<section class="documents-page">
	<div class="view-header">
		<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:file-text" width="28" height="28" /></span><div><h1>Documents</h1><p>All your documents from connected sources</p></div></div>
	</div>
	<div class="documents-layout">
		<DocumentsSourceCards {isLayoutEditing} {isWidgetVisible} />
		<DocumentsNavigation {isLayoutEditing} {isWidgetVisible} />
		<DocumentsList {documents} {isLayoutEditing} {isWidgetVisible} />
		<aside class="stacked-rail">
			<DocumentsProcessingJobs
				jobs={documentProcessingJobs}
				isLoading={isDocumentProcessingJobsLoading}
				detailError={documentProcessingDetailError}
				retryingJobId={retryingDocumentProcessingJobId}
				{isLayoutEditing}
				{isWidgetVisible}
				onRetry={retryFailedDocumentProcessingJob}
			/>
			<DocumentsInsights personList={[]} {isLayoutEditing} {isWidgetVisible} />
		</aside>
	</div>
</section>
