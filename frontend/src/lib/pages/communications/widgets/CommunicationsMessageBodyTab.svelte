<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { RenderedMessageContent } from '$lib/services/communications';
	import type {
		CommunicationAttachment,
		CommunicationMessageDetailItem,
		CommunicationMessageSummary,
		MailMessageInsight,
		MessageAnalyzeResponse,
		WorkflowActionKind,
		WorkflowState
	} from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	type DetailMessage = CommunicationMessageDetailItem | CommunicationMessageSummary;

	interface Props {
		selectedMessage: DetailMessage;
		selectedMessageContent: RenderedMessageContent;
		selectedMessageHtml: string;
		selectedMessageOriginalSrcdoc: string;
		selectedAttachments: CommunicationAttachment[];
		aiAnalysisResult: MessageAnalyzeResponse | null;
		mailMessageInsight: MailMessageInsight | null;
		isMailActionRunning: boolean;
		isMailStateTransitioning: boolean;
		mailActionStatus: string;
		mailActionError: string;
		workflowState: WorkflowState | null;
		importanceScore: number | null;
		summary: string | null;
		confidenceLabel: string;
		evidence: string[];
		labels: string[];
		readActionLabel: string;
		readActionIcon: string;
		messageTime: (msg: unknown) => string;
		attachmentIcon: (contentType: string) => string;
		onWorkflowAction: (action: WorkflowActionKind) => void;
		onToggleReadState: () => void;
		onGenerateAiReply: () => void;
		onExtractTasks: () => void;
		onExtractNotes: () => void;
		onTranslate: () => void;
		onAddLabel: (label: string) => void;
	}

	let {
		selectedMessage,
		selectedMessageContent,
		selectedMessageHtml,
		selectedMessageOriginalSrcdoc,
		selectedAttachments,
		aiAnalysisResult,
		mailMessageInsight,
		isMailActionRunning,
		isMailStateTransitioning,
		mailActionStatus,
		mailActionError,
		workflowState,
		importanceScore,
		summary,
		confidenceLabel,
		evidence,
		labels,
		readActionLabel,
		readActionIcon,
		messageTime,
		attachmentIcon,
		onWorkflowAction,
		onToggleReadState,
		onGenerateAiReply,
		onExtractTasks,
		onExtractNotes,
		onTranslate,
		onAddLabel
	}: Props = $props();

	let labelText = $state('');
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
