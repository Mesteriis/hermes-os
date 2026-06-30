import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ZoomObservedCallsPanel boundary', () => {
  it('preserves call evidence helpers after removing the observed calls Vue panel', () => {
    const runtimeQuerySource = readFileSync(
      new URL('../queries/useZoomRuntimeQuery.ts', import.meta.url),
      'utf8'
    )
    const helperSource = readFileSync(new URL('./zoomEvidence.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./ZoomObservedCallsPanel.vue', import.meta.url))).toBe(false)
    expect(runtimeQuerySource).toContain('useZoomProviderCallsQuery')
    expect(runtimeQuerySource).toContain('useZoomCallTranscriptQuery')
    expect(helperSource).toContain('extractZoomRecordingRefs')
    expect(helperSource).toContain('formatZoomTranscriptProvenance')
    expect(helperSource).not.toContain('.vue')
  })
})
