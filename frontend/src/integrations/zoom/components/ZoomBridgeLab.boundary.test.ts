import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ZoomBridgeLab boundary', () => {
  it('uses the existing bridge mutations instead of raw fetch calls', () => {
    const source = readFileSync(new URL('./ZoomBridgeLab.vue', import.meta.url), 'utf8')

    expect(source).toContain('useBridgeZoomMeetingMutation')
    expect(source).toContain('useBridgeZoomRecordingMutation')
    expect(source).toContain('useBridgeZoomTranscriptMutation')
    expect(source).toContain('useImportZoomTranscriptFileMutation')
    expect(source).toContain('handleBridgeMeeting')
    expect(source).toContain('handleBridgeRecording')
    expect(source).toContain('handleBridgeTranscript')
    expect(source).toContain('handleImportTranscriptFile')
    expect(source).not.toContain('fetch(')
  })
})
