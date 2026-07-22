import { describe, expect, it, vi } from 'vitest'
import { applyMailProviderCommandDiagnosticsRealtimePatch } from './realtimeMailProviderCommandPatches'

describe('Mail provider command realtime patches', () => {
  it('ignores an event with a non-object payload', () => {
    const setQueryData = vi.fn()
    const getQueriesData = <TData>(): Array<[readonly unknown[], TData | undefined]> => [
      [['communications', 'mail', 'provider-command-diagnostics', 'account-1'], undefined]
    ]
    const queryClient = {
      getQueriesData,
      setQueryData
    }

    expect(applyMailProviderCommandDiagnosticsRealtimePatch(JSON.stringify({
      event: {
        event_type: 'communication.provider_command.status_changed.v1',
        payload: 'malformed'
      }
    }), queryClient)).toBe(false)
    expect(setQueryData).not.toHaveBeenCalled()
  })
})
