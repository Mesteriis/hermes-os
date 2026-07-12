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

  it('explains an unstructured AI response as a review item instead of an empty summary', () => {
    expect(mailInspectorSummary({
      ai_summary: null,
      message_metadata: {}
    }, {
      message_id: 'msg:1',
      ai_state: 'REVIEW_REQUIRED',
      review_reason: 'model_response_invalid',
      last_error: null,
      retry_count: 0,
      next_attempt_at: null,
      processing_lease_expires_at: null,
      created_at: '2026-07-12T00:00:00Z',
      updated_at: '2026-07-12T00:00:00Z'
    })).toContain('unstructured response')
  })
})
