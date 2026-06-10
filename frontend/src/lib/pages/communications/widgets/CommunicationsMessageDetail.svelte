<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		selectedCommunication: unknown | null;
		selectedCommunicationDetail: unknown | null;
		aiAnalysisResult: unknown | null;
		isMailStateTransitioning: boolean;
		isAiAnswerSubmitting: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		handleWorkflowStateTransition: (msgId: string, state: string) => void;
		askAiAboutSelectedMessage: () => void;
		messageTime: (msg: unknown) => string;
		senderLabel: (sender: string) => string;
		attachmentIcon: (contentType: string) => string;
		formatBytes: (bytes: number) => string;
	}

	let {
		selectedCommunication,
		selectedCommunicationDetail,
		aiAnalysisResult,
		isMailStateTransitioning,
		isAiAnswerSubmitting,
		isLayoutEditing,
		isWidgetVisible,
		handleWorkflowStateTransition,
		askAiAboutSelectedMessage,
		messageTime,
		senderLabel,
		attachmentIcon,
		formatBytes
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-message-detail" data-widget-hidden={!isWidgetVisible('communications-message-detail')}>
	<WidgetEditChrome widgetId="communications-message-detail" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel chat-pane">
		{#if selectedCommunication}
			<header>
				<img src="/assets/hermes-reference-avatar.png" alt="" />
				<div><h2>{senderLabel((selectedCommunication as Record<string, unknown>).sender as string)}</h2><p>{(selectedCommunication as Record<string, unknown>).subject as string}</p></div>
				<div class="chat-actions">
					<button type="button" onclick={() => void handleWorkflowStateTransition((selectedCommunication as Record<string, unknown>).message_id as string, 'needs_action')} disabled={isMailStateTransitioning} title="Mark as Needs Action"><Icon icon="tabler:alert-triangle" width="17" height="17" /></button>
					<button type="button" onclick={() => void handleWorkflowStateTransition((selectedCommunication as Record<string, unknown>).message_id as string, 'waiting')} disabled={isMailStateTransitioning} title="Mark as Waiting"><Icon icon="tabler:clock-hour-4" width="17" height="17" /></button>
					<button type="button" onclick={() => void handleWorkflowStateTransition((selectedCommunication as Record<string, unknown>).message_id as string, 'done')} disabled={isMailStateTransitioning} title="Mark as Done"><Icon icon="tabler:circle-check" width="17" height="17" /></button>
					<button type="button" onclick={() => void handleWorkflowStateTransition((selectedCommunication as Record<string, unknown>).message_id as string, 'archived')} disabled={isMailStateTransitioning} title="Archive"><Icon icon="tabler:archive" width="17" height="17" /></button>
					<button type="button" onclick={() => void askAiAboutSelectedMessage()} disabled={isAiAnswerSubmitting}><Icon icon="tabler:sparkles" width="17" height="17" /></button>
				</div>
			</header>
			<div class="chat-body">
				{#if aiAnalysisResult && (aiAnalysisResult as Record<string, unknown>).message_id === (selectedCommunication as Record<string, unknown>).message_id}
					<article class="ai-analysis-card">
						<strong><Icon icon="tabler:sparkles" width="16" height="16" />AI Analysis</strong>
						{#if (aiAnalysisResult as Record<string, unknown>).category}<p><em>Category:</em> {(aiAnalysisResult as Record<string, unknown>).category as string}</p>{/if}
						{#if (aiAnalysisResult as Record<string, unknown>).summary}<p><em>Summary:</em> {(aiAnalysisResult as Record<string, unknown>).summary as string}</p>{/if}
						{#if (aiAnalysisResult as Record<string, unknown>).importance_score != null}<p><em>Importance:</em> {(aiAnalysisResult as Record<string, unknown>).importance_score as number}/100</p>{/if}
						<p><em>State:</em> <span class="state-badge {(aiAnalysisResult as Record<string, unknown>).workflow_state as string}">{(aiAnalysisResult as Record<string, unknown>).workflow_state as string}</span></p>
					</article>
				{/if}
				<div class="date-divider">{messageTime((selectedCommunicationDetail as Record<string, unknown>)?.message ?? selectedCommunication)}</div>
				<article class="bubble inbound">
					<strong>{(selectedCommunication as Record<string, unknown>).subject as string}</strong><br />
					{(selectedCommunicationDetail as Record<string, unknown>)?.message ? ((selectedCommunicationDetail as Record<string, unknown>).message as Record<string, unknown>).body_text : (selectedCommunication as Record<string, unknown>).body_text_preview as string}
					<time>{messageTime((selectedCommunicationDetail as Record<string, unknown>)?.message ?? selectedCommunication)}</time>
				</article>
				{#each ((selectedCommunicationDetail as Record<string, unknown>)?.attachments as unknown[] ?? []) as attachment}
					<article class="attachment-bubble">
						<Icon icon={attachmentIcon((attachment as Record<string, unknown>).content_type as string)} width="34" height="34" />
						<span>
							<strong>{(attachment as Record<string, unknown>).filename as string ?? (attachment as Record<string, unknown>).provider_attachment_id as string}</strong>
							<small>{formatBytes((attachment as Record<string, unknown>).size_bytes as number)} · {(attachment as Record<string, unknown>).content_type as string} · {(attachment as Record<string, unknown>).scan_status as string}</small>
						</span>
						<button type="button" disabled><Icon icon="tabler:download" width="16" height="16" /></button>
					</article>
				{/each}
			</div>
			<footer class="composer">
				<input placeholder="Sending is not available yet" disabled />
				<button type="button" disabled><Icon icon="tabler:paperclip" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:send" width="18" height="18" /></button>
			</footer>
		{:else}
			<div class="empty-panel fill">Select a local message.</div>
		{/if}
	</section>
</div>
