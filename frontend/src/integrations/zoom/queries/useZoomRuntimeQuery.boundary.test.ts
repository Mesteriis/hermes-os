import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('zoom runtime query mutation invalidation boundary', () => {
  it('invalidates derived provider call caches for provider sync without exposing diagnostic mutations', () => {
    const source = readFileSync(new URL('./useZoomRuntimeQuery.ts', import.meta.url), 'utf8')

    expect(source).toContain('function invalidateZoomDerived')
    expect(source).toContain("queryClient.invalidateQueries({ queryKey: zoomQueryKeys.providerCalls })")
    expect(source).toContain("queryClient.invalidateQueries({ queryKey: zoomQueryKeys.callTranscript })")
    expect(source).toContain('useSyncZoomRecordingsMutation')
    expect(source).not.toContain('useSetupZoomSyntheticAccountMutation')
    expect(source).not.toContain('useBridgeZoomMeetingMutation')
    expect(source).not.toContain('useBridgeZoomRecordingMutation')
    expect(source).not.toContain('useBridgeZoomTranscriptMutation')
    expect(source).not.toContain('useImportZoomTranscriptFileMutation')
  })
})
