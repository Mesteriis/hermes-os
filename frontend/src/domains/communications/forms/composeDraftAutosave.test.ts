import { afterEach, describe, expect, it, vi } from 'vitest'
import type { ComposeFormModel } from '../types/communications'
import {
	buildComposeDraftPayload,
	composeDraftHasAutosaveContent,
	useComposeDraftAutosave
} from './composeDraftAutosave'

function composeForm(overrides: Partial<ComposeFormModel> = {}): ComposeFormModel {
	return {
		mode: 'compose',
		draftId: 'draft-1',
		accountId: 'account-1',
		toText: '',
		ccText: '',
		bccText: '',
		subject: '',
		body: '',
		bodyHtml: null,
		bodyFormat: 'plain',
		scheduledSendAt: '',
			undoSendSeconds: null,
			inReplyTo: null,
			attachments: [],
			...overrides
	}
}

describe('compose draft autosave', () => {
	afterEach(() => {
		vi.useRealTimers()
	})

	it('builds an autosave draft payload from the compose form', () => {
		const payload = buildComposeDraftPayload(
			composeForm({
				mode: 'reply',
				toText: 'Alex <alex@example.com>, team@example.org',
				ccText: 'copy@example.org',
				subject: '',
				body: 'Autosaved body',
				inReplyTo: 'provider-message-1'
			})
		)

		expect(payload).toEqual({
			draft_id: 'draft-1',
			account_id: 'account-1',
			to_recipients: ['alex@example.com', 'team@example.org'],
			cc_recipients: ['copy@example.org'],
			bcc_recipients: [],
			subject: '',
			body_text: 'Autosaved body',
				body_html: null,
				in_reply_to: 'provider-message-1',
				attachment_ids: [],
				scheduled_send_at: null,
			status: 'draft',
			metadata: { compose_mode: 'reply' }
		})
	})

	it('detects whether a draft has content worth autosaving', () => {
		expect(composeDraftHasAutosaveContent(composeForm())).toBe(false)
		expect(composeDraftHasAutosaveContent(composeForm({ toText: 'recipient@example.com' }))).toBe(true)
		expect(composeDraftHasAutosaveContent(composeForm({ body: 'Draft body' }))).toBe(true)
		expect(composeDraftHasAutosaveContent(composeForm({ bodyHtml: '<p>Draft body</p>', bodyFormat: 'html' }))).toBe(true)
		expect(composeDraftHasAutosaveContent(composeForm({
			attachments: [{
				attachmentId: 'attachment-1',
				filename: 'report.pdf',
				contentType: 'application/pdf',
				sizeBytes: 42,
				scanStatus: 'clean',
				uploadStatus: 'ready',
				error: ''
			}]
		}))).toBe(true)
	})

	it('includes body_html for HTML drafts', () => {
		const payload = buildComposeDraftPayload(
			composeForm({
				body: 'Plain fallback',
				bodyHtml: '<p>HTML body</p>',
				bodyFormat: 'html'
			})
		)

		expect(payload.body_text).toBe('Plain fallback')
		expect(payload.body_html).toBe('<p>HTML body</p>')
	})

	it('includes scheduled_send_at for scheduled drafts', () => {
		const payload = buildComposeDraftPayload(
			composeForm({
				scheduledSendAt: '2026-06-15T10:30'
			})
		)

		expect(payload.scheduled_send_at).toBe(new Date('2026-06-15T10:30').toISOString())
	})

	it('debounces autosave and persists the latest compose state', async () => {
		vi.useFakeTimers()
		let currentForm = composeForm({ body: 'First body' })
		const saveDraft = vi.fn().mockResolvedValue(undefined)
		const autosave = useComposeDraftAutosave({
			delayMs: 2000,
			formSource: () => currentForm,
			saveDraft
		})

		autosave.schedule()
		currentForm = composeForm({ body: 'Second body' })
		autosave.schedule()

		await vi.advanceTimersByTimeAsync(1999)
		expect(saveDraft).not.toHaveBeenCalled()

		await vi.advanceTimersByTimeAsync(1)
		expect(saveDraft).toHaveBeenCalledOnce()
		expect(saveDraft).toHaveBeenCalledWith({
			draft_id: 'draft-1',
			account_id: 'account-1',
			to_recipients: [],
			cc_recipients: [],
			bcc_recipients: [],
			subject: '',
			body_text: 'Second body',
				body_html: null,
				in_reply_to: null,
				attachment_ids: [],
				scheduled_send_at: null,
			status: 'draft',
			metadata: { compose_mode: 'compose' }
		})
	})

	it('flushes a pending autosave without saving twice', async () => {
		vi.useFakeTimers()
		const saveDraft = vi.fn().mockResolvedValue(undefined)
		const autosave = useComposeDraftAutosave({
			delayMs: 2000,
			formSource: () => composeForm({ subject: 'Flush me' }),
			saveDraft
		})

		autosave.schedule()
		await autosave.flush()
		await vi.advanceTimersByTimeAsync(2000)

		expect(saveDraft).toHaveBeenCalledOnce()
	})
})
