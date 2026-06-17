import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram realtime invalidation handling', () => {
  it('invalidates telegram chat and message queries for telegram message events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-45',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.message.created',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'messages'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'chats'],
    })
  })

  it('invalidates telegram runtime-related queries for telegram sync progress events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-46',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.sync.progress',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(3)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'chats'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'messages'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'runtime'],
    })
  })

  it('invalidates telegram message and runtime queries for command status events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-47',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'messages'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'runtime'],
    })
  })

  it('invalidates telegram message and media search queries for media events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-48',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.download.progress',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'messages'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'search', 'media'],
    })
  })

  it('invalidates telegram chat and runtime queries for typing events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-49',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.typing.changed',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'chats'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'runtime'],
    })
  })

  it('invalidates telegram member queries for participant updates', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-50',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.participant.updated',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'chat-members'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'chats'],
    })
  })
})
