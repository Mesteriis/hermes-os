import type { InfiniteData } from '@tanstack/vue-query'
import type {
	BulkMessageActionRequest,
	CommunicationDraft,
	CommunicationOutboxItem,
	CommunicationMessageDetailResponse,
	CommunicationMessagesResponse,
	OutboxListResponse,
	MailSyncStatus
} from '../types/communications'
import type {
	FolderMessage,
	FolderMessageListResponse,
	MailFolder,
	MailFolderListResponse
} from '../types/folders'
import type { MailSavedSearch, SavedSearchListResponse } from '../types/savedSearches'
import type { MailAiStateRecord } from '../types/aiState'
import {
	applyBulkMessageActionToMailDetail,
	applyBulkMessageActionToMailList
} from './optimisticMailUpdates'
import {
	aiStateValue,
	folderMessageValue,
	folderValue,
	isRecord,
	normalizeBulkAction,
	normalizeMessageIds,
	nullableNumberValue,
	nullableStringValue,
	numberValue,
	outboxStatusValue,
	savedSearchValue,
	storedEventEnvelope,
	stringValue,
	type AiStatePatchPayload,
	type DraftPatchPayload,
	type FolderMessagePatchPayload,
	type CommunicationMessagePatchPayload,
	type MailRealtimePatchQueryClient,
	type OutboxPatchPayload,
	type SyncPatchPayload
} from './realtimePatchShared'

export type { MailRealtimePatchQueryClient } from './realtimePatchShared'

type AvailableMailRealtimePatchQueryClient = Required<
	Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>
>

export function applyMailRealtimePatch(
	eventData: string,
	queryClient: MailRealtimePatchQueryClient
): boolean {
	const { getQueriesData, setQueryData } = queryClient
	if (!getQueriesData || !setQueryData) return false
	const availableQueryClient: AvailableMailRealtimePatchQueryClient = {
		getQueriesData,
		setQueryData
	}

	if (applyAiStateRealtimePatch(eventData, availableQueryClient)) return true
	if (applyOutboxRealtimePatch(eventData, availableQueryClient)) return true
	if (applyDraftRealtimePatch(eventData, availableQueryClient)) return true
	if (applyFolderRealtimePatch(eventData, availableQueryClient)) return true
	if (applyFolderMessageRealtimePatch(eventData, availableQueryClient)) return true
	if (applySavedSearchRealtimePatch(eventData, availableQueryClient)) return true
	if (applySyncRealtimePatch(eventData, availableQueryClient)) return true

	const request = bulkActionRequestFromEvent(eventData)
	if (!request) return false

	let patched = false
	for (const [queryKey, data] of availableQueryClient.getQueriesData<InfiniteData<CommunicationMessagesResponse>>({
		queryKey: ['communications-mail-list']
	})) {
		availableQueryClient.setQueryData(queryKey, () =>
			applyBulkMessageActionToMailList(data, request, queryKey)
		)
		patched = true
	}

	for (const messageId of request.message_ids) {
		const queryKey = ['communications-message', messageId] as const
		availableQueryClient.setQueryData<CommunicationMessageDetailResponse | null | undefined>(queryKey, (data) =>
			applyBulkMessageActionToMailDetail(data, request)
		)
		patched = true
	}

	return patched
}

function applyAiStateRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	if (envelope?.event?.event_type !== 'mail.ai_state.changed') return false

	const payload = envelope.event.payload as AiStatePatchPayload | undefined
	const messageId = stringValue(payload?.message_id)
	const aiState = aiStateValue(payload?.ai_state)
	if (!messageId || !aiState) return false

	queryClient.setQueryData<MailAiStateRecord | null | undefined>(
		['communications-ai-state', messageId],
		(data) => ({
			message_id: messageId,
			ai_state: aiState,
			review_reason:
				typeof payload?.review_required === 'boolean' && payload.review_required
					? data?.review_reason ?? null
					: data?.review_reason ?? null,
			last_error:
				typeof payload?.failed === 'boolean' && payload.failed
					? data?.last_error ?? null
					: data?.last_error ?? null,
			created_at: data?.created_at ?? new Date().toISOString(),
			updated_at: new Date().toISOString()
		})
	)

	return true
}

function applySyncRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	const eventType = event?.event_type
	if (
		eventType !== 'mail.sync.started' &&
		eventType !== 'mail.sync.progress' &&
		eventType !== 'mail.sync.completed' &&
		eventType !== 'mail.sync.failed' &&
		eventType !== 'mail.sync.skipped'
	) {
		return false
	}

	const payload = event?.payload as SyncPatchPayload | undefined
	const accountId = stringValue(payload?.account_id)
	if (!accountId) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<MailSyncStatus[]>({
		queryKey: ['communications', 'mail', 'sync-statuses']
	})) {
		const updated = patchSyncStatuses(data, accountId, payload)
		if (updated !== data) {
			queryClient.setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchSyncStatuses(
	items: MailSyncStatus[] | undefined,
	accountId: string,
	payload: SyncPatchPayload | undefined
): MailSyncStatus[] | undefined {
	if (!items || !payload) return items

	let changed = false
	const patched = items.map((item) => {
		if (item.account_id !== accountId) return item
		changed = true
		return {
			...item,
			status: stringValue(payload.status) ?? item.status,
			phase: stringValue(payload.phase) ?? item.phase,
			progress_mode: stringValue(payload.progress_mode) ?? item.progress_mode,
			progress_percent:
				typeof payload.progress_percent === 'undefined'
					? item.progress_percent
					: nullableNumberValue(payload.progress_percent),
			processed_messages: numberValue(payload.processed_messages) ?? item.processed_messages,
			estimated_total_messages:
				typeof payload.estimated_total_messages === 'undefined'
					? item.estimated_total_messages
					: nullableNumberValue(payload.estimated_total_messages),
			current_batch_size: numberValue(payload.current_batch_size) ?? item.current_batch_size,
			next_run_at:
				typeof payload.next_run_at === 'undefined'
					? item.next_run_at
					: nullableStringValue(payload.next_run_at),
			last_error_code:
				typeof payload.error_code === 'undefined'
					? item.last_error_code
					: nullableStringValue(payload.error_code),
			last_fetched_messages: numberValue(payload.fetched_messages) ?? item.last_fetched_messages,
			last_projected_messages:
				numberValue(payload.projected_messages) ?? item.last_projected_messages,
			last_upserted_persons:
				numberValue(payload.upserted_persons) ?? item.last_upserted_persons,
			last_upserted_organizations:
				numberValue(payload.upserted_organizations) ?? item.last_upserted_organizations
		}
	})

	return changed ? patched : items
}

function applyFolderMessageRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	const eventType = event?.event_type
	if (
		eventType !== 'mail.folder_message.copied' &&
		eventType !== 'mail.folder_message.moved'
	) {
		return false
	}

	const payload = event?.payload as FolderMessagePatchPayload | undefined
	const folderMessage = folderMessageValue(payload?.message)
	const messageId = stringValue(payload?.message_id)
	if (!folderMessage || !messageId) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<FolderMessageListResponse>>({
		queryKey: ['communications-folder-messages']
	})) {
		const updated = patchFolderMessageList(data, queryKey, eventType, folderMessage, messageId)
		if (updated !== data) {
			queryClient.setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchFolderMessageList(
	data: InfiniteData<FolderMessageListResponse> | undefined,
	queryKey: readonly unknown[],
	eventType: string,
	folderMessage: FolderMessage,
	messageId: string
): InfiniteData<FolderMessageListResponse> | undefined {
	if (!data) return data

	const queryFolderId = typeof queryKey[1] === 'string' ? queryKey[1] : null
	let changed = false
	const pages = data.pages.map((page, pageIndex) => {
		if (queryFolderId === folderMessage.folder_id) {
			const existingIndex = page.items.findIndex((item) => item.message_id === folderMessage.message_id)
			if (existingIndex >= 0) {
				changed = true
				const items = page.items.slice()
				items[existingIndex] = folderMessage
				return {
					...page,
					items: sortFolderMessages(items)
				}
			}

			if (pageIndex === 0) {
				changed = true
				return {
					...page,
					items: sortFolderMessages([folderMessage, ...page.items])
				}
			}
		}

		if (eventType === 'mail.folder_message.moved' && queryFolderId !== folderMessage.folder_id) {
			const updated = page.items.filter((item) => item.message_id !== messageId)
			if (updated.length !== page.items.length) {
				changed = true
				return {
					...page,
					items: updated
				}
			}
		}

		return page
	})

	return changed ? { ...data, pages } : data
}

function sortFolderMessages(items: FolderMessage[]): FolderMessage[] {
	return items
		.slice()
		.sort((left, right) => {
			const addedAt = Date.parse(right.added_at) - Date.parse(left.added_at)
			if (Number.isFinite(addedAt) && addedAt !== 0) return addedAt
			return left.message_id.localeCompare(right.message_id)
		})
}

function applySavedSearchRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	const eventType = event?.event_type
	if (
		eventType !== 'mail.saved_search.created' &&
		eventType !== 'mail.saved_search.updated' &&
		eventType !== 'mail.saved_search.deleted'
	) {
		return false
	}

	const savedSearch = savedSearchValue(event?.payload)
	if (!savedSearch) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<SavedSearchListResponse>>({
		queryKey: ['communications-saved-searches']
	})) {
		const updated = patchSavedSearchList(data, queryKey, eventType, savedSearch)
		if (updated !== data) {
			queryClient.setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchSavedSearchList(
	data: InfiniteData<SavedSearchListResponse> | undefined,
	queryKey: readonly unknown[],
	eventType: string,
	savedSearch: MailSavedSearch
): InfiniteData<SavedSearchListResponse> | undefined {
	if (!data) return data

	const matchesQuery = savedSearchMatchesQuery(queryKey, savedSearch)
	if (eventType === 'mail.saved_search.deleted' || !matchesQuery) {
		return removeSavedSearchFromPages(data, savedSearch.saved_search_id)
	}

	let found = false
	let changed = false
	const pages = data.pages.map((page) => {
		const existingIndex = page.items.findIndex((item) => item.saved_search_id === savedSearch.saved_search_id)
		if (existingIndex < 0) return page

		found = true
		changed = true
		const items = page.items.slice()
		items[existingIndex] = savedSearch
		return { ...page, items: sortSavedSearches(items) }
	})

	if (eventType === 'mail.saved_search.created' && !found && pages.length > 0) {
		const [firstPage, ...restPages] = pages
		return {
			...data,
			pages: [{ ...firstPage, items: sortSavedSearches([savedSearch, ...firstPage.items]) }, ...restPages]
		}
	}

	return changed ? { ...data, pages } : data
}

function removeSavedSearchFromPages(
	data: InfiniteData<SavedSearchListResponse>,
	savedSearchId: string
): InfiniteData<SavedSearchListResponse> {
	let changed = false
	const pages = data.pages.map((page) => {
		const items = page.items.filter((item) => item.saved_search_id !== savedSearchId)
		if (items.length === page.items.length) return page
		changed = true
		return { ...page, items }
	})

	return changed ? { ...data, pages } : data
}

function savedSearchMatchesQuery(queryKey: readonly unknown[], savedSearch: MailSavedSearch): boolean {
	const isSmartFolder = queryKey[1]
	if (typeof isSmartFolder === 'boolean' && savedSearch.is_smart_folder !== isSmartFolder) {
		return false
	}

	const accountId = queryKey[2]
	if (typeof accountId === 'string' && accountId.trim()) {
		return savedSearch.account_id === accountId
	}

	return true
}

function sortSavedSearches(items: MailSavedSearch[]): MailSavedSearch[] {
	return items
		.slice()
		.sort((left, right) => left.sort_order - right.sort_order || left.name.localeCompare(right.name))
}

function applyFolderRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	const eventType = event?.event_type
	if (
		eventType !== 'mail.folder.created' &&
		eventType !== 'mail.folder.updated' &&
		eventType !== 'mail.folder.deleted'
	) {
		return false
	}

	const folder = folderValue(event?.payload)
	if (!folder) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<MailFolderListResponse>>({
		queryKey: ['communications-folders']
	})) {
		const updated = patchFolderList(data, queryKey, eventType, folder)
		if (updated !== data) {
			queryClient.setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchFolderList(
	data: InfiniteData<MailFolderListResponse> | undefined,
	queryKey: readonly unknown[],
	eventType: string,
	folder: MailFolder
): InfiniteData<MailFolderListResponse> | undefined {
	if (!data || !folderMatchesFolderQuery(queryKey, folder)) return data

	let changed = false
	const pages = data.pages.map((page, pageIndex) => {
		const existingIndex = page.items.findIndex((item) => item.folder_id === folder.folder_id)

		if (eventType === 'mail.folder.deleted') {
			if (existingIndex < 0) return page
			changed = true
			return {
				...page,
				items: page.items.filter((item) => item.folder_id !== folder.folder_id)
			}
		}

		if (existingIndex >= 0) {
			changed = true
			const items = page.items.slice()
			items[existingIndex] = folder
			return {
				...page,
				items: sortFolders(items)
			}
		}

		if (eventType === 'mail.folder.created' && pageIndex === 0) {
			changed = true
			return {
				...page,
				items: sortFolders([folder, ...page.items])
			}
		}

		return page
	})

	return changed ? { ...data, pages } : data
}

function folderMatchesFolderQuery(queryKey: readonly unknown[], folder: MailFolder): boolean {
	const accountId = queryKey[1]
	if (typeof accountId !== 'string' || !accountId.trim()) return true
	return folder.account_id === accountId
}

function sortFolders(folders: MailFolder[]): MailFolder[] {
	return folders
		.slice()
		.sort((left, right) => left.sort_order - right.sort_order || left.name.localeCompare(right.name))
}

function applyDraftRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	if (event?.event_type !== 'mail.draft.deleted') return false

	const payload = event.payload as DraftPatchPayload | undefined
	const draftId = stringValue(payload?.draft_id)
	if (!draftId) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<CommunicationDraft[]>({
		queryKey: ['communications-drafts']
	})) {
		const updated = removeDraft(data, draftId)
		if (updated !== data) {
			queryClient.setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function removeDraft(drafts: CommunicationDraft[] | undefined, draftId: string): CommunicationDraft[] | undefined {
	if (!drafts) return drafts
	const updated = drafts.filter((draft) => draft.draft_id !== draftId)
	return updated.length === drafts.length ? drafts : updated
}

function applyOutboxRealtimePatch(
	eventData: string,
	queryClient: Required<Pick<MailRealtimePatchQueryClient, 'getQueriesData' | 'setQueryData'>>
): boolean {
	const envelope = storedEventEnvelope(eventData)
	const event = envelope?.event
	const eventType = event?.event_type
	if (
		eventType !== 'mail.outbox.sent' &&
		eventType !== 'mail.outbox.failed' &&
		eventType !== 'mail.outbox.retry_scheduled' &&
		eventType !== 'mail.outbox.delivery_status_changed' &&
		eventType !== 'mail.read_receipt.recorded'
	) {
		return false
	}

	const payload = event?.payload as OutboxPatchPayload | undefined
	const outboxId = stringValue(payload?.outbox_id)
	if (!outboxId) return false

	let patched = false
	for (const [queryKey, data] of queryClient.getQueriesData<InfiniteData<OutboxListResponse>>({
		queryKey: ['communications-outbox']
	})) {
		const updated = patchOutboxItems(data, queryKey, outboxId, eventType, payload)
		if (updated !== data) {
			queryClient.setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchOutboxItems(
	data: InfiniteData<OutboxListResponse> | undefined,
	queryKey: readonly unknown[],
	outboxId: string,
	eventType: string,
	payload: OutboxPatchPayload | undefined
): InfiniteData<OutboxListResponse> | undefined {
	if (!data || !Array.isArray(data.pages)) return data

	let changed = false
	const pages = data.pages.map((page) => {
		let pageChanged = false
		const items: CommunicationOutboxItem[] = []

		for (const item of page?.items ?? []) {
			if (item.outbox_id !== outboxId) {
				items.push(item)
				continue
			}
			const next = patchOutboxItem(item, eventType, payload)
			if (next !== item) {
				pageChanged = true
				changed = true
			}
			if (!outboxRealtimeQueryMatches(queryKey, next)) {
				pageChanged = true
				changed = true
				continue
			}
			items.push(next)
		}

		return pageChanged ? { ...page, items } : page
	})

	return changed ? { ...data, pages } : data
}

function outboxRealtimeQueryMatches(queryKey: readonly unknown[], item: CommunicationOutboxItem): boolean {
	const queryAccountId = queryKey[1]
	const queryStatus = queryKey[2]

	if (typeof queryAccountId === 'string' && queryAccountId !== item.account_id) return false
	if (typeof queryStatus === 'string' && queryStatus !== item.status) return false
	return true
}

function patchOutboxItem(
	item: CommunicationOutboxItem,
	eventType: string,
	payload: OutboxPatchPayload | undefined
): CommunicationOutboxItem {
	if (!payload) return item

	if (eventType === 'mail.outbox.delivery_status_changed') {
		const deliveryStatus = stringValue(payload.delivery_status)
		if (!deliveryStatus) return item
		return {
			...item,
			metadata: {
				...item.metadata,
				delivery_status: {
					delivery_status: deliveryStatus,
					smtp_status: nullableStringValue(payload.smtp_status),
					source_kind: nullableStringValue(payload.source_kind),
					recorded_at: nullableStringValue(payload.recorded_at)
				}
			}
		}
	}

	if (eventType === 'mail.read_receipt.recorded') {
		const receiptKind = stringValue(payload.receipt_kind)
		if (!receiptKind) return item
		return {
			...item,
			metadata: {
				...item.metadata,
				latest_read_receipt: {
					receipt_id: nullableStringValue(payload.receipt_id),
					receipt_kind: receiptKind,
					read_at: nullableStringValue(payload.read_at),
					source_kind: nullableStringValue(payload.source_kind)
				}
			}
		}
	}

	return {
		...item,
		status: outboxStatusValue(payload.status) ?? item.status,
		provider_message_id: nullableStringValue(payload.provider_message_id),
		last_error: nullableStringValue(payload.last_error),
		send_attempts: numberValue(payload.send_attempts) ?? item.send_attempts,
		scheduled_send_at: nullableStringValue(payload.scheduled_send_at),
		undo_deadline_at: nullableStringValue(payload.undo_deadline_at),
		sent_at: nullableStringValue(payload.sent_at)
	}
}

function bulkActionRequestFromEvent(eventData: string): BulkMessageActionRequest | null {
	const envelope = storedEventEnvelope(eventData)
	if (!envelope) return null

	const event = envelope.event
	const eventType = event?.event_type
	if (typeof eventType !== 'string' || !eventType.startsWith('mail.message.')) return null

	const payload = event?.payload as CommunicationMessagePatchPayload | undefined
	const action = normalizeBulkAction(payload?.action)
	const messageIds = normalizeMessageIds(payload?.message_ids)
	if (!action || messageIds.length === 0) return null

	const request: BulkMessageActionRequest = {
		action,
		message_ids: messageIds
	}

	const actionParameters = payload?.action_parameters
	if (isRecord(actionParameters)) {
		const label = actionParameters.label
		if (typeof label === 'string' && label.trim()) {
			request.label = label.trim()
		}

		const snoozeUntil = actionParameters.snooze_until
		if (typeof snoozeUntil === 'string' && snoozeUntil.trim()) {
			request.snooze_until = snoozeUntil.trim()
		}
	}

	return request
}
