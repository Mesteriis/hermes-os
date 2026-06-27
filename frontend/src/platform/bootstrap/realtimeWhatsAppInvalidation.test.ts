import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('whatsapp realtime invalidation handling', () => {
	it('invalidates whatsapp message queries for whatsapp message events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-10',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.message.created',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(1)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'messages'],
		})
	})

	it('invalidates whatsapp conversation and message queries for dialog events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-10b',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.dialog.updated',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(3)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'conversations'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'conversation-detail'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'messages'],
		})
	})

	it('invalidates whatsapp session and capability queries for runtime lifecycle events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-11',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.runtime.status_changed',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(5)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'sessions'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'capabilities'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'account-capabilities'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'status'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'health'],
		})
	})

	it('invalidates whatsapp session and capability queries for media lifecycle events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-12',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.media.download.progress',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(7)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'commands'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'sessions'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'capabilities'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'account-capabilities'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'status'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'health'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-media'],
		})
	})

	it('invalidates whatsapp conversation queries for participant events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-13',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.participant.changed',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(4)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'conversations'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'conversation-detail'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'chat-members'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-contacts'],
		})
	})

	it('invalidates whatsapp runtime sync queries for presence, calls and statuses', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-presence-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.presence.changed',
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-call-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.call.updated',
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-status-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.status.updated',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-presence'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-calls'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-statuses'],
		})
	})

	it('invalidates whatsapp runtime sync queries for chats, history and members on sync lifecycle events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-sync-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.sync.completed',
						payload: {
							scope: 'history',
							provider_chat_id: 'chat-1',
						},
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-chats'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-history'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-members'],
		})
	})
})
