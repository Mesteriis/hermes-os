<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { DocumentProcessingJob } from '$lib/api';

	interface Props {
		jobs: DocumentProcessingJob[];
		isLoading: boolean;
		detailError: string;
		retryingJobId: string | null;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onRetry: (job: DocumentProcessingJob) => void;
	}

	let { jobs, isLoading, detailError, retryingJobId, isLayoutEditing, isWidgetVisible, onRetry }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="documents-processing-jobs" data-widget-hidden={!isWidgetVisible('documents-processing-jobs')}>
	<WidgetEditChrome widgetId="documents-processing-jobs" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Processing Jobs</h2>
		{#if isLoading}
			<div class="graph-strip-message"><span>Loading jobs.</span></div>
		{:else}
			{#each jobs.slice(0, 5) as job}
				<div class="job-row">
					<strong>{job.document_id}</strong>
					<span class={`status-chip ${job.status}`}>{job.status}</span>
					<small>{job.step} · {job.queued_at}</small>
					{#if job.status === 'failed'}
						<button type="button" onclick={() => void onRetry(job)} disabled={retryingJobId === job.document_id}>
							{retryingJobId === job.document_id ? 'Retrying...' : 'Retry'}
						</button>
					{/if}
				</div>
			{/each}
			{#if detailError}<p class="inline-error">{detailError}</p>{/if}
		{/if}
	</section>
</div>
