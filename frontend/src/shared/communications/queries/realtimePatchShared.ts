type WorkflowState = 'new' | 'reviewed' | 'needs_action' | 'waiting' | 'done' | 'archived' | 'muted' | 'spam'
type LocalMessageState = 'active' | 'trash' | 'all'
type BulkMessageAction =
	| 'mark_read'
	| 'mark_unread'
	| 'archive'
	| 'trash'
	| 'restore'
	| 'pin'
	| 'unpin'
	| 'important'
	| 'not_important'
	| 'add_label'
	| 'remove_label'
	| 'snooze'
type CommunicationAiState = 'NEW' | 'PROCESSING' | 'PROCESSED' | 'REVIEW_REQUIRED' | 'FAILED' | 'ARCHIVED'

type CommunicationOutboxItem = {
	status: 'queued' | 'scheduled' | 'sending' | 'sent' | 'failed' | 'canceled'
}

type CacheKeyFilterField = `query${'Key'}`
type CacheKeyFilter = {
	[Field in CacheKeyFilterField]: readonly unknown[]
}

export type CommunicationFolder = {
	folder_id: string
	account_id: string | null
	name: string
	description: string | null
	color: string | null
	sort_order: number
	message_count: number
	created_at: string
	updated_at: string
}

export type FolderMessage = {
	folder_id: string
	message_id: string
	account_id: string
	subject: string
	sender: string
	occurred_at: string | null
	projected_at: string
	workflow_state: WorkflowState
	local_state: LocalMessageState
	added_at: string
	attachment_count: number
}

export type CommunicationSavedSearch = {
	saved_search_id: string
	name: string
	description: string | null
	account_id: string | null
	query: string
	workflow_state: WorkflowState | null
	local_state: LocalMessageState
	channel_kind: string | null
	is_smart_folder: boolean
	sort_order: number
	message_count: number
	created_at: string
	updated_at: string
}

export type MailRealtimePatchQueryClient = {
	getQueriesData?: <TData>(filters: CacheKeyFilter) => Array<[
		readonly unknown[],
		TData | undefined
	]>
	setQueryData?: <TData>(
		key: readonly unknown[],
		updater: TData | ((data: TData | undefined) => TData | undefined)
	) => unknown
}

export type StoredEventEnvelope = {
	event?: {
		event_type?: unknown
		payload?: unknown
	}
}

export type CommunicationMessagePatchPayload = {
	action?: unknown
	action_parameters?: unknown
	message_ids?: unknown
}

export type OutboxPatchPayload = {
	outbox_id?: unknown
	account_id?: unknown
	status?: unknown
	provider_message_id?: unknown
	last_error?: unknown
	send_attempts?: unknown
	scheduled_send_at?: unknown
	undo_deadline_at?: unknown
	sent_at?: unknown
	delivery_status?: unknown
	smtp_status?: unknown
	source_kind?: unknown
	recorded_at?: unknown
	receipt_id?: unknown
	provider_record_id?: unknown
	receipt_kind?: unknown
	read_at?: unknown
}

export type AiStatePatchPayload = {
	message_id?: unknown
	ai_state?: unknown
	review_required?: unknown
	failed?: unknown
}

export type DraftPatchPayload = {
	draft_id?: unknown
	account_id?: unknown
}

export type FolderMessagePatchPayload = {
	operation?: unknown
	folder_id?: unknown
	message_id?: unknown
	message?: unknown
}

export type SyncPatchPayload = {
	account_id?: unknown
	status?: unknown
	phase?: unknown
	progress_mode?: unknown
	progress_percent?: unknown
	processed_messages?: unknown
	estimated_total_messages?: unknown
	current_batch_size?: unknown
	fetched_messages?: unknown
	projected_messages?: unknown
	upserted_persons?: unknown
	upserted_organizations?: unknown
	error_code?: unknown
	next_run_at?: unknown
}

const AI_STATES = new Set<CommunicationAiState>([
	'NEW',
	'PROCESSING',
	'PROCESSED',
	'REVIEW_REQUIRED',
	'FAILED',
	'ARCHIVED'
])

const WORKFLOW_STATES = new Set<WorkflowState>([
	'new',
	'reviewed',
	'needs_action',
	'waiting',
	'done',
	'archived',
	'muted',
	'spam'
])

const LOCAL_MESSAGE_STATES = new Set<LocalMessageState>(['active', 'trash', 'all'])

const BULK_ACTIONS = new Set<BulkMessageAction>([
	'mark_read',
	'mark_unread',
	'archive',
	'trash',
	'restore',
	'pin',
	'unpin',
	'important',
	'not_important',
	'add_label',
	'remove_label',
	'snooze'
])

export function storedEventEnvelope(eventData: string): StoredEventEnvelope | null {
	try {
		return JSON.parse(eventData) as StoredEventEnvelope
	} catch {
		return null
	}
}

export function stringValue(value: unknown): string | null {
	return typeof value === 'string' && value.trim() ? value.trim() : null
}

export function nullableStringValue(value: unknown): string | null {
	return typeof value === 'string' && value.trim() ? value.trim() : null
}

export function numberValue(value: unknown): number | null {
	const number = Number(value)
	return Number.isFinite(number) ? number : null
}

export function nullableNumberValue(value: unknown): number | null {
	if (value === null || typeof value === 'undefined') return null
	const number = Number(value)
	return Number.isFinite(number) ? number : null
}

export function outboxStatusValue(value: unknown): CommunicationOutboxItem['status'] | null {
	const status = stringValue(value)
	if (
		status === 'queued' ||
		status === 'scheduled' ||
		status === 'sending' ||
		status === 'sent' ||
		status === 'failed' ||
		status === 'canceled'
	) {
		return status
	}
	return null
}

export function aiStateValue(value: unknown): CommunicationAiState | null {
	if (typeof value !== 'string') return null
	return AI_STATES.has(value as CommunicationAiState) ? (value as CommunicationAiState) : null
}

function workflowStateValue(value: unknown): WorkflowState | null {
	if (value === null || typeof value === 'undefined') return null
	if (typeof value !== 'string') return null
	return WORKFLOW_STATES.has(value as WorkflowState) ? (value as WorkflowState) : null
}

function localMessageStateValue(value: unknown): LocalMessageState | null {
	if (typeof value !== 'string') return null
	return LOCAL_MESSAGE_STATES.has(value as LocalMessageState) ? (value as LocalMessageState) : null
}

export function normalizeBulkAction(value: unknown): BulkMessageAction | null {
	if (typeof value !== 'string') return null
	return BULK_ACTIONS.has(value as BulkMessageAction) ? (value as BulkMessageAction) : null
}

export function normalizeMessageIds(value: unknown): string[] {
	if (!Array.isArray(value)) return []
	return value
		.filter((messageId): messageId is string => typeof messageId === 'string')
		.map((messageId) => messageId.trim())
		.filter(Boolean)
}

export function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function folderValue(value: unknown): CommunicationFolder | null {
	if (!isRecord(value)) return null
	const folderId = stringValue(value.folder_id)
	const name = stringValue(value.name)
	const sortOrder = numberValue(value.sort_order)
	const messageCount = numberValue(value.message_count)
	const createdAt = stringValue(value.created_at)
	const updatedAt = stringValue(value.updated_at)
	if (!folderId || !name || sortOrder === null || messageCount === null || !createdAt || !updatedAt) {
		return null
	}

	return {
		folder_id: folderId,
		account_id: nullableStringValue(value.account_id),
		name,
		description: nullableStringValue(value.description),
		color: nullableStringValue(value.color),
		sort_order: sortOrder,
		message_count: messageCount,
		created_at: createdAt,
		updated_at: updatedAt
	}
}

export function folderMessageValue(value: unknown): FolderMessage | null {
	if (!isRecord(value)) return null
	const folderId = stringValue(value.folder_id)
	const messageId = stringValue(value.message_id)
	const accountId = stringValue(value.account_id)
	const subject = typeof value.subject === 'string' ? value.subject : null
	const sender = typeof value.sender === 'string' ? value.sender : null
	const projectedAt = stringValue(value.projected_at)
	const workflowState = workflowStateValue(value.workflow_state)
	const localState = localMessageStateValue(value.local_state)
	const addedAt = stringValue(value.added_at)
	const attachmentCount = numberValue(value.attachment_count)
	if (
		!folderId ||
		!messageId ||
		!accountId ||
		subject === null ||
		sender === null ||
		!projectedAt ||
		!workflowState ||
		!localState ||
		!addedAt ||
		attachmentCount === null
	) {
		return null
	}

	return {
		folder_id: folderId,
		message_id: messageId,
		account_id: accountId,
		subject,
		sender,
		occurred_at: nullableStringValue(value.occurred_at),
		projected_at: projectedAt,
		workflow_state: workflowState,
		local_state: localState,
		added_at: addedAt,
		attachment_count: attachmentCount
	}
}

export function savedSearchValue(value: unknown): CommunicationSavedSearch | null {
	if (!isRecord(value)) return null
	const savedSearchId = stringValue(value.saved_search_id)
	const name = stringValue(value.name)
	const query = typeof value.query === 'string' ? value.query : null
	const localState = localMessageStateValue(value.local_state)
	const sortOrder = numberValue(value.sort_order)
	const messageCount = numberValue(value.message_count)
	const createdAt = stringValue(value.created_at)
	const updatedAt = stringValue(value.updated_at)
	if (
		!savedSearchId ||
		!name ||
		query === null ||
		!localState ||
		sortOrder === null ||
		messageCount === null ||
		typeof value.is_smart_folder !== 'boolean' ||
		!createdAt ||
		!updatedAt
	) {
		return null
	}

	return {
		saved_search_id: savedSearchId,
		name,
		description: nullableStringValue(value.description),
		account_id: nullableStringValue(value.account_id),
		query,
		workflow_state: workflowStateValue(value.workflow_state),
		local_state: localState,
		channel_kind: nullableStringValue(value.channel_kind),
		is_smart_folder: value.is_smart_folder,
		sort_order: sortOrder,
		message_count: messageCount,
		created_at: createdAt,
		updated_at: updatedAt
	}
}
