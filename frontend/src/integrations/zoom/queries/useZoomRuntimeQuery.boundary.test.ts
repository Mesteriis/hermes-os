import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('zoom runtime query mutation invalidation boundary', () => {
  it('invalidates derived provider call caches for local ingest and provider sync mutations', () => {
    const source = readFileSync(new URL('./useZoomRuntimeQuery.ts', import.meta.url), 'utf8')

    expect(source).toContain('function invalidateZoomDerived')
    expect(source).toContain("queryClient.invalidateQueries({ queryKey: zoomQueryKeys.providerCalls })")
    expect(source).toContain("queryClient.invalidateQueries({ queryKey: zoomQueryKeys.callTranscript })")
    expect(source).toContain('useBridgeZoomMeetingMutation')
    expect(source).toContain('useBridgeZoomRecordingMutation')
    expect(source).toContain('useBridgeZoomTranscriptMutation')
    expect(source).toContain('useImportZoomTranscriptFileMutation')
    expect(source).toContain('useSyncZoomRecordingsMutation')
  })
})
