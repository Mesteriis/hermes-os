<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import CommunicationsMessageAttachmentsTab from './CommunicationsMessageAttachmentsTab.svelte';
	import CommunicationsMessageBodyTab from './CommunicationsMessageBodyTab.svelte';
	import CommunicationsMessageHeadersTab from './CommunicationsMessageHeadersTab.svelte';
	import CommunicationsMessageRelatedTab from './CommunicationsMessageRelatedTab.svelte';
	import CommunicationsMessageTimelineTab from './CommunicationsMessageTimelineTab.svelte';
	import { apiBaseUrl } from '$lib/config';
	import { currentLocale, t } from '$lib/i18n';
	import type { RelatedCommunicationMessage, RenderedMessageContent } from '$lib/services/communications';
	import { originalMailSrcdoc, renderMessageContent } from '$lib/services/communications';
	import type { MessageContextTab } from '$lib/stores/communications';
	import type {
		CommunicationMessageDetail,
		CommunicationMessageSummary,
		MailMessageInsight,
		MessageAnalyzeResponse,
		WorkflowActionKind,
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
		onImportant: () => void;
		onMute: () => void;
		onTrash: () => void;
		onRestore: () => void;
		onSnooze: () => void;
		onAddLabel: (label: string) => void;
		onExport: (format: 'md' | 'eml' | 'json') => void;
		onGenerateAiReply: () => void;
		onExtractTasks: () => void;
		onExtractNotes: () => void;
		onTranslate: () => void;
		onWorkflowAction: (action: WorkflowActionKind) => void;
		onToggleReadState: () => void;
		onOpenInspector: (mode: 'context' | 'contact' | 'organization') => void;
		onSelectRelatedMessage: (messageId: string) => void;
		activeTab: MessageContextTab;
		onActiveTabChange: (tab: MessageContextTab) => void;
		messageTime: (msg: unknown) => string;
		messageContentText?: (value: string) => string;
		messageContentHtml?: (value: string) => RenderedMessageContent;
		relatedMessages: RelatedCommunicationMessage[];
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
		onImportant,
		onMute,
		onTrash,
		onRestore,
		onSnooze,
		onAddLabel,
		onExport,
		onGenerateAiReply,
		onExtractTasks,
		onExtractNotes,
		onTranslate,
		onWorkflowAction,
		onToggleReadState,
		onOpenInspector,
		onSelectRelatedMessage,
		activeTab,
		onActiveTabChange,
		messageTime,
		messageContentText = (value: string) => value.trim(),
		messageContentHtml = renderMessageContent,
		relatedMessages,
		senderLabel,
		attachmentIcon,
		formatBytes
	}: Props = $props();

	type HeaderEntry = {
		name: string;
		value: string;
	};

	type DetailMessage = NonNullable<CommunicationMessageDetail['message'] | CommunicationMessageSummary>;

	const selectedMessage = $derived(selectedCommunicationDetail?.message ?? selectedCommunication);
	const selectedMessageOriginalHtml = $derived.by(() => selectedCommunicationDetail?.message.body_html?.trim() ?? '');
	const selectedMessageOriginalSrcdoc = $derived.by(() =>
		selectedMessageOriginalHtml
			? originalMailSrcdoc(selectedMessageOriginalHtml, {
				messageId: selectedMessage?.message_id,
				apiBaseUrl
			})
			: ''
	);
	const selectedMessageRawBody = $derived.by(() => {
		const rawBody =
			selectedMessageOriginalHtml ||
			selectedCommunicationDetail?.message.body_text ||
			selectedCommunication?.body_text_preview ||
			'';
		return rawBody.trim();
	});
	const selectedMessageContent = $derived.by(() => {
		if (!selectedMessageRawBody) {
			return { html: '', mode: 'text' } satisfies RenderedMessageContent;
		}
		return messageContentHtml(selectedMessageRawBody);
	});
	const selectedMessageHtml = $derived.by(() =>
		selectedMessageContent.html || messageContentHtml(messageContentText(selectedMessageRawBody)).html
	);
	const selectedAttachments = $derived(selectedCommunicationDetail?.attachments ?? []);
	const messageMetadata = $derived((selectedMessage?.message_metadata ?? {}) as Record<string, unknown>);
	const labels = $derived(Array.isArray(messageMetadata.labels) ? (messageMetadata.labels as string[]) : []);
	const isImportant = $derived(messageMetadata.important === true);
	const importantActionLabel = $derived(isImportant ? 'Remove important' : 'Mark important');
	const importantActionIcon = $derived(isImportant ? 'tabler:star-off' : 'tabler:star');
	const headers = $derived.by(() => headersFromMetadata(messageMetadata));
	const workflowState = $derived(workflowStateFromMessage(selectedMessage));
	const readActionLabel = $derived(workflowState === 'reviewed' ? 'Mark unread' : 'Mark read');
	const readActionIcon = $derived(workflowState === 'reviewed' ? 'tabler:mail-opened' : 'tabler:mail-check');
	const selectedAnalysis = $derived(
		aiAnalysisResult && aiAnalysisResult.message_id === selectedMessage?.message_id ? aiAnalysisResult : null
	);
	const relatedCount = $derived(
		(mailMessageInsight?.tasks.length ?? 0) +
		(mailMessageInsight?.notes.length ?? 0) +
		(selectedAttachments.length ? 1 : 0) +
		relatedMessages.length
	);
	const importanceScore = $derived(selectedAnalysis ? selectedAnalysis.importance_score : importanceScoreFromMessage(selectedMessage));
	const summary = $derived(selectedAnalysis ? selectedAnalysis.summary : summaryFromMessage(selectedMessage));
	const confidenceLabel = $derived(selectedAnalysis?.confidence != null ? `${Math.round(selectedAnalysis.confidence * 100)}%` : _('Unknown'));
	const evidence = $derived(selectedAnalysis ? selectedAnalysis.evidence : mailMessageInsight?.explain?.reasons ?? []);

	function headersFromMetadata(metadata: Record<string, unknown>): HeaderEntry[] {
		const rawHeaders = metadata.headers;
		if (Array.isArray(rawHeaders)) {
			return rawHeaders
				.map((item) => {
					if (!item || typeof item !== 'object') return null;
					const header = item as Record<string, unknown>;
					const name = typeof header.name === 'string' ? header.name : '';
					const value = typeof header.value === 'string' ? header.value : '';
					if (!name.trim() || !value.trim()) return null;
					return { name, value };
				})
				.filter((item): item is HeaderEntry => item !== null);
		}
		if (rawHeaders && typeof rawHeaders === 'object') {
			return Object.entries(rawHeaders as Record<string, unknown>)
				.map(([name, value]) => ({ name, value: String(value) }))
				.filter((header) => header.name.trim() && header.value.trim());
		}
		return [];
	}

	function workflowStateFromMessage(message: DetailMessage | null): WorkflowState | null {
		const value = optionalMessageField(message, 'workflow_state');
		return isWorkflowState(value) ? value : null;
	}

	function importanceScoreFromMessage(message: DetailMessage | null): number | null {
		const value = optionalMessageField(message, 'importance_score');
		return typeof value === 'number' ? value : null;
	}

	function summaryFromMessage(message: DetailMessage | null): string | null {
		const value = optionalMessageField(message, 'ai_summary');
		return typeof value === 'string' ? value : null;
	}

	function optionalMessageField(message: DetailMessage | null, field: string): unknown {
		return message && field in message ? (message as Record<string, unknown>)[field] : null;
	}

	function isWorkflowState(value: unknown): value is WorkflowState {
		return (
			value === 'new' ||
			value === 'reviewed' ||
			value === 'needs_action' ||
			value === 'waiting' ||
			value === 'done' ||
			value === 'archived' ||
			value === 'muted' ||
			value === 'spam'
		);
	}
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-message-detail" data-widget-hidden={!isWidgetVisible('communications-message-detail')}>
	<WidgetEditChrome widgetId="communications-message-detail" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel chat-pane">
		{#if selectedMessage}
			<header class="workspace-message-header">
				<div class="message-title-line">
					<span class="round-icon emerald"><Icon icon="tabler:messages" width="18" height="18" /></span>
					<div>
						<h2>{selectedMessage.subject}</h2>
						<p>{_('Source')}: {selectedMessage.channel_kind} · {senderLabel(selectedMessage.sender)} → {selectedMessage.recipients.length} {_('participants')} · {messageTime(selectedMessage)}</p>
					</div>
				</div>
				<div class="chat-actions">
					<button type="button" onclick={() => onOpenInspector('context')} title={_('Context')}><Icon icon="tabler:network" width="17" height="17" /></button>
					<button type="button" onclick={() => onOpenInspector('contact')} title={_('Contact')}><Icon icon="tabler:user-circle" width="17" height="17" /></button>
					<button type="button" onclick={() => onOpenInspector('organization')} title={_('Organization')}><Icon icon="tabler:building" width="17" height="17" /></button>
					<button type="button" onclick={() => void askAiAboutSelectedMessage()} disabled={isAiAnswerSubmitting} title={_('Ask AI')}><Icon icon="tabler:sparkles" width="17" height="17" /></button>
				</div>
			</header>
			<nav class="message-context-tabs">
				<button type="button" class:active={activeTab === 'message'} onclick={() => onActiveTabChange('message')}>{_('Message')}</button>
				<button type="button" class:active={activeTab === 'attachments'} onclick={() => onActiveTabChange('attachments')}>{_('Attachments')} <em>{selectedAttachments.length}</em></button>
				<button type="button" class:active={activeTab === 'headers'} onclick={() => onActiveTabChange('headers')}>{_('Headers')}</button>
				<button type="button" class:active={activeTab === 'related'} onclick={() => onActiveTabChange('related')}>{_('Related')} <em>{relatedCount}</em></button>
				<button type="button" class:active={activeTab === 'timeline'} onclick={() => onActiveTabChange('timeline')}>{_('Timeline')}</button>
			</nav>
			<div class="chat-body">
				{#if activeTab === 'message'}
					<CommunicationsMessageBodyTab
						{selectedMessage}
						{selectedMessageContent}
						{selectedMessageHtml}
						{selectedMessageOriginalSrcdoc}
						{selectedAttachments}
						{aiAnalysisResult}
						{mailMessageInsight}
						{isMailActionRunning}
						{isMailStateTransitioning}
						{mailActionStatus}
						{mailActionError}
						{workflowState}
						{importanceScore}
						{summary}
						{confidenceLabel}
						{evidence}
						{labels}
						{readActionLabel}
						{readActionIcon}
						{messageTime}
						{attachmentIcon}
						{onWorkflowAction}
						{onToggleReadState}
						{onGenerateAiReply}
						{onExtractTasks}
						{onExtractNotes}
						{onTranslate}
						{onAddLabel}
					/>
				{:else if activeTab === 'attachments'}
					<CommunicationsMessageAttachmentsTab {selectedAttachments} {attachmentIcon} {formatBytes} />
				{:else if activeTab === 'headers'}
					<CommunicationsMessageHeadersTab {headers} />
				{:else if activeTab === 'related'}
					<CommunicationsMessageRelatedTab
						{selectedMessage}
						{relatedMessages}
						{mailMessageInsight}
						{isMailActionRunning}
						{importantActionIcon}
						{importantActionLabel}
						{messageTime}
						{senderLabel}
						{onSelectRelatedMessage}
						{onPin}
						{onImportant}
						{onMute}
						{onTrash}
						{onRestore}
						{onSnooze}
						{onExport}
					/>
				{:else}
					<CommunicationsMessageTimelineTab {selectedMessage} {messageTime} />
				{/if}
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
