import { describe, expect, it } from 'vitest'
import type { InfiniteData } from '@tanstack/vue-query'
import type {
	CommunicationMessageSummary,
	CommunicationDraft,
	CommunicationOutboxItem,
	CommunicationMessagesResponse
} from '../types/communications'
import {
	applyBulkMessageActionToMailDetail,
	applyBulkMessageActionToMailList,
	markOutboxItemCanceled,
	removeDraftFromDraftList,
	upsertDraftInDraftList,
	upsertOutboxItem
} from './optimisticMailUpdates'

function message(overrides: Partial<CommunicationMessageSummary>): CommunicationMessageSummary {
	return {
		message_id: 'msg-1',
		raw_record_id: 'raw-1',
		account_id: 'account-1',
		provider_record_id: 'provider-1',
		subject: 'Quarterly update',
		sender: 'sender@example.com',
		recipients: ['recipient@example.com'],
		body_text_preview: 'Preview',
		occurred_at: '2026-06-14T10:00:00Z',
		projected_at: '2026-06-14T10:01:00Z',
		channel_kind: 'email',
		conversation_id: 'thread-1',
		sender_display_name: 'Sender',
		delivery_state: 'delivered',
		workflow_state: 'new',
		importance_score: null,
		ai_category: null,
		ai_summary: null,
		ai_summary_generated_at: null,
		message_metadata: {},
		attachment_count: 0,
		local_state: 'active',
		local_state_changed_at: null,
		...overrides
	}
}

function mailList(items: CommunicationMessageSummary[]): InfiniteData<CommunicationMessagesResponse> {
	return {
		pages: [
			{
				items,
				next_cursor: null,
				has_more: false
			}
		],
		pageParams: [null]
	}
}

function draft(overrides: Partial<CommunicationDraft> = {}): CommunicationDraft {
	return {
		draft_id: 'draft-1',
		account_id: 'account-1',
		persona_id: null,
		to_recipients: ['reader@example.com'],
		cc_recipients: [],
		bcc_recipients: [],
		subject: 'Draft subject',
		body_text: 'Draft body',
		body_html: null,
		in_reply_to: null,
		references: [],
		attachment_ids: [],
		attachments: [],
		status: 'draft',
		scheduled_send_at: null,
		send_attempts: 0,
		last_error: null,
		metadata: {},
		created_at: '2026-06-15T10:00:00Z',
		updated_at: '2026-06-15T10:00:00Z',
		...overrides
	}
}

function outboxItem(overrides: Partial<CommunicationOutboxItem> = {}): CommunicationOutboxItem {
	return {
		outbox_id: 'outbox-1',
		account_id: 'account-1',
		draft_id: 'draft-1',
		to_recipients: ['reader@example.com'],
		cc_recipients: [],
		bcc_recipients: [],
		subject: 'Queued subject',
		body_text: 'Queued body',
		body_html: null,
		status: 'queued',
		scheduled_send_at: null,
		undo_deadline_at: '2026-06-15T10:05:00Z',
		send_attempts: 0,
		claimed_at: null,
		sent_at: null,
		provider_message_id: null,
		last_error: null,
		metadata: {},
		created_at: '2026-06-15T10:00:00Z',
		updated_at: '2026-06-15T10:00:00Z',
		...overrides
	}
}

describe('optimistic mail updates', () => {
	it('marks selected list messages as reviewed without changing unrelated messages', () => {
		const unread = message({ message_id: 'msg-1', workflow_state: 'new' })
		const unrelated = message({ message_id: 'msg-2', workflow_state: 'needs_action' })

		const updated = applyBulkMessageActionToMailList(mailList([unread, unrelated]), {
			action: 'mark_read',
			message_ids: ['msg-1']
		})

		expect(updated?.pages[0]?.items).toEqual([
			{ ...unread, workflow_state: 'reviewed' },
			unrelated
		])
		expect(updated?.pages[0]?.items[0]).not.toBe(unread)
		expect(updated?.pages[0]?.items[1]).toBe(unrelated)
	})

	it('removes changed messages from filtered list caches they no longer match', () => {
		const first = message({ message_id: 'msg-1', local_state: 'active' })
		const second = message({ message_id: 'msg-2', local_state: 'active' })

		const archived = applyBulkMessageActionToMailList(
			mailList([first, second]),
			{
				action: 'archive',
				message_ids: ['msg-1']
			},
			['communications-list', 'account-1', 'new', 'email', undefined, undefined]
		)
		const trashed = applyBulkMessageActionToMailList(
			mailList([first, second]),
			{
				action: 'trash',
				message_ids: ['msg-2']
			},
			['communications-list', 'account-1', undefined, 'email', undefined, undefined]
		)

		expect(archived?.pages[0]?.items).toEqual([second])
		expect(trashed?.pages[0]?.items).toEqual([first])
	})

	it('updates selected message detail state when the detail cache is present', () => {
		const detail = {
			message: {
				...message({ message_id: 'msg-1', workflow_state: 'reviewed' }),
				body_text: 'Full body',
				body_html: null,
				local_state_reason: null
			},
			attachments: []
		}

		const updated = applyBulkMessageActionToMailDetail(detail, {
			action: 'mark_unread',
			message_ids: ['msg-1']
		})

		expect(updated?.message.workflow_state).toBe('new')
		expect(updated?.attachments).toBe(detail.attachments)
	})

	it('upserts saved drafts while preserving unrelated draft rows', () => {
		const existing = draft({ draft_id: 'draft-1', subject: 'Old subject' })
		const unrelated = draft({ draft_id: 'draft-2', subject: 'Unrelated' })
		const updatedDraft = draft({ draft_id: 'draft-1', subject: 'Updated subject' })
		const newDraft = draft({ draft_id: 'draft-3', subject: 'New subject' })

		const replaced = upsertDraftInDraftList([existing, unrelated], updatedDraft)
		const inserted = upsertDraftInDraftList([existing], newDraft)

		expect(replaced).toEqual([updatedDraft, unrelated])
		expect(replaced?.[1]).toBe(unrelated)
		expect(inserted).toEqual([newDraft, existing])
	})

	it('removes deleted drafts without changing unchanged draft caches', () => {
		const first = draft({ draft_id: 'draft-1' })
		const second = draft({ draft_id: 'draft-2' })
		const drafts = [first, second]

		expect(removeDraftFromDraftList(drafts, 'draft-1')).toEqual([second])
		expect(removeDraftFromDraftList(drafts, 'missing')).toBe(drafts)
	})

	it('upserts and cancel-patches outbox rows for undo-send UX', () => {
		const queued = outboxItem({ outbox_id: 'outbox-1', status: 'queued' })
		const other = outboxItem({ outbox_id: 'outbox-2', status: 'scheduled' })
		const sent = outboxItem({ outbox_id: 'outbox-1', status: 'sent', sent_at: '2026-06-15T10:06:00Z' })

		expect(upsertOutboxItem([queued, other], sent)).toEqual([sent, other])
		expect(upsertOutboxItem([other], queued)).toEqual([queued, other])

		const canceled = markOutboxItemCanceled([queued, other], 'outbox-1')
		expect(canceled?.[0]).toMatchObject({
			outbox_id: 'outbox-1',
			status: 'canceled',
			undo_deadline_at: null
		})
		expect(canceled?.[1]).toBe(other)
	})
})
