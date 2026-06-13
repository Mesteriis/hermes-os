<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
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

	let labelText = $state('');
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
	const workflowState = $derived(selectedMessage && 'workflow_state' in selectedMessage ? selectedMessage.workflow_state : null);
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
	const importanceScore = $derived(selectedAnalysis ? selectedAnalysis.importance_score : selectedMessage && 'importance_score' in selectedMessage ? selectedMessage.importance_score ?? null : null);
	const summary = $derived(selectedAnalysis ? selectedAnalysis.summary : selectedMessage && 'ai_summary' in selectedMessage ? selectedMessage.ai_summary ?? null : null);
	const confidenceLabel = $derived(selectedAnalysis?.confidence != null ? `${Math.round(selectedAnalysis.confidence * 100)}%` : _('Unknown'));
	const evidence = $derived(selectedAnalysis ? selectedAnalysis.evidence : mailMessageInsight?.explain?.reasons ?? []);
	let mailRenderHost = $state<HTMLElement | null>(null);
	let originalMailFrame = $state<HTMLIFrameElement | null>(null);
	let mailShadowHost: HTMLElement | null = null;
	let mailShadowRoot: ShadowRoot | null = null;

	$effect(() => {
		const host = mailRenderHost;
		const html = selectedMessageHtml;
		const mode = selectedMessageContent.mode;
		if (!host) return;
		if (mailShadowHost !== host) {
			mailShadowHost = host;
			mailShadowRoot = host.shadowRoot ?? host.attachShadow({ mode: 'open' });
		}
		if (!mailShadowRoot) return;
		mailShadowRoot.replaceChildren();
		const styleElement = document.createElement('style');
		styleElement.textContent = mailBodyShadowCss;
		const bodyElement = document.createElement('article');
		bodyElement.className = `mail-body ${mode === 'html' ? 'html-content' : 'text-content'}`;
		bodyElement.innerHTML = html;
		mailShadowRoot.append(styleElement, bodyElement);
	});

	$effect(() => {
		const frame = originalMailFrame;
		const srcdoc = selectedMessageOriginalSrcdoc;
		if (!frame || !srcdoc) return;

		const resize = () => {
			resizeOriginalMailFrame(frame);
			window.setTimeout(() => resizeOriginalMailFrame(frame), 250);
			window.setTimeout(() => resizeOriginalMailFrame(frame), 1000);
		};

		resize();
		frame.addEventListener('load', resize);
		return () => frame.removeEventListener('load', resize);
	});

	const submitLabel = () => {
		const label = labelText.trim();
		if (!label) return;
		onAddLabel(label);
		labelText = '';
	};

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

	function resizeOriginalMailFrame(frame: HTMLIFrameElement): void {
		try {
			const doc = frame.contentDocument;
			if (!doc) return;

			const body = doc.body;
			const root = doc.documentElement;
			const height = Math.max(
				body?.scrollHeight ?? 0,
				body?.offsetHeight ?? 0,
				root?.scrollHeight ?? 0,
				root?.offsetHeight ?? 0,
				320
			);
			frame.style.height = `${Math.min(Math.max(height, 320), 6000)}px`;
		} catch {
			frame.style.height = '720px';
		}
	}

	const mailBodyShadowCss = `
		:host {
			all: initial;
			display: block;
			width: 100%;
			color-scheme: light;
			contain: content;
		}

		.mail-body {
			box-sizing: border-box;
			display: block;
			width: 100%;
			min-height: 96px;
			overflow: auto;
			border: 1px solid #d8dee4;
			border-radius: 10px;
			background: #ffffff;
			color: #1f2933;
			padding: 20px;
			font-family: Arial, Helvetica, sans-serif;
			font-size: 14px;
			line-height: 1.5;
			overflow-wrap: break-word;
		}

		.mail-body *,
		.mail-body *::before,
		.mail-body *::after {
			box-sizing: border-box;
		}

		.mail-body p,
		.mail-body div,
		.mail-body blockquote,
		.mail-body ul,
		.mail-body ol,
		.mail-body table,
		.mail-body pre,
		.mail-body h1,
		.mail-body h2,
		.mail-body h3,
		.mail-body h4,
		.mail-body h5,
		.mail-body h6 {
			margin: 0;
		}

		.mail-body p + p,
		.mail-body div + div,
		.mail-body p + div,
		.mail-body div + p,
		.mail-body blockquote + p,
		.mail-body table + p,
		.mail-body pre + p {
			margin-top: 12px;
		}

		.mail-body h1,
		.mail-body h2,
		.mail-body h3,
		.mail-body h4,
		.mail-body h5,
		.mail-body h6 {
			color: #111827;
			font-weight: 700;
			line-height: 1.25;
		}

		.mail-body h1 { font-size: 24px; }
		.mail-body h2 { font-size: 20px; }
		.mail-body h3 { font-size: 18px; }
		.mail-body h4,
		.mail-body h5,
		.mail-body h6 { font-size: 16px; }

		.mail-body a {
			color: #0645ad;
			text-decoration: underline;
		}

		.mail-body a:hover {
			color: #043a8b;
		}

		.mail-body ul,
		.mail-body ol {
			padding-left: 22px;
		}

		.mail-body blockquote {
			border-left: 3px solid #d8dee4;
			color: #57606a;
			padding-left: 12px;
		}

		.mail-body table {
			max-width: 100%;
			border-collapse: collapse;
		}

		.mail-body td,
		.mail-body th {
			border: 0;
			padding: 4px;
			text-align: left;
			vertical-align: top;
		}

		.mail-body pre,
		.mail-body code {
			border-radius: 4px;
			background: #f6f8fa;
			font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
		}

		.mail-body pre {
			overflow-x: auto;
			padding: 10px;
			white-space: pre-wrap;
		}

		.mail-body code {
			padding: 1px 4px;
		}

		.mail-image-placeholder {
			display: inline-flex;
			max-width: 100%;
			border: 1px solid #d8dee4;
			border-radius: 4px;
			padding: 3px 7px;
			color: #57606a;
			font-size: 12px;
		}
	`;
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
					<article class="message-content-surface">
						{#if selectedMessageOriginalSrcdoc}
							<iframe
								class="mail-original-frame"
								title={_('Original message')}
								sandbox="allow-same-origin allow-popups allow-popups-to-escape-sandbox"
								referrerpolicy="no-referrer"
								bind:this={originalMailFrame}
								srcdoc={selectedMessageOriginalSrcdoc}
							></iframe>
						{:else}
							<div class="mail-render-host" data-render-mode={selectedMessageContent.mode} bind:this={mailRenderHost}></div>
						{/if}
						<time>{messageTime(selectedMessage)}</time>
					</article>
					{#if selectedAttachments.length}
						<div class="attachment-strip">
							{#each selectedAttachments.slice(0, 3) as attachment}
								<span><Icon icon={attachmentIcon(attachment.content_type)} width="16" height="16" />{attachment.filename ?? attachment.provider_attachment_id}</span>
							{/each}
						</div>
					{/if}
					<section class="mail-insight-panel message-intelligence-card">
						<header><strong>{_('Message Intelligence')}</strong><span>{aiAnalysisResult?.source ?? _('local')}</span></header>
						<div class="mail-status-grid intelligence-grid">
							<span>{_('Importance')}</span><strong>{importanceScore != null ? `${importanceScore}/100` : _('Unknown')}</strong>
							<span>{_('Summary')}</span><strong>{summary ?? _('Not available')}</strong>
							<span>{_('Needs Reply')}</span><strong>{workflowState === 'needs_action' ? _('Yes') : _('No')}</strong>
							<span>{_('Confidence')}</span><strong>{confidenceLabel}</strong>
							<span>{_('Source')}</span><strong>{aiAnalysisResult?.source ?? _('local_heuristic')}</strong>
							<span>{_('Evidence')}</span><strong>{evidence.length ? evidence.slice(0, 2).join(' · ') : _('Not available')}</strong>
						</div>
						<div class="workflow-action-row">
							<button type="button" onclick={() => onWorkflowAction('reply')} disabled={isMailActionRunning}><Icon icon="tabler:arrow-back-up" width="16" height="16" />{_('Reply')}</button>
							<button type="button" onclick={() => onWorkflowAction('create_task')} disabled={isMailActionRunning}><Icon icon="tabler:square-check" width="16" height="16" />{_('Create Task')}</button>
							<button type="button" onclick={() => onWorkflowAction('create_note')} disabled={isMailActionRunning}><Icon icon="tabler:notes" width="16" height="16" />{_('Create Note')}</button>
							<button type="button" onclick={() => onWorkflowAction('create_event')} disabled={isMailActionRunning}><Icon icon="tabler:calendar-plus" width="16" height="16" />{_('Create Event')}</button>
							<button type="button" onclick={() => onWorkflowAction('link_document')} disabled={isMailActionRunning}><Icon icon="tabler:file-symlink" width="16" height="16" />{_('Link Document')}</button>
							<button type="button" onclick={() => onWorkflowAction('archive')} disabled={isMailActionRunning}><Icon icon="tabler:archive" width="16" height="16" />{_('Archive')}</button>
							<button type="button" onclick={onToggleReadState} disabled={isMailStateTransitioning}><Icon icon={readActionIcon} width="16" height="16" />{_(readActionLabel)}</button>
						</div>
						{#if mailMessageInsight?.messageId === selectedMessage.message_id}
							<div class="mail-action-grid compact">
								<button type="button" onclick={onGenerateAiReply} disabled={isMailActionRunning}><Icon icon="tabler:message-spark" width="16" height="16" />{_('AI Reply')}</button>
								<button type="button" onclick={onExtractTasks} disabled={isMailActionRunning}><Icon icon="tabler:list-check" width="16" height="16" />{_('Extract Tasks')}</button>
								<button type="button" onclick={onExtractNotes} disabled={isMailActionRunning}><Icon icon="tabler:notes" width="16" height="16" />{_('Extract Notes')}</button>
								<button type="button" onclick={onTranslate} disabled={isMailActionRunning}><Icon icon="tabler:language" width="16" height="16" />{_('Translate')}</button>
							</div>
						{/if}
					</section>
					<form class="inline-label-form" onsubmit={(event) => { event.preventDefault(); submitLabel(); }}>
						<input bind:value={labelText} placeholder={_('Add label')} autocomplete="off" />
						<button type="submit" disabled={isMailActionRunning || !labelText.trim()}><Icon icon="tabler:tag-plus" width="16" height="16" /></button>
					</form>
					{#if labels.length}<div class="mail-chip-row">{#each labels as label}<span>{label}</span>{/each}</div>{/if}
					{#if mailActionStatus}<p class="form-status">{_(mailActionStatus)}</p>{/if}
					{#if mailActionError}<p class="form-status error">{mailActionError}</p>{/if}
				{:else if activeTab === 'attachments'}
					{#each selectedAttachments as attachment}
						<article class="attachment-bubble">
							<Icon icon={attachmentIcon(attachment.content_type)} width="34" height="34" />
							<span>
								<strong>{attachment.filename ?? attachment.provider_attachment_id}</strong>
								<small>{formatBytes(attachment.size_bytes)} · {attachment.content_type} · {attachment.scan_status}</small>
							</span>
							<button type="button" disabled><Icon icon="tabler:download" width="16" height="16" /></button>
						</article>
					{:else}
						<div class="empty-panel">{_('No attachments')}</div>
					{/each}
				{:else if activeTab === 'headers'}
					<div class="headers-table">
						{#each headers.slice(0, 24) as header}
							<span>{header.name}</span><strong>{header.value}</strong>
						{:else}
							<p>{_('No headers available')}</p>
						{/each}
					</div>
				{:else if activeTab === 'related'}
					{#if relatedMessages.length}
						<section class="related-link-list">
							<h3>{_('Related conversations')}</h3>
							{#each relatedMessages as relatedMessage}
								<button type="button" onclick={() => onSelectRelatedMessage(relatedMessage.message_id)}>
									<span>
										<Icon icon={relatedMessage.relation === 'same_conversation' ? 'tabler:messages' : 'tabler:user-circle'} width="15" height="15" />
										{relatedMessage.relation === 'same_conversation' ? _('Same conversation') : _('Same contact')}
									</span>
									<strong>{relatedMessage.subject}</strong>
									<small>{senderLabel(relatedMessage.sender)} · {messageTime(relatedMessage)}</small>
								</button>
							{/each}
						</section>
					{/if}
					<div class="related-workspace">
						<button type="button" onclick={onPin} disabled={isMailActionRunning}><Icon icon="tabler:pin" width="16" height="16" />{_('Pin')}</button>
						<button type="button" onclick={onImportant} disabled={isMailActionRunning}><Icon icon={importantActionIcon} width="16" height="16" />{_(importantActionLabel)}</button>
						<button type="button" onclick={onMute} disabled={isMailActionRunning}><Icon icon="tabler:volume-off" width="16" height="16" />{_('Mute')}</button>
						{#if selectedMessage.local_state === 'trash'}
							<button type="button" onclick={onRestore} disabled={isMailActionRunning}><Icon icon="tabler:restore" width="16" height="16" />{_('Restore')}</button>
						{:else}
							<button type="button" onclick={onTrash} disabled={isMailActionRunning}><Icon icon="tabler:trash" width="16" height="16" />{_('Delete')}</button>
						{/if}
						<button type="button" onclick={onSnooze} disabled={isMailActionRunning}><Icon icon="tabler:alarm-snooze" width="16" height="16" />{_('Snooze')}</button>
						<button type="button" onclick={() => onExport('md')} disabled={isMailActionRunning}><Icon icon="tabler:file-export" width="16" height="16" />MD</button>
						<button type="button" onclick={() => onExport('eml')} disabled={isMailActionRunning}><Icon icon="tabler:mail-forward" width="16" height="16" />EML</button>
						<button type="button" onclick={() => onExport('json')} disabled={isMailActionRunning}><Icon icon="tabler:braces" width="16" height="16" />JSON</button>
					</div>
					{#if mailMessageInsight?.aiReply?.body}
						<article class="mail-result-card"><strong>{mailMessageInsight.aiReply.subject ?? _('AI Reply')}</strong><p>{mailMessageInsight.aiReply.body}</p></article>
					{/if}
					{#if mailMessageInsight?.tasks.length}
						<article class="mail-result-card"><strong>{_('Extracted Tasks')}</strong>{#each mailMessageInsight.tasks as task}<p>{task.title}</p>{/each}</article>
					{/if}
					{#if mailMessageInsight?.notes.length}
						<article class="mail-result-card"><strong>{_('Extracted Notes')}</strong>{#each mailMessageInsight.notes as note}<p>{note.title}: {note.content}</p>{/each}</article>
					{/if}
				{:else}
					<div class="timeline-list">
						<p><Icon icon="tabler:circle-dot" width="14" height="14" />{_('Received')} · {messageTime(selectedMessage)}</p>
						<p><Icon icon="tabler:circle-dot" width="14" height="14" />{_('Projected locally')} · {messageTime(selectedMessage)}</p>
					</div>
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
