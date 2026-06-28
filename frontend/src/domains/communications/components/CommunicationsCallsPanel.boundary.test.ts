import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsCallsPanel boundary', () => {
  it('uses provider-neutral calls queries and supports calls plus meetings modes', () => {
    const source = readFileSync(new URL('./CommunicationsCallsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useProviderCallsQuery')
    expect(source).toContain('useProviderCallTranscriptQuery')
    expect(source).toContain("mode: 'calls' | 'meetings'")
    expect(source).toContain("mode === 'meetings'")
    expect(source).toContain("props.mode === 'meetings' ? 'zoom' : undefined")
    expect(source).toContain("meetingProvider(call) === 'zoom'")
    expect(source).toContain('meetingParticipants(selectedCall).length')
    expect(source).toContain('meetingRecordingRefs(selectedCall).length')
    expect(source).toContain("{{ t('Recording references') }}")
    expect(source).toContain("{{ t('Open join URL') }}")
    expect(source).not.toContain('fetch(')
  })
})
