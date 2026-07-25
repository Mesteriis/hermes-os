import { describe, expect, it, vi } from 'vitest'
import { telegramQueryKeys } from '../../integrations/telegram/queries/telegramQueryKeys'
import { handleRealtimeEvent } from './realtime'

function telegramEvent(eventType: string) {
  return {
    id: `telegram:${eventType}`,
    event: 'event',
    data: JSON.stringify({ event: { event_type: eventType } }),
  }
}

describe('telegram realtime invalidation ownership', () => {
  it('invalidates only integration-owned caches for provider message events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(telegramEvent('telegram.message.created'), queryClient)

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: telegramQueryKeys.chats,
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: telegramQueryKeys.runtime,
    })
  })

  it('invalidates provider projections and runtime state after sync', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(telegramEvent('telegram.sync.completed'), queryClient)

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(4)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: telegramQueryKeys.chats,
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: telegramQueryKeys.folders,
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: telegramQueryKeys.chatMembers,
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: telegramQueryKeys.runtime,
    })
  })

  it('invalidates only the integration command queue and runtime for command events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(telegramEvent('telegram.command.status_changed'), queryClient)

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: telegramQueryKeys.runtime,
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: telegramQueryKeys.commands,
    })
  })

  it('keeps participants and folders under Telegram integration query roots', () => {
    const participantClient = { invalidateQueries: vi.fn() }
    const folderClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(telegramEvent('telegram.participant.updated'), participantClient)
    handleRealtimeEvent(telegramEvent('telegram.folders.changed'), folderClient)

    expect(participantClient.invalidateQueries).toHaveBeenNthCalledWith(1, {
      queryKey: telegramQueryKeys.chatMembers,
    })
    expect(participantClient.invalidateQueries).toHaveBeenNthCalledWith(2, {
      queryKey: telegramQueryKeys.chats,
    })
    expect(folderClient.invalidateQueries).toHaveBeenNthCalledWith(1, {
      queryKey: telegramQueryKeys.folders,
    })
    expect(folderClient.invalidateQueries).toHaveBeenNthCalledWith(2, {
      queryKey: telegramQueryKeys.chats,
    })
  })
})
