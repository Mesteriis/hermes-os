import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('zoom realtime invalidation handling', () => {
  it('invalidates zoom integration keys and current call caches for zoom events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'zoom-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'zoom.transcript.observed',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(8)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'accounts'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'capabilities'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'runtime', 'status'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'webhook-subscriptions'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'provider-calls'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'provider-call-transcript'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'recording-imports'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'audit-events'],
    })
  })
})
