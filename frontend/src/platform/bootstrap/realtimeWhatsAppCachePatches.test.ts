import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'
import type { RealtimeQueryClient } from './realtime'

describe('whatsapp realtime cache patch handling', () => {
	it('patches cached whatsapp conversation rows for dialog lifecycle events', () => {
		const conversationsKey = ['communications', 'whatsapp', 'conversations', 'account-1', 50]
		const conversationDetailKey = ['communications', 'whatsapp', 'conversation-detail', 'wa-chat-1']
		const conversations = [
			{
				conversation_id: 'wa-chat-1',
				account_id: 'account-1',
				provider_chat_id: 'wa-chat-1',
				chat_kind: 'group',
				title: 'Family',
				last_message_at: '2026-06-16T09:00:00Z',
				metadata: {
					is_pinned: false,
					is_archived: false,
					is_muted: false,
					is_unread: false,
					unread_count: 0,
				},
				created_at: '2026-06-16T09:00:00Z',
				updated_at: '2026-06-16T09:00:00Z',
			},
		]
		const detail = conversations[0]
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function'
				? String(queryKey[2]) === 'conversation-detail'
					? updater(detail)
					: updater(conversations)
				: updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) =>
				String(queryKey[2]) === 'conversation-detail'
					? [[conversationDetailKey, detail]]
					: [[conversationsKey, conversations]]
			),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-dialog-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.dialog.updated',
						payload: {
							account_id: 'account-1',
							conversation_id: 'wa-chat-1',
							provider_chat_id: 'wa-chat-1',
							chat_title: 'Family',
							chat_kind: 'group',
							is_pinned: true,
							is_archived: true,
							is_muted: true,
							is_unread: true,
							unread_count: 4,
							participant_count: 7,
							observed_at: '2026-06-16T09:02:00Z',
						},
					},
				}),
			},
			queryClient
		)

		const patchedConversationList = setQueryData.mock.results[0]?.value
		const patchedConversationDetail = setQueryData.mock.results[1]?.value
		expect(patchedConversationList[0].metadata).toMatchObject({
			is_pinned: true,
			is_archived: true,
			is_muted: true,
			is_unread: true,
			unread_count: 4,
			participant_count: 7,
		})
		expect(patchedConversationDetail.metadata).toMatchObject({
			is_pinned: true,
			is_archived: true,
			is_muted: true,
			is_unread: true,
			unread_count: 4,
			participant_count: 7,
		})
	})

	it('patches cached whatsapp message reaction summary for reaction events', () => {
		const messageKey = ['communications', 'whatsapp', 'messages', 'account-1', 'chat-1', 50]
		const messages = [
			{
				message_id: 'wa-msg-1',
				raw_record_id: 'raw-1',
				account_id: 'account-1',
				provider_message_id: 'provider-1',
				provider_chat_id: 'chat-1',
				chat_title: 'Chat',
				sender: 'sender-1',
				sender_display_name: 'Sender',
				text: 'Hello',
				occurred_at: '2026-06-16T09:00:00Z',
				projected_at: '2026-06-16T09:00:01Z',
				channel_kind: 'whatsapp_web' as const,
				delivery_state: 'received',
				metadata: {},
			},
		]
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function' ? updater(messages) : updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn().mockReturnValue([[messageKey, messages]]),
			setQueryData,
		}

		handleRealtimeEvent(
			{
				id: 'wa-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.reaction.changed',
						payload: {
							account_id: 'account-1',
							provider_chat_id: 'chat-1',
							message_id: 'wa-msg-1',
							reaction: '+1',
							is_active: true,
						},
					},
				}),
			},
			queryClient
		)

		const patchedMessages = setQueryData.mock.results[0]?.value
		expect(patchedMessages[0].metadata.reaction_summary.reactions[0]).toMatchObject({
			reaction: '+1',
			count: 1,
		})
	})

	it('patches cached whatsapp session link-state and runtime metadata events', () => {
		const sessionsKey = ['integrations', 'whatsapp', 'sessions', 'account-1', 50]
		const runtimeStatusKey = ['integrations', 'whatsapp', 'runtime', 'status', 'account-1']
		const sessions = [
			{
				session_id: 'session-1',
				account_id: 'account-1',
				device_name: 'Hermes Desktop',
				companion_runtime: 'fixture' as const,
				link_state: 'qr_pending' as const,
				local_state_path: 'docker/data/whatsapp/session-1',
				last_sync_at: null,
				metadata: {},
				created_at: '2026-06-16T09:00:00Z',
				updated_at: '2026-06-16T09:00:00Z',
			},
		]
		const runtimeStatus = {
			account_id: 'account-1',
			provider_kind: 'whatsapp_web',
			provider_shape: 'whatsapp_web_companion',
			runtime_kind: 'fixture',
			status: 'qr_pending',
			fixture_runtime: true,
			live_runtime_available: false,
			live_send_available: false,
			qr_pairing_available: true,
			pair_code_available: true,
			media_download_available: false,
			media_upload_available: false,
			session_restore_available: false,
			session_secret_ref: null,
			runtime_blockers: ['whatsapp_session_link_required'],
			last_error: null,
			updated_at: '2026-06-16T09:00:00Z',
		}
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function'
				? String(queryKey[3]) === 'status'
					? updater(runtimeStatus)
					: updater(sessions)
				: updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) =>
				String(queryKey[3]) === 'status'
					? [[runtimeStatusKey, runtimeStatus]]
					: [[sessionsKey, sessions]]
			),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-2',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.session.link_state_changed',
						payload: {
							account_id: 'account-1',
							link_state: 'linked',
							occurred_at: '2026-06-16T09:01:00Z',
						},
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-3',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.runtime.status_changed',
						payload: {
							account_id: 'account-1',
							status: 'available',
							source: 'runtime_start',
							occurred_at: '2026-06-16T09:02:00Z',
						},
					},
				}),
			},
			queryClient
		)

		expect(setQueryData.mock.results[0]?.value[0].link_state).toBe('linked')
		expect(setQueryData.mock.results[1]?.value.status).toBe('linked')
		expect(setQueryData.mock.results[2]?.value[0].metadata.runtime_status).toBe('available')
		expect(setQueryData.mock.results[2]?.value[0].metadata.runtime_status_source).toBe(
			'runtime_start'
		)
		expect(setQueryData.mock.results[3]?.value.status).toBe('available')
		expect(setQueryData.mock.results[3]?.value.runtime_blockers).toEqual([
			'whatsapp_session_link_required',
		])
	})

	it('patches cached whatsapp provider commands for command status events', () => {
		const commandsKey = ['integrations', 'whatsapp', 'commands', 'account-1', 25]
		const commands = [
			{
				command_id: 'wa-cmd-1',
				account_id: 'account-1',
				command_kind: 'publish_status',
				idempotency_key: 'status:1',
				provider_chat_id: 'status-feed',
				provider_message_id: null,
				capability_state: 'blocked',
				action_class: 'provider_write',
				confirmation_decision: 'not_required',
				status: 'queued',
				retry_count: 0,
				max_retries: 3,
				last_error: null,
				result_payload: {},
				audit_metadata: {},
				provider_state: {},
				reconciliation_status: 'not_observed',
				next_attempt_at: null,
				last_attempt_at: null,
				provider_observed_at: null,
				reconciled_at: null,
				dead_lettered_at: null,
				completed_at: null,
				created_at: '2026-06-16T09:00:00Z',
				updated_at: '2026-06-16T09:00:00Z',
			},
		]
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function' ? updater(commands) : updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) =>
				String(queryKey[2]) === 'commands' ? [[commandsKey, commands]] : []
			),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-cmd-status-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.command.status_changed',
						payload: {
							account_id: 'account-1',
							command_id: 'wa-cmd-1',
							command_kind: 'publish_status',
							provider_chat_id: 'status-feed',
							status: 'completed',
							reconciliation_status: 'observed',
							provider_observed_at: '2026-06-16T09:01:00Z',
							completed_at: '2026-06-16T09:01:00Z',
						},
					},
				}),
			},
			queryClient
		)

		expect(setQueryData).toHaveBeenCalledTimes(1)
		expect(setQueryData.mock.results[0]?.value[0]).toMatchObject({
			command_id: 'wa-cmd-1',
			status: 'completed',
			reconciliation_status: 'observed',
			provider_observed_at: '2026-06-16T09:01:00Z',
			completed_at: '2026-06-16T09:01:00Z',
		})
	})

	it('patches cached whatsapp presence and call sync snapshots from direct runtime events', () => {
		const presenceKey = ['integrations', 'whatsapp', 'runtime', 'sync-presence', 'account-1', 'chat-1', 8]
		const callsKey = ['integrations', 'whatsapp', 'runtime', 'sync-calls', 'account-1', 'chat-1', 8]
		let presenceItems = [
			{
				identity_id: 'identity-1',
				account_id: 'account-1',
				channel_kind: 'whatsapp_web',
				provider_chat_id: 'chat-1',
				provider_identity_id: 'wa:+111',
				identity_kind: 'whatsapp',
				display_name: 'Alice',
				address: '+111',
				presence_state: 'offline',
				last_seen_at: '2026-06-16T09:00:00Z',
				observed_at: '2026-06-16T09:00:00Z',
				identity_metadata: {},
			},
		]
		let callItems = [
			{
				call_id: 'call-1',
				account_id: 'account-1',
				provider_call_id: 'provider-call-1',
				provider_chat_id: 'chat-1',
				direction: 'incoming',
				call_state: 'ringing',
				started_at: '2026-06-16T09:00:00Z',
				ended_at: null,
				observed_at: '2026-06-16T09:00:00Z',
				metadata: {},
			},
		]
		const setQueryData = vi.fn((queryKey, updater) => {
			if (typeof updater !== 'function') return updater
			if (String(queryKey[3]) === 'sync-presence') {
				presenceItems = updater(presenceItems)
				return presenceItems
			}
			if (String(queryKey[3]) === 'sync-calls') {
				callItems = updater(callItems)
				return callItems
			}
			return undefined
		})
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) => {
				if (String(queryKey[3]) === 'sync-presence') return [[presenceKey, presenceItems]]
				if (String(queryKey[3]) === 'sync-calls') return [[callsKey, callItems]]
				return []
			}),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-presence-patch-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.presence.changed',
						payload: {
							account_id: 'account-1',
							identity_id: 'identity-1',
							provider_chat_id: 'chat-1',
							provider_identity_id: 'wa:+111',
							identity_kind: 'whatsapp',
							display_name: 'Alice Cooper',
							address: '+111',
							presence_state: 'typing',
							last_seen_at: '2026-06-16T09:01:00Z',
							observed_at: '2026-06-16T09:01:00Z',
						},
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-call-patch-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.call.updated',
						payload: {
							account_id: 'account-1',
							call_id: 'call-1',
							provider_call_id: 'provider-call-1',
							provider_chat_id: 'chat-1',
							direction: 'incoming',
							call_state: 'connected',
							started_at: '2026-06-16T09:00:00Z',
							ended_at: '2026-06-16T09:05:00Z',
							observed_at: '2026-06-16T09:05:00Z',
						},
					},
				}),
			},
			queryClient
		)

		const patchedPresence = setQueryData.mock.results[0]?.value
		const patchedCalls = setQueryData.mock.results[1]?.value
		expect(patchedPresence[0]).toMatchObject({
			display_name: 'Alice Cooper',
			presence_state: 'typing',
			last_seen_at: '2026-06-16T09:01:00Z',
		})
		expect(patchedCalls[0]).toMatchObject({
			call_state: 'connected',
			ended_at: '2026-06-16T09:05:00Z',
			observed_at: '2026-06-16T09:05:00Z',
		})
	})

	it('patches cached whatsapp status sync snapshots from direct status events', () => {
		const statusesKey = ['integrations', 'whatsapp', 'runtime', 'sync-statuses', 'account-1', 8]
		const statusItems = [
			{
				message_id: 'status-msg-1',
				raw_record_id: 'raw-status-1',
				account_id: 'account-1',
				provider_message_id: 'provider-status-1',
				provider_chat_id: 'whatsapp_status_feed:account-1',
				chat_title: 'status-feed',
				sender: 'sender-1',
				sender_display_name: 'Alice',
				text: 'Old status',
				occurred_at: '2026-06-16T09:00:00Z',
				projected_at: '2026-06-16T09:00:01Z',
				channel_kind: 'whatsapp_web' as const,
				delivery_state: 'published',
				metadata: {
					provider_status_id: 'provider-status-1',
					status_state: 'posted',
				},
			},
		]
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function' ? updater(statusItems) : updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) =>
				String(queryKey[3]) === 'sync-statuses' ? [[statusesKey, statusItems]] : []
			),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-status-patch-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.status.updated',
						payload: {
							account_id: 'account-1',
							message_id: 'status-msg-1',
							raw_record_id: 'raw-status-1',
							provider_status_id: 'provider-status-1',
							sender_id: 'sender-1',
							sender_display_name: 'Alice Cooper',
							sender_identity_kind: 'whatsapp',
							sender_address: '+111',
							sender_push_name: 'Ali',
							occurred_at: '2026-06-16T09:01:00Z',
							status_state: 'viewed',
						},
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-status-patch-2',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.status.deleted',
						payload: {
							account_id: 'account-1',
							message_id: 'status-msg-1',
							raw_record_id: 'raw-status-1',
							provider_status_id: 'provider-status-1',
							tombstone_id: 'tombstone-1',
							actor_class: 'owner',
							reason_class: 'manual_delete',
							observed_at: '2026-06-16T09:02:00Z',
							status_state: 'deleted',
						},
					},
				}),
			},
			queryClient
		)

		const patchedViewedStatus = setQueryData.mock.results[0]?.value
		const patchedDeletedStatus = setQueryData.mock.results[1]?.value
		expect(patchedViewedStatus[0]).toMatchObject({
			sender_display_name: 'Alice Cooper',
			occurred_at: '2026-06-16T09:01:00Z',
		})
		expect(patchedViewedStatus[0].metadata).toMatchObject({
			status_state: 'viewed',
			sender_identity_kind: 'whatsapp',
			sender_address: '+111',
			sender_push_name: 'Ali',
		})
		expect(patchedDeletedStatus[0]).toMatchObject({
			delivery_state: 'deleted',
			occurred_at: '2026-06-16T09:02:00Z',
		})
		expect(patchedDeletedStatus[0].metadata).toMatchObject({
			status_state: 'deleted',
			tombstone_id: 'tombstone-1',
			actor_class: 'owner',
			reason_class: 'manual_delete',
		})
	})

	it('patches cached whatsapp chat and member sync snapshots from dialog and participant events', () => {
		const chatsKey = ['integrations', 'whatsapp', 'runtime', 'sync-chats', 'account-1', 8]
		const membersKey = ['integrations', 'whatsapp', 'runtime', 'sync-members', 'account-1', 'chat-1', 8]
		const chatItems = [
			{
				conversation_id: 'conversation-1',
				account_id: 'account-1',
				channel_kind: 'whatsapp_web',
				provider_chat_id: 'chat-1',
				title: 'Family',
				chat_kind: 'group',
				is_archived: false,
				is_pinned: false,
				is_muted: false,
				is_unread: false,
				unread_count: 0,
				participant_count: 3,
				community_parent_chat_id: null,
				community_parent_title: null,
				invite_link: null,
				is_community_root: false,
				is_broadcast: false,
				is_newsletter: false,
				avatar_metadata: {},
				provider_labels: [],
			},
		]
		const memberItems = [
			{
				participant_id: 'participant-1',
				conversation_id: 'conversation-1',
				account_id: 'account-1',
				provider_chat_id: 'chat-1',
				provider_member_id: 'member-1',
				provider_identity_id: 'wa:+111',
				sender_display_name: 'Alice',
				role: 'member',
				status: 'active',
				identity_kind: 'whatsapp',
				address: '+111',
				is_admin: false,
				is_owner: false,
				participant_metadata: {},
				identity_metadata: {},
			},
		]
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function'
				? String(queryKey[3]) === 'sync-chats'
					? updater(chatItems)
					: updater(memberItems)
				: updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) => {
				if (String(queryKey[3]) === 'sync-chats') return [[chatsKey, chatItems]]
				if (String(queryKey[3]) === 'sync-members') return [[membersKey, memberItems]]
				return []
			}),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-dialog-patch-2',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.dialog.updated',
						payload: {
							account_id: 'account-1',
							conversation_id: 'conversation-1',
							provider_chat_id: 'chat-1',
							chat_title: 'Family HQ',
							chat_kind: 'group',
							is_pinned: true,
							is_archived: true,
							is_muted: true,
							is_unread: true,
							unread_count: 5,
							participant_count: 7,
							provider_labels: ['vip'],
							avatar_metadata: { color: 'green' },
						},
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-member-patch-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.participant.changed',
						payload: {
							account_id: 'account-1',
							conversation_id: 'conversation-1',
							participant_id: 'participant-1',
							provider_chat_id: 'chat-1',
							provider_member_id: 'member-1',
							provider_identity_id: 'wa:+111',
							display_name: 'Alice Cooper',
							role: 'admin',
							status: 'joined',
							previous_role: 'member',
							previous_status: 'active',
							role_changed: true,
							membership_changed: true,
						},
					},
				}),
			},
			queryClient
		)

		const patchedChats = setQueryData.mock.results[0]?.value
		const patchedMembers = setQueryData.mock.results[1]?.value
		expect(patchedChats[0]).toMatchObject({
			title: 'Family HQ',
			is_pinned: true,
			is_archived: true,
			is_muted: true,
			is_unread: true,
			unread_count: 5,
			participant_count: 7,
			provider_labels: ['vip'],
		})
		expect(patchedChats[0].avatar_metadata).toMatchObject({ color: 'green' })
		expect(patchedMembers[0]).toMatchObject({
			sender_display_name: 'Alice Cooper',
			role: 'admin',
			status: 'joined',
		})
		expect(patchedMembers[0].participant_metadata).toMatchObject({
			previous_role: 'member',
			previous_status: 'active',
			role_changed: true,
			membership_changed: true,
		})
	})

	it('patches cached whatsapp contact sync snapshots from participant, presence and status events', () => {
		const contactsKey = ['integrations', 'whatsapp', 'runtime', 'sync-contacts', 'account-1', 8]
		let contactItems = [
			{
				identity_id: 'identity-1',
				account_id: 'account-1',
				channel_kind: 'whatsapp_web',
				provider_identity_id: 'wa:+111',
				identity_kind: 'whatsapp',
				display_name: 'Alice',
				address: '+111',
				push_name: null,
				business_profile: {},
				profile_photo_ref: {},
				display_name_history: ['Alice'],
				identity_metadata: {},
				whatsapp_trace_metadata: {},
				phone_trace_metadata: {},
			},
		]
		const setQueryData = vi.fn((queryKey, updater) => {
			if (typeof updater !== 'function') return updater
			contactItems = updater(contactItems)
			return contactItems
		})
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) =>
				String(queryKey[3]) === 'sync-contacts' ? [[contactsKey, contactItems]] : []
			),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-contact-participant-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.participant.changed',
						payload: {
							account_id: 'account-1',
							identity_id: 'identity-1',
							provider_identity_id: 'wa:+111',
							display_name: 'Alice Cooper',
							identity_kind: 'whatsapp',
						},
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-contact-presence-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.presence.changed',
						payload: {
							account_id: 'account-1',
							identity_id: 'identity-1',
							provider_identity_id: 'wa:+111',
							display_name: 'Alice Runtime',
							address: '+111',
							identity_kind: 'whatsapp',
						},
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-contact-status-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.status.updated',
						payload: {
							account_id: 'account-1',
							sender_id: 'wa:+111',
							sender_display_name: 'Alice Status',
							sender_identity_kind: 'whatsapp',
							sender_address: '+111',
							sender_push_name: 'Ali',
						},
					},
				}),
			},
			queryClient
		)

		const patchedFromParticipant = setQueryData.mock.results[0]?.value
		const patchedFromPresence = setQueryData.mock.results[1]?.value
		const patchedFromStatus = setQueryData.mock.results[2]?.value
		expect(patchedFromParticipant[0]).toMatchObject({
			display_name: 'Alice Cooper',
			identity_kind: 'whatsapp',
		})
		expect(patchedFromPresence[0]).toMatchObject({
			display_name: 'Alice Runtime',
			address: '+111',
		})
		expect(patchedFromStatus[0]).toMatchObject({
			display_name: 'Alice Status',
			push_name: 'Ali',
			address: '+111',
		})
		expect(patchedFromStatus[0].display_name_history).toEqual(
			expect.arrayContaining(['Alice', 'Alice Status'])
		)
	})
})
