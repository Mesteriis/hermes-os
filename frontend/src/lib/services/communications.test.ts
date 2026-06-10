import { describe, expect, it, vi } from 'vitest';
import type { CommunicationMessageSummary, ProviderAccount } from '$lib/api';
import {
	buildComposeDraftPayload,
	buildForwardComposeForm,
	buildMailAccountOptions,
	buildReplyComposeForm,
	summarizeMailResourceSnapshot,
	filterMessagesForWorkbench,
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

function message(overrides: Partial<CommunicationMessageSummary>): CommunicationMessageSummary {
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
