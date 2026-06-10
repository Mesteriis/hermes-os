<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type {
		CommunicationMessageDetail,
		CommunicationMessageSummary,
		MailMessageInsight,
		MessageAnalyzeResponse,
		WorkflowState
	} from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		selectedCommunication: CommunicationMessageSummary | null;
		selectedCommunicationDetail: CommunicationMessageDetail | null;
		aiAnalysisResult: MessageAnalyzeResponse | null;
		mailMessageInsight: MailMessageInsight | null;
		isMailActionRunning: boolean;
		mailActionStatus: string;
		mailActionError: string;
		isMailStateTransitioning: boolean;
		isAiAnswerSubmitting: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		handleWorkflowStateTransition: (msgId: string, state: WorkflowState) => void;
		askAiAboutSelectedMessage: () => void;
		onReply: () => void;
		onForward: () => void;
		onPin: () => void;
		onMute: () => void;
		onSnooze: () => void;
		onAddLabel: (label: string) => void;
		onExport: (format: 'md' | 'eml' | 'json') => void;
		onGenerateAiReply: () => void;
		onExtractTasks: () => void;
		onExtractNotes: () => void;
		onTranslate: () => void;
		messageTime: (msg: unknown) => string;
		senderLabel: (sender: string) => string;
		attachmentIcon: (contentType: string) => string;
		formatBytes: (bytes: number) => string;
	}

	let {
		selectedCommunication,
		selectedCommunicationDetail,
		aiAnalysisResult,
		mailMessageInsight,
		isMailActionRunning,
		mailActionStatus,
		mailActionError,
		isMailStateTransitioning,
		isAiAnswerSubmitting,
		isLayoutEditing,
		isWidgetVisible,
		handleWorkflowStateTransition,
		askAiAboutSelectedMessage,
		onReply,
		onForward,
		onPin,
		onMute,
		onSnooze,
		onAddLabel,
		onExport,
		onGenerateAiReply,
		onExtractTasks,
		onExtractNotes,
		onTranslate,
		messageTime,
		senderLabel,
		attachmentIcon,
		formatBytes
	}: Props = $props();

	let labelText = $state('');
	const selectedMessage = $derived(selectedCommunicationDetail?.message ?? selectedCommunication);
	const selectedMessageBody = $derived(selectedCommunicationDetail?.message.body_text ?? selectedCommunication?.body_text_preview ?? '');
	const selectedAttachments = $derived(selectedCommunicationDetail?.attachments ?? []);
	const messageMetadata = $derived((selectedMessage?.message_metadata ?? {}) as Record<string, unknown>);
	const labels = $derived(Array.isArray(messageMetadata.labels) ? (messageMetadata.labels as string[]) : []);
	const submitLabel = () => {
		const label = labelText.trim();
		if (!label) return;
		onAddLabel(label);
		labelText = '';
	};
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-message-detail" data-widget-hidden={!isWidgetVisible('communications-message-detail')}>
	<WidgetEditChrome widgetId="communications-message-detail" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel chat-pane">
		{#if selectedMessage}
			<header>
				<img src="/assets/hermes-reference-avatar.png" alt="" />
				<div><h2>{senderLabel(selectedMessage.sender)}</h2><p>{selectedMessage.subject}</p></div>
				<div class="chat-actions">
					<button type="button" onclick={() => void handleWorkflowStateTransition(selectedMessage.message_id, 'needs_action')} disabled={isMailStateTransitioning} title={_('Mark as Needs Action')}><Icon icon="tabler:alert-triangle" width="17" height="17" /></button>
					<button type="button" onclick={() => void handleWorkflowStateTransition(selectedMessage.message_id, 'waiting')} disabled={isMailStateTransitioning} title={_('Mark as Waiting')}><Icon icon="tabler:clock-hour-4" width="17" height="17" /></button>
					<button type="button" onclick={() => void handleWorkflowStateTransition(selectedMessage.message_id, 'done')} disabled={isMailStateTransitioning} title={_('Mark as Done')}><Icon icon="tabler:circle-check" width="17" height="17" /></button>
					<button type="button" onclick={() => void handleWorkflowStateTransition(selectedMessage.message_id, 'archived')} disabled={isMailStateTransitioning} title={_('Archive')}><Icon icon="tabler:archive" width="17" height="17" /></button>
					<button type="button" onclick={() => onReply()} title={_('Reply')}><Icon icon="tabler:arrow-back-up" width="17" height="17" /></button>
					<button type="button" onclick={() => onForward()} title={_('Forward')}><Icon icon="tabler:arrow-forward-up" width="17" height="17" /></button>
					<button type="button" onclick={() => void askAiAboutSelectedMessage()} disabled={isAiAnswerSubmitting}><Icon icon="tabler:sparkles" width="17" height="17" /></button>
				</div>
			</header>
			<div class="chat-body">
				<div class="mail-action-grid">
					<button type="button" onclick={onPin} disabled={isMailActionRunning}><Icon icon="tabler:pin" width="16" height="16" />{_('Pin')}</button>
					<button type="button" onclick={onMute} disabled={isMailActionRunning}><Icon icon="tabler:volume-off" width="16" height="16" />{_('Mute')}</button>
					<button type="button" onclick={onSnooze} disabled={isMailActionRunning}><Icon icon="tabler:alarm-snooze" width="16" height="16" />{_('Snooze')}</button>
					<button type="button" onclick={() => onExport('md')} disabled={isMailActionRunning}><Icon icon="tabler:file-export" width="16" height="16" />MD</button>
					<button type="button" onclick={() => onExport('eml')} disabled={isMailActionRunning}><Icon icon="tabler:mail-forward" width="16" height="16" />EML</button>
					<button type="button" onclick={() => onExport('json')} disabled={isMailActionRunning}><Icon icon="tabler:braces" width="16" height="16" />JSON</button>
				</div>
				<form class="inline-label-form" onsubmit={(event) => { event.preventDefault(); submitLabel(); }}>
					<input bind:value={labelText} placeholder={_('Add label')} autocomplete="off" />
					<button type="submit" disabled={isMailActionRunning || !labelText.trim()}><Icon icon="tabler:tag-plus" width="16" height="16" /></button>
				</form>
				{#if labels.length}
					<div class="mail-chip-row">{#each labels as label}<span>{label}</span>{/each}</div>
				{/if}
				{#if mailActionStatus}<p class="form-status">{_(mailActionStatus)}</p>{/if}
				{#if mailActionError}<p class="form-status error">{mailActionError}</p>{/if}
				{#if aiAnalysisResult && aiAnalysisResult.message_id === selectedMessage.message_id}
					<article class="ai-analysis-card">
						<strong><Icon icon="tabler:sparkles" width="16" height="16" />{_('AI Analysis')}</strong>
						{#if aiAnalysisResult.category}<p><em>{_('Category:')}</em> {aiAnalysisResult.category}</p>{/if}
						{#if aiAnalysisResult.summary}<p><em>{_('Summary:')}</em> {aiAnalysisResult.summary}</p>{/if}
						{#if aiAnalysisResult.importance_score != null}<p><em>{_('Importance:')}</em> {aiAnalysisResult.importance_score}/100</p>{/if}
						<p><em>{_('State:')}</em> <span class="state-badge {aiAnalysisResult.workflow_state}">{aiAnalysisResult.workflow_state}</span></p>
					</article>
				{/if}
				{#if mailMessageInsight?.messageId === selectedMessage.message_id}
					<section class="mail-insight-panel">
						<header><strong>{_('Message Intelligence')}</strong><span>{mailMessageInsight.language?.language ?? _('Unknown')}</span></header>
						{#if mailMessageInsight.explain?.reasons.length}
							<ul>{#each mailMessageInsight.explain.reasons as reason}<li>{reason}</li>{/each}</ul>
						{/if}
						<div class="mail-status-grid">
							<span>{_('Auth')}</span><strong>{mailMessageInsight.auth?.risk.risk_summary ?? _('Not checked')}</strong>
							<span>{_('Signature')}</span><strong>{mailMessageInsight.signature?.has_signature ? (mailMessageInsight.signature.signature_type ?? _('Detected')) : _('None detected')}</strong>
							<span>{_('Smart CC')}</span><strong>{mailMessageInsight.smartCc?.suggestions.join(', ') || _('No suggestions')}</strong>
						</div>
						<div class="mail-action-grid compact">
							<button type="button" onclick={onGenerateAiReply} disabled={isMailActionRunning}><Icon icon="tabler:message-spark" width="16" height="16" />{_('AI Reply')}</button>
							<button type="button" onclick={onExtractTasks} disabled={isMailActionRunning}><Icon icon="tabler:list-check" width="16" height="16" />{_('Extract Tasks')}</button>
							<button type="button" onclick={onExtractNotes} disabled={isMailActionRunning}><Icon icon="tabler:notes" width="16" height="16" />{_('Extract Notes')}</button>
							<button type="button" onclick={onTranslate} disabled={isMailActionRunning}><Icon icon="tabler:language" width="16" height="16" />{_('Translate')}</button>
						</div>
						{#if mailMessageInsight.aiReply?.body}
							<article class="mail-result-card"><strong>{mailMessageInsight.aiReply.subject ?? _('AI Reply')}</strong><p>{mailMessageInsight.aiReply.body}</p></article>
						{/if}
						{#if mailMessageInsight.tasks.length}
							<article class="mail-result-card"><strong>{_('Extracted Tasks')}</strong>{#each mailMessageInsight.tasks as task}<p>{task.title}</p>{/each}</article>
						{/if}
						{#if mailMessageInsight.notes.length}
							<article class="mail-result-card"><strong>{_('Extracted Notes')}</strong>{#each mailMessageInsight.notes as note}<p>{note.title}: {note.content}</p>{/each}</article>
						{/if}
						{#if mailMessageInsight.translation}
							<article class="mail-result-card"><strong>{_('Translation')}</strong><p>{mailMessageInsight.translation.text ?? mailMessageInsight.translation.reason}</p></article>
						{/if}
					</section>
				{/if}
				<div class="date-divider">{messageTime(selectedMessage)}</div>
				<article class="bubble inbound">
					<strong>{selectedMessage.subject}</strong><br />
					{selectedMessageBody}
					<time>{messageTime(selectedMessage)}</time>
				</article>
				{#each selectedAttachments as attachment}
					<article class="attachment-bubble">
						<Icon icon={attachmentIcon(attachment.content_type)} width="34" height="34" />
						<span>
							<strong>{attachment.filename ?? attachment.provider_attachment_id}</strong>
							<small>{formatBytes(attachment.size_bytes)} · {attachment.content_type} · {attachment.scan_status}</small>
						</span>
						<button type="button" disabled><Icon icon="tabler:download" width="16" height="16" /></button>
					</article>
				{/each}
			</div>
			<footer class="composer">
				<input placeholder={_('Reply to this message')} readonly onclick={() => onReply()} />
				<button type="button" disabled><Icon icon="tabler:paperclip" width="17" height="17" /></button>
				<button type="button" onclick={() => onReply()}><Icon icon="tabler:send" width="18" height="18" /></button>
			</footer>
		{:else}
			<div class="empty-panel fill">{_('Select a local message.')}</div>
		{/if}
	</section>
</div>
