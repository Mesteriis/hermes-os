import type {
	WhatsAppCallSyncItem,
	WhatsAppChatSyncItem,
	WhatsAppContactSyncItem,
	WhatsAppMediaSyncItem,
	WhatsAppMembersSyncItem,
	WhatsAppPresenceSyncItem,
	WhatsAppProviderCommand,
	WhatsappWebMessage,
} from '../../../shared/communications/types/whatsapp'

export function commandStatusTone(command: WhatsAppProviderCommand): string {
	if (command.status === 'completed') return 'available'
	if (command.status === 'executing' || command.status === 'queued' || command.status === 'retrying') return 'degraded'
	return 'blocked'
}

export function canRetryCommand(command: WhatsAppProviderCommand): boolean {
	return ['failed', 'dead_letter', 'retrying', 'cancelled'].includes(command.status)
}

export function canDeadLetterCommand(command: WhatsAppProviderCommand): boolean {
	return !['completed', 'dead_letter'].includes(command.status)
}

export function commandTimestamp(command: WhatsAppProviderCommand): string {
	const value =
		command.completed_at
		?? command.provider_observed_at
		?? command.last_attempt_at
		?? command.updated_at
	if (!value) return '-'
	const date = new Date(value)
	return Number.isNaN(date.getTime())
		? value
		: new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
		}).format(date)
}

export function providerTargetLabel(command: WhatsAppProviderCommand): string {
	return command.provider_message_id
		? `${command.provider_chat_id} · ${command.provider_message_id}`
		: command.provider_chat_id
}

export function snapshotTimestamp(value: string | null | undefined): string {
	if (!value) return '-'
	const date = new Date(value)
	return Number.isNaN(date.getTime())
		? value
		: new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
		}).format(date)
}

export function runtimeHealthCheckStatus(value: unknown): string {
	if (isRecordValue(value)) {
		if (typeof value.status === 'string' && value.status.trim()) return value.status
		if (typeof value.healthy === 'boolean') return value.healthy ? 'healthy' : 'degraded'
	}
	if (typeof value === 'string' && value.trim()) return value
	return '-'
}

export function runtimeHealthCheckDetail(value: unknown): string {
	if (isRecordValue(value)) {
		const reason = value.reason
		if (typeof reason === 'string' && reason.trim()) return reason
		const error = value.error
		if (typeof error === 'string' && error.trim()) return error
		const details = value.details
		if (typeof details === 'string' && details.trim()) return details
	}
	if (typeof value === 'boolean') return value ? 'ok' : 'blocked'
	if (typeof value === 'number') return String(value)
	return ''
}

function isRecordValue(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function presenceLabel(item: WhatsAppPresenceSyncItem): string {
	return item.display_name ?? item.address ?? item.provider_identity_id
}

export function chatLabel(item: WhatsAppChatSyncItem): string {
	return item.title || item.provider_chat_id
}

export function chatMeta(item: WhatsAppChatSyncItem): string {
	const parts: string[] = []
	if (item.chat_kind) parts.push(item.chat_kind)
	if (typeof item.unread_count === 'number' && item.unread_count > 0) parts.push(`unread ${item.unread_count}`)
	if (typeof item.participant_count === 'number') parts.push(`${item.participant_count} members`)
	if (item.is_archived) parts.push('archived')
	if (item.is_pinned) parts.push('pinned')
	if (item.is_muted) parts.push('muted')
	return parts.join(' · ') || item.provider_chat_id
}

export function historyLabel(item: WhatsappWebMessage): string {
	return item.sender_display_name ?? item.sender ?? item.provider_message_id
}

export function statusLabel(item: WhatsappWebMessage): string {
	return item.sender_display_name ?? item.sender ?? item.provider_message_id
}

export function statusPreview(item: WhatsappWebMessage): string {
	const text = item.text?.trim()
	if (text) return text
	const mediaType = typeof item.metadata?.media_type === 'string' ? item.metadata.media_type : null
	return mediaType ? `[${mediaType}]` : '-'
}

export function callLabel(item: WhatsAppCallSyncItem): string {
	return `${item.direction} · ${item.call_state}`
}

export function contactLabel(item: WhatsAppContactSyncItem): string {
	return item.display_name ?? item.push_name ?? item.address ?? item.provider_identity_id
}

export function mediaLabel(item: WhatsAppMediaSyncItem): string {
	return item.filename ?? item.provider_attachment_id
}

export function memberLabel(item: WhatsAppMembersSyncItem): string {
	return item.sender_display_name ?? item.address ?? item.provider_member_id
}
