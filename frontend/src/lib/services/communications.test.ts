import { describe, expect, it, vi } from 'vitest';
import type { CommunicationMessageSummary, ProviderAccount } from '$lib/api';
import {
	buildComposeDraftPayload,
	buildForwardComposeForm,
	buildMailAccountOptions,
	buildReplyComposeForm,
	buildWorkflowActionRequest,
	conversationPreview,
	downloadMessageExport,
	messageContentText,
	nextReadWorkflowState,
	originalMailSrcdoc,
	relatedMessagesForSelection,
	remoteMailImageProxyUrl,
	renderMessageContent,
	safeMessageExportFilename,
	summarizeMailResourceSnapshot,
	filterMessagesForWorkbench,
	handleDeleteDraft,
	handleSaveDraft,
	sendCapabilityForAccount
} from './communications';

function providerAccount(overrides: Partial<ProviderAccount>): ProviderAccount {
	return {
		account_id: 'account-primary',
		provider_kind: 'imap',
		display_name: 'Primary Mail',
		external_account_id: 'sender@example.com',
		config: {
			smtp_host: 'smtp.example.com',
			smtp_port: 587,
			smtp_starttls: true
		},
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T00:00:00Z',
		...overrides
	};
}

function message(
	overrides: Partial<CommunicationMessageSummary> & Record<string, unknown>
): CommunicationMessageSummary & Record<string, unknown> {
	return {
		message_id: 'message-primary',
		raw_record_id: 'raw-primary',
		account_id: 'imap-primary',
		provider_record_id: 'provider-primary',
		subject: 'Quarterly Alpha Contract',
		sender: 'Alice <alice@example.com>',
		recipients: ['sender@example.com'],
		body_text_preview: 'The alpha renewal needs legal review.',
		occurred_at: '2026-06-10T10:00:00Z',
		projected_at: '2026-06-10T10:01:00Z',
		channel_kind: 'email',
		conversation_id: null,
		sender_display_name: 'Alice',
		delivery_state: 'delivered',
		message_metadata: {},
		attachment_count: 0,
		local_state: 'active',
		local_state_changed_at: null,
		...overrides
	};
}

describe('mail workbench helpers', () => {
	it('filters messages by account and search terms for local workbench state', () => {
		const messages = [
			message({ account_id: 'imap-primary' }),
			message({
				message_id: 'message-secondary',
				account_id: 'icloud-primary',
				subject: 'Beta Invoice',
				body_text_preview: 'Paid already'
			})
		];

		expect(filterMessagesForWorkbench(messages, 'imap-primary', 'alpha legal')).toEqual([
			messages[0]
		]);
		expect(filterMessagesForWorkbench(messages, 'imap-primary', 'beta')).toEqual([]);
	});

	it('gates send capability by provider kind and SMTP config', () => {
		expect(sendCapabilityForAccount(providerAccount({ provider_kind: 'imap' }))).toMatchObject({
			canSend: true,
			transport: 'smtp'
		});
		expect(sendCapabilityForAccount(providerAccount({ provider_kind: 'icloud' }))).toMatchObject({
			canSend: true,
			transport: 'smtp'
		});
		expect(sendCapabilityForAccount(providerAccount({ provider_kind: 'gmail' }))).toMatchObject({
			canSend: false,
			reason: 'Gmail send is unavailable until OAuth send scopes are configured'
		});
		expect(
			sendCapabilityForAccount(providerAccount({ config: { connected_services: ['mail'] } }))
		).toMatchObject({
			canSend: false,
			reason: 'Reconnect this account to enable SMTP send'
		});
	});

	it('builds account chips with capability metadata', () => {
		const options = buildMailAccountOptions([
			providerAccount({ account_id: 'imap-primary', display_name: 'Work IMAP' }),
			providerAccount({ account_id: 'gmail-primary', provider_kind: 'gmail', display_name: 'Gmail' })
		]);

		expect(options).toEqual([
			expect.objectContaining({ accountId: 'imap-primary', label: 'Work IMAP', canSend: true }),
			expect.objectContaining({ accountId: 'gmail-primary', label: 'Gmail', canSend: false })
		]);
	});

	it('maps explicit read and unread actions onto workflow states without erasing action states', () => {
		expect(nextReadWorkflowState('new')).toBe('reviewed');
		expect(nextReadWorkflowState('reviewed')).toBe('new');
		expect(nextReadWorkflowState('needs_action')).toBe('reviewed');
		expect(nextReadWorkflowState('waiting')).toBe('reviewed');
		expect(nextReadWorkflowState(null)).toBe('reviewed');
	});

	it('sanitizes exported message filenames before browser download', () => {
		expect(safeMessageExportFilename('message:../invoice?.eml')).toBe('message_.._invoice_.eml');
		expect(safeMessageExportFilename('')).toBe('message-export.eml');
		expect(
			downloadMessageExport({
				content_type: 'message/rfc822',
				content: 'Subject: Test\r\n\r\nBody',
				filename: 'message.eml'
			})
		).toBe(false);
	});

	it('prepares reply and forward compose state from a selected message', () => {
		const selected = message({});

		expect(buildReplyComposeForm(selected, 'imap-primary')).toMatchObject({
			mode: 'reply',
			account_id: 'imap-primary',
			to_text: 'alice@example.com',
			subject: 'Re: Quarterly Alpha Contract',
			in_reply_to: 'provider-primary',
			references: ['provider-primary']
		});
		expect(buildForwardComposeForm(selected, 'imap-primary').body).toContain(
			'The alpha renewal needs legal review.'
		);
	});

	it('builds intelligent previews without HTML, CSS, or raw header noise', () => {
		const openStyle = '<' + 'style>';
		const closeStyle = '</' + 'style>';
		const noisyPreview = [
			openStyle,
			'body { margin: 0; font-family: Arial; }',
			closeStyle,
			'Content-Type: text/html',
			'<div>Technical assignment and access keys</div>'
		].join('\n');

		expect(conversationPreview(message({ body_text_preview: noisyPreview }))).toBe(
			'Technical assignment and access keys'
		);
		expect(
			conversationPreview(
				message({
					ai_summary: 'Спортмастер прислал техническое задание и ключи доступа.',
					body_text_preview: noisyPreview
				})
			)
		).toBe('Спортмастер прислал техническое задание и ключи доступа.');
		expect(
			conversationPreview(
				message({
					subject: 'Security alert',
					body_text_preview: 'margin:0; padding:0; font-family:Arial;'
				})
			)
		).toBe('Security alert');
		expect(
			conversationPreview(
				message({
					subject: 'CSS newsletter',
					body_text_preview:
						'* { margin: 0; padding: 0; font-family: Verdana; } body { width: 100%; }\nUseful newsletter text'
				})
			)
		).toBe('Useful newsletter text');
		expect(messageContentText('Hello &zwnj; &#8199;world')).toBe('Hello world');
	});

	it('finds related messages by confirmed contact or conversation identity', () => {
		const selected = message({
			message_id: 'message-selected',
			sender: 'Marta <marta@example.com>',
			conversation_id: 'thread-1'
		});
		const sameContact = message({
			message_id: 'message-same-contact',
			sender: 'Marta <marta@example.com>',
			conversation_id: 'thread-2',
			occurred_at: '2026-06-10T11:00:00Z'
		});
		const sameConversation = message({
			message_id: 'message-same-thread',
			sender: 'Other <other@example.com>',
			conversation_id: 'thread-1',
			occurred_at: '2026-06-10T12:00:00Z'
		});
		const unrelated = message({
			message_id: 'message-unrelated',
			sender: 'Other <other@example.com>',
			conversation_id: 'thread-3'
		});

		expect(relatedMessagesForSelection([selected, sameContact, sameConversation, unrelated], selected)).toEqual([
			expect.objectContaining({ message_id: 'message-same-thread', relation: 'same_conversation' }),
			expect.objectContaining({ message_id: 'message-same-contact', relation: 'same_contact' })
		]);
	});

	it('renders message content as sanitized email HTML instead of raw tags', () => {
		const openStyle = '<' + 'style>';
		const closeStyle = '</' + 'style>';
		const openScript = '<' + 'script>';
		const closeScript = '</' + 'script>';
		const html = [
			'Content-Type: text/html',
			'',
			`<html><head>${openStyle}body { margin: 0; color: red; }${closeStyle}</head>`,
			'<body>',
			'<div onclick="alert(1)">Добрый день!</div>',
			'<p>Техническое <strong>задание</strong> и <a href="https://example.com/doc">документ</a></p>',
			'<a href="javascript:alert(1)">bad link</a>',
			`${openScript}alert(1)${closeScript}`,
			'</body></html>'
		].join('\n');

		const rendered = renderMessageContent(html);

		expect(rendered.mode).toBe('html');
		expect(rendered.html).toContain('<div>Добрый день!</div>');
		expect(rendered.html).toContain('<strong>задание</strong>');
		expect(rendered.html).toContain('href="https://example.com/doc"');
		expect(rendered.html).toContain('bad link</a>');
		expect(rendered.html).not.toContain('<style');
		expect(rendered.html).not.toContain('margin: 0');
		expect(rendered.html).not.toContain('<script');
		expect(rendered.html).not.toContain('onclick');
		expect(rendered.html).not.toContain('javascript:');
	});

	it('renders plain message content as escaped paragraphs', () => {
		const rendered = renderMessageContent('Hello <script>alert(1)</script>\n\nSecond line &amp; details');

		expect(rendered.mode).toBe('text');
		expect(rendered.html).toBe(
			'<p>Hello &lt;script&gt;alert(1)&lt;/script&gt;</p><p>Second line &amp; details</p>'
		);
	});

	it('renders escaped html message bodies after decoding entities', () => {
		const rendered = renderMessageContent('&lt;p&gt;Hello&nbsp;&lt;b&gt;world&lt;/b&gt;&lt;/p&gt;');

		expect(rendered.mode).toBe('html');
		expect(rendered.html).toBe('<p>Hello <strong>world</strong></p>');
	});

	it('keeps rich mail link labels instead of visible tracking urls', () => {
		const rendered = renderMessageContent(
			'<p>Footer</p><a href="https://click.email.feverup.com/?qs=tracking-token">Privacy policy</a>' +
				'<a href="https://click.email.feverup.com/unsub_center.aspx?qs=tracking-token">Unsubscribe</a>'
		);

		expect(rendered.mode).toBe('html');
		expect(rendered.html).toContain('href="https://click.email.feverup.com/?qs=tracking-token"');
		expect(rendered.html).toContain('>Privacy policy</a>');
		expect(rendered.html).toContain('>Unsubscribe</a>');
		expect(rendered.html).not.toContain('>https://click.email.feverup.com');
	});

	it('rewrites original remote mail images through the scoped image proxy', () => {
		const styleAttr = 'st' + 'yle';
		const html = [
			'<html><head></head><body>',
			`<table ${styleAttr}="background-image: url('https://image.email.feverup.com/bg.png')">`,
			'<td background="https://image.email.feverup.com/bg-attr.png"></td>',
			'</table>',
			'<img src="https://image.email.feverup.com/lib/a.png?x=1&amp;y=2" alt="Hero">',
			'<img src="cid:part-1" alt="Inline">',
			'<img src="data:image/png;base64,abc" alt="Data">',
			'</body></html>'
		].join('');

		const srcdoc = originalMailSrcdoc(html, {
			messageId: 'message 1',
			apiBaseUrl: 'http://127.0.0.1:8080'
		});

		expect(srcdoc).toContain('<base target="_blank">');
		expect(srcdoc).toContain(
			'http://127.0.0.1:8080/api/v1/communications/messages/message%201/remote-image?url=https%3A%2F%2Fimage.email.feverup.com%2Flib%2Fa.png%3Fx%3D1%26y%3D2'
		);
		expect(srcdoc).toContain(
			"url('http://127.0.0.1:8080/api/v1/communications/messages/message%201/remote-image?url=https%3A%2F%2Fimage.email.feverup.com%2Fbg.png')"
		);
		expect(srcdoc).toContain(
			'background="http://127.0.0.1:8080/api/v1/communications/messages/message%201/remote-image?url=https%3A%2F%2Fimage.email.feverup.com%2Fbg-attr.png"'
		);
		expect(srcdoc).toContain('src="cid:part-1"');
		expect(srcdoc).toContain('src="data:image/png;base64,abc"');
	});

	it('builds stable remote mail image proxy URLs', () => {
		expect(
			remoteMailImageProxyUrl(
				'message/1',
				'https://image.email.feverup.com/lib/a.png?x=1&amp;y=2',
				'http://127.0.0.1:8080/'
			)
		).toBe(
			'http://127.0.0.1:8080/api/v1/communications/messages/message%2F1/remote-image?url=https%3A%2F%2Fimage.email.feverup.com%2Flib%2Fa.png%3Fx%3D1%26y%3D2'
		);
	});

	it('removes invisible email spacer entities and empty image links', () => {
		const rendered = renderMessageContent(
			'<p>Let music lead the nights! &shy; &shy; &amp;shy;</p><a href="https://click.example.invalid/spacer"><img src="https://img.example.invalid/spacer.gif"></a>'
		);

		expect(rendered.mode).toBe('html');
		expect(rendered.html).toContain('Let music lead the nights!');
		expect(rendered.html).not.toContain('&shy;');
		expect(rendered.html).not.toContain('spacer.gif');
		expect(rendered.html).not.toContain('https://click.example.invalid/spacer');
	});

	it('builds workflow action payloads with source provenance and stable command ids', () => {
		const selected = message({
			message_id: 'message-workflow',
			subject: 'ТЗ и ключи доступа'
		});

		expect(buildWorkflowActionRequest('create_task', selected, 'cmd-1')).toEqual({
			command_id: 'cmd-1',
			action: 'create_task',
			source: { kind: 'communication_message', id: 'message-workflow' },
			input: {
				title: 'ТЗ и ключи доступа'
			}
		});
		expect(buildWorkflowActionRequest('archive', selected, 'cmd-2')).toMatchObject({
			command_id: 'cmd-2',
			action: 'archive',
			source: { kind: 'communication_message', id: 'message-workflow' }
		});
	});

	it('builds draft payload without a hardcoded account fallback', () => {
		const payload = buildComposeDraftPayload({
			draft_id: 'draft-1',
			account_id: 'imap-primary',
			to_text: 'a@example.com, b@example.com',
			cc_text: '',
			bcc_text: 'hidden@example.com',
			subject: 'Draft subject',
			body: 'Draft body',
			mode: 'compose',
			in_reply_to: null,
			references: []
		});

		expect(payload).toMatchObject({
			account_id: 'imap-primary',
			to_recipients: ['a@example.com', 'b@example.com'],
			bcc_recipients: ['hidden@example.com']
		});
	});

	it('reports draft save errors without closing compose state', async () => {
		const createDraft = vi.fn(async () => {
			throw new Error('backend unavailable');
		});

		const result = await handleSaveDraft(
			{
				draft_id: 'draft-1',
				account_id: 'imap-primary',
				to_text: 'a@example.com',
				cc_text: '',
				bcc_text: '',
				subject: 'Draft subject',
				body: 'Draft body',
				mode: 'compose',
				in_reply_to: null,
				references: []
			},
			createDraft
		);

		expect(result).toEqual({ success: false, error: 'backend unavailable' });
	});

	it('deletes drafts through the local draft endpoint helper', async () => {
		const deleteDraft = vi.fn(async () => ({ deleted: true }));

		const result = await handleDeleteDraft(' draft-1 ', deleteDraft);

		expect(deleteDraft).toHaveBeenCalledWith('draft-1');
		expect(result).toEqual({ success: true, error: '', deleted: true });
	});

	it('summarizes nonblocked mail resource endpoints for the context rail', () => {
		const summary = summarizeMailResourceSnapshot({
			subscriptions: [{ sender: 'news@example.com', message_count: 4, first_seen: '2026-06-01', last_seen: '2026-06-10', is_newsletter: true, has_unsubscribe: true }],
			duplicates: [{ sha256: 'hash-a', filenames: ['invoice.pdf'], message_ids: ['m1', 'm2'], count: 2 }],
			invoices: [{ invoice_id: 'inv-1', message_id: 'm1', amount: 10, currency: 'EUR', invoice_number: 'A-1', issue_date: null, due_date: null, counterparty: 'Vendor', tax_id: null, status: 'received', linked_project_id: null, linked_person_id: null, metadata: {}, created_at: '2026-06-10T00:00:00Z', updated_at: '2026-06-10T00:00:00Z' }],
			legalDocuments: [{ document_id: 'doc-1', message_id: 'm2', document_type: 'contract', title: 'MSA', parties: ['Vendor'], effective_date: null, expiry_date: null, amount: null, currency: null, status: 'pending_review', linked_project_id: null, risks: ['expiry'], metadata: {}, created_at: '2026-06-10T00:00:00Z', updated_at: '2026-06-10T00:00:00Z' }],
			certificates: [{ cert_id: 'cert-1', owner_name: 'Alice', issuer: 'CA', serial_number: null, fingerprint_sha256: null, valid_from: null, valid_until: null, cert_type: 'smime', provider: 'other', storage_kind: 'encrypted_vault', storage_ref: null, trust_status: 'untrusted', is_revoked: false, usage: ['email'], linked_message_id: null, metadata: {}, created_at: '2026-06-10T00:00:00Z', updated_at: '2026-06-10T00:00:00Z' }],
			expiringCertificates: [],
			personas: [{ persona_id: 'persona-1', name: 'Default', account_id: 'imap-primary', display_name: 'Alice', signature: 'Regards', default_language: 'en', default_tone: 'professional', is_default: true, metadata: {}, created_at: '2026-06-10T00:00:00Z', updated_at: '2026-06-10T00:00:00Z' }],
			templates: [{ template_id: 'template-1', name: 'Follow up', subject_template: 'Re: {{subject}}', body_template: 'Hello', variables: ['subject'], language: 'en', created_at: '2026-06-10T00:00:00Z', updated_at: '2026-06-10T00:00:00Z' }],
			blockers: [{ section: '§8', feature: 'Attachment sandbox', reason: 'external scanner required', resolution: 'add sidecar' }]
		});

		expect(summary).toMatchObject({
			subscriptions: 1,
			duplicates: 1,
			invoices: 1,
			legalDocuments: 1,
			certificates: 1,
			personas: 1,
			templates: 1,
			blockers: 1
		});
	});
});
