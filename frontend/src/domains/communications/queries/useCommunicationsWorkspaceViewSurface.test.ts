import { describe, expect, it } from 'vitest'
import { mailInspectorSummary } from './useCommunicationsWorkspaceViewSurface'

describe('mailInspectorSummary', () => {
  it('uses the structured summary when the backend text summary is absent', () => {
    expect(mailInspectorSummary({
      ai_summary: null,
      message_metadata: {
        ai_summary_contract: {
          key_points: ['Review the renewal proposal'],
          action_items: ['Reply before Friday']
        }
      }
    })).toBe('Review the renewal proposal Reply before Friday')
  })
})
