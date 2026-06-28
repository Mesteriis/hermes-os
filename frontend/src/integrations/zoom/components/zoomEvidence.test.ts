import { describe, expect, it } from 'vitest'

import { extractZoomRecordingRefs, formatZoomTranscriptProvenance } from './zoomEvidence'

describe('zoomEvidence', () => {
  it('extracts only valid recording refs from call metadata', () => {
    const recordingRefs = extractZoomRecordingRefs({
      recording_refs: [
        {
          recording_id: 'rec-1',
          recording_type: 'shared_screen_with_speaker_view',
          file_extension: 'MP4',
        },
        {
          recording_id: '',
          recording_type: 'audio_only',
        },
        null,
      ],
    })

    expect(recordingRefs).toEqual([
      {
        recording_id: 'rec-1',
        recording_type: 'shared_screen_with_speaker_view',
        file_extension: 'MP4',
      },
    ])
  })

  it('formats transcript provenance as stable sorted json', () => {
    const formatted = formatZoomTranscriptProvenance({
      metadata: {
        topic: 'Weekly review',
        account_id: 'zoom-1',
      },
      provider: 'zoom',
    })

    expect(formatted).toBe(`{
  "metadata": {
    "account_id": "zoom-1",
    "topic": "Weekly review"
  },
  "provider": "zoom"
}`)
  })

  it('returns dash when provenance is missing or empty', () => {
    expect(formatZoomTranscriptProvenance(null)).toBe('—')
    expect(formatZoomTranscriptProvenance({})).toBe('—')
  })
})
