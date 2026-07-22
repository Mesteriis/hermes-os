import type { InfiniteData } from '@tanstack/vue-query'
import type {
	BulkMessageActionRequest,
	CommunicationMessageSummary,
	CommunicationDraft,
	CommunicationOutboxItem,
	LocalMessageState,
	CommunicationMessageDetailItem,
	CommunicationMessageDetailResponse,
	CommunicationMessagesResponse,
	WorkflowState
} from '../types/communications'

type MailListFilters = {
	workflowState?: WorkflowState
	localState?: LocalMessageState
}

export function applyBulkMessageActionToMailList(
	data: InfiniteData<CommunicationMessagesResponse> | undefined,
	request: BulkMessageActionRequest,
	queryKey?: readonly unknown[]
): InfiniteData<CommunicationMessagesResponse> | undefined {
	if (!data) return data

	const targetIds = new Set(request.message_ids)
	if (targetIds.size === 0) return data

	const filters = parseMailListFilters(queryKey)
	let changed = false

	const pages = data.pages.map((page) => {
		let pageChanged = false
		const items: CommunicationMessageSummary[] = []

		for (const item of page.items) {
			if (!targetIds.has(item.message_id)) {
				items.push(item)
				continue
			}

			const updated = applyBulkMessageActionToSummary(item, request)
			if (!isVisibleInMailList(updated, filters)) {
				pageChanged = true
				changed = true
				continue
			}

			items.push(updated)
			if (updated !== item) {
				pageChanged = true
				changed = true
			}
		}

		if (!pageChanged) return page
		return { ...page, items }
	})

	if (!changed) return data
	return { ...data, pages }
}

export function applyBulkMessageActionToMailDetail(
	data: CommunicationMessageDetailResponse | null | undefined,
	request: BulkMessageActionRequest
): CommunicationMessageDetailResponse | null | undefined {
	if (!data || !data.message || !request.message_ids.includes(data.message.message_id)) return data

	const updatedMessage = applyBulkMessageActionToDetailItem(data.message, request)
	if (updatedMessage === data.message) return data
	return { ...data, message: updatedMessage }
}

export function upsertDraftInDraftList(
	drafts: CommunicationDraft[] | undefined,
	draft: CommunicationDraft
): CommunicationDraft[] | undefined {
	if (!drafts) return drafts

	const index = drafts.findIndex((item) => item.draft_id === draft.draft_id)
	if (index === -1) return [draft, ...drafts]

	const next = drafts.slice()
	next[index] = draft
	return next
}

export function removeDraftFromDraftList(
	drafts: CommunicationDraft[] | undefined,
	draftId: string
): CommunicationDraft[] | undefined {
	if (!drafts) return drafts
	const next = drafts.filter((draft) => draft.draft_id !== draftId)
	return next.length === drafts.length ? drafts : next
}

export function upsertOutboxItem(
	items: CommunicationOutboxItem[] | undefined,
	item: CommunicationOutboxItem
): CommunicationOutboxItem[] | undefined {
	if (!items) return items

	const index = items.findIndex((existing) => existing.outbox_id === item.outbox_id)
	if (index === -1) return [item, ...items]

	const next = items.slice()
	next[index] = item
	return next
}

export function markOutboxItemCanceled(
	items: CommunicationOutboxItem[] | undefined,
	outboxId: string
): CommunicationOutboxItem[] | undefined {
	if (!items) return items

	let changed = false
	const next = items.map((item) => {
		if (item.outbox_id !== outboxId) return item
		if (item.status === 'canceled' && item.undo_deadline_at === null) return item
		changed = true
		return {
			...item,
			status: 'canceled' as const,
			undo_deadline_at: null,
			last_error: null
		}
	})

	return changed ? next : items
}

function applyBulkMessageActionToSummary(
	message: CommunicationMessageSummary,
	request: BulkMessageActionRequest
): CommunicationMessageSummary {
	const updated = applyBulkMessageActionToBaseMessage(message, request)
	return updated === message ? message : { ...message, ...updated }
}

function applyBulkMessageActionToDetailItem(
	message: CommunicationMessageDetailItem,
	request: BulkMessageActionRequest
): CommunicationMessageDetailItem {
	const updated = applyBulkMessageActionToBaseMessage(message, request)
	return updated === message ? message : { ...message, ...updated }
}

function applyBulkMessageActionToBaseMessage<T extends {
	workflow_state: WorkflowState
	local_state: LocalMessageState
	message_metadata: Record<string, unknown>
}>(
	message: T,
	request: BulkMessageActionRequest
): T | Partial<T> {
	switch (request.action) {
		case 'mark_read':
			return message.workflow_state === 'reviewed' ? message : { workflow_state: 'reviewed' } as Partial<T>
		case 'mark_unread':
			return message.workflow_state === 'new' ? message : { workflow_state: 'new' } as Partial<T>
		case 'archive':
			return message.workflow_state === 'archived' ? message : { workflow_state: 'archived' } as Partial<T>
		case 'trash':
			return message.local_state === 'trash' ? message : { local_state: 'trash' } as Partial<T>
		case 'restore':
			return message.local_state === 'active' ? message : { local_state: 'active' } as Partial<T>
		case 'pin':
			return applyMetadataUpdate(message, { pinned: true })
		case 'unpin':
			return applyMetadataUpdate(message, { pinned: false })
		case 'important':
			return applyMetadataUpdate(message, { important: true })
		case 'not_important':
			return applyMetadataUpdate(message, { important: false })
		case 'star':
			return applyMetadataUpdate(message, { starred: true })
		case 'unstar':
			return applyMetadataUpdate(message, { starred: false })
		case 'add_label':
			return addLabel(message, request.label)
		case 'remove_label':
			return removeLabel(message, request.label)
		case 'snooze':
			return request.snooze_until
				? applyMetadataUpdate(message, { snooze_until: request.snooze_until })
				: message
	}
}

function applyMetadataUpdate<T extends { message_metadata: Record<string, unknown> }>(
	message: T,
	metadataPatch: Record<string, unknown>
): Partial<T> {
	return {
		message_metadata: {
			...message.message_metadata,
			...metadataPatch
		}
	} as Partial<T>
}

function addLabel<T extends { message_metadata: Record<string, unknown> }>(
	message: T,
	label: string | undefined
): T | Partial<T> {
	const normalized = label?.trim()
	if (!normalized) return message

	const labels = currentLabels(message.message_metadata)
	if (labels.includes(normalized)) return message

	return applyMetadataUpdate(message, {
		labels: [...labels, normalized].sort()
	})
}

function removeLabel<T extends { message_metadata: Record<string, unknown> }>(
	message: T,
	label: string | undefined
): T | Partial<T> {
	const normalized = label?.trim()
	if (!normalized) return message

	const labels = currentLabels(message.message_metadata)
	const nextLabels = labels.filter((value) => value !== normalized)
	if (nextLabels.length === labels.length) return message

	return applyMetadataUpdate(message, { labels: nextLabels })
}

function currentLabels(metadata: Record<string, unknown>): string[] {
	const labels = metadata.labels
	if (!Array.isArray(labels)) return []
	return labels.filter((label): label is string => typeof label === 'string' && label.trim() !== '')
}

function parseMailListFilters(queryKey?: readonly unknown[]): MailListFilters {
	const workflowState = queryKey?.[2]
	const localState = queryKey?.[5]

	return {
		workflowState: isWorkflowState(workflowState) ? workflowState : undefined,
		localState: isLocalState(localState) ? localState : undefined
	}
}

function isVisibleInMailList(
	message: CommunicationMessageSummary,
	filters: MailListFilters
): boolean {
	if (filters.workflowState && message.workflow_state !== filters.workflowState) {
		return false
	}

	if (filters.localState === 'all') return true

	const localState = filters.localState ?? 'active'
	return message.local_state === localState
}

function isWorkflowState(value: unknown): value is WorkflowState {
	return typeof value === 'string' && (
		value === 'new' || value === 'reviewed' || value === 'needs_action' ||
		value === 'waiting' || value === 'done' || value === 'archived' ||
		value === 'muted' || value === 'spam'
	)
}

function isLocalState(value: unknown): value is LocalMessageState {
	return typeof value === 'string' && (value === 'active' || value === 'trash' || value === 'all')
}
