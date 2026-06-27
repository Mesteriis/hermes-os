import type {
	WhatsAppCallSyncItem,
	WhatsAppChatSyncItem,
	WhatsAppContactSyncItem,
	WhatsAppMembersSyncItem,
	WhatsAppPresenceSyncItem,
	WhatsappWebMessage,
} from '../types/whatsapp'
import { isRecord, stringValue } from '../../../shared/communications/queries/realtimePatchShared'
import {
	booleanValue,
	integerValue,
	nullableStringValue,
	stringArray,
	type WhatsAppRuntimeEventPayload,
} from './realtimeWhatsAppRuntimePatchValues'

export function patchPresenceList(
	queryKey: readonly unknown[],
	items: WhatsAppPresenceSyncItem[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppPresenceSyncItem[] | undefined {
	if (!items || eventType !== 'whatsapp.presence.changed') return items

	const payloadAccountId = stringValue(payload.account_id)
	if (!payloadAccountId) return items
	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const queryProviderChatId =
		typeof queryKey[5] === 'string' && queryKey[5] !== 'all' ? queryKey[5] : null
	const payloadProviderChatId = nullableStringValue(payload.provider_chat_id, null)
	if (queryProviderChatId && queryProviderChatId !== payloadProviderChatId) return items

	const identityId =
		nullableStringValue(payload.identity_id, null) ??
		stringValue(payload.provider_identity_id)
	if (!identityId) return items

	const nextItem: WhatsAppPresenceSyncItem = {
		identity_id: identityId,
		account_id: payloadAccountId,
		channel_kind: 'whatsapp_web',
		provider_chat_id: payloadProviderChatId,
		provider_identity_id: stringValue(payload.provider_identity_id) ?? identityId,
		identity_kind: stringValue(payload.identity_kind) ?? 'whatsapp',
		display_name: nullableStringValue(payload.display_name, null),
		address: nullableStringValue(payload.address, null),
		presence_state: stringValue(payload.presence_state) ?? 'unknown',
		last_seen_at: nullableStringValue(payload.last_seen_at, null),
		observed_at: nullableStringValue(payload.observed_at, null),
		identity_metadata: {},
	}

	let changed = false
	const updatedItems = items.map((item) => {
		if (
			item.identity_id !== identityId &&
			item.provider_identity_id !== nextItem.provider_identity_id
		) {
			return item
		}
		changed = true
		return {
			...item,
			...nextItem,
			channel_kind: item.channel_kind,
			identity_metadata: item.identity_metadata ?? {},
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}

export function patchCallList(
	queryKey: readonly unknown[],
	items: WhatsAppCallSyncItem[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppCallSyncItem[] | undefined {
	if (!items || eventType !== 'whatsapp.call.updated') return items

	const payloadAccountId = stringValue(payload.account_id)
	const callId = stringValue(payload.call_id)
	const providerCallId = stringValue(payload.provider_call_id)
	const providerChatId = stringValue(payload.provider_chat_id)
	if (!payloadAccountId || !callId || !providerCallId || !providerChatId) return items

	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const queryProviderChatId =
		typeof queryKey[5] === 'string' && queryKey[5] !== 'all' ? queryKey[5] : null
	if (queryProviderChatId && queryProviderChatId !== providerChatId) return items

	const nextItem: WhatsAppCallSyncItem = {
		call_id: callId,
		account_id: payloadAccountId,
		provider_call_id: providerCallId,
		provider_chat_id: providerChatId,
		direction: stringValue(payload.direction) ?? 'unknown',
		call_state: stringValue(payload.call_state) ?? 'unknown',
		started_at: nullableStringValue(payload.started_at, null),
		ended_at: nullableStringValue(payload.ended_at, null),
		observed_at: nullableStringValue(payload.observed_at, null),
		metadata: {},
	}

	let changed = false
	const updatedItems = items.map((item) => {
		if (item.call_id !== callId && item.provider_call_id !== providerCallId) return item
		changed = true
		return {
			...item,
			...nextItem,
			metadata: item.metadata ?? {},
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}

export function patchStatusesList(
	queryKey: readonly unknown[],
	items: WhatsappWebMessage[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsappWebMessage[] | undefined {
	if (!items) return items
	if (eventType !== 'whatsapp.status.updated' && eventType !== 'whatsapp.status.deleted') {
		return items
	}

	const payloadAccountId = stringValue(payload.account_id)
	const messageId = stringValue(payload.message_id)
	if (!payloadAccountId || !messageId) return items

	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const statusState =
		stringValue(payload.status_state) ??
		(eventType === 'whatsapp.status.deleted' ? 'deleted' : 'posted')
	const providerStatusId = stringValue(payload.provider_status_id) ?? messageId
	const nextOccurredAt =
		nullableStringValue(payload.occurred_at, null) ??
		nullableStringValue(payload.observed_at, null)

	const nextItem: WhatsappWebMessage = {
		message_id: messageId,
		raw_record_id: stringValue(payload.raw_record_id) ?? `status:${providerStatusId}`,
		account_id: payloadAccountId,
		provider_message_id: providerStatusId,
		provider_chat_id: `whatsapp_status_feed:${payloadAccountId}`,
		chat_title: 'status-feed',
		sender:
			stringValue(payload.sender_id) ??
			stringValue(payload.sender_address) ??
			providerStatusId,
		sender_display_name:
			nullableStringValue(payload.sender_display_name, null) ??
			nullableStringValue(payload.viewer_display_name, null),
		text: '',
		occurred_at: nextOccurredAt,
		projected_at: nextOccurredAt ?? new Date().toISOString(),
		channel_kind: 'whatsapp_web',
		delivery_state: statusState === 'deleted' ? 'deleted' : 'published',
		metadata: {
			provider_status_id: providerStatusId,
			status_state: statusState,
			...(stringValue(payload.sender_identity_kind)
				? { sender_identity_kind: stringValue(payload.sender_identity_kind) }
				: {}),
			...(stringValue(payload.sender_address)
				? { sender_address: stringValue(payload.sender_address) }
				: {}),
			...(stringValue(payload.sender_push_name)
				? { sender_push_name: stringValue(payload.sender_push_name) }
				: {}),
			...(stringValue(payload.viewer_id) ? { viewer_id: stringValue(payload.viewer_id) } : {}),
			...(stringValue(payload.actor_class)
				? { actor_class: stringValue(payload.actor_class) }
				: {}),
			...(stringValue(payload.reason_class)
				? { reason_class: stringValue(payload.reason_class) }
				: {}),
			...(stringValue(payload.tombstone_id)
				? { tombstone_id: stringValue(payload.tombstone_id) }
				: {}),
		},
	}

	let changed = false
	const updatedItems = items.map((item) => {
		const itemProviderStatusId =
			typeof item.metadata?.provider_status_id === 'string'
				? item.metadata.provider_status_id
				: null
		if (item.message_id !== messageId && itemProviderStatusId !== providerStatusId) return item
		changed = true
		return {
			...item,
			...nextItem,
			text: item.text,
			metadata: {
				...item.metadata,
				...nextItem.metadata,
			},
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}

export function patchChatsList(
	queryKey: readonly unknown[],
	items: WhatsAppChatSyncItem[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppChatSyncItem[] | undefined {
	if (!items || eventType !== 'whatsapp.dialog.updated') return items

	const payloadAccountId = stringValue(payload.account_id)
	const providerChatId = stringValue(payload.provider_chat_id)
	if (!payloadAccountId || !providerChatId) return items

	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const nextItem: WhatsAppChatSyncItem = {
		conversation_id: stringValue(payload.conversation_id) ?? providerChatId,
		account_id: payloadAccountId,
		channel_kind: 'whatsapp_web',
		provider_chat_id: providerChatId,
		title: stringValue(payload.chat_title) ?? providerChatId,
		chat_kind: nullableStringValue(payload.chat_kind, null),
		is_archived: booleanValue(payload.is_archived) ?? false,
		is_pinned: booleanValue(payload.is_pinned) ?? false,
		is_muted: booleanValue(payload.is_muted) ?? false,
		is_unread: booleanValue(payload.is_unread) ?? false,
		unread_count: integerValue(payload.unread_count),
		participant_count: integerValue(payload.participant_count),
		community_parent_chat_id: nullableStringValue(payload.community_parent_chat_id, null),
		community_parent_title: nullableStringValue(payload.community_parent_title, null),
		invite_link: nullableStringValue(payload.invite_link, null),
		is_community_root: booleanValue(payload.is_community_root) ?? false,
		is_broadcast: booleanValue(payload.is_broadcast) ?? false,
		is_newsletter: booleanValue(payload.is_newsletter) ?? false,
		avatar_metadata: isRecord(payload.avatar_metadata)
			? { ...(payload.avatar_metadata as Record<string, unknown>) }
			: {},
		provider_labels: stringArray(payload.provider_labels) ?? [],
	}

	let changed = false
	const updatedItems = items.map((item) => {
		if (item.provider_chat_id !== providerChatId && item.conversation_id !== nextItem.conversation_id) {
			return item
		}
		changed = true
		return {
			...item,
			...nextItem,
			channel_kind: item.channel_kind,
			avatar_metadata: nextItem.avatar_metadata,
			provider_labels: nextItem.provider_labels,
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}

export function patchMembersList(
	queryKey: readonly unknown[],
	items: WhatsAppMembersSyncItem[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppMembersSyncItem[] | undefined {
	if (!items || eventType !== 'whatsapp.participant.changed') return items

	const payloadAccountId = stringValue(payload.account_id)
	const providerChatId = stringValue(payload.provider_chat_id)
	const participantId = stringValue(payload.participant_id)
	const providerMemberId = stringValue(payload.provider_member_id)
	if (!payloadAccountId || !providerChatId || !participantId || !providerMemberId) return items

	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const queryProviderChatId =
		typeof queryKey[5] === 'string' && queryKey[5] !== 'none' ? queryKey[5] : null
	if (queryProviderChatId && queryProviderChatId !== providerChatId) return items

	const nextItem: WhatsAppMembersSyncItem = {
		participant_id: participantId,
		conversation_id: stringValue(payload.conversation_id) ?? providerChatId,
		account_id: payloadAccountId,
		provider_chat_id: providerChatId,
		provider_member_id: providerMemberId,
		provider_identity_id: nullableStringValue(payload.provider_identity_id, null),
		sender_display_name: nullableStringValue(payload.display_name, null),
		role: stringValue(payload.role) ?? 'member',
		status: nullableStringValue(payload.status, null),
		identity_kind: null,
		address: null,
		is_admin: false,
		is_owner: false,
		participant_metadata: {
			...(stringValue(payload.previous_role)
				? { previous_role: stringValue(payload.previous_role) }
				: {}),
			...(stringValue(payload.previous_status)
				? { previous_status: stringValue(payload.previous_status) }
				: {}),
			...(booleanValue(payload.role_changed) !== null
				? { role_changed: booleanValue(payload.role_changed) }
				: {}),
			...(booleanValue(payload.membership_changed) !== null
				? { membership_changed: booleanValue(payload.membership_changed) }
				: {}),
		},
		identity_metadata: {},
	}

	let changed = false
	const updatedItems = items.map((item) => {
		if (
			item.participant_id !== participantId &&
			item.provider_member_id !== providerMemberId
		) {
			return item
		}
		changed = true
		return {
			...item,
			...nextItem,
			identity_kind: item.identity_kind,
			address: item.address,
			is_admin: item.is_admin,
			is_owner: item.is_owner,
			participant_metadata: {
				...item.participant_metadata,
				...nextItem.participant_metadata,
			},
			identity_metadata: item.identity_metadata ?? {},
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}

export function patchContactsList(
	queryKey: readonly unknown[],
	items: WhatsAppContactSyncItem[] | undefined,
	eventType: string,
	payload: WhatsAppRuntimeEventPayload
): WhatsAppContactSyncItem[] | undefined {
	if (
		!items ||
		(eventType !== 'whatsapp.participant.changed' &&
			eventType !== 'whatsapp.presence.changed' &&
			eventType !== 'whatsapp.status.updated')
	) {
		return items
	}

	const payloadAccountId = stringValue(payload.account_id)
	if (!payloadAccountId) return items
	const queryAccountId =
		typeof queryKey[4] === 'string' && queryKey[4] !== 'none' ? queryKey[4] : null
	if (queryAccountId && queryAccountId !== payloadAccountId) return items

	const providerIdentityId =
		stringValue(payload.provider_identity_id) ??
		stringValue(payload.sender_id) ??
		stringValue(payload.identity_id)
	if (!providerIdentityId) return items

	const identityId = stringValue(payload.identity_id) ?? providerIdentityId
	const displayName =
		nullableStringValue(payload.display_name, null) ??
		nullableStringValue(payload.sender_display_name, null)
	const address = nullableStringValue(payload.address, null) ??
		nullableStringValue(payload.sender_address, null)
	const pushName = nullableStringValue(payload.sender_push_name, null)
	const identityKind =
		stringValue(payload.identity_kind) ??
		stringValue(payload.sender_identity_kind) ??
		'whatsapp'

	const nextItem: WhatsAppContactSyncItem = {
		identity_id: identityId,
		account_id: payloadAccountId,
		channel_kind: 'whatsapp_web',
		provider_identity_id: providerIdentityId,
		identity_kind: identityKind,
		display_name: displayName,
		address,
		push_name: pushName,
		business_profile: {},
		profile_photo_ref: {},
		display_name_history: displayName ? [displayName] : [],
		identity_metadata: {},
		whatsapp_trace_metadata: {},
		phone_trace_metadata: {},
	}

	let changed = false
	const updatedItems = items.map((item) => {
		if (
			item.identity_id !== identityId &&
			item.provider_identity_id !== providerIdentityId
		) {
			return item
		}
		changed = true
		const mergedDisplayNameHistory = displayName
			? Array.from(new Set([displayName, ...item.display_name_history]))
			: item.display_name_history
		return {
			...item,
			...nextItem,
			business_profile: item.business_profile ?? {},
			profile_photo_ref: item.profile_photo_ref ?? {},
			identity_metadata: item.identity_metadata ?? {},
			whatsapp_trace_metadata: item.whatsapp_trace_metadata ?? {},
			phone_trace_metadata: item.phone_trace_metadata ?? {},
			display_name_history: mergedDisplayNameHistory,
		}
	})

	if (changed) return updatedItems
	return [nextItem, ...items]
}
