import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ZoomObservedCallsPanel boundary', () => {
  it('loads projected call and transcript evidence through the query layer', () => {
    const source = readFileSync(new URL('./ZoomObservedCallsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useZoomProviderCallsQuery')
    expect(source).toContain('useZoomCallTranscriptQuery')
    expect(source).toContain('extractZoomRecordingRefs')
    expect(source).toContain('formatZoomTranscriptProvenance')
    expect(source).toContain("t('Observed calls')")
    expect(source).toContain("t('Transcript evidence')")
    expect(source).toContain("t('Recording references')")
    expect(source).toContain("t('Transcript provenance')")
    expect(source).not.toContain('fetch(')
  })
})
