import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('Signal Hub realtime invalidation', () => {
  it('invalidates Signal Hub queries for every supported signal event family', () => {
    const signalEventTypes = [
      'signal.raw.telegram.message.observed',
      'signal.accepted.telegram.message',
      'signal.rejected.mail.message',
      'signal.muted.system.runtime',
      'signal.paused.ai.task',
      'signal.resumed.telegram.runtime',
      'signal.replayed.whatsapp.message'
    ] as const

    for (const eventType of signalEventTypes) {
      const queryClient = { invalidateQueries: vi.fn() }

      handleRealtimeEvent(
        {
          id: '42',
          event: 'event',
          data: JSON.stringify({
            event: {
              event_type: eventType
            }
          })
        },
        queryClient
      )

      expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey: ['signal-hub']
      })
      expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(1)
    }
  })

  it('keeps Signal Hub invalidation on lagged stream recovery', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '42',
        event: 'lagged',
        data: JSON.stringify({ skipped: 7 })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['signal-hub']
    })
  })
})
