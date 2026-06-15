import { describe, expect, it } from 'vitest'
import { previewThreadMessageBody, splitThreadMessageBody } from './threadMessageBody'
import type { ThreadMessage } from '../types/communications'

function threadMessage(overrides: Partial<ThreadMessage> = {}): ThreadMessage {
  return {
    message_id: 'message-1',
    provider_record_id: 'provider-1',
    account_id: 'account-1',
    subject: 'Quarterly update',
    sender: 'alex@example.com',
    sender_display_name: 'Alex',
    body_text: 'Body',
    occurred_at: null,
    projected_at: '2026-06-15T10:00:00Z',
    workflow_state: 'new',
    importance_score: null,
    ai_category: null,
    ai_summary: null,
    delivery_state: 'received',
    attachment_count: 0,
    attachments: [],
    ...overrides
  }
}

describe('threadMessageBody', () => {
  it('splits quoted reply tails from the main body', () => {
    expect(splitThreadMessageBody('Thanks for the update.\n\nOn Tue, Alex wrote:\n> Prior note')).toEqual({
      mainText: 'Thanks for the update.',
      quotedText: 'On Tue, Alex wrote:\n> Prior note'
    })
  })

  it('returns the full body as main text when no quoted segment exists', () => {
    expect(splitThreadMessageBody('Line one\nLine two')).toEqual({
      mainText: 'Line one\nLine two',
      quotedText: ''
    })
  })

  it('uses the non-quoted body for expanded preview text', () => {
    const message = threadMessage({
      body_text: 'Thanks for the update.\n\nOn Tue, Alex wrote:\n> Prior note'
    })

    expect(previewThreadMessageBody(message, true)).toBe('Thanks for the update.')
    expect(previewThreadMessageBody(message, false)).toContain('On Tue, Alex wrote:')
  })
})
