import { describe, expect, it } from 'vitest'
import type { ThreadMessage } from '../types/communications'
import {
  defaultExpandedThreadMessageIds,
  hasQuotedThreadMessages,
  summarizeThreadExpansion
} from './threadConversationPresentation'

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

describe('threadConversationPresentation', () => {
  it('auto-expands only the latest message in a thread by default', () => {
    const ids = defaultExpandedThreadMessageIds([
      threadMessage({ message_id: 'message-1' }),
      threadMessage({ message_id: 'message-2' })
    ])

    expect([...ids]).toEqual(['message-2'])
  })

  it('detects whether any thread message has quoted content', () => {
    expect(hasQuotedThreadMessages([
      threadMessage({ body_text: 'Body only' }),
      threadMessage({ body_text: 'Reply\n\nOn Tue, Alex wrote:\n> Prior note' })
    ])).toBe(true)

    expect(hasQuotedThreadMessages([
      threadMessage({ body_text: 'Body only' })
    ])).toBe(false)
  })

  it('summarizes expansion controls from messages and expanded ids', () => {
    expect(summarizeThreadExpansion(
      [
        threadMessage({ message_id: 'message-1' }),
        threadMessage({ message_id: 'message-2' })
      ],
      new Set(['message-2'])
    )).toEqual({
      expandedCount: 1,
      canExpandAll: true,
      canCollapseAll: true
    })
  })
})
